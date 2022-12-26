import type {ApiPromise} from '@polkadot/api'
import type {SubmittableExtrinsic} from '@polkadot/api/types'
import type {Bytes, Compact, u64} from '@polkadot/types-codec'
import type {AccountId, WeightV2} from '@polkadot/types/interfaces'
import type {Codec, ISubmittableResult} from '@polkadot/types/types'
import {
  hexAddPrefix,
  hexStripPrefix,
  hexToU8a,
  stringToHex,
  u8aToHex,
} from '@polkadot/util'
import {
  sr25519Agree,
  sr25519KeypairFromSeed,
  sr25519Sign,
  waitReady,
} from '@polkadot/wasm-crypto'
import axios, {AxiosError} from 'axios'
import {from} from 'rxjs'
import type {CertificateData} from './certificate'
import {decrypt, encrypt} from './lib/aes-256-gcm'
import {randomHex} from './lib/hex'
import {prpc, pruntime_rpc as pruntimeRpc} from './proto'

export type Query = (
  encodedQuery: string,
  certificateData: CertificateData
) => Promise<string>

export type SidevmQuery = (
  bytes: Bytes,
  certificateData: CertificateData
) => Promise<string>

type EncryptedData = {
  iv: string
  pubkey: string
  data: string
}

type CreateEncryptedData = (
  data: string,
  agreementKey: Uint8Array
) => EncryptedData

export type Command = (params: {
  contractId: string
  payload: string
  deposit: number
}) => SubmittableExtrinsic<'promise', ISubmittableResult>

export interface PhalaInstance {
  query: Query
  command: Command
}

type CreateFn = (options: {
  api: ApiPromise
  baseURL: string
  contractId: string
  autoDeposit: boolean
}) => Promise<{
  api: ApiPromise
  sidevmQuery: SidevmQuery
  instantiate: SidevmQuery
}>

export const createPruntimeApi = (baseURL: string) => {
  // Create a http client prepared for protobuf
  const http = axios.create({
    baseURL,
    headers: {
      'Content-Type': 'application/octet-stream',
    },
    responseType: 'arraybuffer',
  }).post

  const pruntimeApi = pruntimeRpc.PhactoryAPI.create(
    async (method, requestData, callback) => {
      try {
        const res = await http<ArrayBuffer>(
          `/prpc/PhactoryAPI.${method.name}`,
          new Uint8Array(requestData)
        )
        callback(null, new Uint8Array(res.data))
      } catch (err: unknown) {
        if (
          err instanceof AxiosError &&
          err.response?.data instanceof ArrayBuffer
        ) {
          const message = new Uint8Array(err.response.data)
          callback(new Error(prpc.PrpcError.decode(message).message))
        } else {
          throw err
        }
      }
    }
  )

  return pruntimeApi
}

export const create: CreateFn = async ({
  api,
  baseURL,
  contractId,
  autoDeposit = false,
}) => {
  await waitReady()

  const pruntimeApi = createPruntimeApi(baseURL)

  // Get public key from remote for encrypting
  const {publicKey} = await pruntimeApi.getInfo({})

  if (!publicKey) throw new Error('No remote pubkey')
  const remotePubkey = hexAddPrefix(publicKey)

  // Generate a keypair for encryption
  // NOTE: each instance only has a pre-generated pair now, it maybe better to generate a new keypair every time encrypting
  const seed = hexToU8a(hexAddPrefix(randomHex(32)))
  const pair = sr25519KeypairFromSeed(seed)
  const [sk, pk] = [pair.slice(0, 64), pair.slice(64)]

  const queryAgreementKey = sr25519Agree(
    hexToU8a(hexAddPrefix(remotePubkey)),
    sk
  )
  let gasPrice = 0
  if (autoDeposit) {
    const contractInfo = await api.query.phalaFatContracts.contracts(contractId)
    const cluster = contractInfo.unwrap().cluster
    const clusterInfo = await api.query.phalaFatContracts.clusters(cluster)
    gasPrice = clusterInfo.unwrap().gasPrice.toNumber()
  }
  const contractKey = (
    await api.query.phalaRegistry.contractKeys(contractId)
  ).toString()

  if (!contractKey) {
    throw new Error(`No contract key for ${contractId}`)
  }

  const commandAgreementKey = sr25519Agree(hexToU8a(contractKey), sk)

  const createEncryptedData: CreateEncryptedData = (data, agreementKey) => {
    const iv = hexAddPrefix(randomHex(12))
    return {
      iv,
      pubkey: u8aToHex(pk),
      data: hexAddPrefix(encrypt(data, agreementKey, hexToU8a(iv))),
    }
  }

  const query: Query = async (encodedQuery, {certificate, pubkey, secret}) => {
    // Encrypt the ContractQuery.
    const encryptedData = createEncryptedData(encodedQuery, queryAgreementKey)
    const encodedEncryptedData = api
      .createType('EncryptedData', encryptedData)
      .toU8a()

    // Sign the encrypted data.
    const signature: pruntimeRpc.ISignature = {
      signedBy: certificate,
      signatureType: pruntimeRpc.SignatureType.Sr25519,
      signature: sr25519Sign(pubkey, secret, encodedEncryptedData),
    }

    // Send request.
    const requestData = {
      encodedEncryptedData,
      signature,
    }
    return pruntimeApi.contractQuery(requestData).then((res) => {
      const {encodedEncryptedData} = res
      const {data: encryptedData, iv} = api
        .createType('EncryptedData', encodedEncryptedData)
        .toJSON() as {
        iv: string
        data: string
      }
      const data = decrypt(encryptedData, queryAgreementKey, iv)
      return hexAddPrefix(data)
    })
  }

  const sidevmQuery: SidevmQuery = async (bytes, certificateData) =>
    query(
      api
        .createType('InkQuery', {
          head: {
            nonce: hexAddPrefix(randomHex(32)),
            id: contractId,
          },
          data: {
            SidevmMessage: bytes,
          },
        })
        .toHex(),
      certificateData
    )

  const instantiate: SidevmQuery = async (payload, certificateData) =>
    query(
      api
        .createType('InkQuery', {
          head: {
            nonce: hexAddPrefix(randomHex(32)),
            id: contractId,
          },
          data: {
            InkInstantiate: payload,
          },
        })
        .toHex(),
      certificateData
    )

  const command: Command = ({contractId, payload, deposit}) => {
    const encodedPayload = api
      .createType('CommandPayload', {
        encrypted: createEncryptedData(payload, commandAgreementKey),
      })
      .toHex()
    return api.tx.phalaFatContracts.pushContractMessage(
      contractId,
      encodedPayload,
      deposit
    )
  }

  const txContracts = (
    dest: AccountId,
    value: number,
    gas: {refTime: number},
    storageDepositLimit: number,
    encParams: Uint8Array
  ) => {
    let deposit = 0
    if (autoDeposit) {
      const gasFee = gas.refTime * gasPrice
      deposit = value + gasFee + (storageDepositLimit || 0)
    }
    return command({
      contractId: dest.toHex(),
      payload: api
        .createType('InkCommand', {
          InkMessage: {
            nonce: hexAddPrefix(randomHex(32)),
            message: api.createType('Vec<u8>', encParams).toHex(),
            transfer: value,
            gasLimit: gas.refTime,
            storageDepositLimit,
          },
        })
        .toHex(),
      deposit,
    })
  }

  Object.defineProperty(txContracts, 'meta', {
    value: {args: []},
    enumerable: true,
  })

  const instantiateWithCode = () => null
  instantiateWithCode.meta = {args: new Array(6)}

  Object.defineProperty(api.tx, 'contracts', {
    value: {
      instantiateWithCode,
      call: txContracts,
    },
    enumerable: true,
  })

  Object.defineProperty(api.rx.call, 'contractsApi', {
    value: {
      call: (
        origin: CertificateData,
        dest: AccountId,
        value: unknown,
        gasLimit: unknown,
        storageDepositLimit: unknown,
        inputData: Bytes
      ) => {
        return from(
          query(
            api
              .createType('InkQuery', {
                head: {
                  nonce: hexAddPrefix(randomHex(32)),
                  id: dest,
                },
                data: {
                  InkMessage: inputData,
                },
              })
              .toHex(),
            origin
          ).then((data) => {
            return api.createType(
              'ContractExecResult',
              (
                api.createType('InkResponse', hexAddPrefix(data)).toJSON() as {
                  result: {ok: {inkMessageReturn: string}}
                }
              ).result.ok.inkMessageReturn
            )
          })
        )
      },
    },
    enumerable: true,
  })

  Object.defineProperty(api.call, 'contractsApi', {
    value: {call: () => null},
    enumerable: true,
  })

  return {api, sidevmQuery, instantiate}
}

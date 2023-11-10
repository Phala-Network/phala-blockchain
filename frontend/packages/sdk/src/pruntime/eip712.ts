import { type ApiPromise } from '@polkadot/api'
import { ApiTypes, type SubmittableExtrinsic } from '@polkadot/api/types'
import { type U256, type U64 } from '@polkadot/types-codec'
import { hexToString, hexToU8a, u8aToHex } from '@polkadot/util'
import { blake2AsU8a, encodeAddress, secp256k1Compress } from '@polkadot/util-crypto'
import type { Account, Address, WalletClient } from 'viem'
import { hashMessage, recoverPublicKey } from 'viem'
import { type signTypedData } from 'viem/wallet'
import { signMessage } from 'viem/wallet'
import { callback } from '../utils/signAndSend'

// keccak256(b"phala/phat-contract")
const SALT = '0x0ea813d1592526d672ea2576d7a07914cef2ca301b35c5eed941f7c897512a00'

type SignTypedDataInput = Parameters<typeof signTypedData>[1]

/**
 * Get compact formatted ether address for a specified account via a Wallet Client.
 */
export async function etherAddressToCompactPubkey(
  client: WalletClient,
  account: Account,
  msg = 'Allows to access the pubkey address.'
) {
  const sign = await signMessage(client, { account, message: msg })
  const hash = hashMessage(msg)
  const recovered = await recoverPublicKey({ hash, signature: sign })
  const compactPubkey = u8aToHex(secp256k1Compress(hexToU8a(recovered)))
  return compactPubkey
}

interface EtherAddressToSubstrateAddressOptions {
  SS58Prefix?: number
  msg?: string
}

/**
 * Convert an Ethereum address to a Substrate address.
 */
export async function etherAddressToSubstrateAddress(
  client: WalletClient,
  account: Account,
  { SS58Prefix = 30, msg }: EtherAddressToSubstrateAddressOptions = {}
) {
  const compactPubkey = await etherAddressToCompactPubkey(client, account, msg)
  const substratePubkey = encodeAddress(blake2AsU8a(hexToU8a(compactPubkey)), SS58Prefix)
  return substratePubkey as Address
}

export function createEip712StructedDataSignCertificate(
  account: Account,
  encodedCert: string,
  ttl: number
): SignTypedDataInput {
  return {
    domain: {
      name: 'Phat Query Certificate',
      version: '1',
      salt: SALT,
    },
    message: {
      description:
        'You are signing a Certificate that can be used to query Phat Contracts using your identity without further prompts.',
      timeToLive: `The Certificate will be valid till block ${ttl}.`,
      encodedCert,
    },
    primaryType: 'IssueQueryCertificate',
    types: {
      EIP712Domain: [
        { name: 'name', type: 'string' },
        { name: 'version', type: 'string' },
        { name: 'salt', type: 'bytes32' },
      ],
      IssueQueryCertificate: [
        { name: 'description', type: 'string' },
        { name: 'timeToLive', type: 'string' },
        { name: 'encodedCert', type: 'bytes' },
      ],
    },
    account,
  }
}

export function createEip712StructedDataSignQuery(account: Account, encodedQuery: string): SignTypedDataInput {
  return {
    domain: {
      name: 'Phat Contract Query',
      version: '1',
      salt: SALT,
    },
    message: {
      description: 'You are signing a query request that would be sent to a Phat Contract.',
      encodedQuery: encodedQuery,
    },
    primaryType: 'PhatContractQuery',
    types: {
      EIP712Domain: [
        { name: 'name', type: 'string' },
        { name: 'version', type: 'string' },
        { name: 'salt', type: 'bytes32' },
      ],
      PhatContractQuery: [
        { name: 'description', type: 'string' },
        { name: 'encodedQuery', type: 'bytes' },
      ],
    },
    account,
  }
}

export interface Eip712Domain {
  name: string
  version: string
  chainId: number
  verifyingContract: Address
}

export function createEip712Domain(api: ApiPromise): Eip712Domain {
  try {
    const name = hexToString(api.consts.evmAccountMapping.eip712Name.toString())
    const version = hexToString(api.consts.evmAccountMapping.eip712Version.toString())
    const chainId = (api.consts.evmAccountMapping.eip712ChainID as U256).toNumber()
    const verifyingContract = api.consts.evmAccountMapping.eip712VerifyingContractAddress.toString() as Address
    return {
      name,
      version,
      chainId,
      verifyingContract,
    }
  } catch (_err) {
    throw new Error(
      'Create Eip712Domain object failed, possibly due to the unavailability of the evmAccountMapping pallet.'
    )
  }
}

export interface SubstrateCall {
  who: string
  callData: string
  nonce: number
}

export async function createSubstrateCall<T extends ApiTypes>(
  api: ApiPromise,
  substrateAddress: string,
  extrinsic: SubmittableExtrinsic<T>
) {
  const nonce = await api.query.evmAccountMapping.accountNonce<U64>(substrateAddress)
  return {
    who: substrateAddress,
    callData: extrinsic.inner.toHex(),
    nonce: nonce.toNumber(),
  }
}

/**
 * @params account Account  The viem WalletAccount instance for signging.
 * @params who string       The SS58 formated address of the account.
 * @params callData string  The encoded call data, usually create with `api.tx.foo.bar.inner.toHex()`
 * @params nonce number     The nonce of the account.
 */
export function createEip712StructedDataSubstrateCall(
  account: Account,
  domain: Eip712Domain,
  message: SubstrateCall
): SignTypedDataInput {
  return {
    account,
    types: {
      EIP712Domain: [
        {
          name: 'name',
          type: 'string',
        },
        {
          name: 'version',
          type: 'string',
        },
        {
          name: 'chainId',
          type: 'uint256',
        },
        {
          name: 'verifyingContract',
          type: 'address',
        },
      ],
      SubstrateCall: [
        { name: 'who', type: 'string' },
        { name: 'callData', type: 'bytes' },
        { name: 'nonce', type: 'uint64' },
      ],
    },
    primaryType: 'SubstrateCall',
    domain: domain,
    message: { ...message },
  }
}

export class unstable_WalletClientSigner {
  //
  // Resources
  //
  #apiPromise: ApiPromise
  #client: WalletClient
  #account: Account

  //
  // Options
  //
  #SS58Prefix: number

  //
  // State
  //
  #domain: Eip712Domain

  #address: Address | undefined

  constructor(api: ApiPromise, client: WalletClient, account: Account, { SS58Prefix = 30 } = {}) {
    this.#apiPromise = api
    this.#client = client
    this.#account = account
    this.#domain = createEip712Domain(api)
    this.#SS58Prefix = SS58Prefix
  }

  async ready(msg?: string): Promise<void> {
    this.#address = await etherAddressToSubstrateAddress(this.#client, this.#account, {
      SS58Prefix: this.#SS58Prefix,
      msg,
    })
  }

  static async create(
    api: ApiPromise,
    client: WalletClient,
    account: Account,
    options?: EtherAddressToSubstrateAddressOptions
  ) {
    const signer = new unstable_WalletClientSigner(api, client, account, options)
    await signer.ready(options?.msg)
    return signer
  }

  /**
   * SS58 format address derived from the Ethereum EOA.
   */
  get address(): Address {
    if (!this.#address) {
      throw new Error('WalletClientSigner is not ready.')
    }
    return this.#address
  }

  /**
   *
   */
  async send(extrinsic: SubmittableExtrinsic<'promise'>) {
    const substrateCall = await createSubstrateCall(this.#apiPromise, this.address, extrinsic)
    const typedData = createEip712StructedDataSubstrateCall(this.#account, this.#domain, substrateCall)
    const signature = await this.#client.signTypedData(typedData)
    return await new Promise(async (resolve, reject) => {
      try {
        await this.#apiPromise.tx.evmAccountMapping
          .metaCall(this.address, substrateCall.callData, substrateCall.nonce, signature, null)
          .send((result) => {
            callback(resolve, reject, result)
          })
      } catch (error) {
        const isCancelled = (error as Error).message.indexOf('Cancelled') !== -1
        Object.defineProperty(error, 'isCancelled', {
          enumerable: false,
          value: isCancelled,
        })
        reject(error)
      }
    })
  }
}

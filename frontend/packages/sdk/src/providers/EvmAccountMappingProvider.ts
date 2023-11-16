import type { ApiPromise, SubmittableResult } from '@polkadot/api'
import type { SubmittableExtrinsic } from '@polkadot/api/types'
import type { ISubmittableResult } from '@polkadot/types/types'
import { hexToU8a } from '@polkadot/util'
import { blake2AsU8a, encodeAddress } from '@polkadot/util-crypto'
import type { Account, Address, WalletClient } from 'viem'
import { type CertificateData, unstable_signEip712Certificate } from '../pruntime/certificate'
import {
  type Eip712Domain,
  type EtherAddressToSubstrateAddressOptions,
  createEip712Domain,
  createEip712StructedDataSubstrateCall,
  createSubstrateCall,
  etherAddressToCompressedPubkey,
} from '../pruntime/eip712'
import { callback } from '../utils/signAndSend'
import { Provider } from './types'

export class unstable_EvmAccountMappingProvider implements Provider {
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

  #compressedPubkey: Address | undefined
  #address: Address | undefined

  constructor(api: ApiPromise, client: WalletClient, account: Account, { SS58Prefix = 30 } = {}) {
    this.#apiPromise = api
    this.#client = client
    this.#account = account
    this.#domain = createEip712Domain(api)
    this.#SS58Prefix = SS58Prefix
  }

  async ready(msg?: string): Promise<void> {
    this.#compressedPubkey = await etherAddressToCompressedPubkey(this.#client, this.#account, msg)
    this.#address = encodeAddress(blake2AsU8a(hexToU8a(this.#compressedPubkey)), this.#SS58Prefix) as Address
  }

  static async create(
    api: ApiPromise,
    client: WalletClient,
    account: Account,
    options?: EtherAddressToSubstrateAddressOptions
  ) {
    const signer = new unstable_EvmAccountMappingProvider(api, client, account, options)
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

  get evmAccount(): Account {
    if (!this.#account) {
      throw new Error('WalletClientSigner is not ready.')
    }
    return this.#account
  }

  get compressedPubkey(): Address {
    if (!this.#compressedPubkey) {
      throw new Error('WalletClientSigner is not ready.')
    }
    return this.#compressedPubkey
  }

  /**
   *
   */
  async send<TSubmittableResult extends SubmittableResult = SubmittableResult>(
    extrinsic: SubmittableExtrinsic<'promise'>,
    transform?: (input: ISubmittableResult) => ISubmittableResult
  ): Promise<TSubmittableResult> {
    const substrateCall = await createSubstrateCall(this.#apiPromise, this.address, extrinsic)
    const typedData = createEip712StructedDataSubstrateCall(this.#account, this.#domain, substrateCall)
    const signature = await this.#client.signTypedData(typedData)
    return await new Promise(async (resolve, reject) => {
      try {
        const _extrinsic = this.#apiPromise.tx.evmAccountMapping.metaCall(
          this.address,
          substrateCall.callData,
          substrateCall.nonce,
          signature,
          null
        )
        if (transform) {
          return _extrinsic.withResultTransform(transform).send((result) => callback(resolve, reject, result))
        } else {
          return _extrinsic.send((result) => callback(resolve, reject, result))
        }
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

  async signCertificate(ttl?: number): Promise<CertificateData> {
    if (!this.#compressedPubkey) {
      throw new Error('WalletClientSigner is not ready.')
    }
    return await unstable_signEip712Certificate({
      client: this.#client,
      account: this.#account,
      compressedPubkey: this.#compressedPubkey,
      ttl,
    })
  }

  async adjustStake(contractId: string, amount: number): Promise<void> {
    await this.send(this.#apiPromise.tx.phalaPhatTokenomic.adjustStake(contractId, amount))
  }
}

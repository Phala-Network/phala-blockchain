import type { ApiPromise, SubmittableResult } from '@polkadot/api'
import { type SubmittableExtrinsic } from '@polkadot/api/types'
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
  etherAddressToCompactPubkey,
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

  #compactPubkey: Address | undefined
  #address: Address | undefined

  constructor(api: ApiPromise, client: WalletClient, account: Account, { SS58Prefix = 30 } = {}) {
    this.#apiPromise = api
    this.#client = client
    this.#account = account
    this.#domain = createEip712Domain(api)
    this.#SS58Prefix = SS58Prefix
  }

  async ready(msg?: string): Promise<void> {
    this.#compactPubkey = await etherAddressToCompactPubkey(this.#client, this.#account, msg)
    this.#address = encodeAddress(blake2AsU8a(hexToU8a(this.#compactPubkey)), this.#SS58Prefix) as Address
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

  get proxiedEvmAccount(): Account {
    if (!this.#account) {
      throw new Error('WalletClientSigner is not ready.')
    }
    return this.#account
  }

  /**
   *
   */
  async send<TSubmittableResult extends SubmittableResult = SubmittableResult>(
    extrinsic: SubmittableExtrinsic<'promise'>
  ): Promise<TSubmittableResult> {
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

  async signCertificate(ttl?: number): Promise<CertificateData> {
    if (!this.#compactPubkey) {
      throw new Error('WalletClientSigner is not ready.')
    }
    return await unstable_signEip712Certificate({
      client: this.#client,
      account: this.#account,
      compactPubkey: this.#compactPubkey,
      ttl,
    })
  }
}

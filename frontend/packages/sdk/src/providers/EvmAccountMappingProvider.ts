import type { ApiPromise, SubmittableResult } from '@polkadot/api'
import type { SubmittableExtrinsic } from '@polkadot/api/types'
import type { u16 } from '@polkadot/types/primitive'
import type { ISubmittableResult } from '@polkadot/types/types'
import { encodeAddress } from '@polkadot/util-crypto'
import type { Account, Address, Hex, WalletClient } from 'viem'
import { type CertificateData, signEip712Certificate } from '../pruntime/certificate'
import {
  type Eip712Domain,
  createEip712Domain,
  createEip712StructedDataSubstrateCall,
  createSubstrateCall,
  evmPublicKeyToSubstratePubkey,
  recoverEvmPubkey,
} from '../pruntime/eip712'
import { callback } from '../utils/signAndSend'
import { Provider } from './types'

type AccountLike = Account | { address: Address }

export interface EvmCaller {
  compressedPubkey: `0x${string}`
  address: Address
}

export interface EvmAccountMappingProviderOptions {
  SS58Prefix?: number
}

/**
 * @class EvmAccountMappingProvider
 */
export class EvmAccountMappingProvider implements Provider {
  static readonly identity = 'evmAccountMapping'

  //
  // Resources
  //
  #apiPromise: ApiPromise
  #client: WalletClient
  #account: AccountLike

  //
  // Options
  //
  #SS58Prefix: number

  //
  // State
  //
  #domain: Eip712Domain

  // The Substrate Address
  #address: string | undefined

  #recoveredPubkey:
    | {
        compressed: Hex
        uncompressed: Hex
      }
    | undefined

  #cachedCert: CertificateData | undefined
  #certExpiredAt: number | undefined

  constructor(
    api: ApiPromise,
    client: WalletClient,
    account: AccountLike,
    { SS58Prefix = undefined }: EvmAccountMappingProviderOptions = {}
  ) {
    this.#apiPromise = api
    this.#client = client
    this.#account = account
    this.#domain = createEip712Domain(api)
    this.#SS58Prefix = SS58Prefix || (api.consts.system?.ss58Prefix as u16).toNumber() || 42
  }

  get name(): 'evmAccountMapping' {
    return EvmAccountMappingProvider.identity
  }

  async ready(msg?: string): Promise<void> {
    const version = this.#apiPromise.consts.evmAccountMapping.eip712Version.toString()
    if (version === '0x31') {
      this.#recoveredPubkey = await recoverEvmPubkey(this.#client, this.#account as Account, msg)
      this.#address = encodeAddress(evmPublicKeyToSubstratePubkey(this.#recoveredPubkey.compressed), this.#SS58Prefix)
    } else if (version === '0x32') {
      this.#recoveredPubkey = await recoverEvmPubkey(this.#client, this.#account as Account, msg)
      this.#address = encodeAddress(evmPublicKeyToSubstratePubkey(this.#recoveredPubkey.uncompressed), this.#SS58Prefix)
    } else {
      throw new Error(
        `Unsupported evm_account_mapping pallet version: const evmAccountMapping.eip712Version = ${version}`
      )
    }
  }

  static async create(
    api: ApiPromise,
    client: WalletClient,
    account: AccountLike,
    options?: { msg?: string; SS58Prefix?: number }
  ) {
    const signer = new EvmAccountMappingProvider(api, client, account, options)
    await signer.ready(options?.msg)
    return signer
  }

  /**
   * SS58 format address derived from the Ethereum EOA.
   */
  get address(): string {
    if (!this.#address) {
      throw new Error('WalletClientSigner is not ready.')
    }
    return this.#address
  }

  get evmAccount(): AccountLike {
    if (!this.#account) {
      throw new Error('WalletClientSigner is not ready.')
    }
    return this.#account
  }

  get compressedPubkey(): Address {
    if (!this.#recoveredPubkey) {
      throw new Error('WalletClientSigner is not ready.')
    }
    return this.#recoveredPubkey.compressed
  }

  get evmCaller(): EvmCaller {
    if (!this.#recoveredPubkey) {
      throw new Error('WalletClientSigner is not ready.')
    }
    return {
      compressedPubkey: this.#recoveredPubkey.compressed,
      address: this.#account.address,
    }
  }

  /**
   *
   */
  async send<TSubmittableResult extends SubmittableResult = SubmittableResult>(
    extrinsic: SubmittableExtrinsic<'promise'>,
    transform?: (input: ISubmittableResult) => ISubmittableResult
  ): Promise<TSubmittableResult> {
    const substrateCall = await createSubstrateCall(this.#apiPromise, this.address, extrinsic)
    const typedData = createEip712StructedDataSubstrateCall(this.#account as Account, this.#domain, substrateCall)
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

  /**
   * Sign a certificate off-chain query. Optional ttl in seconds. ttl is
   * 0x7fffffff by default.
   *
   * @param ttl? number
   */
  async signCertificate(ttl: number = 0x7f_fff_fff): Promise<CertificateData> {
    if (!this.#recoveredPubkey) {
      throw new Error('WalletClientSigner is not ready.')
    }
    const now = Date.now()
    const isExpired = this.#certExpiredAt && this.#certExpiredAt < now
    if (this.#cachedCert && !isExpired) {
      return this.#cachedCert
    }
    this.#cachedCert = await signEip712Certificate({
      client: this.#client,
      account: this.#account as Account,
      compressedPubkey: this.#recoveredPubkey.compressed,
      ttl,
    })
    this.#certExpiredAt = now + ttl * 1_000
    return this.#cachedCert
  }

  get isCertificateExpired(): boolean {
    if (!this.#cachedCert) {
      return true
    }
    const now = Date.now()
    return !!(this.#certExpiredAt && this.#certExpiredAt < now)
  }

  get hasCertificate(): boolean {
    const now = Date.now()
    return !!(this.#cachedCert && !(this.#certExpiredAt && this.#certExpiredAt < now))
  }

  revokeCertificate(): void {
    this.#cachedCert = undefined
    this.#certExpiredAt = undefined
  }

  async adjustStake(contractId: string, amount: number): Promise<void> {
    await this.send(this.#apiPromise.tx.phalaPhatTokenomic.adjustStake(contractId, amount))
  }
}

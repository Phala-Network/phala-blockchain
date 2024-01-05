import type { ApiPromise, SubmittableResult } from '@polkadot/api'
import { Keyring } from '@polkadot/api'
import type { SubmittableExtrinsic } from '@polkadot/api/types'
import type { KeyringPair, KeyringPair$Meta } from '@polkadot/keyring/types'
import { cryptoWaitReady } from '@polkadot/util-crypto'
import type { KeypairType, Prefix } from '@polkadot/util-crypto/types'
import { type CertificateData, signCertificate } from '../pruntime/certificate'
import signAndSend from '../utils/signAndSend'
import { Provider } from './types'

export interface KeyringPairCreate {
  ss58Format?: Prefix
  meta?: KeyringPair$Meta
  type?: KeypairType
}

/**
 * @class KeyringPairProvider
 */
export class KeyringPairProvider implements Provider {
  static readonly identity = 'keyring'

  readonly #apiPromise: ApiPromise
  readonly #pair: KeyringPair

  #cachedCert: CertificateData | undefined
  #certExpiredAt: number | undefined

  constructor(api: ApiPromise, pair: KeyringPair) {
    this.#apiPromise = api
    this.#pair = pair
  }

  static async create(api: ApiPromise, pair: KeyringPair) {
    return new KeyringPairProvider(api, pair)
  }

  static async createFromSURI(api: ApiPromise, suri: string, options?: KeyringPairCreate) {
    await cryptoWaitReady()
    const { ss58Format, meta, type } = options || {}
    const keyring = new Keyring({ type, ss58Format })
    const pair = keyring.addFromUri(suri, meta, type)
    return new KeyringPairProvider(api, pair)
  }

  get address() {
    return this.#pair.address
  }

  get name(): 'keyring' {
    return KeyringPairProvider.identity
  }

  /**
   * Send an extrinsic to the network.
   */
  send<TSubmittableResult extends SubmittableResult = SubmittableResult>(
    extrinsic: SubmittableExtrinsic<'promise', TSubmittableResult>
  ): Promise<TSubmittableResult> {
    return signAndSend(extrinsic, this.#pair)
  }

  /**
   * Get a signed certificate from the account bind in the provider.
   */
  async signCertificate(ttl: number = 0x7fffffff): Promise<CertificateData> {
    const now = Date.now()
    const isExpired = this.#certExpiredAt && this.#certExpiredAt < now
    if (this.#cachedCert && !isExpired) {
      return this.#cachedCert
    }
    this.#cachedCert = await signCertificate({ pair: this.#pair, ttl })
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

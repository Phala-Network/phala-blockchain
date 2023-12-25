import type { SubmittableResult } from '@polkadot/api'
import type { SubmittableExtrinsic } from '@polkadot/api/types'
import type { ISubmittableResult } from '@polkadot/types/types'
import type { CertificateData } from '../pruntime/certificate'

export interface Provider {
  /**
   * The SS58 format address to use for this provider.
   */
  address: Readonly<string>

  name: Readonly<string>

  /**
   * Send an extrinsic to the network.
   */
  send<TSubmittableResult extends SubmittableResult = SubmittableResult>(
    extrinsic: SubmittableExtrinsic<'promise', TSubmittableResult>,
    transform?: (input: ISubmittableResult) => TSubmittableResult
  ): Promise<TSubmittableResult>

  /**
   * Get a signed certificate from the account bind in the provider.
   */
  signCertificate(ttl?: number): Promise<CertificateData>

  adjustStake(contractId: string, amount: number): Promise<void>
}

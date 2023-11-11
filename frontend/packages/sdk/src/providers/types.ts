import type { SubmittableResult } from '@polkadot/api'
import type { SubmittableExtrinsic } from '@polkadot/api/types'
import type { ISubmittableResult } from '@polkadot/types/types'
import type { Address } from 'viem'
import type { CertificateData } from '../pruntime/certificate'

export interface Provider {
  /**
   * The SS58 format address to use for this provider.
   */
  address: Readonly<Address>

  /**
   * Send an extrinsic to the network.
   */
  send<TSubmittableResult extends SubmittableResult = SubmittableResult>(
    extrinsic: SubmittableExtrinsic<'promise'>,
    transform?: (input: ISubmittableResult) => ISubmittableResult
  ): Promise<TSubmittableResult>

  /**
   * Get a signed certificate from the account bind in the provider.
   */
  signCertificate(ttl?: number): Promise<CertificateData>
}

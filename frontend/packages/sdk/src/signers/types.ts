import type { SubmittableResult } from '@polkadot/api'
import type { SubmittableExtrinsic } from '@polkadot/api/types'
import type { Address } from 'viem'
import type { CertificateData } from '../pruntime/certificate'

export interface Signer {
  address: Readonly<Address>

  send<TSubmittableResult extends SubmittableResult = SubmittableResult>(
    extrinsic: SubmittableExtrinsic<'promise'>
  ): Promise<TSubmittableResult>

  signCertificate(ttl?: number): Promise<CertificateData>
}

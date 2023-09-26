import type { SubmittableResult } from '@polkadot/api'
import type { Signer as InjectedSigner } from '@polkadot/api/types'
import type { ApiTypes } from '@polkadot/api-base/types/base'
import type { AddressOrPair, SubmittableExtrinsic } from '@polkadot/api-base/types/submittable'

export class SignAndSendError extends Error {
  readonly isCancelled: boolean = false
}

function callback<TSubmittableResult>(
  resolve: (value: TSubmittableResult) => void,
  reject: (reason?: any) => void,
  result: SubmittableResult,
  unsub?: any
) {
  if (result.status.isInBlock) {
    let error
    for (const e of result.events) {
      const {
        event: { data, method, section },
      } = e
      if (section === 'system' && method === 'ExtrinsicFailed') {
        error = data[0]
      }
    }

    ;(unsub as any)()
    if (error) {
      reject(error)
    } else {
      resolve(result as TSubmittableResult)
    }
  } else if (result.status.isInvalid) {
    ;(unsub as any)()
    reject('Invalid transaction')
  }
}

function signAndSend<TSubmittableResult extends SubmittableResult = SubmittableResult>(
  target: SubmittableExtrinsic<ApiTypes>,
  pair: AddressOrPair
): Promise<TSubmittableResult>
function signAndSend<TSubmittableResult extends SubmittableResult = SubmittableResult>(
  target: SubmittableExtrinsic<ApiTypes>,
  address: AddressOrPair,
  signer: InjectedSigner
): Promise<TSubmittableResult>
function signAndSend(target: SubmittableExtrinsic<ApiTypes>, address: AddressOrPair, signer?: InjectedSigner) {
  // Ready -> Broadcast -> InBlock -> Finalized
  return new Promise(async (resolve, reject) => {
    try {
      if (signer) {
        const unsub = await target.signAndSend(address, { signer }, (result) => {
          callback(resolve, reject, result, unsub)
        })
      } else {
        const unsub = await target.signAndSend(address, (result) => {
          callback(resolve, reject, result, unsub)
        })
      }
    } catch (error) {
      const isCancelled = (error as Error).message.indexOf('Cancelled') !== -1
      Object.defineProperty(error, 'isCancelled', {
        enumerable: false,
        value: isCancelled,
      })
      reject(error as SignAndSendError)
    }
  })
}

export default signAndSend

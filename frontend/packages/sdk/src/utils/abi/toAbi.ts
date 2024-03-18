import { Abi } from '@polkadot/api-contract'
import type { ChainProperties } from '@polkadot/types/interfaces'
import type { AbiLike } from '../../types'

export function toAbi<T extends Abi = Abi>(maybeAbi: AbiLike, chainProperties?: ChainProperties): T {
  if (maybeAbi instanceof Abi) {
    return maybeAbi as T
  }
  return new Abi(maybeAbi, chainProperties) as T
}

import type { Abi } from '@polkadot/api-contract/Abi'
import type { Result, Vec, u8 } from '@polkadot/types-codec'
import type { IEnum, IMap } from '@polkadot/types-codec/types'

export interface InkQueryOk extends IEnum {
  asInkMessageReturn: Vec<u8>
}

export interface InkQueryError extends IEnum {}

export interface InkResponse extends IMap {
  nonce: Vec<u8>
  result: Result<InkQueryOk, InkQueryError>
}

export type AbiLike = string | Record<string, unknown> | Abi

export type WasmLike = Uint8Array | string | Buffer | null | undefined

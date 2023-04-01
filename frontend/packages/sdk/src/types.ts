import type { Result, u8, Vec } from "@polkadot/types-codec";
import type { IMap, IEnum } from '@polkadot/types-codec/types';
import type { Abi } from '@polkadot/api-contract/Abi';

export interface InkQueryOk extends IEnum {
  asInkMessageReturn: Vec<u8>
}

export interface InkQueryError extends IEnum {
}

export interface InkResponse extends IMap {
  nonce: Vec<u8>;
  result: Result<InkQueryOk, InkQueryError>;
}

export type AbiLike = string | Record<string, unknown> | Abi

export type WasmLike = Uint8Array | string | Buffer | null | undefined
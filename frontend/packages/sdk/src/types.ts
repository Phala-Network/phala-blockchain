import type { Result, u8, Vec } from "@polkadot/types-codec";
import type { IMap, IEnum } from '@polkadot/types-codec/types';

export interface InkQueryOk extends IEnum {
  asInkMessageReturn: Vec<u8>
}

export interface InkQueryError extends IEnum {
}

export interface InkResponse extends IMap {
  nonce: Vec<u8>;
  result: Result<InkQueryOk, InkQueryError>;
}
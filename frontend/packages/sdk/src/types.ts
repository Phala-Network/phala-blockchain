import type { Abi } from '@polkadot/api-contract/Abi'
import type { AccountId, Balance } from '@polkadot/types/interfaces'
import type { Result, Struct, Vec, u128, u32, u8 } from '@polkadot/types-codec'
import type { IEnum, IMap } from '@polkadot/types-codec/types'
import type { PinkContractPromise, PinkContractQuery, PinkContractTx } from './contracts/PinkContract'
import { type EvmAccountMappingProvider } from './providers/EvmAccountMappingProvider'
import { type KeyringPairProvider } from './providers/KeyringPairProvider'
import { type UIKeyringProvider } from './providers/UIKeyringProvider'

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

export interface FrameSystemAccountInfo extends Struct {
  readonly nonce: u32
  readonly consumers: u32
  readonly providers: u32
  readonly sufficients: u32
  readonly data: PalletBalancesAccountData
}

interface PalletBalancesAccountData extends Struct {
  readonly free: u128
  readonly reserved: u128
  readonly frozen: u128
  readonly flags: u128
}

export type AnyProvider = EvmAccountMappingProvider | UIKeyringProvider | KeyringPairProvider

//
// Typing for the SystemContract
//

export type SystemContract = PinkContractPromise<{
  owner: PinkContractQuery<[], [AccountId]>
  ['system::totalBalanceOf']: PinkContractQuery<[string | AccountId], Balance>
  ['system::freeBalanceOf']: PinkContractQuery<[string | AccountId], Balance>
}>

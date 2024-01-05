import type { Abi } from '@polkadot/api-contract/Abi'
import type { Bool, Enum, Map, Option, Result, Struct, Vec, u128, u16, u32, u64, u8 } from '@polkadot/types'
import type { VecFixed } from '@polkadot/types/codec'
import type { AccountId, Balance } from '@polkadot/types/interfaces'
import type { ITuple } from '@polkadot/types/types'
import type { PinkContractPromise, PinkContractQuery, PinkContractTx } from './contracts/PinkContract'
import { type EvmAccountMappingProvider } from './providers/EvmAccountMappingProvider'
import { type KeyringPairProvider } from './providers/KeyringPairProvider'
import { type UIKeyringProvider } from './providers/UIKeyringProvider'

export interface InkQueryOk extends Enum {
  asInkMessageReturn: Vec<u8>
}

export interface InkQueryError extends Enum {}

export interface InkResponse extends Map {
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

export interface SidevmConfig extends Struct {
  max_code_size: u32
  max_memory_pages: u32
  vital_capacity: u64
  deadline: u32
}

export interface LiteralSideVmConfig {
  max_code_size: number
  max_memory_pages: number
  vital_capacity: number
  deadline: number
}

export interface HookPoint extends Enum {
  readonly isOnBlockEnd: boolean
  readonly asOnBlockEnd: null
  readonly type: 'onBlockEnd'
}

export type SystemContract = PinkContractPromise<
  {
    owner: PinkContractQuery<[], [AccountId]>
    system: {
      version: PinkContractQuery<[], ITuple<[u16, u16, u16]>>
      getDriver: PinkContractQuery<[string], Option<AccountId>>
      getDriver2: PinkContractQuery<[string], Option<ITuple<[u32, AccountId]>>>
      deploySidevmTo: PinkContractQuery<[string | AccountId, VecFixed<u8>]>
      stopSidevmAt: PinkContractQuery<[string | AccountId]>
      setContractWeight: PinkContractQuery<[string | AccountId, u8]>
      totalBalanceOf: PinkContractQuery<[string | AccountId], Balance>
      freeBalanceOf: PinkContractQuery<[string | AccountId], Balance>
      isAdmin: PinkContractQuery<[string | AccountId], Bool>
      doUpgrade: PinkContractQuery<[string | AccountId, [u16 | number, u16 | number, u16 | number]]>
      codeExists: PinkContractQuery<[string | VecFixed<u8>, 'Ink' | 'Sidevm'], Bool> // TODO: enum
      codeHash: PinkContractQuery<[string | AccountId], VecFixed<u8>> // TODO: enum
      driverHistory: PinkContractQuery<[string], Option<Vec<ITuple<[u32, AccountId]>>>>
      currentEventChainHead: PinkContractQuery<[], ITuple<[u64, VecFixed<u8>]>>
      deploySidevmToWorkers: PinkContractQuery<
        [
          string | AccountId, // contract
          string | VecFixed<u8>, // code_hash
          string[] | Vec<VecFixed<u8>>, // workers
          LiteralSideVmConfig | SidevmConfig, // config
        ]
      >
      setSidevmDeadline: PinkContractQuery<[string | AccountId, number | u32]>
    }
    // @deprecated Backward compatible,
    ['system::version']: PinkContractQuery<[], ITuple<[u16, u16, u16]>>
    ['system::getDriver']: PinkContractQuery<[string], Option<AccountId>>
    ['system::getDriver2']: PinkContractQuery<[string], Option<ITuple<[u32, AccountId]>>>
    ['system::deploySidevmTo']: PinkContractQuery<[string | AccountId, VecFixed<u8>]>
    ['system::stopSidevmAt']: PinkContractQuery<[string | AccountId]>
    ['system::setContractWeight']: PinkContractQuery<[string | AccountId, u8]>
    ['system::totalBalanceOf']: PinkContractQuery<[string | AccountId], Balance>
    ['system::freeBalanceOf']: PinkContractQuery<[string | AccountId], Balance>
    ['system::isAdmin']: PinkContractQuery<[string | AccountId], Bool>
    ['system::doUpgrade']: PinkContractQuery<[string | AccountId, [u16 | number, u16 | number, u16 | number]]>
    ['system::codeExists']: PinkContractQuery<[string | VecFixed<u8>, 'Ink' | 'Sidevm'], Bool>
    ['system::codeHash']: PinkContractQuery<[string | AccountId], VecFixed<u8>>
    ['system::driverHistory']: PinkContractQuery<[string], Option<Vec<ITuple<[u32, AccountId]>>>>
    ['system::currentEventChainHead']: PinkContractQuery<[], ITuple<[u64, VecFixed<u8>]>>
    ['system::deploySidevmToWorkers']: PinkContractQuery<
      [
        string | AccountId, // contract
        string | VecFixed<u8>, // code_hash
        string[] | Vec<VecFixed<u8>>, // workers
        LiteralSideVmConfig | SidevmConfig, // config
      ]
    >
    ['system::setSidevmDeadline']: PinkContractQuery<[string | AccountId, number | u32]>
  },
  {
    system: {
      grantAdmin: PinkContractTx<[AccountId]>
      setDriver: PinkContractTx<[string, AccountId]>
      setHook: PinkContractTx<
        [
          'onBlockEnd' | HookPoint, // hook
          string | AccountId, // contract
          number | `0x${string}` | u32, // selector
          number | bigint | u64, // gas_limit
        ]
      >
      upgradeSystemContract: PinkContractTx<[]>
      upgradeRuntime: PinkContractTx<[[u32 | number, u32 | number]]>
    }
    contractDeposit: {
      changeDeposit: PinkContractTx<
        [
          string | AccountId, // contract
          number | bigint | Balance, // deposit
        ]
      >
    }
    // @deprecated Backward compatible,
    ['system::grantAdmin']: PinkContractTx<[AccountId]>
    ['system::setDriver']: PinkContractTx<[string, AccountId]>
    ['system::setHook']: PinkContractTx<
      [
        'onBlockEnd' | HookPoint, // hook
        string | AccountId, // contract
        number | `0x${string}` | u32, // selector
        number | bigint | u64, // gas_limit
      ]
    >
    ['system::upgradeSystemContract']: PinkContractTx<[]>
    ['system::upgradeRuntime']: PinkContractTx<[[u32 | number, u32 | number]]>
    ['contractDeposit::changeDeposit']: PinkContractTx<
      [
        string | AccountId, // contract
        number | bigint | Balance, // deposit
      ]
    >
  }
>

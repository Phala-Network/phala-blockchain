import type {ApiTypes} from '@polkadot/api/types'
import type {AugmentedQueryAt} from '@polkadot/api/types/storage'
import type {Option, Vec} from '@polkadot/types/codec'
import type {u128} from '@polkadot/types/primitive'
import type {Codec, Observable} from '@polkadot/types/types'
import type {
  AccountId32,
  H256,
  Sr25519Signature,
} from '@polkadot/types/interfaces'

type ContractId = H256 | string

type ContractClusterId = H256 | string

interface BasicContractInfo extends Codec {
  deployer: AccountId32
  cluster: ContractClusterId
}

interface ClusterInfo extends Codec {
  owner: AccountId32
  permission: unknown // @fixme ClusterPermission<AccountId>
  workers: Vec<Sr25519Signature>
  systemContract: ContractId
  gasPrice: u128
  depositPerItem: u128
  depositPerByte: u128
}

declare module '@polkadot/api/types/storage' {
  export interface AugmentedQueries<ApiType extends ApiTypes> {
    phalaFatContracts: {
      contracts: AugmentedQueryAt<
        ApiType,
        (contractId?: ContractId) => Observable<Option<BasicContractInfo>>
      >

      clusters: AugmentedQueryAt<
        ApiType,
        (clusterId?: ContractClusterId) => Observable<Option<ClusterInfo>>
      >
    }
  }
}

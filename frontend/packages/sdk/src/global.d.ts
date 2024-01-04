import type { ApiTypes } from '@polkadot/api/types'
import type { Text, U256 } from '@polkadot/types'
import { H160 } from '@polkadot/types/interfaces'

declare module 'browserify-cipher' {
  export { createCipheriv, createDecipheriv } from 'crypto'
}

declare module 'randombytes' {
  export { randombytes as default } from 'crypto'
}

declare module '@polkadot/api/types/consts' {
  export interface AugmentedConsts<ApiType extends ApiTypes> {
    evmAccountMapping: {
      eip712Name: Text
      eip712Version: Text
      eip712ChainID: U256
      eip712VerifyingContractAddress: H160
    }
  }

  export interface QueryableConsts<ApiType extends ApiTypes> extends AugmentedConsts<ApiType> {}
}

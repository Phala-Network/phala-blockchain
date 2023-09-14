import type { ApiOptions } from '@polkadot/api/types'
import { TypeRegistry, typeDefinitions } from '@polkadot/types'
import type { RegistryTypes } from '@polkadot/types/types'
import SubstrateLookupTypes from '@polkadot/types-augment/lookup/substrate'
import { types } from './lib/types'

export const phalaRegistryTypes = { ...types, ...typeDefinitions, ...SubstrateLookupTypes } as unknown as RegistryTypes

export const phalaTypes = new TypeRegistry()

phalaTypes.register(phalaRegistryTypes)

export function options(options: ApiOptions = {}): ApiOptions {
  return {
    ...options,
    types: {
      ...phalaRegistryTypes,
      ...(options.types || {}),
    } as unknown as ApiOptions['types'],
  }
}

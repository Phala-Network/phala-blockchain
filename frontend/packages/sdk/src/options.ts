import type { ApiOptions } from "@polkadot/api/types";

import { typeDefinitions } from '@polkadot/types';
import * as types from './lib/types';

export function options(options: ApiOptions = {}): ApiOptions {
    return {
        types: {
          ...types,
          ...typeDefinitions,
        } as unknown as ApiOptions['types'],
        ...options
    }
}

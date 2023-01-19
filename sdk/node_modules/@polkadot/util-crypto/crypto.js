// Copyright 2017-2022 @polkadot/util-crypto authors & contributors
// SPDX-License-Identifier: Apache-2.0

import { isReady, waitReady } from '@polkadot/wasm-crypto';
export const cryptoIsReady = isReady;
export function cryptoWaitReady() {
  return waitReady().then(() => {
    if (!isReady()) {
      throw new Error('Unable to initialize @polkadot/util-crypto');
    }
    return true;
  }).catch(() => false);
}
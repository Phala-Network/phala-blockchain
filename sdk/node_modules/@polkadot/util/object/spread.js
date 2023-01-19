// Copyright 2017-2022 @polkadot/util authors & contributors
// SPDX-License-Identifier: Apache-2.0

/**
 * @name objectSpread
 * @summary Concats all sources into the destination
 */
export function objectSpread(dest, ...sources) {
  for (let i = 0; i < sources.length; i++) {
    const src = sources[i];
    if (src) {
      if (typeof src.entries === 'function') {
        for (const [key, value] of src.entries()) {
          dest[key] = value;
        }
      } else {
        Object.assign(dest, src);
      }
    }
  }
  return dest;
}
// Copyright 2017-2022 @polkadot/util authors & contributors
// SPDX-License-Identifier: Apache-2.0

/** @internal */
export function createCmp(cmp) {
  return (...items) => {
    if (items.length === 0) {
      throw new Error('Must provide one or more arguments');
    }
    let result = items[0];
    for (let i = 1; i < items.length; i++) {
      if (cmp(items[i], result)) {
        result = items[i];
      }
    }
    return result;
  };
}
"use strict";

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.objectSpread = objectSpread;
// Copyright 2017-2022 @polkadot/util authors & contributors
// SPDX-License-Identifier: Apache-2.0

/**
 * @name objectSpread
 * @summary Concats all sources into the destination
 */
function objectSpread(dest) {
  for (let i = 0; i < (arguments.length <= 1 ? 0 : arguments.length - 1); i++) {
    const src = i + 1 < 1 || arguments.length <= i + 1 ? undefined : arguments[i + 1];
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
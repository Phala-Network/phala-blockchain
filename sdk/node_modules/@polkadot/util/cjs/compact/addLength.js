"use strict";

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.compactAddLength = compactAddLength;
var _u8a = require("../u8a");
var _toU8a = require("./toU8a");
// Copyright 2017-2022 @polkadot/util authors & contributors
// SPDX-License-Identifier: Apache-2.0

/**
 * @name compactAddLength
 * @description Adds a length prefix to the input value
 * @example
 * <BR>
 *
 * ```javascript
 * import { compactAddLength } from '@polkadot/util';
 *
 * console.log(compactAddLength(new Uint8Array([0xde, 0xad, 0xbe, 0xef]))); // Uint8Array([4 << 2, 0xde, 0xad, 0xbe, 0xef])
 * ```
 */
function compactAddLength(input) {
  return (0, _u8a.u8aConcatStrict)([(0, _toU8a.compactToU8a)(input.length), input]);
}
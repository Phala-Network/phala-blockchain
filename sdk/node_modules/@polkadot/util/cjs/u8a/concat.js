"use strict";

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.u8aConcat = u8aConcat;
exports.u8aConcatStrict = u8aConcatStrict;
var _toU8a = require("./toU8a");
// Copyright 2017-2022 @polkadot/util authors & contributors
// SPDX-License-Identifier: Apache-2.0

/**
 * @name u8aConcat
 * @summary Creates a concatenated Uint8Array from the inputs.
 * @description
 * Concatenates the input arrays into a single `UInt8Array`.
 * @example
 * <BR>
 *
 * ```javascript
 * import { { u8aConcat } from '@polkadot/util';
 *
 * u8aConcat(
 *   new Uint8Array([1, 2, 3]),
 *   new Uint8Array([4, 5, 6])
 * ); // [1, 2, 3, 4, 5, 6]
 * ```
 */
function u8aConcat() {
  const u8as = new Array(arguments.length);
  let length = 0;
  for (let i = 0; i < arguments.length; i++) {
    u8as[i] = (0, _toU8a.u8aToU8a)(i < 0 || arguments.length <= i ? undefined : arguments[i]);
    length += u8as[i].length;
  }
  return u8aConcatStrict(u8as, length);
}

/**
 * @name u8aConcatStrict
 * @description A strict version of [[u8aConcat]], accepting only Uint8Array inputs
 */
function u8aConcatStrict(u8as) {
  let length = arguments.length > 1 && arguments[1] !== undefined ? arguments[1] : 0;
  let offset = 0;
  if (!length) {
    for (let i = 0; i < u8as.length; i++) {
      length += u8as[i].length;
    }
  }
  const result = new Uint8Array(length);
  for (let i = 0; i < u8as.length; i++) {
    result.set(u8as[i], offset);
    offset += u8as[i].length;
  }
  return result;
}
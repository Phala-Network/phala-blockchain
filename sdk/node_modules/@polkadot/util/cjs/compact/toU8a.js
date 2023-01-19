"use strict";

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.compactToU8a = compactToU8a;
var _bn = require("../bn");
var _u8a = require("../u8a");
// Copyright 2017-2022 @polkadot/util authors & contributors
// SPDX-License-Identifier: Apache-2.0

const MAX_U8 = _bn.BN_TWO.pow(new _bn.BN(8 - 2)).isub(_bn.BN_ONE);
const MAX_U16 = _bn.BN_TWO.pow(new _bn.BN(16 - 2)).isub(_bn.BN_ONE);
const MAX_U32 = _bn.BN_TWO.pow(new _bn.BN(32 - 2)).isub(_bn.BN_ONE);
const BL_16 = {
  bitLength: 16
};
const BL_32 = {
  bitLength: 32
};

/**
 * @name compactToU8a
 * @description Encodes a number into a compact representation
 * @example
 * <BR>
 *
 * ```javascript
 * import { compactToU8a } from '@polkadot/util';
 *
 * console.log(compactToU8a(511, 32)); // Uint8Array([0b11111101, 0b00000111])
 * ```
 */
function compactToU8a(value) {
  const bn = (0, _bn.bnToBn)(value);
  if (bn.lte(MAX_U8)) {
    return new Uint8Array([bn.toNumber() << 2]);
  } else if (bn.lte(MAX_U16)) {
    return (0, _bn.bnToU8a)(bn.shln(2).iadd(_bn.BN_ONE), BL_16);
  } else if (bn.lte(MAX_U32)) {
    return (0, _bn.bnToU8a)(bn.shln(2).iadd(_bn.BN_TWO), BL_32);
  }
  const u8a = (0, _bn.bnToU8a)(bn);
  let length = u8a.length;

  // adjust to the minimum number of bytes
  while (u8a[length - 1] === 0) {
    length--;
  }
  if (length < 4) {
    throw new Error('Invalid length, previous checks match anything less than 2^30');
  }
  return (0, _u8a.u8aConcatStrict)([
  // subtract 4 as minimum (also catered for in decoding)
  new Uint8Array([(length - 4 << 2) + 0b11]), u8a.subarray(0, length)]);
}
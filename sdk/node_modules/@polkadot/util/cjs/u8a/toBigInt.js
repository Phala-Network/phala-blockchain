"use strict";

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.u8aToBigInt = u8aToBigInt;
var _xBigint = require("@polkadot/x-bigint");
var _consts = require("../bi/consts");
// Copyright 2017-2022 @polkadot/util authors & contributors
// SPDX-License-Identifier: Apache-2.0

const U8_MAX = (0, _xBigint.BigInt)(256);
const U16_MAX = (0, _xBigint.BigInt)(256 * 256);

/**
 * @name u8aToBigInt
 * @summary Creates a BigInt from a Uint8Array object.
 */
function u8aToBigInt(value) {
  let {
    isLe = true,
    isNegative = false
  } = arguments.length > 1 && arguments[1] !== undefined ? arguments[1] : {};
  if (!value || !value.length) {
    return (0, _xBigint.BigInt)(0);
  }
  const u8a = isLe ? value : value.reverse();
  const dvI = new DataView(u8a.buffer, u8a.byteOffset);
  const mod = u8a.length % 2;
  let result = (0, _xBigint.BigInt)(0);

  // This is mostly written for readability (with the single isNegative shortcut),
  // as opposed to performance, e.g. `u8aToBn` does loop unrolling, etc.
  if (isNegative) {
    for (let i = u8a.length - 2; i >= mod; i -= 2) {
      result = result * U16_MAX + (0, _xBigint.BigInt)(dvI.getUint16(i, true) ^ 0xffff);
    }
    if (mod) {
      result = result * U8_MAX + (0, _xBigint.BigInt)(dvI.getUint8(0) ^ 0xff);
    }
  } else {
    for (let i = u8a.length - 2; i >= mod; i -= 2) {
      result = result * U16_MAX + (0, _xBigint.BigInt)(dvI.getUint16(i, true));
    }
    if (mod) {
      result = result * U8_MAX + (0, _xBigint.BigInt)(dvI.getUint8(0));
    }
  }
  return isNegative ? result * -_consts._1n - _consts._1n : result;
}
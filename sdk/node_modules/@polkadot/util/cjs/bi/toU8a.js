"use strict";

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.nToU8a = nToU8a;
var _xBigint = require("@polkadot/x-bigint");
var _consts = require("./consts");
var _toBigInt = require("./toBigInt");
// Copyright 2017-2022 @polkadot/util authors & contributors
// SPDX-License-Identifier: Apache-2.0

const DIV = (0, _xBigint.BigInt)(256);
const NEG_MASK = (0, _xBigint.BigInt)(0xff);
function toU8a(value, isLe, isNegative) {
  const arr = [];
  if (isNegative) {
    value = (value + _consts._1n) * -_consts._1n;
  }
  while (value !== _consts._0n) {
    const mod = value % DIV;
    const val = Number(isNegative ? mod ^ NEG_MASK : mod);
    if (isLe) {
      arr.push(val);
    } else {
      arr.unshift(val);
    }
    value = (value - mod) / DIV;
  }
  return Uint8Array.from(arr);
}

/**
 * @name nToU8a
 * @summary Creates a Uint8Array object from a bigint.
 */
function nToU8a(value) {
  let {
    bitLength = -1,
    isLe = true,
    isNegative = false
  } = arguments.length > 1 && arguments[1] !== undefined ? arguments[1] : {};
  const valueBi = (0, _toBigInt.nToBigInt)(value);
  if (valueBi === _consts._0n) {
    return bitLength === -1 ? new Uint8Array(1) : new Uint8Array(Math.ceil((bitLength || 0) / 8));
  }
  const u8a = toU8a(valueBi, isLe, isNegative);
  if (bitLength === -1) {
    return u8a;
  }
  const byteLength = Math.ceil((bitLength || 0) / 8);
  const output = new Uint8Array(byteLength);
  if (isNegative) {
    output.fill(0xff);
  }
  output.set(u8a, isLe ? 0 : byteLength - u8a.length);
  return output;
}
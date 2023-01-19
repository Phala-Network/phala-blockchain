"use strict";

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.hexToBigInt = hexToBigInt;
var _xBigint = require("@polkadot/x-bigint");
var _toBigInt = require("../u8a/toBigInt");
var _toU8a = require("./toU8a");
// Copyright 2017-2022 @polkadot/util authors & contributors
// SPDX-License-Identifier: Apache-2.0

/**
 * @name hexToBigInt
 * @summary Creates a BigInt instance object from a hex string.
 */
function hexToBigInt(value) {
  let {
    isLe = false,
    isNegative = false
  } = arguments.length > 1 && arguments[1] !== undefined ? arguments[1] : {};
  return !value || value === '0x' ? (0, _xBigint.BigInt)(0) : (0, _toBigInt.u8aToBigInt)((0, _toU8a.hexToU8a)(value), {
    isLe,
    isNegative
  });
}
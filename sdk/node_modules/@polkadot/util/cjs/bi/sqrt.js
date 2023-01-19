"use strict";

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.SQRT_MAX_SAFE_INTEGER = void 0;
exports.nSqrt = nSqrt;
var _xBigint = require("@polkadot/x-bigint");
var _consts = require("./consts");
var _toBigInt = require("./toBigInt");
// Copyright 2017-2022 @polkadot/util authors & contributors
// SPDX-License-Identifier: Apache-2.0

/** @internal */
const SQRT_MAX_SAFE_INTEGER = (0, _xBigint.BigInt)(94906265);

/**
 * @name nSqrt
 * @summary Calculates the integer square root of a bigint
 */
exports.SQRT_MAX_SAFE_INTEGER = SQRT_MAX_SAFE_INTEGER;
function nSqrt(value) {
  const n = (0, _toBigInt.nToBigInt)(value);
  if (n < _consts._0n) {
    throw new Error('square root of negative numbers is not supported');
  }

  // https://stackoverflow.com/questions/53683995/javascript-big-integer-square-root/
  // shortcut <= 2^53 - 1 to use the JS utils
  if (n <= _consts._2pow53n) {
    // ~~ is more performant that Math.floor
    return (0, _xBigint.BigInt)(~~Math.sqrt(Number(n)));
  }

  // Use sqrt(MAX_SAFE_INTEGER) as starting point. since we already know the
  // output will be larger than this, we expect this to be a safe start
  let x0 = SQRT_MAX_SAFE_INTEGER;
  while (true) {
    const x1 = n / x0 + x0 >> _consts._1n;
    if (x0 === x1 || x0 === x1 - _consts._1n) {
      return x0;
    }
    x0 = x1;
  }
}
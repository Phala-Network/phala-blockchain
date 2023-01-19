// Copyright 2017-2022 @polkadot/util authors & contributors
// SPDX-License-Identifier: Apache-2.0

import { BigInt } from '@polkadot/x-bigint';
import { _0n, _1n, _2pow53n } from "./consts.js";
import { nToBigInt } from "./toBigInt.js";

/** @internal */
export const SQRT_MAX_SAFE_INTEGER = BigInt(94906265);

/**
 * @name nSqrt
 * @summary Calculates the integer square root of a bigint
 */
export function nSqrt(value) {
  const n = nToBigInt(value);
  if (n < _0n) {
    throw new Error('square root of negative numbers is not supported');
  }

  // https://stackoverflow.com/questions/53683995/javascript-big-integer-square-root/
  // shortcut <= 2^53 - 1 to use the JS utils
  if (n <= _2pow53n) {
    // ~~ is more performant that Math.floor
    return BigInt(~~Math.sqrt(Number(n)));
  }

  // Use sqrt(MAX_SAFE_INTEGER) as starting point. since we already know the
  // output will be larger than this, we expect this to be a safe start
  let x0 = SQRT_MAX_SAFE_INTEGER;
  while (true) {
    const x1 = n / x0 + x0 >> _1n;
    if (x0 === x1 || x0 === x1 - _1n) {
      return x0;
    }
    x0 = x1;
  }
}
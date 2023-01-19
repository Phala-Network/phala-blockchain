"use strict";

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.floatToU8a = floatToU8a;
// Copyright 2017-2022 @polkadot/util authors & contributors
// SPDX-License-Identifier: Apache-2.0

// eslint-disable-next-line @typescript-eslint/ban-types

/**
 * @name floatToU8a
 * @description Converts a float into a U8a representation (While we don't use BE in SCALE
 * we still allow for either representation, although, as elsewhere, isLe is default)
 */
function floatToU8a() {
  let value = arguments.length > 0 && arguments[0] !== undefined ? arguments[0] : 0.0;
  let {
    bitLength = 32,
    isLe = true
  } = arguments.length > 1 && arguments[1] !== undefined ? arguments[1] : {};
  if (bitLength !== 32 && bitLength !== 64) {
    throw new Error('Invalid bitLength provided, expected 32 or 64');
  }
  const result = new Uint8Array(bitLength / 8);
  const dv = new DataView(result.buffer, result.byteOffset);
  if (bitLength === 32) {
    dv.setFloat32(0, Number(value), isLe);
  } else {
    dv.setFloat64(0, Number(value), isLe);
  }
  return result;
}
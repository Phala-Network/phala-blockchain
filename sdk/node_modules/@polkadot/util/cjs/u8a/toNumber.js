"use strict";

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.u8aToNumber = u8aToNumber;
// Copyright 2017-2022 @polkadot/util authors & contributors
// SPDX-License-Identifier: Apache-2.0

/**
 * @name u8aToNumber
 * @summary Creates a number from a Uint8Array object.
 */
function u8aToNumber(value) {
  let {
    isNegative = false
  } = arguments.length > 1 && arguments[1] !== undefined ? arguments[1] : {};
  const count = value.length;
  if (isNegative) {
    let result = 0;
    switch (count) {
      case 0:
        return 0;
      case 1:
        result = value[0] ^ 0x000000ff;
        break;
      case 2:
        result = value[0] + (value[1] << 8) ^ 0x0000ffff;
        break;
      case 3:
        result = value[0] + (value[1] << 8) + (value[2] << 16) ^ 0x00ffffff;
        break;
      case 4:
        // for the 3rd byte, we don't << 24 - since JS converts all bitwise operators to
        // 32-bit, in the case where the top-most bit is set this yields a negative value
        result = value[0] + (value[1] << 8) + (value[2] << 16) + value[3] * 0x1000000 ^ 0xffffffff;
        break;
      case 5:
        result = (value[0] + (value[1] << 8) + (value[2] << 16) + value[3] * 0x1000000 ^ 0xffffffff) + (value[4] ^ 0xff) * 0x100000000;
        break;
      case 6:
        result = (value[0] + (value[1] << 8) + (value[2] << 16) + value[3] * 0x1000000 ^ 0xffffffff) + (value[4] + (value[5] << 8) ^ 0x0000ffff) * 0x100000000;
        break;
      default:
        throw new Error('Value more than 48-bits cannot be reliably converted');
    }
    return result * -1 - 1;
  }
  switch (count) {
    case 0:
      return 0;
    case 1:
      return value[0];
    case 2:
      return value[0] + (value[1] << 8);
    case 3:
      return value[0] + (value[1] << 8) + (value[2] << 16);
    case 4:
      // for the 3rd byte, we don't << 24 - since JS converts all bitwise operators to
      // 32-bit, in the case where the top-most bit is set this yields a negative value
      return value[0] + (value[1] << 8) + (value[2] << 16) + value[3] * 0x1000000;
    case 5:
      return value[0] + (value[1] << 8) + (value[2] << 16) + (value[3] + (value[4] << 8)) * 0x1000000;
    case 6:
      return value[0] + (value[1] << 8) + (value[2] << 16) + (value[3] + (value[4] << 8) + (value[5] << 16)) * 0x1000000;
    default:
      throw new Error('Value more than 48-bits cannot be reliably converted');
  }
}
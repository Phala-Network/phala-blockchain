"use strict";

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.u8aToBn = u8aToBn;
var _bn = require("../bn/bn");
// Copyright 2017-2022 @polkadot/util authors & contributors
// SPDX-License-Identifier: Apache-2.0

/**
 * @name u8aToBn
 * @summary Creates a BN from a Uint8Array object.
 * @description
 * `UInt8Array` input values return the actual BN. `null` or `undefined` values returns an `0x0` value.
 * @param value The value to convert
 * @param options Options to pass while converting
 * @param options.isLe Convert using Little Endian (default)
 * @param options.isNegative Convert using two's complement
 * @example
 * <BR>
 *
 * ```javascript
 * import { u8aToBn } from '@polkadot/util';
 *
 * u8aToHex(new Uint8Array([0x68, 0x65, 0x6c, 0x6c, 0xf])); // 0x68656c0f
 * ```
 */
function u8aToBn(value) {
  let {
    isLe = true,
    isNegative = false
  } = arguments.length > 1 && arguments[1] !== undefined ? arguments[1] : {};
  const count = value.length;

  // shortcut for <= u48 values - in this case the manual conversion
  // here seems to be more efficient than passing the full array
  if (count <= 6) {
    if (isNegative) {
      let result = 0;
      if (isLe) {
        // Most common case i{8, 16, 32} default LE SCALE-encoded
        // For <= 32, we also optimize the xor to a single op
        // (see the comments around unrolling in the next section)
        switch (count) {
          case 0:
            return new _bn.BN(0);
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
          default:
            // 6
            result = (value[0] + (value[1] << 8) + (value[2] << 16) + value[3] * 0x1000000 ^ 0xffffffff) + (value[4] + (value[5] << 8) ^ 0x0000ffff) * 0x100000000;
            break;
        }
      } else {
        for (let i = 0; i < count; i++) {
          result = result * 0x100 + (value[i] ^ 0xff);
        }
      }
      return count ? new _bn.BN(result * -1 - 1) : new _bn.BN(0);
    } else if (isLe) {
      // Most common case - u{8, 16, 32} default LE SCALE-encoded
      //
      // There are some slight benefits in unrolling this specific loop,
      // however it comes with diminishing returns since here the actual
      // `new BN` does seem to take up the bulk of the time
      switch (count) {
        case 0:
          return new _bn.BN(0);
        case 1:
          return new _bn.BN(value[0]);
        case 2:
          return new _bn.BN(value[0] + (value[1] << 8));
        case 3:
          return new _bn.BN(value[0] + (value[1] << 8) + (value[2] << 16));
        case 4:
          // for the 3rd byte, we don't << 24 - since JS converts all bitwise operators to
          // 32-bit, in the case where the top-most bit is set this yields a negative value
          return new _bn.BN(value[0] + (value[1] << 8) + (value[2] << 16) + value[3] * 0x1000000);
        case 5:
          return new _bn.BN(value[0] + (value[1] << 8) + (value[2] << 16) + (value[3] + (value[4] << 8)) * 0x1000000);
        default:
          // 6
          return new _bn.BN(value[0] + (value[1] << 8) + (value[2] << 16) + (value[3] + (value[4] << 8) + (value[5] << 16)) * 0x1000000);
      }
    } else {
      let result = 0;
      for (let i = 0; i < count; i++) {
        result = result * 0x100 + value[i];
      }
      return new _bn.BN(result);
    }
  }
  return isNegative ? new _bn.BN(value, isLe ? 'le' : 'be').fromTwos(value.length * 8) : new _bn.BN(value, isLe ? 'le' : 'be');
}
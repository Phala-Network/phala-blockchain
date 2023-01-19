"use strict";

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.numberToHex = numberToHex;
var _fixLength = require("../hex/fixLength");
// Copyright 2017-2022 @polkadot/util authors & contributors
// SPDX-License-Identifier: Apache-2.0

/**
 * @name numberToHex
 * @summary Creates a hex value from a number.
 * @description
 * `null`/`undefined`/`NaN` inputs returns an empty `0x` result. `number` input values return the actual bytes value converted to a `hex`. With `bitLength` set, it converts the number to the equivalent size.
 * @example
 * <BR>
 *
 * ```javascript
 * import { numberToHex } from '@polkadot/util';
 *
 * numberToHex(0x1234); // => '0x1234'
 * numberToHex(0x1234, 32); // => 0x00001234
 * ```
 */
function numberToHex(value) {
  let bitLength = arguments.length > 1 && arguments[1] !== undefined ? arguments[1] : -1;
  const hex = (!value || Number.isNaN(value) ? 0 : value).toString(16);
  return (0, _fixLength.hexFixLength)(hex.length % 2 ? `0${hex}` : hex, bitLength, true);
}
"use strict";

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.isAscii = isAscii;
var _toU8a = require("../u8a/toU8a");
var _hex = require("./hex");
var _string = require("./string");
// Copyright 2017-2022 @polkadot/util authors & contributors
// SPDX-License-Identifier: Apache-2.0

/** @internal */
function isAsciiStr(str) {
  const count = str.length | 0;
  for (let i = 0; i < count; i++) {
    const b = str.charCodeAt(i);

    // check is inlined here, it is faster than making a call
    if (b < 32 || b > 126) {
      return false;
    }
  }
  return true;
}

/** @internal */
function isAsciiBytes(u8a) {
  const count = u8a.length | 0;
  for (let i = 0; i < count; i++) {
    const b = u8a[i] | 0;

    // check is inlined here, it is faster than making a call
    if (b < 32 || b > 126) {
      return false;
    }
  }
  return true;
}

/**
 * @name isAscii
 * @summary Tests if the input is printable ASCII
 * @description
 * Checks to see if the input string or Uint8Array is printable ASCII, 32-127 + formatters
 */
function isAscii(value) {
  return (0, _string.isString)(value) ? (0, _hex.isHex)(value) ? isAsciiBytes((0, _toU8a.u8aToU8a)(value)) : isAsciiStr(value) : value ? isAsciiBytes(value) : false;
}
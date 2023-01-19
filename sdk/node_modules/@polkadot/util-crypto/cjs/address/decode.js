"use strict";

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.decodeAddress = decodeAddress;
var _util = require("@polkadot/util");
var _base = require("../base58");
var _checksum = require("./checksum");
var _defaults = require("./defaults");
// Copyright 2017-2022 @polkadot/util-crypto authors & contributors
// SPDX-License-Identifier: Apache-2.0

// Original implementation: https://github.com/paritytech/polka-ui/blob/4858c094684769080f5811f32b081dd7780b0880/src/polkadot.js#L6

function decodeAddress(encoded, ignoreChecksum) {
  let ss58Format = arguments.length > 2 && arguments[2] !== undefined ? arguments[2] : -1;
  if (!encoded) {
    throw new Error('Invalid empty address passed');
  }
  if ((0, _util.isU8a)(encoded) || (0, _util.isHex)(encoded)) {
    return (0, _util.u8aToU8a)(encoded);
  }
  try {
    const decoded = (0, _base.base58Decode)(encoded);
    if (!_defaults.defaults.allowedEncodedLengths.includes(decoded.length)) {
      throw new Error('Invalid decoded address length');
    }
    const [isValid, endPos, ss58Length, ss58Decoded] = (0, _checksum.checkAddressChecksum)(decoded);
    if (!isValid && !ignoreChecksum) {
      throw new Error('Invalid decoded address checksum');
    } else if (ss58Format !== -1 && ss58Format !== ss58Decoded) {
      throw new Error(`Expected ss58Format ${ss58Format}, received ${ss58Decoded}`);
    }
    return decoded.slice(ss58Length, endPos);
  } catch (error) {
    throw new Error(`Decoding ${encoded}: ${error.message}`);
  }
}
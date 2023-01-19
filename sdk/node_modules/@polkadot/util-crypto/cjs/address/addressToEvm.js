"use strict";

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.addressToEvm = addressToEvm;
var _decode = require("./decode");
// Copyright 2017-2022 @polkadot/util-crypto authors & contributors
// SPDX-License-Identifier: Apache-2.0

/**
 * @name addressToEvm
 * @summary Converts an SS58 address to its corresponding EVM address.
 */
function addressToEvm(address, ignoreChecksum) {
  return (0, _decode.decodeAddress)(address, ignoreChecksum).subarray(0, 20);
}
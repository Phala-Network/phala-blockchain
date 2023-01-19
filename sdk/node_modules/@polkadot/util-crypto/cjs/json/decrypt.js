"use strict";

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.jsonDecrypt = jsonDecrypt;
var _util = require("@polkadot/util");
var _base = require("../base64");
var _decryptData = require("./decryptData");
// Copyright 2017-2022 @polkadot/util-crypto authors & contributors
// SPDX-License-Identifier: Apache-2.0

function jsonDecrypt(_ref, passphrase) {
  let {
    encoded,
    encoding
  } = _ref;
  if (!encoded) {
    throw new Error('No encrypted data available to decode');
  }
  return (0, _decryptData.jsonDecryptData)((0, _util.isHex)(encoded) ? (0, _util.hexToU8a)(encoded) : (0, _base.base64Decode)(encoded), passphrase, Array.isArray(encoding.type) ? encoding.type : [encoding.type]);
}
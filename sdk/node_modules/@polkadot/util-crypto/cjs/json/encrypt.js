"use strict";

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.jsonEncrypt = jsonEncrypt;
var _util = require("@polkadot/util");
var _nacl = require("../nacl");
var _scrypt = require("../scrypt");
var _encryptFormat = require("./encryptFormat");
// Copyright 2017-2022 @polkadot/util-crypto authors & contributors
// SPDX-License-Identifier: Apache-2.0

function jsonEncrypt(data, contentType, passphrase) {
  let isEncrypted = false;
  let encoded = data;
  if (passphrase) {
    const {
      params,
      password,
      salt
    } = (0, _scrypt.scryptEncode)(passphrase);
    const {
      encrypted,
      nonce
    } = (0, _nacl.naclEncrypt)(encoded, password.subarray(0, 32));
    isEncrypted = true;
    encoded = (0, _util.u8aConcat)((0, _scrypt.scryptToU8a)(salt, params), nonce, encrypted);
  }
  return (0, _encryptFormat.jsonEncryptFormat)(encoded, contentType, isEncrypted);
}
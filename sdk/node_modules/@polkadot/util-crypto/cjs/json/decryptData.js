"use strict";

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.jsonDecryptData = jsonDecryptData;
var _util = require("@polkadot/util");
var _nacl = require("../nacl");
var _scrypt = require("../scrypt");
var _constants = require("./constants");
// Copyright 2017-2022 @polkadot/util-crypto authors & contributors
// SPDX-License-Identifier: Apache-2.0

function jsonDecryptData(encrypted, passphrase) {
  let encType = arguments.length > 2 && arguments[2] !== undefined ? arguments[2] : _constants.ENCODING;
  if (!encrypted) {
    throw new Error('No encrypted data available to decode');
  } else if (encType.includes('xsalsa20-poly1305') && !passphrase) {
    throw new Error('Password required to decode encrypted data');
  }
  let encoded = encrypted;
  if (passphrase) {
    let password;
    if (encType.includes('scrypt')) {
      const {
        params,
        salt
      } = (0, _scrypt.scryptFromU8a)(encrypted);
      password = (0, _scrypt.scryptEncode)(passphrase, salt, params).password;
      encrypted = encrypted.subarray(_constants.SCRYPT_LENGTH);
    } else {
      password = (0, _util.stringToU8a)(passphrase);
    }
    encoded = (0, _nacl.naclDecrypt)(encrypted.subarray(_constants.NONCE_LENGTH), encrypted.subarray(0, _constants.NONCE_LENGTH), (0, _util.u8aFixLength)(password, 256, true));
  }
  if (!encoded) {
    throw new Error('Unable to decode using the supplied passphrase');
  }
  return encoded;
}
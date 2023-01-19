"use strict";

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.deriveAddress = deriveAddress;
var _key = require("../key");
var _sr = require("../sr25519");
var _decode = require("./decode");
var _encode = require("./encode");
// Copyright 2017-2022 @polkadot/util-crypto authors & contributors
// SPDX-License-Identifier: Apache-2.0

function filterHard(_ref) {
  let {
    isHard
  } = _ref;
  return isHard;
}

/**
 * @name deriveAddress
 * @summary Creates a sr25519 derived address from the supplied and path.
 * @description
 * Creates a sr25519 derived address based on the input address/publicKey and the uri supplied.
 */
function deriveAddress(who, suri, ss58Format) {
  const {
    path
  } = (0, _key.keyExtractPath)(suri);
  if (!path.length || path.every(filterHard)) {
    throw new Error('Expected suri to contain a combination of non-hard paths');
  }
  let publicKey = (0, _decode.decodeAddress)(who);
  for (const {
    chainCode
  } of path) {
    publicKey = (0, _sr.sr25519DerivePublic)(publicKey, chainCode);
  }
  return (0, _encode.encodeAddress)(publicKey, ss58Format);
}
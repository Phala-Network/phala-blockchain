"use strict";

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.cryptoIsReady = void 0;
exports.cryptoWaitReady = cryptoWaitReady;
var _wasmCrypto = require("@polkadot/wasm-crypto");
// Copyright 2017-2022 @polkadot/util-crypto authors & contributors
// SPDX-License-Identifier: Apache-2.0

const cryptoIsReady = _wasmCrypto.isReady;
exports.cryptoIsReady = cryptoIsReady;
function cryptoWaitReady() {
  return (0, _wasmCrypto.waitReady)().then(() => {
    if (!(0, _wasmCrypto.isReady)()) {
      throw new Error('Unable to initialize @polkadot/util-crypto');
    }
    return true;
  }).catch(() => false);
}
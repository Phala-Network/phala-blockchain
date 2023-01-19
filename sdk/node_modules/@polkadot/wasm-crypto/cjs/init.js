"use strict";

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.bridge = void 0;
exports.initBridge = initBridge;

var _wasmBridge = require("@polkadot/wasm-bridge");

var _wasmCryptoInit = require("@polkadot/wasm-crypto-init");

// Copyright 2019-2022 @polkadot/wasm-crypto authors & contributors
// SPDX-License-Identifier: Apache-2.0

/**
 * @name bridge
 * @description
 * The JS <-> WASM bridge that is in operation. For the specific package
 * it is a global, i.e. all operations happens on this specific bridge
 */
const bridge = new _wasmBridge.Bridge(_wasmCryptoInit.createWasm);
/**
 * @name initBridge
 * @description
 * Creates a new bridge interface with the (optional) initialization function
 */

exports.bridge = bridge;

async function initBridge(createWasm) {
  return bridge.init(createWasm);
}
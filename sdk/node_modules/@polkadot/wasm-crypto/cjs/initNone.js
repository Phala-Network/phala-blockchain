"use strict";

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.initWasm = initWasm;

var _none = require("@polkadot/wasm-crypto-init/cjs/none");

var _init = require("./init");

// Copyright 2019-2022 @polkadot/wasm-crypto authors & contributors
// SPDX-License-Identifier: Apache-2.0

/**
 * @name initWasm
 * @description
 * For historic purposes and for tighter control on init, specifically performing
 * a WASM initialization with no interface whatsoever (no WASM, no ASM.js)
 *
 * Generally should not be used unless you want explicit control over which
 * interfaces are initialized.
 */
async function initWasm() {
  await (0, _init.initBridge)(_none.createWasm);
}

initWasm().catch(() => {// cannot happen, initWasm doesn't throw
});
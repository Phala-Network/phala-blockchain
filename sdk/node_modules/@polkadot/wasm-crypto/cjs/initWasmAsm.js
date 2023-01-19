"use strict";

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.initWasm = initWasm;

var _both = require("@polkadot/wasm-crypto-init/cjs/both");

var _init = require("./init");

// Copyright 2019-2022 @polkadot/wasm-crypto authors & contributors
// SPDX-License-Identifier: Apache-2.0

/**
 * @name initWasm
 * @description
 * For historic purposes and for tighter control on init, specifically performing
 * a WASM initialization with ASM and an ASM.js fallback
 *
 * Generally should not be used unless you want explicit control over which
 * interfaces are initialized.
 */
async function initWasm() {
  await (0, _init.initBridge)(_both.createWasm);
}

initWasm().catch(() => {// cannot happen, initWasm doesn't throw
});
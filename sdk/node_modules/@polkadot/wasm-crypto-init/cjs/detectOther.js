"use strict";

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.default = void 0;

var _packageInfo = require("@polkadot/wasm-bridge/cjs/packageInfo");

var _packageInfo2 = require("@polkadot/wasm-crypto-asmjs/cjs/packageInfo");

var _packageInfo3 = require("@polkadot/wasm-crypto-wasm/cjs/packageInfo");

// Copyright 2017-2022 @polkadot/wasm-crypto-init authors & contributors
// SPDX-License-Identifier: Apache-2.0
var _default = [_packageInfo.packageInfo, _packageInfo2.packageInfo, _packageInfo3.packageInfo];
exports.default = _default;
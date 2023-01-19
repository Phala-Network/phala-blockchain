"use strict";

Object.defineProperty(exports, "__esModule", {
  value: true
});
Object.defineProperty(exports, "packageInfo", {
  enumerable: true,
  get: function () {
    return _packageInfo.packageInfo;
  }
});
exports.wasmBytes = void 0;

var _wasmUtil = require("@polkadot/wasm-util");

var _bytes = require("./cjs/bytes.js");

var _packageInfo = require("./packageInfo");

// Copyright 2019-2022 @polkadot/wasm-crypto-wasm authors & contributors
// SPDX-License-Identifier: Apache-2.0

/**
 * @name wasmBytes
 * @description
 * The decoded WASM interface as exposed by this package.
 *
 * The build process will output into cjs/* into a compressed base64 format.
 * Upon loading the exposed bytes will be decoded and decompressed form this
 * specific format and returned.
 */
const wasmBytes = (0, _wasmUtil.unzlibSync)((0, _wasmUtil.base64Decode)(_bytes.bytes, new Uint8Array(_bytes.lenIn)), new Uint8Array(_bytes.lenOut));
exports.wasmBytes = wasmBytes;
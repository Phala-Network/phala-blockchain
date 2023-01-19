"use strict";

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.hasWasm = exports.hasProcess = exports.hasEsm = exports.hasDirname = exports.hasCjs = exports.hasBuffer = exports.hasBigInt = void 0;
var _xBigint = require("@polkadot/x-bigint");
// Copyright 2017-2022 @polkadot/util authors & contributors
// SPDX-License-Identifier: Apache-2.0

/** true if the environment has proper BigInt support */
const hasBigInt = typeof _xBigint.BigInt === 'function' && typeof _xBigint.BigInt.asIntN === 'function';

/** true if the environment has support for Buffer */
exports.hasBigInt = hasBigInt;
const hasBuffer = typeof Buffer !== 'undefined';

/** true if the environment is CJS */
exports.hasBuffer = hasBuffer;
const hasCjs = typeof require === 'function' && typeof module !== 'undefined';

/** true if the environment has __dirname available */
exports.hasCjs = hasCjs;
const hasDirname = typeof __dirname !== 'undefined';

/** true if the environment is ESM */
exports.hasDirname = hasDirname;
const hasEsm = !hasCjs;

/** true if the environment has process available (typically Node.js) */
exports.hasEsm = hasEsm;
const hasProcess = typeof process === 'object';

/** true if the environment has WebAssembly available */
exports.hasProcess = hasProcess;
const hasWasm = typeof WebAssembly !== 'undefined';
exports.hasWasm = hasWasm;
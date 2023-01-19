"use strict";

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.exposeGlobal = exposeGlobal;
exports.extractGlobal = extractGlobal;
Object.defineProperty(exports, "packageInfo", {
  enumerable: true,
  get: function () {
    return _packageInfo.packageInfo;
  }
});
exports.xglobal = void 0;
var _packageInfo = require("./packageInfo");
// Copyright 2017-2022 @polkadot/x-global authors & contributors
// SPDX-License-Identifier: Apache-2.0

function evaluateThis(fn) {
  return fn('return this');
}
const xglobal = typeof globalThis !== 'undefined' ? globalThis : typeof global !== 'undefined' ? global : typeof self !== 'undefined' ? self : typeof window !== 'undefined' ? window : evaluateThis(Function);
exports.xglobal = xglobal;
function extractGlobal(name, fallback) {
  // Not quite sure why this is here - snuck in with TS 4.7.2 with no real idea
  // (as of now) as to why this looks like an "any" when we do cast it to a T
  //
  // eslint-disable-next-line @typescript-eslint/no-unsafe-return
  return typeof xglobal[name] === 'undefined' ? fallback : xglobal[name];
}
function exposeGlobal(name, fallback) {
  if (typeof xglobal[name] === 'undefined') {
    xglobal[name] = fallback;
  }
}
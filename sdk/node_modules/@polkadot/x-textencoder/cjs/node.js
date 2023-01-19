"use strict";

var _interopRequireDefault = require("@babel/runtime/helpers/interopRequireDefault");
Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.TextEncoder = void 0;
Object.defineProperty(exports, "packageInfo", {
  enumerable: true,
  get: function () {
    return _packageInfo.packageInfo;
  }
});
var _util = _interopRequireDefault(require("util"));
var _xGlobal = require("@polkadot/x-global");
var _packageInfo = require("./packageInfo");
// Copyright 2017-2022 @polkadot/x-textencoder authors & contributors
// SPDX-License-Identifier: Apache-2.0

class Fallback {
  #encoder;
  constructor() {
    this.#encoder = new _util.default.TextEncoder();
  }

  // For a Jest 26.0.1 environment, Buffer !== Uint8Array
  encode(value) {
    return Uint8Array.from(this.#encoder.encode(value));
  }
}
const TextEncoder = (0, _xGlobal.extractGlobal)('TextEncoder', Fallback);
exports.TextEncoder = TextEncoder;
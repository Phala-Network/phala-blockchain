"use strict";

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.Pairs = void 0;
var _util = require("@polkadot/util");
var _utilCrypto = require("@polkadot/util-crypto");
// Copyright 2017-2022 @polkadot/keyring authors & contributors
// SPDX-License-Identifier: Apache-2.0

class Pairs {
  #map = {};
  add(pair) {
    this.#map[(0, _utilCrypto.decodeAddress)(pair.address).toString()] = pair;
    return pair;
  }
  all() {
    return Object.values(this.#map);
  }
  get(address) {
    const pair = this.#map[(0, _utilCrypto.decodeAddress)(address).toString()];
    if (!pair) {
      throw new Error(`Unable to retrieve keypair '${(0, _util.isU8a)(address) || (0, _util.isHex)(address) ? (0, _util.u8aToHex)((0, _util.u8aToU8a)(address)) : address}'`);
    }
    return pair;
  }
  remove(address) {
    delete this.#map[(0, _utilCrypto.decodeAddress)(address).toString()];
  }
}
exports.Pairs = Pairs;
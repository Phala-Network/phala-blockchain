"use strict";

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.DeriveJunction = void 0;
var _util = require("@polkadot/util");
var _asU8a = require("../blake2/asU8a");
var _bn = require("../bn");
// Copyright 2017-2022 @polkadot/util-crypto authors & contributors
// SPDX-License-Identifier: Apache-2.0

const RE_NUMBER = /^\d+$/;
const JUNCTION_ID_LEN = 32;
class DeriveJunction {
  #chainCode = new Uint8Array(32);
  #isHard = false;
  static from(value) {
    const result = new DeriveJunction();
    const [code, isHard] = value.startsWith('/') ? [value.substring(1), true] : [value, false];
    result.soft(RE_NUMBER.test(code) ? new _util.BN(code, 10) : code);
    return isHard ? result.harden() : result;
  }
  get chainCode() {
    return this.#chainCode;
  }
  get isHard() {
    return this.#isHard;
  }
  get isSoft() {
    return !this.#isHard;
  }
  hard(value) {
    return this.soft(value).harden();
  }
  harden() {
    this.#isHard = true;
    return this;
  }
  soft(value) {
    if ((0, _util.isNumber)(value) || (0, _util.isBn)(value) || (0, _util.isBigInt)(value)) {
      return this.soft((0, _util.bnToU8a)(value, _bn.BN_LE_256_OPTS));
    } else if ((0, _util.isHex)(value)) {
      return this.soft((0, _util.hexToU8a)(value));
    } else if ((0, _util.isString)(value)) {
      return this.soft((0, _util.compactAddLength)((0, _util.stringToU8a)(value)));
    } else if (value.length > JUNCTION_ID_LEN) {
      return this.soft((0, _asU8a.blake2AsU8a)(value));
    }
    this.#chainCode.fill(0);
    this.#chainCode.set(value, 0);
    return this;
  }
  soften() {
    this.#isHard = false;
    return this;
  }
}
exports.DeriveJunction = DeriveJunction;
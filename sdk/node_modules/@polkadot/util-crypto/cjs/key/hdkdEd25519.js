"use strict";

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.keyHdkdEd25519 = void 0;
var _ed = require("../ed25519");
var _hdkdDerive = require("./hdkdDerive");
// Copyright 2017-2022 @polkadot/util-crypto authors & contributors
// SPDX-License-Identifier: Apache-2.0

const keyHdkdEd25519 = (0, _hdkdDerive.createSeedDeriveFn)(_ed.ed25519PairFromSeed, _ed.ed25519DeriveHard);
exports.keyHdkdEd25519 = keyHdkdEd25519;
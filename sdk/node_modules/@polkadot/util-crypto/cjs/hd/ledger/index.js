"use strict";

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.hdLedger = hdLedger;
var _ed = require("../../ed25519");
var _mnemonic2 = require("../../mnemonic");
var _validatePath = require("../validatePath");
var _derivePrivate = require("./derivePrivate");
var _master = require("./master");
// Copyright 2017-2022 @polkadot/util-crypto authors & contributors
// SPDX-License-Identifier: Apache-2.0

function hdLedger(_mnemonic, path) {
  const words = _mnemonic.split(' ').map(s => s.trim()).filter(s => s);
  if (![12, 24, 25].includes(words.length)) {
    throw new Error('Expected a mnemonic with 24 words (or 25 including a password)');
  }
  const [mnemonic, password] = words.length === 25 ? [words.slice(0, 24).join(' '), words[24]] : [words.join(' '), ''];
  if (!(0, _mnemonic2.mnemonicValidate)(mnemonic)) {
    throw new Error('Invalid mnemonic passed to ledger derivation');
  } else if (!(0, _validatePath.hdValidatePath)(path)) {
    throw new Error('Invalid derivation path');
  }
  const parts = path.split('/').slice(1);
  let seed = (0, _master.ledgerMaster)(mnemonic, password);
  for (const p of parts) {
    const n = parseInt(p.replace(/'$/, ''), 10);
    seed = (0, _derivePrivate.ledgerDerivePrivate)(seed, n < _validatePath.HARDENED ? n + _validatePath.HARDENED : n);
  }
  return (0, _ed.ed25519PairFromSeed)(seed.slice(0, 32));
}
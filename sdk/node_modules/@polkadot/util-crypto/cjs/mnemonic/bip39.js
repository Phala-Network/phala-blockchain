"use strict";

var _interopRequireDefault = require("@babel/runtime/helpers/interopRequireDefault");
Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.entropyToMnemonic = entropyToMnemonic;
exports.generateMnemonic = generateMnemonic;
exports.mnemonicToEntropy = mnemonicToEntropy;
exports.mnemonicToSeedSync = mnemonicToSeedSync;
exports.validateMnemonic = validateMnemonic;
var _util = require("@polkadot/util");
var _pbkdf = require("../pbkdf2");
var _random = require("../random");
var _sha = require("../sha");
var _bip39En = _interopRequireDefault(require("./bip39-en"));
// Copyright 2017-2022 @polkadot/util-crypto authors & contributors
// SPDX-License-Identifier: Apache-2.0

// Adapted from the bitcoinjs/bip39 source
// https://github.com/bitcoinjs/bip39/blob/1d063b6a6aee4145b34d701037cd3e67f5446ff9/ts_src/index.ts
// Copyright (c) 2014, Wei Lu <luwei.here@gmail.com> and Daniel Cousens <email@dcousens.com>
// ISC Licence
//
// Change made in this version -
//   - Adjust formatting (just eslint differences)
//   - Only English wordlist (this aligns with the wasm-crypto implementation)
//   - Use util-crypto randomAsU8a (instead of randombytes)
//   - Remove setting of wordlist passing of wordlist in functions
//   - Remove mnemonicToSeed (we only use the sync variant)
//   - generateMnemonic takes number of words (instead of strength)

const INVALID_MNEMONIC = 'Invalid mnemonic';
const INVALID_ENTROPY = 'Invalid entropy';
const INVALID_CHECKSUM = 'Invalid mnemonic checksum';
function normalize(str) {
  return (str || '').normalize('NFKD');
}
function binaryToByte(bin) {
  return parseInt(bin, 2);
}
function bytesToBinary(bytes) {
  return bytes.map(x => x.toString(2).padStart(8, '0')).join('');
}
function deriveChecksumBits(entropyBuffer) {
  return bytesToBinary(Array.from((0, _sha.sha256AsU8a)(entropyBuffer))).slice(0, entropyBuffer.length * 8 / 32);
}
function mnemonicToSeedSync(mnemonic, password) {
  return (0, _pbkdf.pbkdf2Encode)((0, _util.stringToU8a)(normalize(mnemonic)), (0, _util.stringToU8a)(`mnemonic${normalize(password)}`)).password;
}
function mnemonicToEntropy(mnemonic) {
  const words = normalize(mnemonic).split(' ');
  if (words.length % 3 !== 0) {
    throw new Error(INVALID_MNEMONIC);
  }

  // convert word indices to 11 bit binary strings
  const bits = words.map(word => {
    const index = _bip39En.default.indexOf(word);
    if (index === -1) {
      throw new Error(INVALID_MNEMONIC);
    }
    return index.toString(2).padStart(11, '0');
  }).join('');

  // split the binary string into ENT/CS
  const dividerIndex = Math.floor(bits.length / 33) * 32;
  const entropyBits = bits.slice(0, dividerIndex);
  const checksumBits = bits.slice(dividerIndex);

  // calculate the checksum and compare
  const matched = entropyBits.match(/(.{1,8})/g);
  const entropyBytes = matched && matched.map(binaryToByte);
  if (!entropyBytes || entropyBytes.length % 4 !== 0 || entropyBytes.length < 16 || entropyBytes.length > 32) {
    throw new Error(INVALID_ENTROPY);
  }
  const entropy = (0, _util.u8aToU8a)(entropyBytes);
  if (deriveChecksumBits(entropy) !== checksumBits) {
    throw new Error(INVALID_CHECKSUM);
  }
  return entropy;
}
function entropyToMnemonic(entropy) {
  // 128 <= ENT <= 256
  if (entropy.length % 4 !== 0 || entropy.length < 16 || entropy.length > 32) {
    throw new Error(INVALID_ENTROPY);
  }
  const matched = `${bytesToBinary(Array.from(entropy))}${deriveChecksumBits(entropy)}`.match(/(.{1,11})/g);
  const mapped = matched && matched.map(binary => _bip39En.default[binaryToByte(binary)]);
  if (!mapped || mapped.length < 12) {
    throw new Error('Unable to map entropy to mnemonic');
  }
  return mapped.join(' ');
}
function generateMnemonic(numWords) {
  return entropyToMnemonic((0, _random.randomAsU8a)(numWords / 3 * 4));
}
function validateMnemonic(mnemonic) {
  try {
    mnemonicToEntropy(mnemonic);
  } catch (e) {
    return false;
  }
  return true;
}
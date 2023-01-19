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

import { stringToU8a, u8aToU8a } from '@polkadot/util';
import { pbkdf2Encode } from "../pbkdf2/index.js";
import { randomAsU8a } from "../random/index.js";
import { sha256AsU8a } from "../sha/index.js";
import DEFAULT_WORDLIST from "./bip39-en.js";
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
  return bytesToBinary(Array.from(sha256AsU8a(entropyBuffer))).slice(0, entropyBuffer.length * 8 / 32);
}
export function mnemonicToSeedSync(mnemonic, password) {
  return pbkdf2Encode(stringToU8a(normalize(mnemonic)), stringToU8a(`mnemonic${normalize(password)}`)).password;
}
export function mnemonicToEntropy(mnemonic) {
  const words = normalize(mnemonic).split(' ');
  if (words.length % 3 !== 0) {
    throw new Error(INVALID_MNEMONIC);
  }

  // convert word indices to 11 bit binary strings
  const bits = words.map(word => {
    const index = DEFAULT_WORDLIST.indexOf(word);
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
  const entropy = u8aToU8a(entropyBytes);
  if (deriveChecksumBits(entropy) !== checksumBits) {
    throw new Error(INVALID_CHECKSUM);
  }
  return entropy;
}
export function entropyToMnemonic(entropy) {
  // 128 <= ENT <= 256
  if (entropy.length % 4 !== 0 || entropy.length < 16 || entropy.length > 32) {
    throw new Error(INVALID_ENTROPY);
  }
  const matched = `${bytesToBinary(Array.from(entropy))}${deriveChecksumBits(entropy)}`.match(/(.{1,11})/g);
  const mapped = matched && matched.map(binary => DEFAULT_WORDLIST[binaryToByte(binary)]);
  if (!mapped || mapped.length < 12) {
    throw new Error('Unable to map entropy to mnemonic');
  }
  return mapped.join(' ');
}
export function generateMnemonic(numWords) {
  return entropyToMnemonic(randomAsU8a(numWords / 3 * 4));
}
export function validateMnemonic(mnemonic) {
  try {
    mnemonicToEntropy(mnemonic);
  } catch (e) {
    return false;
  }
  return true;
}
// Copyright 2017-2022 @polkadot/util-crypto authors & contributors
// SPDX-License-Identifier: Apache-2.0

import { hasBigInt } from '@polkadot/util';
import { bip39Generate, isReady } from '@polkadot/wasm-crypto';
import { generateMnemonic } from "./bip39.js";

/**
 * @name mnemonicGenerate
 * @summary Creates a valid mnemonic string using using [BIP39](https://github.com/bitcoin/bips/blob/master/bip-0039.mediawiki).
 * @example
 * <BR>
 *
 * ```javascript
 * import { mnemonicGenerate } from '@polkadot/util-crypto';
 *
 * const mnemonic = mnemonicGenerate(); // => string
 * ```
 */
export function mnemonicGenerate(numWords = 12, onlyJs) {
  return !hasBigInt || !onlyJs && isReady() ? bip39Generate(numWords) : generateMnemonic(numWords);
}
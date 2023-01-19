// Copyright 2017-2022 @polkadot/util-crypto authors & contributors
// SPDX-License-Identifier: Apache-2.0

import { u8aToU8a } from '@polkadot/util';
import { sr25519Sign as wasmSign } from '@polkadot/wasm-crypto';

/**
 * @name sr25519Sign
 * @description Returns message signature of `message`, using the supplied pair
 */
export function sr25519Sign(message, {
  publicKey,
  secretKey
}) {
  if ((publicKey == null ? void 0 : publicKey.length) !== 32) {
    throw new Error('Expected a valid publicKey, 32-bytes');
  } else if ((secretKey == null ? void 0 : secretKey.length) !== 64) {
    throw new Error('Expected a valid secretKey, 64-bytes');
  }
  return wasmSign(publicKey, secretKey, u8aToU8a(message));
}
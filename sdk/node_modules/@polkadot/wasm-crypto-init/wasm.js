// Copyright 2019-2022 @polkadot/wasm-crypto-init authors & contributors
// SPDX-License-Identifier: Apache-2.0
import { createWasmFn } from '@polkadot/wasm-bridge';
import { wasmBytes } from '@polkadot/wasm-crypto-wasm';
export { packageInfo } from "./packageInfo.js";
/**
 * @name createWasm
 * @description
 * Creates an interface using only WASM
 */

export const createWasm = createWasmFn('crypto', wasmBytes, null);
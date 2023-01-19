// Copyright 2019-2022 @polkadot/wasm-crypto-init authors & contributors
// SPDX-License-Identifier: Apache-2.0
import { createWasmFn } from '@polkadot/wasm-bridge';
export { packageInfo } from "./packageInfo.js";
/**
 * @name createWasm
 * @description
 * Creates an interface using no WASM and no ASM.js
 */

export const createWasm = createWasmFn('crypto', null, null);
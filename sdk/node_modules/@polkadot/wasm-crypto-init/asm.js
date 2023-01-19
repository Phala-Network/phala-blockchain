// Copyright 2019-2022 @polkadot/wasm-crypto-init authors & contributors
// SPDX-License-Identifier: Apache-2.0
import { createWasmFn } from '@polkadot/wasm-bridge';
import { asmJsInit } from '@polkadot/wasm-crypto-asmjs';
export { packageInfo } from "./packageInfo.js";
/**
 * @name createWasm
 * @description
 * Creates an interface using only ASM.js
 */

export const createWasm = createWasmFn('crypto', null, asmJsInit);
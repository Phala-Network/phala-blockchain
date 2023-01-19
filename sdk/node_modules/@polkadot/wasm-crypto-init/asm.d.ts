import type { InitFn } from '@polkadot/wasm-bridge/types';
import type { WasmCryptoInstance } from './types';
export { packageInfo } from './packageInfo';
/**
 * @name createWasm
 * @description
 * Creates an interface using only ASM.js
 */
export declare const createWasm: InitFn<WasmCryptoInstance>;

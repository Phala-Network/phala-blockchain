import type { InitFn, WasmBaseInstance, WasmImports } from './types';
/**
 * @name createWasmFn
 * @description
 * Create a WASM (or ASM.js) creator interface based on the supplied information.
 *
 * It will attempt to create a WASM interface first and if this fails or is not available in
 * the environment, will fallback to attempting to create an ASM.js interface.
 */
export declare function createWasmFn<C extends WasmBaseInstance>(root: string, wasmBytes: null | Uint8Array, asmFn: null | ((wbg: WasmImports) => C)): InitFn<C>;

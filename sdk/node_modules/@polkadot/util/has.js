// Copyright 2017-2022 @polkadot/util authors & contributors
// SPDX-License-Identifier: Apache-2.0

import { BigInt } from '@polkadot/x-bigint';

// Since we run in very different environments, we have to ensure we have all
// the types used here for detection (some of these may require Node definitions,
// which are not available in Deno/browser)

/** true if the environment has proper BigInt support */
export const hasBigInt = typeof BigInt === 'function' && typeof BigInt.asIntN === 'function';

/** true if the environment has support for Buffer */
export const hasBuffer = typeof Buffer !== 'undefined';

/** true if the environment is CJS */
export const hasCjs = typeof require === 'function' && typeof module !== 'undefined';

/** true if the environment has __dirname available */
export const hasDirname = typeof __dirname !== 'undefined';

/** true if the environment is ESM */
export const hasEsm = !hasCjs;

/** true if the environment has process available (typically Node.js) */
export const hasProcess = typeof process === 'object';

/** true if the environment has WebAssembly available */
export const hasWasm = typeof WebAssembly !== 'undefined';
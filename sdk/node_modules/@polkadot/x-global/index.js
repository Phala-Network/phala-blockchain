// Copyright 2017-2022 @polkadot/x-global authors & contributors
// SPDX-License-Identifier: Apache-2.0

export { packageInfo } from "./packageInfo.js";

// Ensure that we are able to run this without any @types/node definitions
// and without having lib: ['dom'] in our TypeScript configuration
// (may not be available in all environments, e.g. Deno springs to mind)

function evaluateThis(fn) {
  return fn('return this');
}
export const xglobal = typeof globalThis !== 'undefined' ? globalThis : typeof global !== 'undefined' ? global : typeof self !== 'undefined' ? self : typeof window !== 'undefined' ? window : evaluateThis(Function);
export function extractGlobal(name, fallback) {
  // Not quite sure why this is here - snuck in with TS 4.7.2 with no real idea
  // (as of now) as to why this looks like an "any" when we do cast it to a T
  //
  // eslint-disable-next-line @typescript-eslint/no-unsafe-return
  return typeof xglobal[name] === 'undefined' ? fallback : xglobal[name];
}
export function exposeGlobal(name, fallback) {
  if (typeof xglobal[name] === 'undefined') {
    xglobal[name] = fallback;
  }
}
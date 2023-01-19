import type { U8aLike } from '../types';
/** @internal */
export declare const U8A_WRAP_ETHEREUM: Uint8Array;
/** @internal */
export declare const U8A_WRAP_PREFIX: Uint8Array;
/** @internal */
export declare const U8A_WRAP_POSTFIX: Uint8Array;
/** @internal */
export declare function u8aIsWrapped(u8a: Uint8Array, withEthereum: boolean): boolean;
/**
 * @name u8aUnwrapBytes
 * @description Removes all <Bytes>...</Bytes> wrappers from the supplied value
 */
export declare function u8aUnwrapBytes(bytes: U8aLike): Uint8Array;
/**
 * @name u8aWrapBytes
 * @description Adds a <Bytes>...</Bytes> wrapper to the supplied value (if not already existing)
 */
export declare function u8aWrapBytes(bytes: U8aLike): Uint8Array;

/// <reference types="bn.js" />
import type { BN } from '../bn/bn';
import type { HexString, NumberOptions, ToBigInt, ToBn } from '../types';
/**
 * @name nToHex
 * @summary Creates a hex value from a bigint object.
 */
export declare function nToHex<ExtToBn extends ToBn | ToBigInt>(value?: ExtToBn | BN | bigint | number | null, { bitLength, isLe, isNegative }?: NumberOptions): HexString;

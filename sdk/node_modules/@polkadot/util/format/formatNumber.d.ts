/// <reference types="bn.js" />
import type { BN } from '../bn/bn';
import type { ToBn } from '../types';
/**
 * @name formatNumber
 * @description Formats a number into string format with thousand seperators
 */
export declare function formatNumber<ExtToBn extends ToBn>(value?: ExtToBn | BN | bigint | number | null): string;

/// <reference types="bn.js" />
import type { BN } from '../bn/bn';
import type { SiDef, ToBn } from '../types';
interface Defaults {
    decimals: number;
    unit: string;
}
interface SetDefaults {
    decimals?: number[] | number;
    unit?: string[] | string;
}
interface Options {
    /**
     * @description The number of decimals
     */
    decimals?: number;
    /**
     * @description Format the number with this specific unit
     */
    forceUnit?: string;
    /**
     * @description Format with SI, i.e. m/M/etc.
     */
    withSi?: boolean;
    /**
     * @description Format with full SI, i.e. mili/Mega/etc.
     */
    withSiFull?: boolean;
    /**
     * @description Add the unit (useful in Balance formats)
     */
    withUnit?: boolean | string;
}
interface BalanceFormatter {
    <ExtToBn extends ToBn>(input?: number | string | BN | bigint | ExtToBn, options?: Options): string;
    calcSi(text: string, decimals?: number): SiDef;
    findSi(type: string): SiDef;
    getDefaults(): Defaults;
    getOptions(decimals?: number): SiDef[];
    setDefaults(defaults: SetDefaults): void;
}
export declare const formatBalance: BalanceFormatter;
export {};

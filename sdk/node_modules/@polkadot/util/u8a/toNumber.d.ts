interface ToNumberOptions {
    /**
     * @description Number is signed, apply two's complement
     */
    isNegative?: boolean;
}
/**
 * @name u8aToNumber
 * @summary Creates a number from a Uint8Array object.
 */
export declare function u8aToNumber(value: Uint8Array, { isNegative }?: ToNumberOptions): number;
export {};

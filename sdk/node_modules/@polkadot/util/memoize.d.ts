import type { Memoized } from './types';
interface Options {
    getInstanceId?: () => string;
}
/**
 * @name memoize
 * @description Memomize the function with a specific instanceId
 */
export declare function memoize<T, F extends (...args: any[]) => T>(fn: F, { getInstanceId }?: Options): Memoized<F>;
export {};

/// <reference types="node" />
import type { HexString } from '@polkadot/util/types';
export type { HexString } from '@polkadot/util/types';
interface DualHash {
    256: (u8a: Uint8Array) => Uint8Array;
    512: (u8a: Uint8Array) => Uint8Array;
}
/** @internal */
export declare function createAsHex<T extends (...args: never[]) => Uint8Array>(fn: T): (...args: Parameters<T>) => HexString;
/** @internal */
export declare function createBitHasher(bitLength: 256 | 512, fn: (data: HexString | Buffer | Uint8Array | string, bitLength: 256 | 512, onlyJs?: boolean) => Uint8Array): (data: HexString | Buffer | Uint8Array | string, onlyJs?: boolean) => Uint8Array;
/** @internal */
export declare function createDualHasher(wa: DualHash, js: DualHash): (value: HexString | Buffer | Uint8Array | string, bitLength?: 256 | 512, onlyJs?: boolean) => Uint8Array;

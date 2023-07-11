import type { bool, Result } from '@polkadot/types';
import type { DecorateMethod } from '@polkadot/api/types';
import type { ISubmittableResult } from '@polkadot/types/types';
import type { KeyringPair } from '@polkadot/keyring/types';
import type { AbiConstructor } from '@polkadot/api-contract/types';
import type { MapConstructorExec } from '@polkadot/api-contract/base/types';

import type { OnChainRegistry } from '../OnChainRegistry';
import type { AbiLike, WasmLike } from '../types';
import type { CertificateData } from '../certificate';

import { SubmittableResult, toPromiseMethod } from '@polkadot/api';
import { ApiBase } from '@polkadot/api/base';
import { Abi } from '@polkadot/api-contract/Abi';
import { createBluePrintTx } from '@polkadot/api-contract/base/util';
import { isUndefined, isWasm, u8aToU8a } from '@polkadot/util';

import { PinkBlueprintPromise } from './PinkBlueprint';


export class InkCodeSubmittableResult extends SubmittableResult {
  readonly registry: OnChainRegistry;
  readonly abi: Abi;
  readonly blueprint?: PinkBlueprintPromise;
  
  #isFinalized: boolean = false

  constructor (result: ISubmittableResult, abi: Abi, registry: OnChainRegistry) {
    super(result);

    this.registry = registry;
    this.abi = abi;

    this.blueprint = new PinkBlueprintPromise(this.registry.api, this.registry, this.abi, this.abi.info.source.wasmHash)
  }

  async waitFinalized(pair: KeyringPair, cert: CertificateData, timeout: number = 10_000) {
    if (this.#isFinalized) {
      return
    }
    if (this.isInBlock || this.isFinalized) {
      const system = this.registry.systemContract!;
      const codeHash = this.abi.info.source.wasmHash.toString();
      const t0 = new Date().getTime();
      while (true) {
        const { output } = await system.query['system::codeExists'](pair.address, { cert }, codeHash, 'Ink')
        if (output && (output as Result<bool, any>).asOk.toPrimitive()) {
          this.#isFinalized = true;
          return
        }
        const t1 = new Date().getTime();
        if (t1 - t0 > timeout) {
          throw new Error('Timeout')
        }
        await new Promise(resolve => setTimeout(resolve, 500));
      }
    }
    throw new Error('Not in block, your Code may upload failed.')
  }
}

export class PinkCodePromise {

  readonly abi: Abi;
  readonly api: ApiBase<'promise'>;
  readonly phatRegistry: OnChainRegistry;

  protected readonly _decorateMethod: DecorateMethod<'promise'>;

  readonly code: Uint8Array;

  readonly #tx: MapConstructorExec<'promise'> = {};

  constructor (api: ApiBase<'promise'>, phatRegistry: OnChainRegistry, abi: AbiLike, wasm: WasmLike) {
    if (!api || !api.isConnected || !api.tx) {
      throw new Error('Your API has not been initialized correctly and is not connected to a chain');
    }
    if (!phatRegistry.isReady()) {
      throw new Error('Your phatRegistry has not been initialized correctly.');
    }

    this.abi = abi instanceof Abi
      ? abi
      : new Abi(abi, api.registry.getChainProperties());
    this.api = api;
    this._decorateMethod = toPromiseMethod;
    this.phatRegistry = phatRegistry

    this.code = isWasm(this.abi.info.source.wasm)
      ? this.abi.info.source.wasm
      : u8aToU8a(wasm);

    if (!isWasm(this.code)) {
      throw new Error('No WASM code provided');
    }

    this.abi.constructors.forEach((c): void => {
      if (isUndefined(this.#tx[c.method])) {
        this.#tx[c.method] = createBluePrintTx(c, (_o, p) => this.#instantiate(c, p));
      }
    });
  }

  public get tx (): MapConstructorExec<'promise'> {
    return this.#tx;
  }

  public upload() {
    return this.#instantiate(0, [])
  }

  #instantiate = (_constructorOrId: AbiConstructor | string | number, _params: unknown[]) => {
    return this.api.tx.phalaPhatContracts.clusterUploadResource(
      this.phatRegistry.clusterId,
      'InkCode',
      this.code.toString()
    ).withResultTransform(
      (result: ISubmittableResult) => {
        return new InkCodeSubmittableResult(result, this.abi, this.phatRegistry)
      }
    );
  };
}

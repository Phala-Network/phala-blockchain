import type { DecorateMethod } from '@polkadot/api/types';
import type { Hash } from '@polkadot/types/interfaces';
import type { ISubmittableResult } from '@polkadot/types/types';
import type { AbiConstructor, BlueprintOptions } from '@polkadot/api-contract/types';
import type { MapConstructorExec } from '@polkadot/api-contract/base/types';

import type { OnChainRegistry } from '../OnChainRegistry';
import type { AbiLike } from '../types';

import { SubmittableResult } from '@polkadot/api';
import { ApiBase } from '@polkadot/api/base';
import { BN_ZERO, isUndefined } from '@polkadot/util';
import { createBluePrintTx } from '@polkadot/api-contract/base/util';
import crypto from 'crypto';

import { Abi } from '@polkadot/api-contract/Abi';
import { toPromiseMethod } from '@polkadot/api';
import { Option } from '@polkadot/types';

function hex(b: string | Uint8Array) {
  if (typeof b != "string") {
    b = Buffer.from(b).toString('hex');
  }
  if (!b.startsWith('0x')) {
    return '0x' + b;
  } else {
    return b;
  }
}


export class PinkBlueprintSubmittableResult extends SubmittableResult {
  readonly registry: OnChainRegistry;
  readonly abi: Abi;
  // readonly contract?: Contract<ApiType>;

  #isReady: boolean = false;

  constructor (result: ISubmittableResult, abi: Abi, registry: OnChainRegistry) {
    super(result);

    this.registry = registry;
    this.abi = abi;
  //   this.contract = contract;
  }

  async waitReady(timeout: number = 120_000) {
    if (this.#isReady) {
      return
    }

    if (this.isInBlock || this.isFinalized) {
      let contractId: string | undefined
      for (const event of this.events) {
        if (event.event.method === 'Instantiating') {
          // tired of TS complaining about the type of event.event.data.contract
          // @ts-ignore
          contractId = event.event.data.contract.toString();
          break;
        }
      }
      if (!contractId) {
        throw new Error('Failed to find contract ID in events, maybe instantiate failed.')
      }

      const t0 = new Date().getTime();
      while (true) {
        const result1 = (await this.registry.api.query.phalaPhatContracts.clusterContracts(this.registry.clusterId)) as unknown as Text[]
        const contractIds = result1.map(i => i.toString())
        if (contractIds.indexOf(contractId) !== -1) {
          const result2 = (await this.registry.api.query.phalaRegistry.contractKeys(contractId)) as unknown as Option<any>
          if (result2.isSome) {
            this.#isReady = true
            return
          }
        }

        const t1 = new Date().getTime();
        if (t1 - t0 > timeout) {
          throw new Error('Timeout')
        }
        await new Promise(resolve => setTimeout(resolve, 1000));
      }
    }
    throw new Error(`instantiate failed for ${this.abi.info.source.wasmHash.toString()}`)
  }
}

export class PinkBlueprintPromise {

  readonly abi: Abi;
  readonly api: ApiBase<'promise'>;
  readonly phatRegistry: OnChainRegistry;

  protected readonly _decorateMethod: DecorateMethod<'promise'>;

  /**
   * @description The on-chain code hash for this blueprint
   */
  readonly codeHash: Hash;

  readonly #tx: MapConstructorExec<'promise'> = {};

  constructor (api: ApiBase<'promise'>, phatRegistry: OnChainRegistry, abi: AbiLike, codeHash: string | Hash | Uint8Array) {
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

    this.codeHash = this.api.registry.createType('Hash', codeHash);

    this.abi.constructors.forEach((c): void => {
      if (isUndefined(this.#tx[c.method])) {
        this.#tx[c.method] = createBluePrintTx(c, (o, p) => this.#deploy(c, o, p));
      }
    });
  }

  public get tx (): MapConstructorExec<'promise'> {
    return this.#tx;
  }

  #deploy = (
    constructorOrId: AbiConstructor | string | number,
    { gasLimit = BN_ZERO, storageDepositLimit = null, value = BN_ZERO }: BlueprintOptions,
    params: unknown[]
  ) => {
    const salt = hex(crypto.randomBytes(4))
    return this.api.tx.phalaPhatContracts.instantiateContract(
      { 'WasmCode': this.abi.info.source.wasmHash.toString() },
      this.abi.findConstructor(constructorOrId).toU8a(params),
      salt,
      this.phatRegistry.clusterId,
      0,  // not transfer any token to the contract during initialization
      gasLimit,
      storageDepositLimit,
      0
    ).withResultTransform(
      (result: ISubmittableResult) => new PinkBlueprintSubmittableResult(result, this.abi, this.phatRegistry)
    );
  };
}

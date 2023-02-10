import type { ApiPromise } from '@polkadot/api';

import { waitReady } from "@polkadot/wasm-crypto";
import { BN } from '@polkadot/util';

import { createPruntimeApi } from './create';
import { pruntime_rpc } from './proto';

export class UnexpectedEndpointError extends Error {
}


export class OnChainRegistry {

  public api: ApiPromise

  public clusterId: string | undefined
  public clusterInfo: Record<string, any> | undefined
  public remotePubkey: string | undefined
  public pruntimeURL: string | undefined

  #ready: boolean = false
  #phactory: pruntime_rpc.PhactoryAPI | undefined

  constructor(api: ApiPromise) {
    this.api = api
  }

  public async waitReady() {
    await waitReady();

    const clusters: [string, any][] = []
    {
      const result = await this.api.query.phalaFatContracts.clusters.entries()
      // @ts-ignore
      result.forEach(([storageKey, value]) => {
        const clusterId = storageKey.toHuman()
      // @ts-ignore
        const clusterInfo = value.unwrap().toHuman()
      // @ts-ignore
        clusters[clusterId] = clusterInfo
      // @ts-ignore
        clusters.push([clusterId, clusterInfo])
      })
    }
    [this.clusterId, this.clusterInfo] = clusters[0]
    // @ts-ignore
    this.remotePubkey = this.clusterInfo.workers[0]

    {
      const result = await this.api.query.phalaRegistry.endpoints(this.remotePubkey)
      /// @ts-ignore
      this.pruntimeURL = result.toHuman()['V1'][0]
    }

    if (!this.pruntimeURL) {
      throw new UnexpectedEndpointError('PRuntime not yet registered on chain.')
    }

    this.#phactory = createPruntimeApi(this.pruntimeURL)

    this.#ready = true
  }

  public async getContractKey(contractId: string) {
    const contractKey = (await this.api.query.phalaRegistry.contractKeys(contractId)).toString();
    if (!contractKey) {
      return undefined
    }
    return contractKey
  }

  public isReady() {
    return this.#ready
  }

  get phactory() {
    if (!this.#ready || !this.#phactory) {
      throw new Error('You need initialize OnChainRegistry first.')
    }
    return this.#phactory
  }

  get gasPrice() {
    if (!this.#ready || !this.clusterInfo) {
      throw new Error('You need initialize OnChainRegistry first.')
    }
    return new BN(this.clusterInfo.gasPrice)
  }
}

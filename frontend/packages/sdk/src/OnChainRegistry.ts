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
        const clusterInfo = value.unwrap().toJSON()
      // @ts-ignore
        clusters[clusterId] = clusterInfo
      // @ts-ignore
        clusters.push([clusterId, clusterInfo])
      })
    }
    this.clusterId = clusters[0][0][0]
    this.clusterInfo = clusters[0][1]

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

    // It might not be a good idea to call getInfo() here, but for now both testnet (POC-5 & closed-beta) not yet
    // upgrade to the latest Phactory API, so we need to call it here to make sure that's compatible.
    try {
      await this.#phactory.getInfo({})
    } catch (err) {
      throw new Error('Phactory API not compatible, you might need downgrade your @phala/sdk or connect to an up-to-date endpoint.')
    }

    this.#ready = true
  }

  public async getContractKey(contractId: string) {
    const contractKey = await this.api.query.phalaRegistry.contractKeys(contractId);
    if (!contractKey) {
      return undefined
    }
    return contractKey.toString();
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


  /**
   * Static factory method returns a ready to use PhatRegistry object.
   */
  static async create(api: ApiPromise) {
    const instance = new OnChainRegistry(api)
    await instance.waitReady()
    return instance
  }
}

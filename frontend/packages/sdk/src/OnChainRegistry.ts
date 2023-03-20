import type { ApiPromise } from '@polkadot/api';

import { waitReady } from "@polkadot/wasm-crypto";
import { BN } from '@polkadot/util';

import { createPruntimeApi } from './create';
import { pruntime_rpc } from './proto';
import { Option, Map, Enum, Vec, U8aFixed, Text } from '@polkadot/types';
import { AccountId } from '@polkadot/types/interfaces';

export class UnexpectedEndpointError extends Error {
}

// @FIXME: We not yet cover `as` and the `OnlyOwner` scenario.
interface ClusterPermission extends Enum {
  readonly isPublic: boolean
}

interface ClusterInfo extends Map {
  owner: AccountId
  permission: ClusterPermission
  workers: Vec<U8aFixed>
  systemContract?: AccountId
  gasPrice?: BN
  depositPerItem?: BN
  depositPerByte?: BN
}

interface VersionedEndpoints extends Enum {
  readonly isV1: boolean
  readonly asV1: Vec<Text>
}

interface CreateOptions {
  autoConnect?: boolean
  clusterId?: string
  workerId?: string
  pruntimeURL?: string
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

  public async getContractKey(contractId: string) {
    const contractKey = await this.api.query.phalaRegistry.contractKeys(contractId);
    if (!contractKey) {
      return undefined
    }
    return contractKey.toString();
  }

  public async getContractKeyOrFail(contractId: string) {
    const contractKey = await this.getContractKey(contractId);
    if (!contractKey) {
      throw new Error(`Contract ${contractId} not found in cluster.`);
    }
    return contractKey;
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
  static async create(api: ApiPromise, options?: CreateOptions) {
    options = { autoConnect: true, ...(options || {}) }
    const instance = new OnChainRegistry(api)
    await waitReady()
    if (options.autoConnect) {
      await instance.connect(options.clusterId, options.workerId, options.pruntimeURL)
    }
    return instance
  }

  public async getClusters(clusterId?: string) {
    if (clusterId) {
      const result = (await this.api.query.phalaFatContracts.clusters(clusterId)) as Option<ClusterInfo>
      if (result.isNone) {
        return null
      }
      return result.unwrap()
    } else {
      const result = await this.api.query.phalaFatContracts.clusters.entries()
      return result.map(([storageKey, value]) => {
        const clusterId = storageKey.args.map(i => i.toPrimitive())[0] as string
        const clusterInfo = (value as Option<ClusterInfo>).unwrap()
        return [clusterId, clusterInfo]
      })
    }
  }

  public async getEndpints(workerId?: U8aFixed | string) {
    if (workerId) {
      if (typeof workerId !== 'string') {
        workerId = workerId.toHex()
      }
      return await this.api.query.phalaRegistry.endpoints<Option<VersionedEndpoints>>(workerId)
    }
    const result = await this.api.query.phalaRegistry.endpoints.entries()
    return result.map(([storageKey, value]) => {
      const workerId = storageKey.args.map(i => i.toPrimitive())[0] as string
      return [workerId, (value as Option<VersionedEndpoints>)]
    })
  }

  public async connect(clusterId?: string | null, workerId?: string | null, pruntimeURL?: string | null) {
    this.#ready = false

    let clusterInfo
    if (clusterId) {
      clusterInfo = await this.getClusters(clusterId)
      if (!clusterInfo) {
        throw new Error(`Cluster not found: ${clusterId}`)
      }
    } else {
      const clusters = await this.getClusters()
      if (!clusters || !Array.isArray(clusters)) {
        throw new Error('No cluster found.')
      }
      if (clusters.length === 0) {
        throw new Error('No cluster found.')
      }
      clusterId = clusters[0][0] as string
      clusterInfo = clusters[0][1] as ClusterInfo
    }

    const endpoints = await this.getEndpints()
    if (!Array.isArray(endpoints) || endpoints.length === 0) {
      throw new Error('No worker found.')
    }
    if (!workerId && !pruntimeURL) {
      workerId = endpoints[0][0] as string
      pruntimeURL = (endpoints[0][1] as Option<VersionedEndpoints>).unwrap().asV1[0].toPrimitive() as string
    } else if (workerId) {
      const endpoint = endpoints.find(([id, _]) => id === workerId)
      if (!endpoint) {
        throw new Error(`Worker not found: ${workerId}`)
      }
      pruntimeURL = (endpoint[1] as Option<VersionedEndpoints>).unwrap().asV1[0].toPrimitive() as string
    } else if (pruntimeURL) {
      const endpoint = endpoints.find(([_, v]) => {
        const url = (v as Option<VersionedEndpoints>).unwrap().asV1[0].toPrimitive() as string
        return url === pruntimeURL
      })
      if (!endpoint) {
        throw new Error(`Worker not found: ${workerId}`)
      }
      workerId = endpoint[0] as string
    }

    this.#phactory = createPruntimeApi(pruntimeURL!)

    // It might not be a good idea to call getInfo() here, but for now both testnet (POC-5 & closed-beta) not yet
    // upgrade to the latest Phactory API, so we need to call it here to make sure that's compatible.
    try {
      await this.#phactory.getInfo({})
    } catch (err) {
      throw new Error('Phactory API not compatible, you might need downgrade your @phala/sdk or connect to an up-to-date endpoint.')
    }
    this.clusterId = clusterId
    this.clusterInfo = clusterInfo
    this.remotePubkey = workerId!
    this.pruntimeURL = pruntimeURL!

    this.#ready = true
  }
}

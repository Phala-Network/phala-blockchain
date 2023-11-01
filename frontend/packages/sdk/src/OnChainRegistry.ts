import { type ApiPromise, Keyring } from '@polkadot/api'
import type { Result, U64 } from '@polkadot/types'
import { Enum, Map, Option, Text, U8aFixed, Vec } from '@polkadot/types'
import { AccountId } from '@polkadot/types/interfaces'
import { BN } from '@polkadot/util'
import { cryptoWaitReady } from '@polkadot/util-crypto'
import systemAbi from './abis/system.json'
import { PinkContractPromise } from './contracts/PinkContract'
import { PinkLoggerContractPromise } from './contracts/PinkLoggerContract'
import { type CertificateData, signCertificate } from './pruntime/certificate'
import createPruntimeClient from './pruntime/createPruntimeClient'
import { pruntime_rpc } from './pruntime/proto'

export class UnexpectedEndpointError extends Error {}

// @FIXME: We not yet cover `as` and the `OnlyOwner` scenario.
interface ClusterPermission extends Enum { readonly isPublic: boolean
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
  systemContractId?: string
  skipCheck?: boolean
}

export interface WorkerInfo {
  pubkey: string
  clusterId: string
  endpoints: {
    default: string
    v1?: string[]
  }
}

export class OnChainRegistry {
  public api: ApiPromise

  public clusterId: string | undefined
  public clusterInfo: ClusterInfo | undefined

  public workerInfo: WorkerInfo | undefined

  #ready: boolean = false
  #phactory: pruntime_rpc.PhactoryAPI | undefined

  #cert: CertificateData | undefined

  #systemContract: PinkContractPromise | undefined
  #loggerContract: PinkLoggerContractPromise | undefined

  constructor(api: ApiPromise) {
    this.api = api
  }

  public async getContractKey(contractId: AccountId | string) {
    const contractKey = await this.api.query.phalaRegistry.contractKeys(contractId)
    if (!contractKey) {
      return undefined
    }
    return contractKey.toString()
  }

  public async getContractKeyOrFail(contractId: string) {
    const contractKey = await this.getContractKey(contractId)
    if (!contractKey) {
      throw new Error(`Contract ${contractId} not found in cluster.`)
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
    if (!this.#ready || !this.clusterInfo || !this.clusterInfo.gasPrice) {
      throw new Error('You need initialize OnChainRegistry first.')
    }
    return this.clusterInfo.gasPrice
  }

  /**
   * Static factory method returns a ready to use PhatRegistry object.
   */
  static async create(api: ApiPromise, options?: CreateOptions) {
    options = { autoConnect: true, ...(options || {}) }
    const instance = new OnChainRegistry(api)
    // We should ensure the wasm & api has been initialized here.
    await Promise.all([cryptoWaitReady(), api.isReady])
    if (options.autoConnect) {
      await instance.connect(
        options.clusterId,
        options.workerId,
        options.pruntimeURL,
        options.systemContractId,
        !!options.skipCheck
      )
    }
    return instance
  }

  public async getAllClusters() {
    const result = await this.api.query.phalaPhatContracts.clusters.entries()
    return result.map(([storageKey, value]) => {
      const clusterId = storageKey.args.map((i) => i.toPrimitive())[0] as string
      const clusterInfo = (value as Option<ClusterInfo>).unwrap()
      return [clusterId, clusterInfo] as const
    })
  }

  public async getClusterInfoById(clusterId: string) {
    const result = (await this.api.query.phalaPhatContracts.clusters(clusterId)) as Option<ClusterInfo>
    if (result.isNone) {
      return null
    }
    return result.unwrap()
  }

  public async getClusters(clusterId?: string) {
    if (clusterId) {
      const result = (await this.api.query.phalaPhatContracts.clusters(clusterId)) as Option<ClusterInfo>
      if (result.isNone) {
        return null
      }
      return result.unwrap()
    } else {
      return await this.getAllClusters()
    }
  }

  public async getEndpoints(workerId?: U8aFixed | string) {
    if (workerId) {
      if (typeof workerId !== 'string') {
        workerId = workerId.toHex()
      }
      return await this.api.query.phalaRegistry.endpoints<Option<VersionedEndpoints>>(workerId)
    }
    const result = await this.api.query.phalaRegistry.endpoints.entries()
    return result.map(([storageKey, value]) => {
      const workerId = storageKey.args.map((i) => i.toPrimitive())[0] as string
      return [workerId, value as Option<VersionedEndpoints>]
    })
  }

  public async getClusterWorkers(clusterId?: string): Promise<WorkerInfo[]> {
    let _clusterId = clusterId || this.clusterId
    if (!_clusterId) {
      throw new Error('You need specified clusterId to list workers inside it.')
    }
    const result = await this.api.query.phalaPhatContracts.clusterWorkers(clusterId)
    const workerIds = result.toJSON() as string[]
    const infos = await this.api.query.phalaRegistry.endpoints.multi(workerIds)
    return infos.map((info, idx) => {
      const workerId = workerIds[idx]
      const endpoints = info.toJSON() as { v1: string[] }
      return {
        pubkey: workerId,
        clusterId: _clusterId!,
        endpoints: {
          default: endpoints.v1[0],
          v1: endpoints.v1,
        }
      }
    })
  }

  async preparePruntimeClientOrThrows(endpoint: string) {
    // It might not be a good idea to call getInfo() here, but for now both testnet (POC-5 & closed-beta) not yet
    // upgrade to the latest Phactory API, so we need to call it here to make sure that's compatible.
    try {
      const phactory = createPruntimeClient(endpoint)
      await phactory.getInfo({})
      return phactory
    } catch (err) {
      console.error(err)
      throw new Error(
        'Phactory API not compatible, you might need downgrade your @phala/sdk or connect to an up-to-date endpoint.'
      )
    }
  }

  async prepareSystemOrThrows(clusterInfo: ClusterInfo) {
    const systemContractId = clusterInfo.systemContract
    if (systemContractId) {
      const systemContractKey = await this.getContractKey(systemContractId)
      if (systemContractKey) {
        this.#systemContract = new PinkContractPromise(this.api, this, systemAbi, systemContractId, systemContractKey)
        this.#loggerContract = await PinkLoggerContractPromise.create(this.api, this, this.#systemContract)
      } else {
        throw new Error(`System contract not found: ${systemContractId}`)
      }
    }
  }

  /**
   * ClusterId: string | null  - Cluster ID, if empty, will try to use the first cluster found in the chain registry.
   * WorkerId: string | null - Worker ID, if empty, will try to use the first worker found in the cluster.
   * PruntimeURL: string | null - Pruntime URL, if empty, will try to use the pruntime URL of the selected worker.
   * systemContractId: string | AccountId | null - System contract ID, if empty, will try to use the system contract ID of the selected cluster.
   * skipCheck: boolean | undefined - Skip the check of cluster and worker has been registry on chain or not, it's for cluster
   *                      deployment scenario, where the cluster and worker has not been registry on chain yet.
   */
  public async connect(worker?: WorkerInfo): Promise<void>;
  public async connect(
    clusterId?: string | null,
    workerId?: string | null,
    pruntimeURL?: string | null,
    systemContractId?: string | AccountId,
    skipCheck?: boolean
  ): Promise<void>;
  public async connect(...args: any[]): Promise<void> {
    this.#ready = false

    if (args.length === 0 || args.length === 1) {
      // Scenario 1: connect to default worker.
      if (args.length === 0) {
        const clusters = await this.getAllClusters()
        if (!clusters || clusters.length === 0) {
          throw new Error('No cluster found.')
        }
        const [clusterId, clusterInfo] = clusters[0]
        const workers = await this.getClusterWorkers(clusterId)
        this.#phactory = await this.preparePruntimeClientOrThrows(workers[0].endpoints.default)
        this.clusterId = clusterId
        this.clusterInfo = clusterInfo
        this.workerInfo = workers[0]
        this.#ready = true
        await this.prepareSystemOrThrows(clusterInfo)
        return
      }

      // Scenario 2: connect to specified worker.
      if (args.length === 1 && args[0] instanceof Object) {
        const worker = args[0] as WorkerInfo
        const clusterInfo = await this.getClusterInfoById(worker.clusterId)
        if (!clusterInfo) {
          throw new Error(`Cluster not found: ${worker.clusterId}`)
        }
        this.#phactory = await this.preparePruntimeClientOrThrows(worker.endpoints.default)
        this.clusterId = worker.clusterId
        this.clusterInfo = clusterInfo
        this.workerInfo = worker
        this.#ready = true
        await this.prepareSystemOrThrows(clusterInfo)
        return
      }
    }

    // legacy support.
    let clusterId = args[0] as string | undefined
    let workerId = args[1] as string | undefined
    let pruntimeURL = args[2] as string | undefined
    let systemContractId = args[3] as string | AccountId | undefined
    const skipCheck = args[4] as boolean | undefined

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

    if (!skipCheck) {
      const endpoints = await this.getEndpoints()
      if (!Array.isArray(endpoints) || endpoints.length === 0) {
        throw new Error('No worker found.')
      }
      if (!workerId && !pruntimeURL) {
        workerId = endpoints[0][0] as string
        pruntimeURL = (endpoints[0][1] as Option<VersionedEndpoints>).unwrap().asV1[0].toPrimitive() as string
      } else if (pruntimeURL) {
        const endpoint = endpoints.find(([_, v]) => {
          const url = (v as Option<VersionedEndpoints>).unwrap().asV1[0].toPrimitive() as string
          return url === pruntimeURL
        })
        if (endpoint) {
          workerId = endpoint[0] as string
        }
      } else if (workerId) {
        const endpoint = endpoints.find(([id, _]) => id === workerId)
        if (!endpoint) {
          throw new Error(`Worker not found: ${workerId}`)
        }
        pruntimeURL = (endpoint[1] as Option<VersionedEndpoints>).unwrap().asV1[0].toPrimitive() as string
      }
    }

    this.#phactory = createPruntimeClient(pruntimeURL!)

    // It might not be a good idea to call getInfo() here, but for now both testnet (POC-5 & closed-beta) not yet
    // upgrade to the latest Phactory API, so we need to call it here to make sure that's compatible.
    try {
      await this.#phactory.getInfo({})
    } catch (err) {
      console.error(err)
      throw new Error(
        'Phactory API not compatible, you might need downgrade your @phala/sdk or connect to an up-to-date endpoint.'
      )
    }
    this.clusterId = clusterId!
    this.workerInfo = {
      pubkey: workerId!,
      clusterId: clusterId!,
      endpoints: {
        default: pruntimeURL!,
        v1: [pruntimeURL!],
      }
    }
    this.clusterInfo = clusterInfo as ClusterInfo

    this.#ready = true

    if (this.clusterInfo && this.clusterInfo.systemContract) {
      systemContractId = this.clusterInfo.systemContract
    }
    if (systemContractId) {
      const systemContractKey = await this.getContractKey(systemContractId)
      if (systemContractKey) {
        this.#systemContract = new PinkContractPromise(this.api, this, systemAbi, systemContractId, systemContractKey)
        this.#loggerContract = await PinkLoggerContractPromise.create(this.api, this, this.#systemContract)
      } else {
        throw new Error(`System contract not found: ${systemContractId}`)
      }
    }
  }

  get systemContract() {
    if (this.#systemContract) {
      return this.#systemContract
    }
    console.warn('System contract not found, you might not connect to a health cluster.')
  }

  async getAnonymousCert(): Promise<CertificateData> {
    if (!this.#cert) {
      const keyring = new Keyring({ type: 'sr25519' })
      const pair = keyring.addFromUri('//Alice')
      this.#cert = await signCertificate({ pair })
    }
    return this.#cert
  }

  resetAnonymousCert() {
    this.#cert = undefined
  }

  async getClusterBalance(address: string | AccountId, cert?: CertificateData) {
    const system = this.#systemContract
    if (!system) {
      throw new Error('System contract not found, you might not connect to a health cluster.')
    }
    if (!cert) {
      cert = await this.getAnonymousCert()
    }
    const [{ output: totalBalanceOf }, { output: freeBalanceOf }] = await Promise.all([
      system.query['system::totalBalanceOf'](cert.address, { cert }, address),
      system.query['system::freeBalanceOf'](cert.address, { cert }, address),
    ])
    return {
      total: (totalBalanceOf as Result<U64, any>).asOk.toBn(),
      free: (freeBalanceOf as Result<U64, any>).asOk.toBn(),
    }
  }

  transferToCluster(address: string | AccountId, amount: number | string | BN) {
    return this.api.tx.phalaPhatContracts.transferToCluster(amount, this.clusterId, address)
  }

  get loggerContract() {
    if (this.#loggerContract) {
      return this.#loggerContract
    }
    console.warn('Logger contract not found, you might not connect to a health cluster.')
  }

  get remotePubkey() {
    return this.workerInfo?.pubkey
  }

  get pruntimeURL() {
    return this.workerInfo?.endpoints.default
  }
}

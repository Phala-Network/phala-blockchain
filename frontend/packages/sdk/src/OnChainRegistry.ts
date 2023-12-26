import { type ApiPromise, Keyring } from '@polkadot/api'
import { type KeyringPair } from '@polkadot/keyring/types'
import { Enum, Map, Option, Text, U8aFixed, Vec } from '@polkadot/types'
import { AccountId } from '@polkadot/types/interfaces'
import { BN } from '@polkadot/util'
import { cryptoWaitReady } from '@polkadot/util-crypto'
import systemAbi from './abis/system.json'
import { PinkContractPromise } from './contracts/PinkContract'
import { PinkLoggerContractPromise } from './contracts/PinkLoggerContract'
import { type PhalaTypesVersionedWorkerEndpoints, ackFirst } from './ha/ack-first'
import { KeyringPairProvider } from './providers/KeyringPairProvider'
import { type CertificateData, signCertificate } from './pruntime/certificate'
import createPruntimeClient from './pruntime/createPruntimeClient'
import { pruntime_rpc } from './pruntime/proto'
import type { SystemContract } from './types'

export class UnexpectedEndpointError extends Error {}

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

export interface CreateOptions {
  autoConnect?: boolean
  clusterId?: string
  workerId?: string
  pruntimeURL?: string
  systemContractId?: string
  skipCheck?: boolean
  strategy?:
    | 'ack-first'
    | ((
        api: ApiPromise,
        clusterId: string
      ) => Promise<Readonly<[string, string, ReturnType<typeof createPruntimeClient>]>>)
}

export interface WorkerInfo {
  pubkey: string
  clusterId: string
  endpoints: {
    default: string
    v1?: string[]
  }
}

export interface PartialWorkerInfo {
  clusterId?: string
  pruntimeURL: string
}

export interface StrategicWorkerInfo {
  clusterId?: string
  strategy:
    | 'ack-first'
    | ((api: ApiPromise, clusterId: string) => Promise<Readonly<[string, ReturnType<typeof createPruntimeClient>]>>)
}

export class OnChainRegistry {
  public api: ApiPromise

  public clusterId: string | undefined
  public clusterInfo: ClusterInfo | undefined

  public workerInfo: WorkerInfo | undefined

  #ready: boolean = false
  #phactory: pruntime_rpc.PhactoryAPI | undefined

  #alice: KeyringPair | undefined
  #cert: CertificateData | undefined

  #systemContract: SystemContract | undefined
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
      if (!options.clusterId && !options.workerId && !options.pruntimeURL) {
        await instance.connect()
      }
      // If user specified the pruntimeURL, it should use it first.
      else if (options.pruntimeURL) {
        const workerInfo: PartialWorkerInfo = {
          clusterId: options.clusterId,
          pruntimeURL: options.pruntimeURL,
        }
        await instance.connect(workerInfo)
      } else if (options.clusterId && !options.strategy) {
        await instance.connect({
          clusterId: options.clusterId,
          strategy: 'ack-first',
        } as StrategicWorkerInfo)
      } else if (options.strategy) {
        await instance.connect({
          clusterId: options.clusterId,
          strategy: options.strategy,
        } as StrategicWorkerInfo)
      }
      // Failed back to backward compatible mode.
      else {
        console.warn('Failed back to legacy connection mode, please use pruntimeURL instead.')
        await instance.connect(
          options.clusterId,
          options.workerId,
          options.pruntimeURL,
          options.systemContractId,
          !!options.skipCheck
        )
      }
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
      const clusters = await this.getAllClusters()
      if (!clusters || clusters.length === 0) {
        throw new Error('You need specified clusterId to list workers inside it.')
      }
      _clusterId = clusters[0][0] as string
    }
    const result = await this.api.query.phalaPhatContracts.clusterWorkers(_clusterId)
    const workerIds = result.toJSON() as string[]
    const infos =
      await this.api.query.phalaRegistry.endpoints.multi<Option<PhalaTypesVersionedWorkerEndpoints>>(workerIds)

    return infos
      .map((i, idx) => [workerIds[idx], i] as const)
      .filter(([_, maybeEndpoint]) => maybeEndpoint.isSome)
      .map(
        ([workerId, maybeEndpoint]) =>
          ({
            pubkey: workerId,
            clusterId: _clusterId!,
            endpoints: {
              default: maybeEndpoint.unwrap().asV1[0].toString(),
              v1: maybeEndpoint.unwrap().asV1.map((i) => i.toString()),
            },
          }) as WorkerInfo
      )
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
        const provider = await KeyringPairProvider.create(this.api, this.alice)
        this.#systemContract = new PinkContractPromise(
          this.api,
          this,
          systemAbi,
          systemContractId,
          systemContractKey,
          provider
        )
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
  public async connect(worker?: WorkerInfo | PartialWorkerInfo | StrategicWorkerInfo): Promise<void>
  public async connect(
    clusterId?: string | null,
    workerId?: string | null,
    pruntimeURL?: string | null,
    systemContractId?: string | AccountId,
    skipCheck?: boolean
  ): Promise<void>
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
        const [workerId, endpoint, phactory] = await ackFirst()(this.api, clusterId)
        this.#phactory = phactory
        this.clusterId = clusterId
        this.clusterInfo = clusterInfo
        this.workerInfo = {
          pubkey: workerId,
          clusterId: clusterId,
          endpoints: {
            default: endpoint,
            v1: [endpoint],
          },
        }
        this.#ready = true
        await this.prepareSystemOrThrows(clusterInfo)
        return
      }

      // Scenario 2: connect to specified worker.
      if (args.length === 1 && args[0] instanceof Object) {
        if (args[0].strategy) {
          let clusterId = args[0].clusterId
          let clusterInfo
          if (!clusterId) {
            const clusters = await this.getAllClusters()
            if (!clusters || clusters.length === 0) {
              throw new Error('No cluster found.')
            }
            ;[clusterId, clusterInfo] = clusters[0]
          } else {
            clusterInfo = await this.getClusterInfoById(clusterId)
            if (!clusterInfo) {
              throw new Error(`Cluster not found: ${clusterId}`)
            }
          }
          if (args[0].strategy === 'ack-first') {
            const [workerId, endpoint, phactory] = await ackFirst()(this.api, clusterId)
            this.#phactory = phactory
            this.clusterId = clusterId
            this.clusterInfo = clusterInfo
            this.workerInfo = {
              pubkey: workerId,
              clusterId: clusterId,
              endpoints: {
                default: endpoint,
                v1: [endpoint],
              },
            }
            this.#ready = true
            await this.prepareSystemOrThrows(clusterInfo)
          } else if (typeof args[0].strategy === 'function') {
            const [workerId, phactory] = await args[0].strategy(this.api, clusterId)
            this.#phactory = phactory
            this.clusterId = clusterId
            this.clusterInfo = clusterInfo
            this.workerInfo = {
              pubkey: workerId,
              clusterId: clusterId,
              endpoints: {
                default: phactory.endpoint,
                v1: [phactory.endpoint],
              },
            }
            this.#ready = true
            await this.prepareSystemOrThrows(clusterInfo)
          } else {
            throw new Error(`Unknown strategy: ${args[0].strategy}`)
          }
        }
        // Minimal connection settings, only PRuntimeURL has been specified.
        // clusterId is optional here since we can find it from `getClusterInfo`
        // API
        else if (args[0].pruntimeURL) {
          const partialInfo = args[0] as PartialWorkerInfo
          const pruntimeURL = partialInfo.pruntimeURL
          let clusterId = partialInfo.clusterId
          if (!pruntimeURL) {
            throw new Error('pruntimeURL is required.')
          }
          // We don't here preparePruntimeClientOrThrows here because we need
          // getting related info from PRtunime, we don't need extra check here.
          const phactory = createPruntimeClient(pruntimeURL)
          if (!clusterId) {
            const clusterInfoQuery = await phactory.getClusterInfo({})
            if (clusterInfoQuery?.info?.id) {
              clusterId = clusterInfoQuery.info.id as string
            } else {
              throw new Error(`getClusterInfo is unavailable, please ensure ${pruntimeURL} is valid PRuntime endpoint.`)
            }
          }
          const clusterInfo = await this.getClusterInfoById(clusterId)
          if (!clusterInfo) {
            throw new Error(`Cluster not found: ${partialInfo.clusterId}`)
          }
          const nodeInfo = await phactory.getInfo({})
          if (!nodeInfo || !nodeInfo.publicKey) {
            throw new Error(`Get PRuntime Pubkey failed.`)
          }
          this.#phactory = phactory
          this.clusterId = clusterId
          this.clusterInfo = clusterInfo
          this.workerInfo = {
            pubkey: nodeInfo.publicKey,
            clusterId: clusterId,
            endpoints: {
              default: pruntimeURL,
            },
          }
          this.#ready = true
          await this.prepareSystemOrThrows(clusterInfo)
        } else {
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
        }
        return
      }
    }

    console.warn('Deprecated: connect to dedicated worker via legacy mode, please migrate to the new API.')

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
      },
    }
    this.clusterInfo = clusterInfo as ClusterInfo

    this.#ready = true

    if (this.clusterInfo && this.clusterInfo.systemContract) {
      systemContractId = this.clusterInfo.systemContract
    }
    if (systemContractId) {
      const systemContractKey = await this.getContractKey(systemContractId)
      if (systemContractKey) {
        const provider = await KeyringPairProvider.create(this.api, this.alice)
        this.#systemContract = new PinkContractPromise(
          this.api,
          this,
          systemAbi,
          systemContractId,
          systemContractKey,
          provider
        )
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

  get alice() {
    if (!this.#alice) {
      const keyring = new Keyring({ type: 'sr25519' })
      this.#alice = keyring.addFromUri('//Alice')
    }
    return this.#alice
  }

  async getAnonymousCert(): Promise<CertificateData> {
    if (!this.#cert) {
      this.#cert = await signCertificate({ pair: this.alice })
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
      total: totalBalanceOf.asOk,
      free: freeBalanceOf.asOk,
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

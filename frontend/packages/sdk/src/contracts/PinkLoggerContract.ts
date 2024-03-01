import type { ApiPromise } from '@polkadot/api'
import { Keyring } from '@polkadot/api'
import { Abi } from '@polkadot/api-contract'
import type { DecodedEvent } from '@polkadot/api-contract/types'
import type { KeyringPair } from '@polkadot/keyring/types'
import type { Enum, Struct, Text, U8 } from '@polkadot/types'
import type { AccountId } from '@polkadot/types/interfaces'
import type { Result } from '@polkadot/types-codec'
import { hexAddPrefix, hexToString, hexToU8a, u8aToHex } from '@polkadot/util'
import { blake2AsU8a, sr25519Agreement } from '@polkadot/util-crypto'
import type { OnChainRegistry } from '../OnChainRegistry'
import { phalaTypes } from '../options'
import { type CertificateData, generatePair, signCertificate } from '../pruntime/certificate'
import { InkQuerySidevmMessage } from '../pruntime/coders'
import { pinkQuery } from '../pruntime/pinkQuery'
import { type pruntime_rpc } from '../pruntime/proto'
import type { AbiLike, InkResponse } from '../types'
import { isPascalCase, snakeToPascalCase } from '../utils/snakeToPascalCase'
import { ContractInitialError } from './Errors'
import { type PinkContractPromise } from './PinkContract'

export type LogTypeLiteral = 'Log' | 'Event' | 'MessageOutput' | 'QueryIn' | 'TooLarge'

export interface GetLogRequest {
  contract?: string
  from: number
  count: number
  block_number?: number
  type?: LogTypeLiteral | LogTypeLiteral[]

  // Event type logs specified filter.
  topic?: LiteralTopic

  // Use for decode event data
  abi?: AbiLike

  // MessageOutput type logs specified filter.
  nonce?: string
}

export interface SerMessageLog {
  type: 'Log'
  sequence: number
  blockNumber: number
  contract: string
  entry: string
  execMode: string
  timestamp: number
  level: number
  message: string
}

export interface SerMessageEvent {
  type: 'Event'
  sequence: number
  blockNumber: number
  contract: string
  topics: string[]
  payload: string
}

export interface SerMessageEventWithDecoded<Decoded extends DecodedEvent = DecodedEvent> {
  type: 'Event'
  sequence: number
  blockNumber: number
  contract: string
  topics: string[]
  payload: string
  decoded?: Decoded
}

interface OutputOk {
  ok: {
    flags: string[]
    data: string
  }
}

interface OutputErr {
  err: {
    module?: {
      error: string
      index: number
    }
    other?: null
  }
}

export interface SerMessageMessageOutputRaw {
  type: 'MessageOutput'
  sequence: number
  blockNumber: number
  origin: string
  contract: string
  nonce: string
  output: string
}

export interface SerMessageMessageOutput {
  type: 'MessageOutput'
  sequence: number
  blockNumber: number
  origin: string
  contract: string
  nonce: string
  output: {
    gasConsumed: {
      refTime: number
      proofSize: number
    }
    gasRequired: {
      refTime: number
      proofSize: number
    }
    storageDeposit: {
      charge: number
    }
    debugMessage: string
    result: OutputOk | OutputErr
  }
}

export interface SerMessageQueryIn {
  type: 'QueryIn'
  sequence: number
  user: string
}

export interface SerMessageTooLarge {
  type: 'TooLarge'
}

export type SerInnerMessage =
  | SerMessageLog
  | SerMessageEvent
  | SerMessageMessageOutputRaw
  | SerMessageQueryIn
  | SerMessageTooLarge

export type SerMessage<TEvent extends DecodedEvent = DecodedEvent> =
  | SerMessageLog
  | SerMessageEventWithDecoded<TEvent>
  | SerMessageMessageOutput
  | SerMessageQueryIn
  | SerMessageTooLarge

export interface GetLogResponse {
  records: SerMessage[]
  next: number
}

export interface LogServerInfo {
  programVersion: [number, number, number]
  nextSequence: number
  memoryCapacity: number
  memoryUsage: number
  currentNumberOfRecords: number
  estimatedCurrentSize: number
}

interface SidevmQueryContext {
  phactory: pruntime_rpc.PhactoryAPI
  remotePubkey: string
  address: AccountId
  cert: CertificateData
}

interface ContractExecResultOk extends Struct {
  flags: Text[]
  data: Text
}

interface ModuleError extends Struct {
  index: U8
  error: Text
}

interface ContractExecResultErr extends Enum {
  asModule: ModuleError
  asOther: Text
  isModule: boolean
  isOther: boolean
}

interface ContractExecResult extends Struct {
  result: Result<ContractExecResultOk, ContractExecResultErr>
}

function sidevmQueryWithReader({ phactory, remotePubkey, address, cert }: SidevmQueryContext) {
  return async function unsafeRunSidevmQuery<T>(sidevmMessage: Record<string, any>): Promise<T> {
    const [sk, pk] = generatePair()
    const encodedQuery = InkQuerySidevmMessage(address, sidevmMessage)
    const queryAgreementKey = sr25519Agreement(sk, hexToU8a(hexAddPrefix(remotePubkey)))
    const response = await pinkQuery(phactory, pk, queryAgreementKey, encodedQuery.toHex(), cert)
    const inkResponse = phalaTypes.createType<InkResponse>('InkResponse', response)
    if (inkResponse.result.isErr) {
      let error = `[${inkResponse.result.asErr.index}] ${inkResponse.result.asErr.type}`
      if (inkResponse.result.asErr.type === 'RuntimeError') {
        error = `${error}: ${inkResponse.result.asErr.value}`
      }
      throw new Error(error)
    }
    const payload = inkResponse.result.asOk.asInkMessageReturn.toString()
    const parsed = payload.substring(0, 2) === '0x' ? JSON.parse(hexToString(payload)) : JSON.parse(payload)
    if (parsed.error) {
      throw new Error(parsed.error)
    }
    return parsed
  }
}

function postProcessLogRecord<TDecodedEvent extends DecodedEvent = DecodedEvent>(
  messages: SerInnerMessage[],
  abiLike?: AbiLike
): SerMessage<TDecodedEvent>[] {
  let abi: Abi | undefined
  if (abiLike) {
    abi = abiLike instanceof Abi ? abiLike : new Abi(abiLike)
  }

  return messages.map((message) => {
    if (message.type === 'MessageOutput') {
      const execResult = phalaTypes.createType<ContractExecResult>('ContractExecResult', hexToU8a(message.output))
      const output = execResult.toJSON() as unknown as SerMessageMessageOutput['output']
      if (
        execResult.result.isErr &&
        execResult.result.asErr.isModule &&
        execResult.result.asErr.asModule?.index.toNumber() === 4
      ) {
        const err = phalaTypes.createType('ContractError', execResult.result.asErr.asModule.error)
        output.result = {
          err: {
            ...(output.result as OutputErr).err,
            module: {
              error: err.toJSON() as string,
              index: 4,
            },
          },
        } as OutputErr
      }
      return { ...message, output }
    } else if (message.type === 'Event' && abi) {
      try {
        const decoded = abi.decodeEvent(hexToU8a(message.payload)) as TDecodedEvent
        return { ...message, decoded }
      } catch (_err) {
        // silent
      }
    }
    return message
  })
}

export function buildGetLogRequest(
  params: any[],
  getFrom: (x: Partial<GetLogRequest>) => number,
  getDefaults: () => Partial<GetLogRequest>
): GetLogRequest {
  let request = getDefaults()
  switch (params.length) {
    case 0:
      request.from = getFrom(request)
      break

    case 1:
      if (typeof params[0] === 'number') {
        request.count = params[0]
        request.from = getFrom(request)
      } else {
        request.from = getFrom(params[0])
        if (params[0].count) {
          request.count = params[0].count
        }
        request = { ...params[0], ...request }
      }
      break

    case 2:
      request.count = params[0]
      if (typeof params[1] === 'number') {
        request.from = params[1]
      } else {
        request.from = getFrom(request)
        request = { ...params[1], ...request }
      }
      break

    case 3:
      request = { ...params[2], count: params[0], from: params[1] }
      break

    default:
      throw new Error('Unexpected parameters.')
  }
  return request as GetLogRequest
}

// LiteralTopic it looks like `System::Event', the contract name in PascalCase, split by `::', and then the event name.
export type LiteralTopic = `${Capitalize<string>}::${string}`

export function getTopicHash(topic: LiteralTopic): string {
  if (topic.indexOf('::') === -1) {
    throw new Error('Invalid topic.')
  }
  let [contract, event] = topic.split('::')
  if (!isPascalCase(contract)) {
    contract = snakeToPascalCase(contract)
  }
  event = `${contract}::${event}`

  const length = event.length
  const encoded = phalaTypes.createType(`(Vec<u8>, [u8; ${length}])`, [null, event]).toU8a()
  if (encoded.length > 32) {
    return u8aToHex(blake2AsU8a(encoded))
  } else {
    return u8aToHex(encoded) + '00'.repeat(32 - encoded.length)
  }
}

export class PinkLoggerContractPromise {
  #phactory: pruntime_rpc.PhactoryAPI
  #remotePubkey: string
  #address: AccountId
  #pair: KeyringPair
  #systemContractId: string | AccountId | undefined

  static async create(
    _api: ApiPromise,
    registry: OnChainRegistry,
    systemContract: PinkContractPromise,
    pair?: KeyringPair
  ): Promise<PinkLoggerContractPromise> {
    let _pair: KeyringPair | undefined = pair
    if (!_pair) {
      const keyring = new Keyring({ type: 'sr25519' })
      _pair = keyring.addFromUri('//Alice')
    }
    const cert = await signCertificate({ pair: _pair })
    const { output } = await systemContract.query['system::getDriver'](_pair.address, { cert }, 'PinkLogger')
    const contractId = (output as Result<Text, any>).asOk.toHex()
    if (!contractId) {
      throw new ContractInitialError('No PinkLogger contract registered in the cluster.')
    }
    const systemContractId = systemContract.address?.toHex()
    if (!registry.phactory || !registry.remotePubkey) {
      throw new Error('No Pruntime connection found.')
    }
    return new PinkLoggerContractPromise(registry.phactory, registry.remotePubkey, _pair, contractId, systemContractId)
  }

  // constructor(api: ApiPromise, registry: OnChainRegistry, contractId: string | AccountId, pair: KeyringPair, systemContractId: string) {
  constructor(
    phactory: pruntime_rpc.PhactoryAPI,
    remotePubkey: string,
    pair: KeyringPair,
    contractId: string | AccountId,
    systemContractId?: string | AccountId
  ) {
    this.#phactory = phactory
    this.#remotePubkey = remotePubkey
    this.#address = phalaTypes.createType('AccountId', contractId)
    this.#pair = pair
    this.#systemContractId = systemContractId
  }

  protected async getSidevmQueryContext(): Promise<SidevmQueryContext> {
    const cert = await signCertificate({ pair: this.#pair })
    const address = this.#address as AccountId
    const phactory = this.#phactory
    const remotePubkey = this.#remotePubkey
    return { phactory, remotePubkey, address, cert } as const
  }

  /**
   * This method call `GetLog` directly, and return the raw result. All encapulation methods is based on this method.
   * We keep this one for testing purpose, and it should less likely to be used in production.
   */
  async getLogRaw(query: { from?: number; count?: number; contract?: string | AccountId } = {}) {
    const ctx = await this.getSidevmQueryContext()
    const unsafeRunSidevmQuery = sidevmQueryWithReader(ctx)
    return await unsafeRunSidevmQuery<{ records: SerInnerMessage[]; next: number }>({
      action: 'GetLog',
      from: query.from,
      count: query.count,
      contract: query.contract,
    })
  }

  get address() {
    return this.#address
  }

  /**
   * Get log records from the contract.
   *
   * @deprecated
   */
  async getLog(contract: AccountId | string, from: number = 0, count: number = 100): Promise<GetLogResponse> {
    const result = await this.getLogRaw({
      contract,
      from,
      count,
    })
    return { records: postProcessLogRecord(result.records), next: result.next } as const
  }

  /**
   * Get the logger info. It use for probe the logger status.
   */
  async getInfo(): Promise<LogServerInfo> {
    const ctx = await this.getSidevmQueryContext()
    const unsafeRunSidevmQuery = sidevmQueryWithReader(ctx)
    return await unsafeRunSidevmQuery({ action: 'GetInfo' })
  }

  /**
   * Fetching the log records, from the latest one back to the oldest one.
   */
  async tail(): Promise<GetLogResponse>
  async tail(counts: number): Promise<GetLogResponse>
  async tail(request: Partial<GetLogRequest>): Promise<GetLogResponse>
  async tail(counts: number, from: number): Promise<GetLogResponse>
  async tail(counts: number, request: Omit<GetLogRequest, 'from' | 'count'>): Promise<GetLogResponse>
  async tail(counts: number, from: number, request?: Omit<GetLogRequest, 'from' | 'count'>): Promise<GetLogResponse>
  async tail(...params: any[]): Promise<GetLogResponse> {
    const { abi, type, topic, nonce, ...request }: GetLogRequest = buildGetLogRequest(
      params,
      (x) => {
        if (!x.from) {
          return x.count ? -x.count : -10
        }
        return -(x.from + (x.count || 10))
      },
      () => ({ count: 10 })
    )
    const result = await this.getLogRaw(request)
    if (type) {
      if (Array.isArray(type)) {
        result.records = result.records.filter((record) => type.includes(record.type))
      } else if (type === 'Event' && topic) {
        const topicHash = getTopicHash(topic)
        result.records = result.records.filter((record) => record.type === type && record.topics[0] === topicHash)
      } else if (type === 'MessageOutput' && nonce) {
        result.records = result.records.filter((record) => record.type === type && record.nonce === nonce)
      } else {
        result.records = result.records.filter((record) => record.type === type)
      }
    }
    return { records: postProcessLogRecord(result.records, abi), next: result.next } as const
  }

  /**
   * Fetching the log records, from the oldest one to the latest one.
   */
  async head(): Promise<GetLogResponse>
  async head(counts: number): Promise<GetLogResponse>
  async head(request: Partial<GetLogRequest>): Promise<GetLogResponse>
  async head(counts: number, from: number): Promise<GetLogResponse>
  async head(counts: number, request: Omit<GetLogRequest, 'from' | 'count'>): Promise<GetLogResponse>
  async head(counts: number, from: number, request?: Omit<GetLogRequest, 'from' | 'count'>): Promise<GetLogResponse>
  async head(...params: any[]): Promise<GetLogResponse> {
    const { abi, type, topic, nonce, ...request }: GetLogRequest = buildGetLogRequest(
      params,
      (x) => x.from || 0,
      () => ({ from: 0, count: 10 })
    )
    const result = await this.getLogRaw(request)
    if (type) {
      if (Array.isArray(type)) {
        result.records = result.records.filter((record) => type.includes(record.type))
      } else if (type === 'Event' && topic) {
        const topicHash = getTopicHash(topic)
        result.records = result.records.filter((record) => record.type === type && record.topics[0] === topicHash)
      } else if (type === 'MessageOutput' && nonce) {
        result.records = result.records.filter((record) => record.type === type && record.nonce === nonce)
      } else {
        result.records = result.records.filter((record) => record.type === type)
      }
    }
    return { records: postProcessLogRecord(result.records, abi), next: result.next } as const
  }

  /// System Contract Related.

  setSystemContract(contract: PinkContractPromise | string) {
    if (typeof contract === 'string') {
      this.#systemContractId = contract
    } else {
      this.#systemContractId = contract.address?.toHex()
    }
  }

  async headSystemLog(counts: number = 10, from: number = 0) {
    if (!this.#systemContractId) {
      throw new Error('System contract ID is not set.')
    }
    const contract =
      typeof this.#systemContractId === 'string' ? this.#systemContractId : this.#systemContractId.toHex()
    return this.head(counts, from, { contract })
  }

  async tailSystemLog(counts: number = 10, from: number = -10) {
    if (!this.#systemContractId) {
      throw new Error('System contract ID is not set.')
    }
    const contract =
      typeof this.#systemContractId === 'string' ? this.#systemContractId : this.#systemContractId.toHex()
    return this.tail(counts, from, { contract })
  }
}

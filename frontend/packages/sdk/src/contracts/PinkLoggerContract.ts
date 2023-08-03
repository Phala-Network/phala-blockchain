import type { ApiPromise } from '@polkadot/api'
import type { Text } from '@polkadot/types'
import type { Result } from '@polkadot/types-codec'
import type { KeyringPair } from '@polkadot/keyring/types'
import type { AccountId } from '@polkadot/types/interfaces'
import type { OnChainRegistry } from '../OnChainRegistry'
import type { InkResponse } from '../types'

import { Keyring } from '@polkadot/api'
import { hexAddPrefix, hexToU8a, stringToHex, hexToString } from '@polkadot/util'
import { sr25519Agree } from "@polkadot/wasm-crypto";

import { PinkContractPromise, pinkQuery } from './PinkContract'
import { ContractInitialError } from './Errors'
import { type CertificateData, generatePair, signCertificate } from '../certificate'
import { randomHex } from '../lib/hex'
import { phalaTypes } from '../options'
import { type pruntime_rpc } from '../proto'


interface GetLogRequest {
  contract?: string
  from: number
  count: number
  block_number?: number
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

export interface SerMessageMessageOutput {
  type: 'MessageOutput'
  sequence: number
  blockNumber: number
  origin: string
  contract: string
  nonce: string
  output: string
}

export interface SerMessageQueryIn {
  type: 'QueryIn'
  sequence: number
  user: string
}

export interface SerMessageTooLarge {
  type: 'TooLarge'
}

export type SerMessage = SerMessageLog | SerMessageEvent | SerMessageMessageOutput | SerMessageQueryIn | SerMessageTooLarge

export interface LogServerInfo {
  programVersion: [ number, number, number ]
  nextSequence: number
  memoryCapacity: number
  memoryUsage: number
  currentNumberOfRecords: number
  estimatedCurrentSize: number
}


function InkQuery(contractId: AccountId, { sidevmMessage }: { sidevmMessage?: Record<string, any> } = {}) {
  const head = {
    nonce: hexAddPrefix(randomHex(32)),
    id: contractId,
  }
  let data: Record<string, string> = {}
  if (sidevmMessage) {
    data['SidevmMessage'] = stringToHex(JSON.stringify(sidevmMessage))
  } else {
    throw new Error('InkQuery construction failed: sidevmMessage is required.')
  }
  return phalaTypes.createType('InkQuery', { head, data })
}

interface SidevmQueryContext {
  api: ApiPromise
  phactory: pruntime_rpc.PhactoryAPI
  remotePubkey: string
  address: AccountId
  cert: CertificateData
}

function sidevmQueryWithReader({ api, phactory, remotePubkey, address, cert }: SidevmQueryContext) {
  return async function unsafeRunSidevmQuery<T>(sidevmMessage: Record<string, any>): Promise<T> {
    const [sk, pk] = generatePair()
    const encodedQuery = InkQuery(address, { sidevmMessage })
    const queryAgreementKey = sr25519Agree(hexToU8a(hexAddPrefix(remotePubkey)), sk)
    const response = await pinkQuery(api, phactory, pk, queryAgreementKey, encodedQuery.toHex(), cert)
    const inkResponse = api.createType<InkResponse>('InkResponse', response)
    if (inkResponse.result.isErr) {
      let error = `[${inkResponse.result.asErr.index}] ${inkResponse.result.asErr.type}`
      if (inkResponse.result.asErr.type === 'RuntimeError') {
        error = `${error}: ${inkResponse.result.asErr.value}`
      }
      throw new Error(error)
    }
    const payload = inkResponse.result.asOk.asInkMessageReturn.toString()
    if (payload.substring(0, 2) === '0x') {
      return JSON.parse(hexToString(payload))
    }
    return JSON.parse(payload)
  }
}

function buildGetLogRequest(params: any[], getFrom: (x: Partial<GetLogRequest>) => number, getDefaults: () => Partial<GetLogRequest>): GetLogRequest {
  let request = getDefaults()
  switch (params.length) {
    case 0:
      break

    case 1:
      if (typeof params[0] === 'number') {
        request.count = params[0]
        request.from = getFrom(request)
      } else {
        request = { ...params[0], ...request, }
      }
      break

    case 2:
      request.count = params[0]
      request.from = getFrom(request)
      if (typeof params[1] === 'number') {
        request.from = params[1]
      } else {
        request = { ...params[0], ...request, }
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


export class PinkLoggerContractPromise {
  #api: ApiPromise
  #phatRegistry: OnChainRegistry
  #address: AccountId;
  #pair: KeyringPair
  #systemContractId: string | undefined

  static async create(api: ApiPromise, registry: OnChainRegistry, systemContract: PinkContractPromise, pair?: KeyringPair): Promise<PinkLoggerContractPromise> {
    let _pair: KeyringPair | undefined = pair
    if (!_pair) {
      const keyring = new Keyring({ type: 'sr25519' });
      _pair = keyring.addFromUri('//Alice')
    }
    const cert = await signCertificate({ pair: _pair })
    const { output } = await systemContract.query['system::getDriver'](_pair.address, { cert }, 'PinkLogger')
    const contractId = (output as Result<Text, any>).asOk.toHex()
    if (!contractId) {
      throw new ContractInitialError('No PinkLogger contract registered in the cluster.')
    }
    const systemContractId = systemContract.address?.toHex()
    return new PinkLoggerContractPromise(api, registry, contractId, pair, systemContractId)
  }

  constructor(api: ApiPromise, registry: OnChainRegistry, contractId: string | AccountId, pair?: KeyringPair, systemContractId?: string) {
    this.#api = api
    this.#phatRegistry = registry
    this.#address = api.createType('AccountId', contractId);
    if (!pair) {
      const keyring = new Keyring({ type: 'sr25519' });
      this.#pair = keyring.addFromUri('//Alice')
    } else {
      this.#pair = pair
    }
    this.#systemContractId = systemContractId
  }

  protected async getSidevmQueryContext(): Promise<SidevmQueryContext> {
    if (!this.#phatRegistry.phactory || !this.#phatRegistry.remotePubkey) {
      throw new Error('No Pruntime connection found.')
    }
    const cert = await signCertificate({ pair: this.#pair });
    const api = this.#api as ApiPromise
    const address = this.#address as AccountId
    const phactory = this.#phatRegistry.phactory
    const remotePubkey = this.#phatRegistry.remotePubkey
    return { api, phactory, remotePubkey, address, cert } as const
  }

  async getLog(contract: AccountId | string, from: number = 0, count: number = 100): Promise<SerMessage[]> {
    const ctx = await this.getSidevmQueryContext()
    const unsafeRunSidevmQuery = sidevmQueryWithReader(ctx)
    return await unsafeRunSidevmQuery({ action: 'GetLog', contract, from, count })
  }

  async getInfo(): Promise<LogServerInfo> {
    const ctx = await this.getSidevmQueryContext()
    const unsafeRunSidevmQuery = sidevmQueryWithReader(ctx)
    return await unsafeRunSidevmQuery({ action: 'GetInfo' })
  }

  async tail(): Promise<SerMessage[]>;
  async tail(counts: number): Promise<SerMessage[]>;
  async tail(filters: Pick<GetLogRequest, 'contract' | 'block_number'>): Promise<SerMessage[]>;
  async tail(counts: number, filters: Pick<GetLogRequest, 'contract' | 'block_number'>): Promise<SerMessage[]>;
  async tail(counts: number, from: number, filters?: Pick<GetLogRequest, 'contract' | 'block_number'>): Promise<SerMessage[]>;
  async tail(...params: any[]): Promise<SerMessage[]> {
    const request: GetLogRequest = buildGetLogRequest(
      params,
      (x) => x.count ? -x.count : -10,
      () => ({ from: -10, count: 10 })
    )
    const ctx = await this.getSidevmQueryContext()
    const unsafeRunSidevmQuery = sidevmQueryWithReader(ctx)
    return await unsafeRunSidevmQuery({ action: 'GetLog', ...request })
  }

  async head(): Promise<SerMessage[]>;
  async head(counts: number): Promise<SerMessage[]>;
  async head(filters: Pick<GetLogRequest, 'contract' | 'block_number'>): Promise<SerMessage[]>;
  async head(counts: number, filters: Pick<GetLogRequest, 'contract' | 'block_number'>): Promise<SerMessage[]>;
  async head(counts: number, from: number, filters?: Pick<GetLogRequest, 'contract' | 'block_number'>): Promise<SerMessage[]>;
  async head(...params: any[]): Promise<SerMessage[]> {
    const request: GetLogRequest = buildGetLogRequest(params, () => 0, () => ({ from: 0, count: 10 }))
    const ctx = await this.getSidevmQueryContext()
    const unsafeRunSidevmQuery = sidevmQueryWithReader(ctx)
    return await unsafeRunSidevmQuery({ action: 'GetLog', ...request })
  }

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
    return this.head(counts, from, { contract: this.#systemContractId })
  }

  async tailSystemLog(counts: number = 10, from: number = -10) {
    if (!this.#systemContractId) {
      throw new Error('System contract ID is not set.')
    }
    return this.tail(counts, from, { contract: this.#systemContractId })
  }
}

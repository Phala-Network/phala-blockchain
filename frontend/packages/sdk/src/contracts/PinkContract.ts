import type { ApiPromise } from '@polkadot/api'
import { toPromiseMethod } from '@polkadot/api'
import type { ApiBase } from '@polkadot/api/base'
import type { SubmittableExtrinsic } from '@polkadot/api/submittable/types'
import type { DecorateMethod, Signer as InjectedSigner } from '@polkadot/api/types'
import { Abi } from '@polkadot/api-contract/Abi'
import { ContractSubmittableResult } from '@polkadot/api-contract/base/Contract'
import type { ContractCallResult, ContractCallSend, MessageMeta } from '@polkadot/api-contract/base/types'
import { convertWeight, withMeta } from '@polkadot/api-contract/base/util'
import type { AbiMessage, ContractCallOutcome, ContractOptions, DecodedEvent } from '@polkadot/api-contract/types'
import { applyOnEvent } from '@polkadot/api-contract/util'
import type { Bytes, Null, Result, Struct, Text, Vec, u8 } from '@polkadot/types'
import type { AccountId, ContractExecResult, EventRecord } from '@polkadot/types/interfaces'
import type { Codec, IEnum, IKeyringPair, ISubmittableResult, Registry } from '@polkadot/types/types'
import { BN, BN_ZERO, hexAddPrefix, hexToU8a, isHex } from '@polkadot/util'
import { sr25519Agreement, sr25519PairFromSeed } from '@polkadot/util-crypto'
import { from } from 'rxjs'
import type { OnChainRegistry } from '../OnChainRegistry'
import { type Provider } from '../providers/types'
import type { CertificateData } from '../pruntime/certificate'
import { EncryptedInkCommand, InkQueryMessage, PlainInkCommand } from '../pruntime/coders'
import { pinkQuery } from '../pruntime/pinkQuery'
import type { AbiLike, AnyProvider, FrameSystemAccountInfo } from '../types'
import assert from '../utils/assert'
import { BN_MAX_SUPPLY } from '../utils/constants'
import { randomHex } from '../utils/hex'
import signAndSend from '../utils/signAndSend'

export type PinkContractCallOutcome<ResultType> = {
  output: ResultType
} & Omit<ContractCallOutcome, 'output'>

export interface ILooseResult<O, E extends Codec = Codec> extends IEnum {
  readonly asErr: E
  readonly asOk: O
  readonly isErr: boolean
  readonly isOk: boolean
}

export interface PinkContractQuery<
  TParams extends Array<any> = any[],
  DefaultResultType = Codec,
  DefaultErrType extends Codec = Codec,
> extends MessageMeta {
  <ResultType = DefaultResultType, ErrType extends Codec = DefaultErrType>(
    origin: string | AccountId | Uint8Array,
    options: PinkContractQueryOptions,
    ...params: TParams
  ): ContractCallResult<'promise', PinkContractCallOutcome<ILooseResult<ResultType, ErrType>>>
}

export interface MapMessageInkQuery {
  [message: string]: PinkContractQuery
}

export interface PinkContractOptions extends ContractOptions {
  // Deposit to caller's cluster account to pay the gas fee. It useful when caller's cluster account
  // won't have enough funds and eliminate one `transferToCluster` transaction.
  deposit?: bigint | BN | string | number

  //
  plain?: boolean

  nonce?: `0x${string}`
}

interface SendOptions {
  cert?: CertificateData
}

export type PinkContractSendOptions =
  | (PinkContractOptions & SendOptions & { address: string | AccountId; signer: InjectedSigner })
  | (PinkContractOptions & SendOptions & { pair: IKeyringPair })
  | (PinkContractOptions & SendOptions & { provider: Provider })

export interface PinkContractTx<TParams extends Array<any> = any[]> extends MessageMeta {
  (options: PinkContractOptions, ...params: TParams): SubmittableExtrinsic<'promise'>
}

export interface MapMessageTx {
  [message: string]: PinkContractTx
}

export interface PinkContractQueryOptions {
  cert: CertificateData
  salt?: string
  estimating?: boolean
  deposit?: number | bigint | BN | string
  transfer?: number | bigint | BN | string
}

class PinkContractSubmittableResult extends ContractSubmittableResult {
  readonly #registry: OnChainRegistry

  #isFinalized: boolean = false
  #contract: PinkContractPromise
  #message: AbiMessage
  #nonce: string

  constructor(
    registry: OnChainRegistry,
    contract: PinkContractPromise,
    nonce: string,
    message: AbiMessage,
    result: ISubmittableResult,
    contractEvents?: DecodedEvent[]
  ) {
    super(result, contractEvents)
    this.#registry = registry
    this.#contract = contract
    this.#message = message
    this.#nonce = nonce
  }

  get nonce() {
    return this.#nonce
  }

  protected async throwsOnErrorLog(chainHeight: number): Promise<void> {
    const logger = this.#registry.loggerContract
    if (!logger) {
      return
    }
    const { records } = await logger.tail(10, { contract: this.#contract.address.toHex() })
    const sinceSubmitted = records.filter(
      (i) => (i.type === 'Log' || i.type === 'MessageOutput') && i.blockNumber >= chainHeight
    )
    sinceSubmitted.reverse()
    sinceSubmitted.forEach((msg) => {
      if (msg.type === 'MessageOutput' && 'ok' in msg.output.result) {
        const { ok } = msg.output.result
        if (ok.flags.length && ok.flags[0] === 'Revert' && this.#message.returnType) {
          const returns = this.#contract.abi.registry.createType(this.#message.returnType.type, hexToU8a(ok.data))
          throw new Error(JSON.stringify(returns.toHuman()))
        }
      } else if (msg.type === 'Log' && msg.execMode === 'transaction') {
        throw new Error(msg.message)
      }
    })
  }

  async waitFinalized(
    predicate?: () => Promise<boolean>,
    options?: { timeout?: number; blocks?: number }
  ): Promise<void> {
    if (this.#isFinalized) {
      return
    }
    if (!this.isInBlock && !this.isFinalized) {
      throw new Error('Contract transaction submit failed.')
    }
    const codeHash = this.status.asInBlock.toString()
    const block = await this.#registry.api.rpc.chain.getBlock(codeHash)
    const chainHeight = block.block.header.number.toNumber()
    const t0 = new Date().getTime()
    const timeout = options?.timeout ?? 120_000
    const blocks = options?.blocks ?? 10
    if (!predicate) {
      while (true) {
        await this.throwsOnErrorLog(chainHeight)
        const { blocknum: currentHeight } = await this.#registry.phactory.getInfo({})
        if (currentHeight > chainHeight) {
          this.#isFinalized = true
          return
        }
        if (currentHeight - blocks > chainHeight) {
          throw new Error('Timeout')
        }
        if (new Date().getTime() - t0 > timeout) {
          throw new Error('Timeout')
        }
        await new Promise((resolve) => setTimeout(resolve, 1_000))
      }
    } else {
      while (true) {
        await this.throwsOnErrorLog(chainHeight)
        const { blocknum: currentHeight } = await this.#registry.phactory.getInfo({})
        const isOk = await predicate()
        if (isOk) {
          this.#isFinalized = true
          return
        }
        if (currentHeight - blocks > chainHeight) {
          throw new Error('Timeout')
        }
        if (new Date().getTime() - t0 > timeout) {
          throw new Error('Timeout')
        }
        await new Promise((resolve) => setTimeout(resolve, 1_000))
      }
    }
  }
}

export interface PinkContractSend<TParams extends Array<any> = any[]> extends MessageMeta {
  (options: PinkContractSendOptions, ...params: TParams): Promise<PinkContractSubmittableResult>
}

export interface MapMessageSend {
  [message: string]: PinkContractSend
}

interface InkQueryOk extends IEnum {
  readonly isInkMessageReturn: boolean
  readonly asInkMessageReturn: Vec<u8>
}

interface InkQueryError extends IEnum {
  readonly isBadOrigin: boolean
  readonly asBadOrigin: Null

  readonly isRuntimeError: boolean
  readonly asRuntimeError: Text

  readonly isSidevmNotFound: boolean
  readonly asSidevmNotFound: Null

  readonly isNoResponse: boolean
  readonly asNoResponse: Null

  readonly isServiceUnavailable: boolean
  readonly asServiceUnavailable: Null

  readonly isTimeout: boolean
  readonly asTimeout: Null
}

interface InkResponse extends Struct {
  nonce: Text
  result: Result<InkQueryOk, InkQueryError>
}

//
//

interface ProxyCallbackOptions {
  path: string[]
  args: unknown[]
}
type ProxyCallback = (opts: ProxyCallbackOptions) => unknown

const noop = () => {
  /* noop */
}

function createInnerProxy<TProxied = unknown>(callback: ProxyCallback, path: string[]): TProxied {
  const proxy = new Proxy(noop, {
    get(_obj, key) {
      if (typeof key !== 'string' || key === 'then') {
        return undefined
      }
      return createInnerProxy(callback, [...path, key]) as TProxied
    },
    apply(_target, _thisArg, args) {
      const isApply = path[path.length - 1] === 'apply'
      return callback({
        path: isApply ? path.slice(0, -1) : path,
        args: isApply ? args[1] : args,
      })
    },
  })
  return proxy as TProxied
}

//
//

type QueryProxyArgs<TParams extends Array<any> = any[]> = {
  args: TParams
  origin?: string | AccountId | Uint8Array
} & Partial<PinkContractQueryOptions> &
  unknown

type QueryProxy<T, TParams extends Array<any> = any[], ResultType = Codec, ErrType extends Codec = Codec> = {
  [k in keyof T]: <TOverrideResultType = ResultType>(
    args?: QueryProxyArgs<TParams>
  ) => ContractCallResult<'promise', PinkContractCallOutcome<ILooseResult<TOverrideResultType, ErrType>>>
}

type TxProxyArgs<TParams extends Array<any> = any[]> = {
  args: TParams
} & Partial<PinkContractSendOptions> &
  unknown

type TxProxy<T, TParams extends Array<any> = any[]> = {
  [k in keyof T]: (args?: TxProxyArgs<TParams>) => Promise<PinkContractSubmittableResult>
}

type PinkQueryMap = Record<string, PinkContractQuery | Record<string, PinkContractQuery>>

type PinkCommandMap = Record<string, PinkContractTx | Record<string, PinkContractTx>>

//
//
//
export class PinkContractPromise<
  TQueries extends PinkQueryMap = PinkQueryMap,
  TTransactions extends PinkCommandMap = PinkCommandMap,
> {
  readonly abi: Abi
  readonly api: ApiBase<'promise'>
  readonly address: AccountId
  readonly contractKey: string
  readonly phatRegistry: OnChainRegistry

  protected readonly _decorateMethod: DecorateMethod<'promise'>

  readonly #query: MapMessageInkQuery = {}
  readonly #tx: MapMessageTx = {}

  protected _provider: AnyProvider | undefined = undefined

  constructor(
    api: ApiBase<'promise'>,
    phatRegistry: OnChainRegistry,
    abi: AbiLike,
    address: string | AccountId,
    contractKey: string,
    provider?: AnyProvider
  ) {
    if (!api || !api.isConnected || !api.tx) {
      throw new Error('Your API has not been initialized correctly and is not connected to a chain')
    }
    if (!phatRegistry.isReady()) {
      throw new Error('Your phatRegistry has not been initialized correctly.')
    }

    this.abi = abi instanceof Abi ? abi : new Abi(abi, api.registry.getChainProperties())
    this.api = api
    this._decorateMethod = toPromiseMethod
    this.phatRegistry = phatRegistry

    this.address = this.registry.createType('AccountId', address)
    this.contractKey = contractKey

    this._provider = provider

    this.abi.messages.forEach((meta): void => {
      if (meta.isMutating) {
        this.#tx[meta.method] = withMeta(
          meta,
          (options: PinkContractOptions, ...params: unknown[]): SubmittableExtrinsic<'promise'> => {
            return this.#inkCommand(meta, options, params)
          }
        )
        this.#query[meta.method] = withMeta(
          meta,
          (
            origin: string | AccountId | Uint8Array,
            options: PinkContractQueryOptions,
            ...params: unknown[]
          ): ContractCallResult<'promise', ContractCallOutcome> => {
            return this.#inkQuery(true, meta, options, params).send(origin)
          }
        )
      } else {
        this.#query[meta.method] = withMeta(
          meta,
          (
            origin: string | AccountId | Uint8Array,
            options: PinkContractQueryOptions,
            ...params: unknown[]
          ): ContractCallResult<'promise', ContractCallOutcome> => {
            return this.#inkQuery(false, meta, options, params).send(origin)
          }
        )
      }
    })
  }

  get provider(): AnyProvider | undefined {
    return this._provider
  }

  set provider(provider: AnyProvider) {
    if (!provider || typeof provider.send !== 'function' || typeof provider.signCertificate !== 'function') {
      throw new Error('The provider implementation is not valid')
    }
    this._provider = provider
  }

  public withProvider(provider: AnyProvider): PinkContractPromise<TQueries, TTransactions> {
    this.provider = provider
    return this
  }

  private qProxyInstance: unknown = undefined

  public get q() {
    type TMethods = TQueries & TTransactions
    type TReMap = {
      [k in keyof TMethods]: TMethods[k] extends PinkContractQuery
        ? <ResultType extends Codec = Codec, TParams extends Array<any> = any[], ErrType extends Codec = Codec>(
            args?: QueryProxyArgs<TParams>
          ) => ContractCallResult<'promise', PinkContractCallOutcome<ILooseResult<ResultType, ErrType>>>
        : QueryProxy<TMethods[k]>
    } & unknown

    if (!this.qProxyInstance) {
      this.qProxyInstance = createInnerProxy<TReMap>(async ({ path, args: [arg] }) => {
        const { args = [], origin, ...options } = (arg || {}) as QueryProxyArgs
        const key = path.join('::')
        if (!this.provider) {
          throw new Error('The provider is not set')
        }
        let _origin = origin || this.provider.address
        if (!options.cert) {
          options.cert = await this.provider.signCertificate()
          _origin = options.cert.address
        }
        return await this.query[key](_origin, options as PinkContractQueryOptions, ...args)
      }, [])
    }
    return this.qProxyInstance as TReMap
  }

  private execProxyInstance: unknown = undefined

  public get exec() {
    type TReMap = {
      [k in keyof TTransactions]: TTransactions[k] extends PinkContractTx
        ? <TParams extends Array<any> = any[]>(args?: TxProxyArgs<TParams>) => Promise<PinkContractSubmittableResult>
        : TxProxy<TTransactions[k]>
    } & unknown

    if (!this.execProxyInstance) {
      this.execProxyInstance = createInnerProxy<TReMap>(async ({ path, args: [arg] }) => {
        const { args = [], ..._options } = (arg || {}) as TxProxyArgs
        const key = path.join('::')
        if (!this.provider) {
          throw new Error('The provider is not set')
        }
        const meta = this.abi.messages.filter((i) => i.method === key)
        if (!meta || !meta.length) {
          throw new Error('Method not found')
        }
        const options: PinkContractSendOptions = {
          cert: await this.provider.signCertificate(),
          address: this.provider.address,
          provider: this.provider,
          ..._options,
        }
        return this._send(key, options, ...args)
      }, [])
    }
    return this.execProxyInstance as TReMap
  }

  public get send() {
    return new Proxy(
      {},
      {
        get: (_target, prop, _receiver) => {
          const meta = this.abi.messages.filter((i) => i.method === prop)
          if (!meta || !meta.length) {
            throw new Error('Method not found')
          }
          return withMeta(meta[0], (options: PinkContractSendOptions, ...arags: unknown[]) => {
            return this._send(prop as string, options, ...arags)
          })
        },
      }
    ) as MapMessageSend
  }

  public get registry(): Registry {
    return this.api.registry
  }

  public get query(): TQueries & { [k in keyof TTransactions]: PinkContractQuery } {
    return this.#query as TQueries & { [k in keyof TTransactions]: PinkContractQuery }
  }

  public get tx(): MapMessageTx {
    return this.#tx as MapMessageTx
  }

  #inkQuery = (
    isEstimating: boolean,
    messageOrId: AbiMessage | string | number,
    options: PinkContractQueryOptions,
    params: unknown[]
  ): ContractCallSend<'promise'> => {
    const message = this.abi.findMessage(messageOrId)
    const api = this.api as ApiPromise

    if (!options.cert) {
      throw new Error(
        'You need to provide the `cert` parameter in the options to process a Phat Contract query. ' +
          'Please check the document for a more detailed code snippet: https://www.npmjs.com/package/@phala/sdk'
      )
    }

    const { cert } = options

    // Generate a keypair for encryption
    // NOTE: each instance only has a pre-generated pair now, it maybe better to generate a new keypair every time encrypting
    const seed = hexToU8a(hexAddPrefix(randomHex(32)))
    const pair = sr25519PairFromSeed(seed)
    const [sk, pk] = [pair.secretKey, pair.publicKey]

    const queryAgreementKey = sr25519Agreement(sk, hexToU8a(hexAddPrefix(this.phatRegistry.remotePubkey)))

    const inkQueryInternal = async (origin: string | AccountId | Uint8Array): Promise<ContractCallOutcome> => {
      if (typeof origin === 'string') {
        assert(origin === cert.address, 'origin must be the same as the certificate address')
      } else if (origin.hasOwnProperty('verify') && origin.hasOwnProperty('adddress')) {
        throw new Error('Contract query expected AccountId as first parameter but since we got signer object here.')
      } else {
        assert(origin.toString() === cert.address, 'origin must be the same as the certificate address')
      }

      const payload = InkQueryMessage(
        this.address,
        message.toU8a(params),
        options.deposit,
        options.transfer,
        options.estimating !== undefined ? !!options.estimating : isEstimating
      )
      const data = await pinkQuery(this.phatRegistry.phactory, pk, queryAgreementKey, payload.toHex(), cert)
      const inkResponse = api.createType<InkResponse>('InkResponse', data)
      if (inkResponse.result.isErr) {
        // @FIXME: not sure this is enough as not yet tested
        throw new Error(`InkResponse Error: ${inkResponse.result.asErr.toString()}`)
      }
      if (!inkResponse.result.asOk.isInkMessageReturn) {
        // @FIXME: not sure this is enough as not yet tested
        throw new Error(`Unexpected InkMessageReturn: ${inkResponse.result.asOk.toJSON()?.toString()}`)
      }
      const { debugMessage, gasConsumed, gasRequired, result, storageDeposit } = api.createType<ContractExecResult>(
        'ContractExecResult',
        inkResponse.result.asOk.asInkMessageReturn.toString()
      )
      return {
        debugMessage: debugMessage,
        gasConsumed: gasConsumed,
        gasRequired: gasRequired && !convertWeight(gasRequired).v1Weight.isZero() ? gasRequired : gasConsumed,
        output:
          result.isOk && message.returnType
            ? this.abi.registry.createTypeUnsafe(
                message.returnType.lookupName || message.returnType.type,
                [result.asOk.data.toU8a(true)],
                { isPedantic: true }
              )
            : null,
        result,
        storageDeposit,
      }
    }

    return {
      send: this._decorateMethod((origin: string | AccountId | Uint8Array) => from(inkQueryInternal(origin))),
    }
  }

  #inkCommand = (
    messageOrId: AbiMessage | string | number,
    options: PinkContractOptions,
    params: unknown[]
  ): SubmittableExtrinsic<'promise'> => {
    options.nonce && assert(isHex(options.nonce) && options.nonce.length === 66, 'Invalid nonce provided')
    const nonce = options.nonce || hexAddPrefix(randomHex(32))
    const command = options.plain ? PlainInkCommand : EncryptedInkCommand
    const message = this.abi.findMessage(messageOrId)
    const payload = command(
      this.contractKey,
      message.toU8a(params),
      nonce,
      options.value,
      convertWeight(options.gasLimit || BN_ZERO).v2Weight,
      options.storageDepositLimit
    )
    return this.api.tx.phalaPhatContracts
      .pushContractMessage(this.address, payload.toHex(), options.deposit || BN_ZERO)
      .withResultTransform((result: ISubmittableResult) => {
        return new PinkContractSubmittableResult(
          this.phatRegistry,
          this,
          nonce,
          message,
          result,
          applyOnEvent(result, ['ContractEmitted', 'ContractExecution'], (records: EventRecord[]) => {
            return records
              .map(
                ({
                  event: {
                    data: [, data],
                  },
                }): DecodedEvent | null => {
                  try {
                    return this.abi.decodeEvent(data as Bytes)
                  } catch (error) {
                    console.error(`Unable to decode contract event: ${(error as Error).message}`)
                    return null
                  }
                }
              )
              .filter((decoded): decoded is DecodedEvent => !!decoded)
          })
        )
      })
  }

  private async _send(messageOrId: string, options: PinkContractSendOptions, ...args: unknown[]) {
    const { cert: userCert, ...rest } = options
    const txOptions: PinkContractOptions = {
      gasLimit: options.gasLimit,
      value: options.value,
      storageDepositLimit: options.storageDepositLimit,
      plain: options.plain,
      nonce: options.nonce,
    }

    const tx = this.#tx[messageOrId]
    if (!tx) {
      throw new Error(`Message not found: ${messageOrId}`)
    }

    const address = 'provider' in rest ? rest.provider.address : 'signer' in rest ? rest.address : rest.pair.address
    const cert = userCert || (await this.phatRegistry.getAnonymousCert())

    const estimate = this.#query[messageOrId]
    if (!estimate) {
      throw new Error(`Message not found: ${messageOrId}`)
    }

    const { gasPrice } = this.phatRegistry.clusterInfo ?? {}
    if (!gasPrice) {
      throw new Error('No Gas Price or deposit Per Byte from cluster info.')
    }

    const [clusterBalance, onchainBalance, { gasRequired, storageDeposit }] = await Promise.all([
      this.phatRegistry.getClusterBalance(address),
      this.api.query.system.account<FrameSystemAccountInfo>(address),
      estimate(cert.address, { cert, deposit: BN_MAX_SUPPLY }, ...args),
    ])

    // calculate the total costs
    const gasLimit = gasRequired.refTime.toBn()
    const storageDepositFee = storageDeposit.isCharge ? storageDeposit.asCharge.toBn() : BN_ZERO
    const minRequired = gasLimit.mul(gasPrice).add(storageDepositFee)

    // Auto deposit.
    if (clusterBalance.free.lt(minRequired)) {
      const deposit = minRequired.sub(clusterBalance.free)
      if (onchainBalance.data.free.lt(deposit)) {
        throw new Error(`Not enough balance to pay for gas and storage deposit: ${minRequired.toNumber()}`)
      }
      txOptions.deposit = deposit
    }

    // gasLimit is required, so we set it to the estimated value if not provided.
    if (!txOptions.gasLimit) {
      txOptions.gasLimit = gasRequired.refTime.toBn()
    }

    if ('provider' in rest) {
      options.nonce && assert(isHex(options.nonce) && options.nonce.length === 66, 'Invalid nonce provided')
      const nonce = options.nonce || hexAddPrefix(randomHex(32))
      return await rest.provider.send(tx(txOptions, ...args), (result: ISubmittableResult) => {
        return new PinkContractSubmittableResult(
          this.phatRegistry,
          this,
          nonce,
          this.abi.findMessage(messageOrId),
          result,
          applyOnEvent(result, ['ContractEmitted', 'ContractExecution'], (records: EventRecord[]) => {
            return records
              .map(
                ({
                  event: {
                    data: [, data],
                  },
                }): DecodedEvent | null => {
                  try {
                    return this.abi.decodeEvent(data as Bytes)
                  } catch (error) {
                    console.error(`Unable to decode contract event: ${(error as Error).message}`)
                    return null
                  }
                }
              )
              .filter((decoded): decoded is DecodedEvent => !!decoded)
          })
        )
      })
    } else if ('signer' in rest) {
      return await signAndSend(tx(txOptions, ...args), rest.address, rest.signer)
    } else {
      return await signAndSend(tx(txOptions, ...args), rest.pair)
    }
  }
}

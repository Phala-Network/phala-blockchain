import { SubmittableResult, toPromiseMethod } from '@polkadot/api'
import { ApiBase } from '@polkadot/api/base'
import type { SubmittableExtrinsic } from '@polkadot/api/submittable/types'
import type { ApiTypes, DecorateMethod, Signer as InjectedSigner } from '@polkadot/api/types'
import { Abi } from '@polkadot/api-contract/Abi'
import type { ContractCallResult, MessageMeta } from '@polkadot/api-contract/base/types'
import { createBluePrintTx, withMeta } from '@polkadot/api-contract/base/util'
import type { AbiConstructor, BlueprintOptions, ContractCallOutcome } from '@polkadot/api-contract/types'
import { type Option } from '@polkadot/types'
import type { AccountId, ContractInstantiateResult, Hash } from '@polkadot/types/interfaces'
import type { IKeyringPair, ISubmittableResult } from '@polkadot/types/types'
import { BN, BN_ZERO, hexAddPrefix, hexToU8a, isUndefined } from '@polkadot/util'
import { sr25519Agree, sr25519KeypairFromSeed } from '@polkadot/wasm-crypto'
import { from } from 'rxjs'
import type { OnChainRegistry } from '../OnChainRegistry'
import { phalaTypes } from '../options'
import type { CertificateData } from '../pruntime/certificate'
import { InkQueryInstantiate } from '../pruntime/coders'
import { pinkQuery } from '../pruntime/pinkQuery'
import type { AbiLike, FrameSystemAccountInfo, InkQueryError, InkResponse } from '../types'
import assert from '../utils/assert'
import { BN_MAX_SUPPLY } from '../utils/constants'
import { randomHex } from '../utils/hex'
import signAndSend from '../utils/signAndSend'
import { PinkContractPromise } from './PinkContract'

export interface PinkContractInstantiateCallOutcome extends ContractCallOutcome {
  salt: string
}

interface ContractInkQuery<ApiType extends ApiTypes> extends MessageMeta {
  (
    origin: string | AccountId | Uint8Array,
    ...params: unknown[]
  ): ContractCallResult<ApiType, PinkContractInstantiateCallOutcome>
}

interface MapMessageInkQuery<ApiType extends ApiTypes> {
  [message: string]: ContractInkQuery<ApiType>
}

interface PinkContractInstantiateResult extends ContractInstantiateResult {
  salt: string
}

export interface PinkInstantiateQueryOptions {
  cert: CertificateData
  salt?: string
  transfer?: bigint | string | number | BN
  deposit?: bigint | string | number | BN
}

export interface PinkBlueprintOptions extends BlueprintOptions {
  // Deposit to caller's cluster account to pay the gas fee. It useful when caller's cluster account
  // won't have enough funds and eliminate one `transferToCluster` transaction.
  deposit?: bigint | BN | string | number
}

export interface PinkBlueprintDeploy<ApiType extends ApiTypes> extends MessageMeta {
  (options: PinkBlueprintOptions, ...params: unknown[]): SubmittableExtrinsic<ApiType, PinkBlueprintSubmittableResult>
}

export interface PinkMapConstructorExec<ApiType extends ApiTypes> {
  [message: string]: PinkBlueprintDeploy<ApiType>
}

interface SendOptions {
  cert?: CertificateData
}

export type PinkBlueprintSendOptions =
  | (PinkBlueprintOptions & SendOptions & { address: string | AccountId; signer: InjectedSigner })
  | (PinkBlueprintOptions & SendOptions & { pair: IKeyringPair })

export interface PinkBlueprintSend<TParams extends Array<any> = any[]> extends MessageMeta {
  (options: PinkBlueprintSendOptions, ...params: TParams): Promise<PinkBlueprintSubmittableResult>
}

interface MapBlueprintSend {
  [message: string]: PinkBlueprintSend
}

export class PinkBlueprintSubmittableResult extends SubmittableResult {
  readonly registry: OnChainRegistry
  readonly abi: Abi
  readonly contractId?: string

  #isFinalized: boolean = false
  #contract?: PinkContractPromise

  constructor(result: ISubmittableResult, abi: Abi, registry: OnChainRegistry, contractId?: string) {
    super(result)

    this.registry = registry
    this.abi = abi
    this.contractId = contractId
  }

  async waitFinalized(timeout: number = 120_000) {
    if (this.#isFinalized) {
      return
    }

    if (this.isInBlock || this.isFinalized) {
      let contractId: string | undefined
      for (const event of this.events) {
        if (event.event.method === 'Instantiating') {
          // tired of TS complaining about the type of event.event.data.contract
          // @ts-ignore
          contractId = event.event.data.contract.toString()
          break
        }
      }
      if (!contractId) {
        throw new Error('Failed to find contract ID in events, maybe instantiate failed.')
      }
      const logger = this.registry.loggerContract

      const t0 = new Date().getTime()
      while (true) {
        if (logger) {
          const { records } = await logger.tail(10, { contract: contractId })
          if (
            records.length > 0 &&
            records[0].type === 'Log' &&
            records[0].execMode === 'transaction' &&
            records[0].message.indexOf('instantiate failed') !== -1
          ) {
            throw new Error(records[0].message)
          }
        }

        const result1 = (await this.registry.api.query.phalaPhatContracts.clusterContracts(
          this.registry.clusterId
        )) as unknown as Text[]
        const contractIds = result1.map((i) => i.toString())
        if (contractIds.indexOf(contractId) !== -1) {
          const result2 = (await this.registry.api.query.phalaRegistry.contractKeys(
            contractId
          )) as unknown as Option<any>
          if (result2.isSome) {
            this.#isFinalized = true
            if (this.contractId) {
              const contractKey = await this.registry.getContractKeyOrFail(this.contractId)
              this.#contract = new PinkContractPromise(
                this.registry.api,
                this.registry,
                this.abi,
                this.contractId,
                contractKey
              )
            }
            return
          }
        }

        const t1 = new Date().getTime()
        if (t1 - t0 > timeout) {
          throw new Error('Timeout')
        }
        await new Promise((resolve) => setTimeout(resolve, 1000))
      }
    }
    throw new Error(`instantiate failed for ${this.abi.info.source.wasmHash.toString()}`)
  }

  get contract() {
    if (!this.#contract) {
      throw new Error('contract is not ready yet, please call waitFinalized first')
    }
    return this.#contract!
  }
}

export class PinkBlueprintPromise {
  readonly abi: Abi
  readonly api: ApiBase<'promise'>
  readonly phatRegistry: OnChainRegistry

  protected readonly _decorateMethod: DecorateMethod<'promise'>

  /**
   * @description The on-chain code hash for this blueprint
   */
  readonly codeHash: Hash

  readonly #query: MapMessageInkQuery<'promise'> = {}
  readonly #tx: PinkMapConstructorExec<'promise'> = {}

  constructor(
    api: ApiBase<'promise'>,
    phatRegistry: OnChainRegistry,
    abi: AbiLike,
    codeHash: string | Hash | Uint8Array
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

    this.codeHash = phalaTypes.createType('Hash', codeHash)

    this.abi.constructors.forEach((meta): void => {
      if (isUndefined(this.#tx[meta.method])) {
        this.#tx[meta.method] = createBluePrintTx(meta, (o, p) =>
          this.#deploy(meta, o, p)
        ) as PinkBlueprintDeploy<'promise'>
        this.#query[meta.method] = withMeta(
          meta,
          (
            origin: string | AccountId | Uint8Array,
            options: PinkInstantiateQueryOptions,
            ...params: unknown[]
          ): ContractCallResult<'promise', PinkContractInstantiateCallOutcome> =>
            this.#estimateGas(meta, options, params).send(origin)
        )
      }
    })
  }

  public get tx(): PinkMapConstructorExec<'promise'> {
    return this.#tx
  }

  public get query(): MapMessageInkQuery<'promise'> {
    return this.#query
  }

  public get send() {
    return new Proxy(
      {},
      {
        get: (_target, prop, _receiver) => {
          const meta = this.abi.constructors.filter((i) => i.method === prop)
          if (!meta || !meta.length) {
            throw new Error('Method not found')
          }
          return withMeta(meta[0], (options: PinkBlueprintSendOptions, ...arags: unknown[]) => {
            return this.#send(prop as string, options, ...arags)
          })
        },
      }
    ) as MapBlueprintSend
  }

  #deploy = (
    constructorOrId: AbiConstructor | string | number,
    { gasLimit = BN_ZERO, storageDepositLimit = null, value = BN_ZERO, deposit = BN_ZERO, salt }: PinkBlueprintOptions,
    params: unknown[]
  ) => {
    if (!salt) {
      salt = randomHex(4)
    }
    const codeHash = this.abi.info.source.wasmHash.toString()
    return this.api.tx.phalaPhatContracts
      .instantiateContract(
        { WasmCode: codeHash },
        this.abi.findConstructor(constructorOrId).toU8a(params),
        salt,
        this.phatRegistry.clusterId,
        value, // not transfer any token to the contract during initialization
        gasLimit,
        storageDepositLimit,
        deposit
      )
      .withResultTransform((result: ISubmittableResult) => {
        let maybeContactId: string | undefined
        const instantiateEvent = result.events.filter((i) => i.event.method === 'Instantiating')[0]
        if (instantiateEvent) {
          const contractId = (instantiateEvent.event.data as any).contract
          if (contractId) {
            maybeContactId = contractId.toString()
          }
        }
        return new PinkBlueprintSubmittableResult(result, this.abi, this.phatRegistry, maybeContactId)
      })
  }

  #estimateGas = (
    constructorOrId: AbiConstructor | string | number,
    options: PinkInstantiateQueryOptions,
    params: unknown[]
  ) => {
    // Generate a keypair for encryption
    // NOTE: each instance only has a pre-generated pair now, it maybe better to generate a new keypair every time encrypting
    const seed = hexToU8a(hexAddPrefix(randomHex(32)))
    const pair = sr25519KeypairFromSeed(seed)
    const [sk, pk] = [pair.slice(0, 64), pair.slice(64)]
    const { cert } = options

    const queryAgreementKey = sr25519Agree(hexToU8a(hexAddPrefix(this.phatRegistry.remotePubkey)), sk)

    const inkQueryInternal = async (origin: string | AccountId | Uint8Array) => {
      if (typeof origin === 'string') {
        assert(origin === cert.address, 'origin must be the same as the certificate address')
      } else if (origin.hasOwnProperty('verify') && origin.hasOwnProperty('adddress')) {
        throw new Error('Contract query expected AccountId as first parameter but since we got signer object here.')
      } else {
        assert(origin.toString() === cert.address, 'origin must be the same as the certificate address')
      }
      if (!this.phatRegistry.systemContract) {
        throw new Error(
          'The associated System Contract was not set up for You OnChainRegistry, causing the estimate gas to fail.'
        )
      }

      const salt = options.salt || randomHex(4)
      const payload = InkQueryInstantiate(
        this.phatRegistry.systemContract.address,
        this.abi.info.source.wasmHash,
        this.abi.findConstructor(constructorOrId).toU8a(params),
        salt,
        options.deposit,
        options.transfer
      )
      const rawResponse = await pinkQuery(this.phatRegistry.phactory, pk, queryAgreementKey, payload.toHex(), cert)
      const response = phalaTypes.createType<InkResponse>('InkResponse', rawResponse)
      if (response.result.isErr) {
        return phalaTypes.createType<InkQueryError>('InkQueryError', response.result.asErr.toHex())
      }
      const result = phalaTypes.createType<ContractInstantiateResult>(
        'ContractInstantiateResult',
        response.result.asOk.asInkMessageReturn.toHex()
      )
      ;(result as PinkContractInstantiateResult).salt = salt

      if (result.result.isErr) {
        const err = result.result.asErr
        if (err.isModule && err.asModule.index.toNumber() === 4) {
          const contractError = phalaTypes.createType('ContractError', result.result.asErr.asModule.error)
          throw new Error(`Estimation failed: ${contractError.toHuman()}`)
        }
        throw new Error('Estimation failed: ' + JSON.stringify(result.result.asErr.toHuman()))
      }

      return result
    }

    return {
      send: this._decorateMethod((origin: string | AccountId | Uint8Array) => from(inkQueryInternal(origin))),
    }
  }

  async #send(constructorOrId: string, options: PinkBlueprintSendOptions, ...args: unknown[]) {
    const { cert: userCert, ...rest } = options
    const txOptions: PinkBlueprintOptions = {
      gasLimit: options.gasLimit,
      value: options.value,
      storageDepositLimit: options.storageDepositLimit,
    }

    const tx = this.#tx[constructorOrId]
    if (!tx) {
      throw new Error(`Constructor not found: ${constructorOrId}`)
    }

    const address = 'signer' in rest ? rest.address : rest.pair.address
    const cert = userCert || (await this.phatRegistry.getAnonymousCert())
    const estimate = this.#query[constructorOrId]
    if (!estimate) {
      throw new Error(`Constructor not found: ${constructorOrId}`)
    }

    const { gasPrice } = this.phatRegistry.clusterInfo ?? {}
    if (!gasPrice) {
      throw new Error('No Gas Price or deposit Per Byte from cluster info.')
    }

    const [clusterBalance, onchainBalance, { gasRequired, storageDeposit }] = await Promise.all([
      this.phatRegistry.getClusterBalance(address),
      this.api.query.system.account<FrameSystemAccountInfo>(address),
      // We estimating the gas & storage deposit cost with deposit propose.
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
      txOptions.gasLimit = gasLimit
    }

    if ('signer' in rest) {
      return await signAndSend(tx(txOptions, ...args), rest.address, rest.signer)
    } else {
      return await signAndSend(tx(txOptions, ...args), rest.pair)
    }
  }
}

import { SubmittableResult, toPromiseMethod } from '@polkadot/api'
import { ApiBase } from '@polkadot/api/base'
import type { DecorateMethod, SubmittableExtrinsic } from '@polkadot/api/types'
import { Abi } from '@polkadot/api-contract/Abi'
import type { MessageMeta } from '@polkadot/api-contract/base/types'
import type { AbiConstructor } from '@polkadot/api-contract/types'
import type { KeyringPair } from '@polkadot/keyring/types'
import type { Result, bool } from '@polkadot/types'
import type { ISubmittableResult } from '@polkadot/types/types'
import { hexToU8a, isU8a, isWasm, u8aToHex } from '@polkadot/util'
import type { OnChainRegistry } from '../OnChainRegistry'
import type { Provider } from '../providers/types'
import type { CertificateData } from '../pruntime/certificate'
import type { AbiLike } from '../types'
import { toAbi } from '../utils/abi/toAbi'
import { PinkBlueprintPromise } from './PinkBlueprint'

export interface PinkCodeSendOptions {
  provider: Provider
}

export class InkCodeSubmittableResult extends SubmittableResult {
  readonly client: OnChainRegistry
  readonly abi: Abi
  readonly blueprint: PinkBlueprintPromise

  #isFinalized: boolean

  constructor(result: ISubmittableResult, abi: Abi, client: OnChainRegistry) {
    super(result)

    this.#isFinalized = false
    this.client = client
    this.abi = abi

    this.blueprint = new PinkBlueprintPromise(this.client, this.abi, this.abi.info.source.wasmHash)
  }

  async waitFinalized(): Promise<void>
  async waitFinalized(timeout: number): Promise<void>
  async waitFinalized(cert: CertificateData): Promise<void>
  async waitFinalized(cert: CertificateData, timeout: number): Promise<void>
  async waitFinalized(pair: KeyringPair, cert: CertificateData, timeout: number): Promise<void>
  async waitFinalized(...args: unknown[]): Promise<void> {
    if (this.#isFinalized) {
      return
    }
    let timeout = 10_000
    let cert, address
    args = args || []
    switch (args.length) {
      case 0:
        cert = await this.client.getAnonymousCert()
        address = cert.address
        break

      case 1:
        if (typeof args[0] === 'number') {
          timeout = args[0] as number
          cert = await this.client.getAnonymousCert()
          address = cert.address
        } else {
          cert = args[0] as CertificateData
          address = cert.address
        }
        break

      case 2:
        cert = args[0] as CertificateData
        address = cert.address
        timeout = args[1] as number
        break

      case 3:
        cert = args[1] as CertificateData
        address = cert.address
        timeout = args[2] as number
        break

      default:
        throw new Error('Invalid arguments')
    }

    if (this.isInBlock || this.isFinalized) {
      const system = this.client.systemContract!
      const codeHash = this.abi.info.source.wasmHash.toString()
      const t0 = new Date().getTime()
      while (true) {
        const { output } = await system.query['system::codeExists'](address, { cert }, codeHash, 'Ink')
        if (output && (output as Result<bool, any>).asOk.toPrimitive()) {
          this.#isFinalized = true
          return
        }
        const t1 = new Date().getTime()
        if (t1 - t0 > timeout) {
          throw new Error('Timeout')
        }
        await new Promise((resolve) => setTimeout(resolve, 500))
      }
    }
    throw new Error('Not in block, your Code may upload failed.')
  }
}

interface PinkBlueprintDeploy extends MessageMeta {
  (): SubmittableExtrinsic<'promise', InkCodeSubmittableResult>
}

type PinkMapConstructorExec = Record<string, PinkBlueprintDeploy>

export class PinkCodePromise {
  readonly abi: Abi
  readonly api: ApiBase<'promise'>
  readonly client: OnChainRegistry

  protected readonly _decorateMethod: DecorateMethod<'promise'>

  readonly code: Uint8Array

  readonly #tx: PinkMapConstructorExec = {}

  constructor(client: OnChainRegistry, abi: AbiLike, wasm?: Uint8Array | string | Buffer | null | undefined) {
    if (!client.isReady()) {
      throw new Error('Your client has not been initialized correctly.')
    }
    const api = client.api
    if (!api || !api.isConnected || !api.tx) {
      throw new Error('Your API has not been initialized correctly and is not connected to a chain')
    }

    this.abi = toAbi(abi, api.registry.getChainProperties())
    this.api = api
    this._decorateMethod = toPromiseMethod
    this.client = client

    // NOTE: we only tested with the .contract file & wasm in Uint8Array.
    if (isWasm(this.abi.info.source.wasm)) {
      this.code = this.abi.info.source.wasm
    } else if (isU8a(wasm)) {
      this.code = wasm
    } else if (typeof wasm === 'string' && wasm.substring(0, 2) === '0x') {
      this.code = hexToU8a(wasm)
    } else {
      throw new Error('`wasm` should hex encoded string or Uint8Array.')
    }

    if (!isWasm(this.code)) {
      throw new Error('No WASM code provided')
    }

    this.#tx = new Proxy(
      {},
      {
        get: (_target, prop, _receiver) => {
          const meta = this.abi.constructors.filter((i) => i.method === prop)
          if (!meta || !meta.length) {
            throw new Error('Method not found')
          }
          return () => this.#instantiate(meta[0], []) as SubmittableExtrinsic<'promise', InkCodeSubmittableResult>
        },
      }
    ) as PinkMapConstructorExec
  }

  public get tx(): PinkMapConstructorExec {
    return this.#tx
  }

  public upload() {
    return this.#instantiate(0, [])
  }

  public async send({ provider }: PinkCodeSendOptions): Promise<InkCodeSubmittableResult> {
    return await provider.send<InkCodeSubmittableResult>(
      this.api.tx.phalaPhatContracts.clusterUploadResource(this.client.clusterId, 'InkCode', u8aToHex(this.code)),
      (result) => new InkCodeSubmittableResult(result, this.abi, this.client)
    )
  }

  public async hasExists() {
    if (!this.client.systemContract) {
      throw new Error('System contract not found in the cluster, maybe the cluster is not ready yet.')
    }
    const cert = await this.client.getAnonymousCert()
    const { output } = await this.client.systemContract.query['system::codeExists']<bool>(
      this.client.alice.address,
      { cert },
      this.abi.info.source.wasmHash.toHex(),
      'Ink'
    )
    const hasExists = !!(output && output.isOk && output.asOk.isTrue)
    return hasExists
  }

  public getBlueprint() {
    return new PinkBlueprintPromise(this.client, this.abi, this.abi.info.source.wasmHash)
  }

  #instantiate = (_constructorOrId: AbiConstructor | string | number, _params: unknown[]) => {
    return this.api.tx.phalaPhatContracts
      .clusterUploadResource(this.client.clusterId, 'InkCode', u8aToHex(this.code))
      .withResultTransform((result: ISubmittableResult) => {
        return new InkCodeSubmittableResult(result, this.abi, this.client)
      }) as SubmittableExtrinsic<'promise', InkCodeSubmittableResult>
  }
}

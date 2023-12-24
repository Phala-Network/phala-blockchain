import { ApiPromise, Keyring, WsProvider } from '@polkadot/api'
import type { AccountId } from '@polkadot/types/interfaces'
import type { HexString } from '@polkadot/util/types'
import { cryptoWaitReady } from '@polkadot/util-crypto'
import { PinkContractPromise } from './contracts/PinkContract'
import { PinkLoggerContractPromise } from './contracts/PinkLoggerContract'
import { type CreateOptions, OnChainRegistry } from './OnChainRegistry'
import { options } from './options'
import createPruntimeClient from './pruntime/createPruntimeClient'
import type { AbiLike } from './types'
import { type LiteralRpc, fetchMetadata } from './utils/fetchMetadata'

export type GetClientOptions = {
  transport: LiteralRpc | WsProvider

  // Provides metadata instead loading via RPC when initializing the client.
  // It's optional since if the RPC under the phala.network domain, we will
  // try to preload the metadata via HTTP unless the `noPreloadMetadata` is
  // set to true.
  metadata?: Record<string, HexString>
  noPreloadMetadata?: boolean
} & CreateOptions

export async function getClient(opts: GetClientOptions): Promise<OnChainRegistry> {
  const { transport, metadata: _metadata, noPreloadMetadata, ...rest } = opts
  const provider = typeof transport === 'string' ? new WsProvider(transport) : transport
  let metadata = _metadata
  if (typeof transport === 'string' && !metadata && transport.indexOf('phala.network/') !== -1 && !noPreloadMetadata) {
    metadata = await fetchMetadata(transport)
  }
  const api = await ApiPromise.create(options({ provider, noInitWarn: true, metadata }))
  return await OnChainRegistry.create(api, rest)
}

export type GetContractOptions = {
  client: OnChainRegistry
  contractId: string
  abi: AbiLike
}

export async function getContract(options: GetContractOptions): Promise<PinkContractPromise> {
  const { client, contractId, abi } = options
  const contractKey = await client.getContractKeyOrFail(contractId)
  return new PinkContractPromise(client.api, client, abi, contractId, contractKey)
}

export type GetLoggerOptions =
  | GetClientOptions
  | {
      contractId: string | AccountId
      pruntimeURL: string
      systemContract?: string | AccountId
    }

export async function getLogger(options: GetLoggerOptions): Promise<PinkLoggerContractPromise> {
  if ('transport' in options) {
    const client = await getClient(options)
    if (!client.loggerContract) {
      throw new Error('Logger contract not found in the cluster.')
    }
    return client.loggerContract
  }
  // This is off-chain only mode.
  else if ('contractId' in options && 'pruntimeURL' in options) {
    await cryptoWaitReady()
    const keyring = new Keyring({ type: 'sr25519' })
    const alice = keyring.addFromUri('//Alice')
    const { contractId, pruntimeURL } = options
    const phactory = createPruntimeClient(pruntimeURL)
    const info = await phactory.getInfo({})
    return new PinkLoggerContractPromise(phactory, info.publicKey!, alice, contractId, options.systemContract)
  } else {
    throw new Error('Invalid options.')
  }
}

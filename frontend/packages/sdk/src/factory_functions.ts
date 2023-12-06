import { ApiPromise, Keyring, WsProvider } from '@polkadot/api'
import type { AccountId } from '@polkadot/types/interfaces'
import { cryptoWaitReady } from '@polkadot/util-crypto'
import { PinkContractPromise } from './contracts/PinkContract'
import { PinkLoggerContractPromise } from './contracts/PinkLoggerContract'
import { type CreateOptions, OnChainRegistry } from './OnChainRegistry'
import { options } from './options'
import createPruntimeClient from './pruntime/createPruntimeClient'
import type { AbiLike } from './types'

export type GetClientOptions = {
  transport: string | WsProvider
} & CreateOptions

export async function getClient(opts: GetClientOptions): Promise<OnChainRegistry> {
  const { transport, ...rest } = opts
  const provider = typeof transport === 'string' ? new WsProvider(transport) : transport
  const api = await ApiPromise.create(options({ provider, noInitWarn: true }))
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

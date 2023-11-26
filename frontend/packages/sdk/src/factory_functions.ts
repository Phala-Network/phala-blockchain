import { ApiPromise, WsProvider } from '@polkadot/api'
import { PinkContractPromise } from './contracts/PinkContract'
import { type CreateOptions, OnChainRegistry } from './OnChainRegistry'
import { options } from './options'
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

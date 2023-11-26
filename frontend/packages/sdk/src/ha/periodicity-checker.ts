import { type ApiPromise } from '@polkadot/api'
import fetch from 'cross-fetch'
import createPruntimeClient from '../pruntime/createPruntimeClient'

/**
 * This strategy is not rely on on-chain worker list avoiding extract RPC community. Instead of
 * this, it fetch worker list from remote JSON file.
 *
 * It useful when you work with off-chain queries only and elimate the on-chain RPC dependency.
 */

const PRUNTIME_NODE_LIST = 'https://phala-network.github.io/pruntime-node-list/nodes.json'

export function periodicityChecker(url?: string) {
  return async function (
    _apiPromise: ApiPromise,
    clusterId: string
  ): Promise<Readonly<[string, string, ReturnType<typeof createPruntimeClient>]>> {
    const resp = await fetch(url || PRUNTIME_NODE_LIST)
    const data = await resp.json()
    const endpoints = data?.[clusterId] || []
    if (endpoints.length === 0) {
      throw new Error('No worker available.')
    }
    const picked = endpoints[Math.floor(Math.random() * endpoints.length)]
    const client = createPruntimeClient(picked)
    const info = await client.getInfo({})
    return [`0x${info.ecdhPublicKey || ''}`, picked, client] as const
  }
}

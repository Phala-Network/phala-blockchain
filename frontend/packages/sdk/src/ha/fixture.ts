import { type ApiPromise } from '@polkadot/api'
import createPruntimeClient from '../pruntime/createPruntimeClient'

export function fixture(endpoints: string[]) {
  return async function (
    _apiPromise: ApiPromise,
    _clusterId: string
  ): Promise<Readonly<[string, string, ReturnType<typeof createPruntimeClient>]>> {
    if (endpoints.length === 0) {
      throw new Error('No worker available.')
    }
    const picked = endpoints[Math.floor(Math.random() * endpoints.length)]
    const client = createPruntimeClient(picked)
    const info = await client.getInfo({})
    return [`0x${info.ecdhPublicKey || ''}`, picked, client] as const
  }
}

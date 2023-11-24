import { type ApiPromise } from '@polkadot/api'
import type { Enum, Option, Text, U8aFixed, Vec } from '@polkadot/types'
import createPruntimeClient from '../pruntime/createPruntimeClient'

interface PhalaTypesVersionedWorkerEndpoints extends Enum {
  readonly isV1: boolean
  readonly asV1: Vec<Text>
  readonly type: 'V1'
}

async function ack(
  workerId: string,
  endpoint: string
): Promise<Readonly<[string, string, ReturnType<typeof createPruntimeClient>]>> {
  const client = createPruntimeClient(endpoint)
  const info = await client.getInfo({})
  const actually = `0x${info.ecdhPublicKey || ''}`
  if (actually === workerId) {
    return [workerId, endpoint, client] as const
  }
  throw new Error('On-chain worker ID not match to the worker ECDH PublicKey.')
}

/**
 * This is the most simple strategy which pulling list of workers and check their available
 * or not. Return first one that ack succeed.
 */
export function ackFirst() {
  return async function ackFirst(
    apiPromise: ApiPromise,
    clusterId: string
  ): Promise<Readonly<[string, string, ReturnType<typeof createPruntimeClient>]>> {
    const workersQuery = await apiPromise.query.phalaPhatContracts.clusterWorkers<Vec<U8aFixed>>(clusterId)
    const workerIds = workersQuery.map((i) => i.toHex())
    const endpointsQuery =
      await apiPromise.query.phalaRegistry.endpoints.multi<Option<PhalaTypesVersionedWorkerEndpoints>>(workerIds)
    const pairs = endpointsQuery
      .map((i, idx) => [workerIds[idx], i] as const)
      .filter(([_, maybeEndpoint]) => maybeEndpoint.isSome)
      .map(([workerId, maybeEndpoint]) => [workerId, maybeEndpoint.unwrap().asV1[0].toString()])
    try {
      return await Promise.any(pairs.map(([workerId, endpoint]) => ack(workerId, endpoint)))
    } catch (_err) {
      throw new Error('No worker available.')
    }
  }
}

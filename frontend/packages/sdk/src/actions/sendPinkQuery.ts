import type { ContractExecResult } from '@polkadot/types/interfaces'
import type { Codec } from '@polkadot/types-codec/types'
import type { OnChainRegistry } from '../OnChainRegistry'
import { phalaTypes } from '../options'
import { InkQueryMessage } from '../pruntime/coders'
import { pinkQuery } from '../pruntime/pinkQuery'
import { WorkerAgreementKey } from '../pruntime/WorkerAgreementKey'
import type { AbiLike, AnyProvider } from '../types'
import { toAbi } from '../utils/abi/toAbi'

export type SendPinkQueryParameters = {
  address: string
  abi: AbiLike
  functionName: string
  args?: any[]
  provider: AnyProvider
}

export async function sendPinkQuery<TResult extends Codec = Codec>(
  client: OnChainRegistry,
  parameters: SendPinkQueryParameters
): Promise<TResult | null> {
  const { address, functionName, provider } = parameters
  if (!client.workerInfo?.pubkey) {
    throw new Error('Worker pubkey not found')
  }

  const abi = toAbi(parameters.abi)
  const args = parameters.args || []

  const message = abi.findMessage(functionName)
  if (!message) {
    throw new Error(`Message not found: ${functionName}`)
  }
  const encodedArgs = message.toU8a(args)
  const inkMessage = InkQueryMessage(address, encodedArgs)
  const argument = new WorkerAgreementKey(client.workerInfo.pubkey)

  const cert = await provider.signCertificate()
  const inkResponse = await pinkQuery(client.phactory, argument, inkMessage.toHex(), cert)

  if (inkResponse.result.isErr) {
    // @FIXME: not sure this is enough as not yet tested
    throw new Error(`InkResponse Error: ${inkResponse.result.asErr.toString()}`)
  }
  if (!inkResponse.result.asOk.isInkMessageReturn) {
    // @FIXME: not sure this is enough as not yet tested
    throw new Error(`Unexpected InkMessageReturn: ${inkResponse.result.asOk.toJSON()?.toString()}`)
  }
  const { result } = phalaTypes.createType<ContractExecResult>(
    'ContractExecResult',
    inkResponse.result.asOk.asInkMessageReturn.toString()
  )
  if (result.isErr) {
    throw new Error(`ContractExecResult Error: ${result.asErr.toString()}`)
  }
  if (message.returnType) {
    return abi.registry.createTypeUnsafe<TResult>(
      message.returnType.lookupName || message.returnType.type,
      [result.asOk.data.toU8a(true)],
      { isPedantic: true }
    )
  }
  return null
}

import { convertWeight } from '@polkadot/api-contract/base/util'
import type { ContractCallOutcome } from '@polkadot/api-contract/types'
import type { ContractExecResult } from '@polkadot/types/interfaces'
import type { OnChainRegistry } from '../OnChainRegistry'
import { phalaTypes } from '../options'
import { InkQueryMessage } from '../pruntime/coders'
import { pinkQuery } from '../pruntime/pinkQuery'
import { WorkerAgreementKey } from '../pruntime/WorkerAgreementKey'
import type { LooseNumber } from '../types'
import { toAbi } from '../utils/abi/toAbi'
import { SendPinkQueryParameters } from './sendPinkQuery'

export type EstimateContractParameters<T> = SendPinkQueryParameters<T> & {
  deposit?: LooseNumber
  transfer?: LooseNumber
}

export async function estimateContract(
  client: OnChainRegistry,
  parameters: EstimateContractParameters<any[]>
): Promise<Omit<ContractCallOutcome, 'output'>> {
  const { address, functionName, provider, deposit, transfer } = parameters
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
  const inkMessage = InkQueryMessage(address, encodedArgs, deposit, transfer, true)
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
  const { debugMessage, gasConsumed, gasRequired, result, storageDeposit } = phalaTypes.createType<ContractExecResult>(
    'ContractExecResult',
    inkResponse.result.asOk.asInkMessageReturn.toString()
  )

  return {
    debugMessage: debugMessage,
    gasConsumed: gasConsumed,
    gasRequired: gasRequired && !convertWeight(gasRequired).v1Weight.isZero() ? gasRequired : gasConsumed,
    result,
    storageDeposit,
  }
}

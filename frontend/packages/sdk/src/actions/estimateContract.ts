import { convertWeight } from '@polkadot/api-contract/base/util'
import type { ContractCallOutcome } from '@polkadot/api-contract/types'
import type { ContractExecResult } from '@polkadot/types/interfaces'
import { BN_ZERO } from '@polkadot/util'
import type { OnChainRegistry } from '../OnChainRegistry'
import { phalaTypes } from '../options'
import { InkQueryMessage } from '../pruntime/coders'
import { pinkQuery } from '../pruntime/pinkQuery'
import { WorkerAgreementKey } from '../pruntime/WorkerAgreementKey'
import type { FrameSystemAccountInfo, LooseNumber } from '../types'
import { toAbi } from '../utils/abi/toAbi'
import { BN_MAX_SUPPLY } from '../utils/constants'
import { SendPinkCommandParameters } from './sendPinkCommand'
import { SendPinkQueryParameters } from './sendPinkQuery'

export type EstimateContractParameters<T> = SendPinkQueryParameters<T> & {
  contractKey: string
  deposit?: LooseNumber
  transfer?: LooseNumber
}

export type EstimateContractResult = Omit<ContractCallOutcome, 'output'> & {
  request: SendPinkCommandParameters
}

export async function estimateContract(
  client: OnChainRegistry,
  parameters: EstimateContractParameters<any[]>
): Promise<EstimateContractResult> {
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
  const inkMessage = InkQueryMessage(address, encodedArgs, deposit || BN_MAX_SUPPLY, transfer, true)
  const argument = new WorkerAgreementKey(client.workerInfo.pubkey)

  const cert = await provider.signCertificate()

  const [clusterBalance, onchainBalance, inkResponse] = await Promise.all([
    client.getClusterBalance(provider.address),
    client.api.query.system.account<FrameSystemAccountInfo>(provider.address),
    pinkQuery(client.phactory, argument, inkMessage.toHex(), cert),
  ])

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

  const { gasPrice } = client.clusterInfo ?? {}
  if (!gasPrice) {
    throw new Error('No Gas Price or deposit Per Byte from cluster info.')
  }

  // calculate the total costs
  const gasLimit = gasRequired.refTime.toBn()
  const storageDepositFee = storageDeposit.isCharge ? storageDeposit.asCharge.toBn() : BN_ZERO
  const minRequired = gasLimit.mul(gasPrice).add(storageDepositFee)

  // Auto deposit.
  let autoDeposit = undefined
  if (clusterBalance.free.lt(minRequired)) {
    const deposit = minRequired.sub(clusterBalance.free)
    if (onchainBalance.data.free.lt(deposit)) {
      throw new Error(`Not enough balance to pay for gas and storage deposit: ${minRequired.toNumber()}`)
    }
    autoDeposit = deposit
  }

  return {
    debugMessage: debugMessage,
    gasConsumed: gasConsumed,
    gasRequired: gasRequired && !convertWeight(gasRequired).v1Weight.isZero() ? gasRequired : gasConsumed,
    result,
    storageDeposit,
    request: {
      ...parameters,
      gasLimit,
      deposit: autoDeposit,
    },
  }
}

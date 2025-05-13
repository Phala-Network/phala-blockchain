import { convertWeight } from '@polkadot/api-contract/base/util'
import { BN_ZERO, hexAddPrefix, isHex } from '@polkadot/util'
import type { OnChainRegistry } from '../OnChainRegistry'
import { EncryptedInkCommand, PlainInkCommand } from '../pruntime/coders'
import type { AbiLike, AnyProvider, HexString, LooseNumber } from '../types'
import { toAbi } from '../utils/abi/toAbi'
import assert from '../utils/assert'
import { randomHex } from '../utils/hex'

export type SendPinkCommandParameters<TArgs = any[]> = {
  address: string
  contractKey: string
  provider: AnyProvider
  abi: AbiLike
  functionName: string
  args?: TArgs
  //
  gasLimit: LooseNumber
  nonce?: HexString
  plain?: boolean
  value?: LooseNumber
  storageDepositLimit?: LooseNumber
  deposit?: LooseNumber
}

export async function sendPinkCommand(client: OnChainRegistry, parameters: SendPinkCommandParameters) {
  const { address, contractKey, functionName, provider, plain, value, gasLimit, storageDepositLimit, deposit } =
    parameters
  if (!client.workerInfo?.pubkey) {
    throw new Error('Worker pubkey not found')
  }

  const abi = toAbi(parameters.abi)
  const args = parameters.args || []

  parameters.nonce && assert(isHex(parameters.nonce) && parameters.nonce.length === 66, 'Invalid nonce provided')
  const nonce = parameters.nonce || hexAddPrefix(randomHex(32))

  const message = abi.findMessage(functionName)
  if (!message) {
    throw new Error(`Message not found: ${functionName}`)
  }
  const encodedArgs = message.toU8a(args)

  const createCommand = plain ? PlainInkCommand : EncryptedInkCommand
  const payload = createCommand(
    contractKey,
    encodedArgs,
    nonce,
    value,
    convertWeight(gasLimit || BN_ZERO).v2Weight,
    storageDepositLimit
  )
  const extrinsic = client.api.tx.phalaPhatContracts.pushContractMessage(address, payload.toHex(), deposit || BN_ZERO)
  const result = await provider.send(extrinsic)
  return result
}

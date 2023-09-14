import system from './abis/system.json'
import { pruntime_rpc } from './proto'

export * from './lib/types'
export * from './lib/hex'
export * from './certificate'
export * from './contracts/PinkCode'
export * from './contracts/PinkContract'
export * from './contracts/PinkBlueprint'
export * from './contracts/PinkLoggerContract'
export * from './OnChainRegistry'
export * from './options'
export * from './metadata'
export * from './eip712'
export { default as createPruntimeClient } from './createPruntimeClient'
export { default as signAndSend } from './signAndSend'

export const PhactoryAPI = pruntime_rpc.PhactoryAPI
export const pruntimeRpc = pruntime_rpc
export const abis = { system }

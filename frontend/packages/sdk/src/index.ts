import system from './abis/system.json'
import { pruntime_rpc } from './pruntime/proto'

export * from './utils/hex'
export * from './pruntime/certificate'
export * from './contracts/PinkCode'
export * from './contracts/PinkContract'
export * from './contracts/PinkBlueprint'
export * from './contracts/PinkLoggerContract'
export * from './OnChainRegistry'
export * from './options'
export * from './abis/fetchers'
export * from './pruntime/eip712'
export { default as createPruntimeClient } from './pruntime/createPruntimeClient'
export { default as signAndSend, SignAndSendError } from './utils/signAndSend'

export const pruntimeRpc = pruntime_rpc
export type PhactoryAPI = pruntime_rpc.PhactoryAPI
export const PhactoryAPI = pruntime_rpc.PhactoryAPI
export const abis = { system }

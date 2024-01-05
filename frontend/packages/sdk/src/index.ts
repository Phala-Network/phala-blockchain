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
export * from './utils/eip712'
export * from './utils/addressConverter'
export * from './providers/types'
export * from './providers/EvmAccountMappingProvider'
export * from './providers/UIKeyringProvider'
export * from './providers/KeyringPairProvider'
export { default as createPruntimeClient } from './pruntime/createPruntimeClient'
export { default as signAndSend, SignAndSendError } from './utils/signAndSend'
export { ackFirst } from './ha/ack-first'
export * from './ha/periodicity-checker'
export * from './ha/fixture'
export * from './factory_functions'
export * from './utils/fetchMetadata'
export * from './types'

export const pruntimeRpc = pruntime_rpc
export type PhactoryAPI = pruntime_rpc.PhactoryAPI
export const PhactoryAPI = pruntime_rpc.PhactoryAPI
export const abis = { system }

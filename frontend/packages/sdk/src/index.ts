import { pruntime_rpc } from "./proto";
import system from './abis/system.json';
import logServer from './abis/log_server.json';

export * from "./lib/types";
export * from "./lib/hex";
export * from "./create";
export * from "./certificate";
export * from "./contracts/PinkCode";
export * from "./contracts/PinkContract";
export * from './contracts/PinkBlueprint';
export * from "./contracts/SystemContract";
export * from "./contracts/PinkLoggerContract";
export * from './OnChainRegistry';

export const PhactoryAPI = pruntime_rpc.PhactoryAPI;
export const abis = { system, logServer };

import { pruntime_rpc } from "./proto";
import system from './abis/system.json';
import log_server from './abis/log_server.json';

export * from "./lib/types";
export * from "./lib/hex";
export * from "./create";
export * from "./certificate";
export * from "./contracts/PinkContract";

export const PhactoryAPI = pruntime_rpc.PhactoryAPI;
export const abis = { system, log_server };

import * as $protobuf from "protobufjs";
import Long = require("long");
/** Namespace prpc. */
export namespace prpc {

    /** Properties of a PrpcError. */
    interface IPrpcError {

        /** The error description */
        message?: (string|null);
    }

    /** The final Error type of RPCs to be serialized to protobuf. */
    class PrpcError implements IPrpcError {

        /**
         * Constructs a new PrpcError.
         * @param [properties] Properties to set
         */
        constructor(properties?: prpc.IPrpcError);

        /** The error description */
        public message: string;

        /**
         * Creates a new PrpcError instance using the specified properties.
         * @param [properties] Properties to set
         * @returns PrpcError instance
         */
        public static create(properties?: prpc.IPrpcError): prpc.PrpcError;

        /**
         * Encodes the specified PrpcError message. Does not implicitly {@link prpc.PrpcError.verify|verify} messages.
         * @param message PrpcError message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: prpc.IPrpcError, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified PrpcError message, length delimited. Does not implicitly {@link prpc.PrpcError.verify|verify} messages.
         * @param message PrpcError message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: prpc.IPrpcError, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a PrpcError message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns PrpcError
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): prpc.PrpcError;

        /**
         * Decodes a PrpcError message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns PrpcError
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): prpc.PrpcError;

        /**
         * Verifies a PrpcError message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a PrpcError message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns PrpcError
         */
        public static fromObject(object: { [k: string]: any }): prpc.PrpcError;

        /**
         * Creates a plain object from a PrpcError message. Also converts values to other types if specified.
         * @param message PrpcError
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: prpc.PrpcError, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this PrpcError to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for PrpcError
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }
}

/** Namespace pruntime_rpc. */
export namespace pruntime_rpc {

    /** Represents a PhactoryAPI */
    class PhactoryAPI extends $protobuf.rpc.Service {

        /**
         * Constructs a new PhactoryAPI service.
         * @param rpcImpl RPC implementation
         * @param [requestDelimited=false] Whether requests are length-delimited
         * @param [responseDelimited=false] Whether responses are length-delimited
         */
        constructor(rpcImpl: $protobuf.RPCImpl, requestDelimited?: boolean, responseDelimited?: boolean);

        /**
         * Creates new PhactoryAPI service using the specified rpc implementation.
         * @param rpcImpl RPC implementation
         * @param [requestDelimited=false] Whether requests are length-delimited
         * @param [responseDelimited=false] Whether responses are length-delimited
         * @returns RPC service. Useful where requests and/or responses are streamed.
         */
        public static create(rpcImpl: $protobuf.RPCImpl, requestDelimited?: boolean, responseDelimited?: boolean): PhactoryAPI;

        /**
         * Calls GetInfo.
         * @param request Empty message or plain object
         * @param callback Node-style callback called with the error, if any, and PhactoryInfo
         */
        public getInfo(request: google.protobuf.IEmpty, callback: pruntime_rpc.PhactoryAPI.GetInfoCallback): void;

        /**
         * Calls GetInfo.
         * @param request Empty message or plain object
         * @returns Promise
         */
        public getInfo(request: google.protobuf.IEmpty): Promise<pruntime_rpc.PhactoryInfo>;

        /**
         * Calls SyncHeader.
         * @param request HeadersToSync message or plain object
         * @param callback Node-style callback called with the error, if any, and SyncedTo
         */
        public syncHeader(request: pruntime_rpc.IHeadersToSync, callback: pruntime_rpc.PhactoryAPI.SyncHeaderCallback): void;

        /**
         * Calls SyncHeader.
         * @param request HeadersToSync message or plain object
         * @returns Promise
         */
        public syncHeader(request: pruntime_rpc.IHeadersToSync): Promise<pruntime_rpc.SyncedTo>;

        /**
         * Calls SyncParaHeader.
         * @param request ParaHeadersToSync message or plain object
         * @param callback Node-style callback called with the error, if any, and SyncedTo
         */
        public syncParaHeader(request: pruntime_rpc.IParaHeadersToSync, callback: pruntime_rpc.PhactoryAPI.SyncParaHeaderCallback): void;

        /**
         * Calls SyncParaHeader.
         * @param request ParaHeadersToSync message or plain object
         * @returns Promise
         */
        public syncParaHeader(request: pruntime_rpc.IParaHeadersToSync): Promise<pruntime_rpc.SyncedTo>;

        /**
         * Calls SyncCombinedHeaders.
         * @param request CombinedHeadersToSync message or plain object
         * @param callback Node-style callback called with the error, if any, and HeadersSyncedTo
         */
        public syncCombinedHeaders(request: pruntime_rpc.ICombinedHeadersToSync, callback: pruntime_rpc.PhactoryAPI.SyncCombinedHeadersCallback): void;

        /**
         * Calls SyncCombinedHeaders.
         * @param request CombinedHeadersToSync message or plain object
         * @returns Promise
         */
        public syncCombinedHeaders(request: pruntime_rpc.ICombinedHeadersToSync): Promise<pruntime_rpc.HeadersSyncedTo>;

        /**
         * Calls DispatchBlocks.
         * @param request Blocks message or plain object
         * @param callback Node-style callback called with the error, if any, and SyncedTo
         */
        public dispatchBlocks(request: pruntime_rpc.IBlocks, callback: pruntime_rpc.PhactoryAPI.DispatchBlocksCallback): void;

        /**
         * Calls DispatchBlocks.
         * @param request Blocks message or plain object
         * @returns Promise
         */
        public dispatchBlocks(request: pruntime_rpc.IBlocks): Promise<pruntime_rpc.SyncedTo>;

        /**
         * Calls InitRuntime.
         * @param request InitRuntimeRequest message or plain object
         * @param callback Node-style callback called with the error, if any, and InitRuntimeResponse
         */
        public initRuntime(request: pruntime_rpc.IInitRuntimeRequest, callback: pruntime_rpc.PhactoryAPI.InitRuntimeCallback): void;

        /**
         * Calls InitRuntime.
         * @param request InitRuntimeRequest message or plain object
         * @returns Promise
         */
        public initRuntime(request: pruntime_rpc.IInitRuntimeRequest): Promise<pruntime_rpc.InitRuntimeResponse>;

        /**
         * Calls GetRuntimeInfo.
         * @param request GetRuntimeInfoRequest message or plain object
         * @param callback Node-style callback called with the error, if any, and InitRuntimeResponse
         */
        public getRuntimeInfo(request: pruntime_rpc.IGetRuntimeInfoRequest, callback: pruntime_rpc.PhactoryAPI.GetRuntimeInfoCallback): void;

        /**
         * Calls GetRuntimeInfo.
         * @param request GetRuntimeInfoRequest message or plain object
         * @returns Promise
         */
        public getRuntimeInfo(request: pruntime_rpc.IGetRuntimeInfoRequest): Promise<pruntime_rpc.InitRuntimeResponse>;

        /**
         * Calls GetEgressMessages.
         * @param request Empty message or plain object
         * @param callback Node-style callback called with the error, if any, and GetEgressMessagesResponse
         */
        public getEgressMessages(request: google.protobuf.IEmpty, callback: pruntime_rpc.PhactoryAPI.GetEgressMessagesCallback): void;

        /**
         * Calls GetEgressMessages.
         * @param request Empty message or plain object
         * @returns Promise
         */
        public getEgressMessages(request: google.protobuf.IEmpty): Promise<pruntime_rpc.GetEgressMessagesResponse>;

        /**
         * Calls ContractQuery.
         * @param request ContractQueryRequest message or plain object
         * @param callback Node-style callback called with the error, if any, and ContractQueryResponse
         */
        public contractQuery(request: pruntime_rpc.IContractQueryRequest, callback: pruntime_rpc.PhactoryAPI.ContractQueryCallback): void;

        /**
         * Calls ContractQuery.
         * @param request ContractQueryRequest message or plain object
         * @returns Promise
         */
        public contractQuery(request: pruntime_rpc.IContractQueryRequest): Promise<pruntime_rpc.ContractQueryResponse>;

        /**
         * Calls GetWorkerState.
         * @param request GetWorkerStateRequest message or plain object
         * @param callback Node-style callback called with the error, if any, and WorkerState
         */
        public getWorkerState(request: pruntime_rpc.IGetWorkerStateRequest, callback: pruntime_rpc.PhactoryAPI.GetWorkerStateCallback): void;

        /**
         * Calls GetWorkerState.
         * @param request GetWorkerStateRequest message or plain object
         * @returns Promise
         */
        public getWorkerState(request: pruntime_rpc.IGetWorkerStateRequest): Promise<pruntime_rpc.WorkerState>;

        /**
         * Calls AddEndpoint.
         * @param request AddEndpointRequest message or plain object
         * @param callback Node-style callback called with the error, if any, and GetEndpointResponse
         */
        public addEndpoint(request: pruntime_rpc.IAddEndpointRequest, callback: pruntime_rpc.PhactoryAPI.AddEndpointCallback): void;

        /**
         * Calls AddEndpoint.
         * @param request AddEndpointRequest message or plain object
         * @returns Promise
         */
        public addEndpoint(request: pruntime_rpc.IAddEndpointRequest): Promise<pruntime_rpc.GetEndpointResponse>;

        /**
         * Calls RefreshEndpointSigningTime.
         * @param request Empty message or plain object
         * @param callback Node-style callback called with the error, if any, and GetEndpointResponse
         */
        public refreshEndpointSigningTime(request: google.protobuf.IEmpty, callback: pruntime_rpc.PhactoryAPI.RefreshEndpointSigningTimeCallback): void;

        /**
         * Calls RefreshEndpointSigningTime.
         * @param request Empty message or plain object
         * @returns Promise
         */
        public refreshEndpointSigningTime(request: google.protobuf.IEmpty): Promise<pruntime_rpc.GetEndpointResponse>;

        /**
         * Calls GetEndpointInfo.
         * @param request Empty message or plain object
         * @param callback Node-style callback called with the error, if any, and GetEndpointResponse
         */
        public getEndpointInfo(request: google.protobuf.IEmpty, callback: pruntime_rpc.PhactoryAPI.GetEndpointInfoCallback): void;

        /**
         * Calls GetEndpointInfo.
         * @param request Empty message or plain object
         * @returns Promise
         */
        public getEndpointInfo(request: google.protobuf.IEmpty): Promise<pruntime_rpc.GetEndpointResponse>;

        /**
         * Calls SignEndpointInfo.
         * @param request SignEndpointsRequest message or plain object
         * @param callback Node-style callback called with the error, if any, and GetEndpointResponse
         */
        public signEndpointInfo(request: pruntime_rpc.ISignEndpointsRequest, callback: pruntime_rpc.PhactoryAPI.SignEndpointInfoCallback): void;

        /**
         * Calls SignEndpointInfo.
         * @param request SignEndpointsRequest message or plain object
         * @returns Promise
         */
        public signEndpointInfo(request: pruntime_rpc.ISignEndpointsRequest): Promise<pruntime_rpc.GetEndpointResponse>;

        /**
         * Calls DerivePhalaI2pKey.
         * @param request Empty message or plain object
         * @param callback Node-style callback called with the error, if any, and DerivePhalaI2pKeyResponse
         */
        public derivePhalaI2pKey(request: google.protobuf.IEmpty, callback: pruntime_rpc.PhactoryAPI.DerivePhalaI2pKeyCallback): void;

        /**
         * Calls DerivePhalaI2pKey.
         * @param request Empty message or plain object
         * @returns Promise
         */
        public derivePhalaI2pKey(request: google.protobuf.IEmpty): Promise<pruntime_rpc.DerivePhalaI2pKeyResponse>;

        /**
         * Calls Echo.
         * @param request EchoMessage message or plain object
         * @param callback Node-style callback called with the error, if any, and EchoMessage
         */
        public echo(request: pruntime_rpc.IEchoMessage, callback: pruntime_rpc.PhactoryAPI.EchoCallback): void;

        /**
         * Calls Echo.
         * @param request EchoMessage message or plain object
         * @returns Promise
         */
        public echo(request: pruntime_rpc.IEchoMessage): Promise<pruntime_rpc.EchoMessage>;

        /**
         * Calls HandoverCreateChallenge.
         * @param request Empty message or plain object
         * @param callback Node-style callback called with the error, if any, and HandoverChallenge
         */
        public handoverCreateChallenge(request: google.protobuf.IEmpty, callback: pruntime_rpc.PhactoryAPI.HandoverCreateChallengeCallback): void;

        /**
         * Calls HandoverCreateChallenge.
         * @param request Empty message or plain object
         * @returns Promise
         */
        public handoverCreateChallenge(request: google.protobuf.IEmpty): Promise<pruntime_rpc.HandoverChallenge>;

        /**
         * Calls HandoverStart.
         * @param request HandoverChallengeResponse message or plain object
         * @param callback Node-style callback called with the error, if any, and HandoverWorkerKey
         */
        public handoverStart(request: pruntime_rpc.IHandoverChallengeResponse, callback: pruntime_rpc.PhactoryAPI.HandoverStartCallback): void;

        /**
         * Calls HandoverStart.
         * @param request HandoverChallengeResponse message or plain object
         * @returns Promise
         */
        public handoverStart(request: pruntime_rpc.IHandoverChallengeResponse): Promise<pruntime_rpc.HandoverWorkerKey>;

        /**
         * Calls HandoverAcceptChallenge.
         * @param request HandoverChallenge message or plain object
         * @param callback Node-style callback called with the error, if any, and HandoverChallengeResponse
         */
        public handoverAcceptChallenge(request: pruntime_rpc.IHandoverChallenge, callback: pruntime_rpc.PhactoryAPI.HandoverAcceptChallengeCallback): void;

        /**
         * Calls HandoverAcceptChallenge.
         * @param request HandoverChallenge message or plain object
         * @returns Promise
         */
        public handoverAcceptChallenge(request: pruntime_rpc.IHandoverChallenge): Promise<pruntime_rpc.HandoverChallengeResponse>;

        /**
         * Calls HandoverReceive.
         * @param request HandoverWorkerKey message or plain object
         * @param callback Node-style callback called with the error, if any, and Empty
         */
        public handoverReceive(request: pruntime_rpc.IHandoverWorkerKey, callback: pruntime_rpc.PhactoryAPI.HandoverReceiveCallback): void;

        /**
         * Calls HandoverReceive.
         * @param request HandoverWorkerKey message or plain object
         * @returns Promise
         */
        public handoverReceive(request: pruntime_rpc.IHandoverWorkerKey): Promise<google.protobuf.Empty>;

        /**
         * Calls ConfigNetwork.
         * @param request NetworkConfig message or plain object
         * @param callback Node-style callback called with the error, if any, and Empty
         */
        public configNetwork(request: pruntime_rpc.INetworkConfig, callback: pruntime_rpc.PhactoryAPI.ConfigNetworkCallback): void;

        /**
         * Calls ConfigNetwork.
         * @param request NetworkConfig message or plain object
         * @returns Promise
         */
        public configNetwork(request: pruntime_rpc.INetworkConfig): Promise<google.protobuf.Empty>;

        /**
         * Calls HttpFetch.
         * @param request HttpRequest message or plain object
         * @param callback Node-style callback called with the error, if any, and HttpResponse
         */
        public httpFetch(request: pruntime_rpc.IHttpRequest, callback: pruntime_rpc.PhactoryAPI.HttpFetchCallback): void;

        /**
         * Calls HttpFetch.
         * @param request HttpRequest message or plain object
         * @returns Promise
         */
        public httpFetch(request: pruntime_rpc.IHttpRequest): Promise<pruntime_rpc.HttpResponse>;

        /**
         * Calls GetContractInfo.
         * @param request GetContractInfoRequest message or plain object
         * @param callback Node-style callback called with the error, if any, and GetContractInfoResponse
         */
        public getContractInfo(request: pruntime_rpc.IGetContractInfoRequest, callback: pruntime_rpc.PhactoryAPI.GetContractInfoCallback): void;

        /**
         * Calls GetContractInfo.
         * @param request GetContractInfoRequest message or plain object
         * @returns Promise
         */
        public getContractInfo(request: pruntime_rpc.IGetContractInfoRequest): Promise<pruntime_rpc.GetContractInfoResponse>;

        /**
         * Calls GetClusterInfo.
         * @param request Empty message or plain object
         * @param callback Node-style callback called with the error, if any, and GetClusterInfoResponse
         */
        public getClusterInfo(request: google.protobuf.IEmpty, callback: pruntime_rpc.PhactoryAPI.GetClusterInfoCallback): void;

        /**
         * Calls GetClusterInfo.
         * @param request Empty message or plain object
         * @returns Promise
         */
        public getClusterInfo(request: google.protobuf.IEmpty): Promise<pruntime_rpc.GetClusterInfoResponse>;

        /**
         * Calls UploadSidevmCode.
         * @param request SidevmCode message or plain object
         * @param callback Node-style callback called with the error, if any, and Empty
         */
        public uploadSidevmCode(request: pruntime_rpc.ISidevmCode, callback: pruntime_rpc.PhactoryAPI.UploadSidevmCodeCallback): void;

        /**
         * Calls UploadSidevmCode.
         * @param request SidevmCode message or plain object
         * @returns Promise
         */
        public uploadSidevmCode(request: pruntime_rpc.ISidevmCode): Promise<google.protobuf.Empty>;

        /**
         * Calls CalculateContractId.
         * @param request ContractParameters message or plain object
         * @param callback Node-style callback called with the error, if any, and ContractId
         */
        public calculateContractId(request: pruntime_rpc.IContractParameters, callback: pruntime_rpc.PhactoryAPI.CalculateContractIdCallback): void;

        /**
         * Calls CalculateContractId.
         * @param request ContractParameters message or plain object
         * @returns Promise
         */
        public calculateContractId(request: pruntime_rpc.IContractParameters): Promise<pruntime_rpc.ContractId>;

        /**
         * Calls GetNetworkConfig.
         * @param request Empty message or plain object
         * @param callback Node-style callback called with the error, if any, and NetworkConfigResponse
         */
        public getNetworkConfig(request: google.protobuf.IEmpty, callback: pruntime_rpc.PhactoryAPI.GetNetworkConfigCallback): void;

        /**
         * Calls GetNetworkConfig.
         * @param request Empty message or plain object
         * @returns Promise
         */
        public getNetworkConfig(request: google.protobuf.IEmpty): Promise<pruntime_rpc.NetworkConfigResponse>;

        /**
         * Calls LoadChainState.
         * @param request ChainState message or plain object
         * @param callback Node-style callback called with the error, if any, and Empty
         */
        public loadChainState(request: pruntime_rpc.IChainState, callback: pruntime_rpc.PhactoryAPI.LoadChainStateCallback): void;

        /**
         * Calls LoadChainState.
         * @param request ChainState message or plain object
         * @returns Promise
         */
        public loadChainState(request: pruntime_rpc.IChainState): Promise<google.protobuf.Empty>;

        /**
         * Calls Stop.
         * @param request StopOptions message or plain object
         * @param callback Node-style callback called with the error, if any, and Empty
         */
        public stop(request: pruntime_rpc.IStopOptions, callback: pruntime_rpc.PhactoryAPI.StopCallback): void;

        /**
         * Calls Stop.
         * @param request StopOptions message or plain object
         * @returns Promise
         */
        public stop(request: pruntime_rpc.IStopOptions): Promise<google.protobuf.Empty>;

        /**
         * Calls LoadStorageProof.
         * @param request StorageProof message or plain object
         * @param callback Node-style callback called with the error, if any, and Empty
         */
        public loadStorageProof(request: pruntime_rpc.IStorageProof, callback: pruntime_rpc.PhactoryAPI.LoadStorageProofCallback): void;

        /**
         * Calls LoadStorageProof.
         * @param request StorageProof message or plain object
         * @returns Promise
         */
        public loadStorageProof(request: pruntime_rpc.IStorageProof): Promise<google.protobuf.Empty>;

        /**
         * Calls TakeCheckpoint.
         * @param request Empty message or plain object
         * @param callback Node-style callback called with the error, if any, and SyncedTo
         */
        public takeCheckpoint(request: google.protobuf.IEmpty, callback: pruntime_rpc.PhactoryAPI.TakeCheckpointCallback): void;

        /**
         * Calls TakeCheckpoint.
         * @param request Empty message or plain object
         * @returns Promise
         */
        public takeCheckpoint(request: google.protobuf.IEmpty): Promise<pruntime_rpc.SyncedTo>;

        /**
         * Calls Statistics.
         * @param request StatisticsReqeust message or plain object
         * @param callback Node-style callback called with the error, if any, and StatisticsResponse
         */
        public statistics(request: pruntime_rpc.IStatisticsReqeust, callback: pruntime_rpc.PhactoryAPI.StatisticsCallback): void;

        /**
         * Calls Statistics.
         * @param request StatisticsReqeust message or plain object
         * @returns Promise
         */
        public statistics(request: pruntime_rpc.IStatisticsReqeust): Promise<pruntime_rpc.StatisticsResponse>;

        /**
         * Calls GenerateClusterStateRequest.
         * @param request Empty message or plain object
         * @param callback Node-style callback called with the error, if any, and SaveClusterStateArguments
         */
        public generateClusterStateRequest(request: google.protobuf.IEmpty, callback: pruntime_rpc.PhactoryAPI.GenerateClusterStateRequestCallback): void;

        /**
         * Calls GenerateClusterStateRequest.
         * @param request Empty message or plain object
         * @returns Promise
         */
        public generateClusterStateRequest(request: google.protobuf.IEmpty): Promise<pruntime_rpc.SaveClusterStateArguments>;

        /**
         * Calls SaveClusterState.
         * @param request SaveClusterStateArguments message or plain object
         * @param callback Node-style callback called with the error, if any, and SaveClusterStateResponse
         */
        public saveClusterState(request: pruntime_rpc.ISaveClusterStateArguments, callback: pruntime_rpc.PhactoryAPI.SaveClusterStateCallback): void;

        /**
         * Calls SaveClusterState.
         * @param request SaveClusterStateArguments message or plain object
         * @returns Promise
         */
        public saveClusterState(request: pruntime_rpc.ISaveClusterStateArguments): Promise<pruntime_rpc.SaveClusterStateResponse>;

        /**
         * Calls LoadClusterState.
         * @param request SaveClusterStateResponse message or plain object
         * @param callback Node-style callback called with the error, if any, and Empty
         */
        public loadClusterState(request: pruntime_rpc.ISaveClusterStateResponse, callback: pruntime_rpc.PhactoryAPI.LoadClusterStateCallback): void;

        /**
         * Calls LoadClusterState.
         * @param request SaveClusterStateResponse message or plain object
         * @returns Promise
         */
        public loadClusterState(request: pruntime_rpc.ISaveClusterStateResponse): Promise<google.protobuf.Empty>;

        /**
         * Calls TryUpgradePinkRuntime.
         * @param request PinkRuntimeVersion message or plain object
         * @param callback Node-style callback called with the error, if any, and Empty
         */
        public tryUpgradePinkRuntime(request: pruntime_rpc.IPinkRuntimeVersion, callback: pruntime_rpc.PhactoryAPI.TryUpgradePinkRuntimeCallback): void;

        /**
         * Calls TryUpgradePinkRuntime.
         * @param request PinkRuntimeVersion message or plain object
         * @returns Promise
         */
        public tryUpgradePinkRuntime(request: pruntime_rpc.IPinkRuntimeVersion): Promise<google.protobuf.Empty>;
    }

    namespace PhactoryAPI {

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#getInfo}.
         * @param error Error, if any
         * @param [response] PhactoryInfo
         */
        type GetInfoCallback = (error: (Error|null), response?: pruntime_rpc.PhactoryInfo) => void;

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#syncHeader}.
         * @param error Error, if any
         * @param [response] SyncedTo
         */
        type SyncHeaderCallback = (error: (Error|null), response?: pruntime_rpc.SyncedTo) => void;

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#syncParaHeader}.
         * @param error Error, if any
         * @param [response] SyncedTo
         */
        type SyncParaHeaderCallback = (error: (Error|null), response?: pruntime_rpc.SyncedTo) => void;

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#syncCombinedHeaders}.
         * @param error Error, if any
         * @param [response] HeadersSyncedTo
         */
        type SyncCombinedHeadersCallback = (error: (Error|null), response?: pruntime_rpc.HeadersSyncedTo) => void;

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#dispatchBlocks}.
         * @param error Error, if any
         * @param [response] SyncedTo
         */
        type DispatchBlocksCallback = (error: (Error|null), response?: pruntime_rpc.SyncedTo) => void;

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#initRuntime}.
         * @param error Error, if any
         * @param [response] InitRuntimeResponse
         */
        type InitRuntimeCallback = (error: (Error|null), response?: pruntime_rpc.InitRuntimeResponse) => void;

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#getRuntimeInfo}.
         * @param error Error, if any
         * @param [response] InitRuntimeResponse
         */
        type GetRuntimeInfoCallback = (error: (Error|null), response?: pruntime_rpc.InitRuntimeResponse) => void;

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#getEgressMessages}.
         * @param error Error, if any
         * @param [response] GetEgressMessagesResponse
         */
        type GetEgressMessagesCallback = (error: (Error|null), response?: pruntime_rpc.GetEgressMessagesResponse) => void;

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#contractQuery}.
         * @param error Error, if any
         * @param [response] ContractQueryResponse
         */
        type ContractQueryCallback = (error: (Error|null), response?: pruntime_rpc.ContractQueryResponse) => void;

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#getWorkerState}.
         * @param error Error, if any
         * @param [response] WorkerState
         */
        type GetWorkerStateCallback = (error: (Error|null), response?: pruntime_rpc.WorkerState) => void;

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#addEndpoint}.
         * @param error Error, if any
         * @param [response] GetEndpointResponse
         */
        type AddEndpointCallback = (error: (Error|null), response?: pruntime_rpc.GetEndpointResponse) => void;

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#refreshEndpointSigningTime}.
         * @param error Error, if any
         * @param [response] GetEndpointResponse
         */
        type RefreshEndpointSigningTimeCallback = (error: (Error|null), response?: pruntime_rpc.GetEndpointResponse) => void;

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#getEndpointInfo}.
         * @param error Error, if any
         * @param [response] GetEndpointResponse
         */
        type GetEndpointInfoCallback = (error: (Error|null), response?: pruntime_rpc.GetEndpointResponse) => void;

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#signEndpointInfo}.
         * @param error Error, if any
         * @param [response] GetEndpointResponse
         */
        type SignEndpointInfoCallback = (error: (Error|null), response?: pruntime_rpc.GetEndpointResponse) => void;

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#derivePhalaI2pKey}.
         * @param error Error, if any
         * @param [response] DerivePhalaI2pKeyResponse
         */
        type DerivePhalaI2pKeyCallback = (error: (Error|null), response?: pruntime_rpc.DerivePhalaI2pKeyResponse) => void;

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#echo}.
         * @param error Error, if any
         * @param [response] EchoMessage
         */
        type EchoCallback = (error: (Error|null), response?: pruntime_rpc.EchoMessage) => void;

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#handoverCreateChallenge}.
         * @param error Error, if any
         * @param [response] HandoverChallenge
         */
        type HandoverCreateChallengeCallback = (error: (Error|null), response?: pruntime_rpc.HandoverChallenge) => void;

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#handoverStart}.
         * @param error Error, if any
         * @param [response] HandoverWorkerKey
         */
        type HandoverStartCallback = (error: (Error|null), response?: pruntime_rpc.HandoverWorkerKey) => void;

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#handoverAcceptChallenge}.
         * @param error Error, if any
         * @param [response] HandoverChallengeResponse
         */
        type HandoverAcceptChallengeCallback = (error: (Error|null), response?: pruntime_rpc.HandoverChallengeResponse) => void;

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#handoverReceive}.
         * @param error Error, if any
         * @param [response] Empty
         */
        type HandoverReceiveCallback = (error: (Error|null), response?: google.protobuf.Empty) => void;

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#configNetwork}.
         * @param error Error, if any
         * @param [response] Empty
         */
        type ConfigNetworkCallback = (error: (Error|null), response?: google.protobuf.Empty) => void;

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#httpFetch}.
         * @param error Error, if any
         * @param [response] HttpResponse
         */
        type HttpFetchCallback = (error: (Error|null), response?: pruntime_rpc.HttpResponse) => void;

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#getContractInfo}.
         * @param error Error, if any
         * @param [response] GetContractInfoResponse
         */
        type GetContractInfoCallback = (error: (Error|null), response?: pruntime_rpc.GetContractInfoResponse) => void;

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#getClusterInfo}.
         * @param error Error, if any
         * @param [response] GetClusterInfoResponse
         */
        type GetClusterInfoCallback = (error: (Error|null), response?: pruntime_rpc.GetClusterInfoResponse) => void;

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#uploadSidevmCode}.
         * @param error Error, if any
         * @param [response] Empty
         */
        type UploadSidevmCodeCallback = (error: (Error|null), response?: google.protobuf.Empty) => void;

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#calculateContractId}.
         * @param error Error, if any
         * @param [response] ContractId
         */
        type CalculateContractIdCallback = (error: (Error|null), response?: pruntime_rpc.ContractId) => void;

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#getNetworkConfig}.
         * @param error Error, if any
         * @param [response] NetworkConfigResponse
         */
        type GetNetworkConfigCallback = (error: (Error|null), response?: pruntime_rpc.NetworkConfigResponse) => void;

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#loadChainState}.
         * @param error Error, if any
         * @param [response] Empty
         */
        type LoadChainStateCallback = (error: (Error|null), response?: google.protobuf.Empty) => void;

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#stop}.
         * @param error Error, if any
         * @param [response] Empty
         */
        type StopCallback = (error: (Error|null), response?: google.protobuf.Empty) => void;

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#loadStorageProof}.
         * @param error Error, if any
         * @param [response] Empty
         */
        type LoadStorageProofCallback = (error: (Error|null), response?: google.protobuf.Empty) => void;

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#takeCheckpoint}.
         * @param error Error, if any
         * @param [response] SyncedTo
         */
        type TakeCheckpointCallback = (error: (Error|null), response?: pruntime_rpc.SyncedTo) => void;

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#statistics}.
         * @param error Error, if any
         * @param [response] StatisticsResponse
         */
        type StatisticsCallback = (error: (Error|null), response?: pruntime_rpc.StatisticsResponse) => void;

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#generateClusterStateRequest}.
         * @param error Error, if any
         * @param [response] SaveClusterStateArguments
         */
        type GenerateClusterStateRequestCallback = (error: (Error|null), response?: pruntime_rpc.SaveClusterStateArguments) => void;

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#saveClusterState}.
         * @param error Error, if any
         * @param [response] SaveClusterStateResponse
         */
        type SaveClusterStateCallback = (error: (Error|null), response?: pruntime_rpc.SaveClusterStateResponse) => void;

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#loadClusterState}.
         * @param error Error, if any
         * @param [response] Empty
         */
        type LoadClusterStateCallback = (error: (Error|null), response?: google.protobuf.Empty) => void;

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#tryUpgradePinkRuntime}.
         * @param error Error, if any
         * @param [response] Empty
         */
        type TryUpgradePinkRuntimeCallback = (error: (Error|null), response?: google.protobuf.Empty) => void;
    }

    /** Properties of a PhactoryInfo. */
    interface IPhactoryInfo {

        /** PhactoryInfo initialized */
        initialized?: (boolean|null);

        /** PhactoryInfo registered */
        registered?: (boolean|null);

        /** PhactoryInfo genesisBlockHash */
        genesisBlockHash?: (string|null);

        /** PhactoryInfo publicKey */
        publicKey?: (string|null);

        /** PhactoryInfo ecdhPublicKey */
        ecdhPublicKey?: (string|null);

        /** PhactoryInfo headernum */
        headernum?: (number|null);

        /** PhactoryInfo paraHeadernum */
        paraHeadernum?: (number|null);

        /** PhactoryInfo blocknum */
        blocknum?: (number|null);

        /** PhactoryInfo stateRoot */
        stateRoot?: (string|null);

        /** PhactoryInfo devMode */
        devMode?: (boolean|null);

        /** PhactoryInfo pendingMessages */
        pendingMessages?: (number|Long|null);

        /** PhactoryInfo score */
        score?: (number|Long|null);

        /** PhactoryInfo gatekeeper */
        gatekeeper?: (pruntime_rpc.IGatekeeperStatus|null);

        /** PhactoryInfo version */
        version?: (string|null);

        /** PhactoryInfo gitRevision */
        gitRevision?: (string|null);

        /** PhactoryInfo memoryUsage */
        memoryUsage?: (pruntime_rpc.IMemoryUsage|null);

        /** PhactoryInfo waitingForParaheaders */
        waitingForParaheaders?: (boolean|null);

        /** PhactoryInfo system */
        system?: (pruntime_rpc.ISystemInfo|null);

        /** PhactoryInfo canLoadChainState */
        canLoadChainState?: (boolean|null);

        /** PhactoryInfo safeModeLevel */
        safeModeLevel?: (number|null);

        /** PhactoryInfo currentBlockTime */
        currentBlockTime?: (number|Long|null);

        /** PhactoryInfo maxSupportedPinkRuntimeVersion */
        maxSupportedPinkRuntimeVersion?: (string|null);
    }

    /** Represents a PhactoryInfo. */
    class PhactoryInfo implements IPhactoryInfo {

        /**
         * Constructs a new PhactoryInfo.
         * @param [properties] Properties to set
         */
        constructor(properties?: pruntime_rpc.IPhactoryInfo);

        /** PhactoryInfo initialized. */
        public initialized: boolean;

        /** PhactoryInfo registered. */
        public registered: boolean;

        /** PhactoryInfo genesisBlockHash. */
        public genesisBlockHash?: (string|null);

        /** PhactoryInfo publicKey. */
        public publicKey?: (string|null);

        /** PhactoryInfo ecdhPublicKey. */
        public ecdhPublicKey?: (string|null);

        /** PhactoryInfo headernum. */
        public headernum: number;

        /** PhactoryInfo paraHeadernum. */
        public paraHeadernum: number;

        /** PhactoryInfo blocknum. */
        public blocknum: number;

        /** PhactoryInfo stateRoot. */
        public stateRoot: string;

        /** PhactoryInfo devMode. */
        public devMode: boolean;

        /** PhactoryInfo pendingMessages. */
        public pendingMessages: (number|Long);

        /** PhactoryInfo score. */
        public score: (number|Long);

        /** PhactoryInfo gatekeeper. */
        public gatekeeper?: (pruntime_rpc.IGatekeeperStatus|null);

        /** PhactoryInfo version. */
        public version: string;

        /** PhactoryInfo gitRevision. */
        public gitRevision: string;

        /** PhactoryInfo memoryUsage. */
        public memoryUsage?: (pruntime_rpc.IMemoryUsage|null);

        /** PhactoryInfo waitingForParaheaders. */
        public waitingForParaheaders: boolean;

        /** PhactoryInfo system. */
        public system?: (pruntime_rpc.ISystemInfo|null);

        /** PhactoryInfo canLoadChainState. */
        public canLoadChainState: boolean;

        /** PhactoryInfo safeModeLevel. */
        public safeModeLevel: number;

        /** PhactoryInfo currentBlockTime. */
        public currentBlockTime: (number|Long);

        /** PhactoryInfo maxSupportedPinkRuntimeVersion. */
        public maxSupportedPinkRuntimeVersion: string;

        /** PhactoryInfo _genesisBlockHash. */
        public _genesisBlockHash?: "genesisBlockHash";

        /** PhactoryInfo _publicKey. */
        public _publicKey?: "publicKey";

        /** PhactoryInfo _ecdhPublicKey. */
        public _ecdhPublicKey?: "ecdhPublicKey";

        /**
         * Creates a new PhactoryInfo instance using the specified properties.
         * @param [properties] Properties to set
         * @returns PhactoryInfo instance
         */
        public static create(properties?: pruntime_rpc.IPhactoryInfo): pruntime_rpc.PhactoryInfo;

        /**
         * Encodes the specified PhactoryInfo message. Does not implicitly {@link pruntime_rpc.PhactoryInfo.verify|verify} messages.
         * @param message PhactoryInfo message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: pruntime_rpc.IPhactoryInfo, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified PhactoryInfo message, length delimited. Does not implicitly {@link pruntime_rpc.PhactoryInfo.verify|verify} messages.
         * @param message PhactoryInfo message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: pruntime_rpc.IPhactoryInfo, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a PhactoryInfo message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns PhactoryInfo
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): pruntime_rpc.PhactoryInfo;

        /**
         * Decodes a PhactoryInfo message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns PhactoryInfo
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): pruntime_rpc.PhactoryInfo;

        /**
         * Verifies a PhactoryInfo message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a PhactoryInfo message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns PhactoryInfo
         */
        public static fromObject(object: { [k: string]: any }): pruntime_rpc.PhactoryInfo;

        /**
         * Creates a plain object from a PhactoryInfo message. Also converts values to other types if specified.
         * @param message PhactoryInfo
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: pruntime_rpc.PhactoryInfo, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this PhactoryInfo to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for PhactoryInfo
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a SystemInfo. */
    interface ISystemInfo {

        /** SystemInfo registered */
        registered?: (boolean|null);

        /** SystemInfo publicKey */
        publicKey?: (string|null);

        /** SystemInfo ecdhPublicKey */
        ecdhPublicKey?: (string|null);

        /** SystemInfo gatekeeper */
        gatekeeper?: (pruntime_rpc.IGatekeeperStatus|null);

        /** SystemInfo numberOfClusters */
        numberOfClusters?: (number|Long|null);

        /** SystemInfo numberOfContracts */
        numberOfContracts?: (number|Long|null);

        /** SystemInfo maxSupportedConsensusVersion */
        maxSupportedConsensusVersion?: (number|null);

        /** SystemInfo genesisBlock */
        genesisBlock?: (number|null);
    }

    /** Represents a SystemInfo. */
    class SystemInfo implements ISystemInfo {

        /**
         * Constructs a new SystemInfo.
         * @param [properties] Properties to set
         */
        constructor(properties?: pruntime_rpc.ISystemInfo);

        /** SystemInfo registered. */
        public registered: boolean;

        /** SystemInfo publicKey. */
        public publicKey: string;

        /** SystemInfo ecdhPublicKey. */
        public ecdhPublicKey: string;

        /** SystemInfo gatekeeper. */
        public gatekeeper?: (pruntime_rpc.IGatekeeperStatus|null);

        /** SystemInfo numberOfClusters. */
        public numberOfClusters: (number|Long);

        /** SystemInfo numberOfContracts. */
        public numberOfContracts: (number|Long);

        /** SystemInfo maxSupportedConsensusVersion. */
        public maxSupportedConsensusVersion: number;

        /** SystemInfo genesisBlock. */
        public genesisBlock: number;

        /**
         * Creates a new SystemInfo instance using the specified properties.
         * @param [properties] Properties to set
         * @returns SystemInfo instance
         */
        public static create(properties?: pruntime_rpc.ISystemInfo): pruntime_rpc.SystemInfo;

        /**
         * Encodes the specified SystemInfo message. Does not implicitly {@link pruntime_rpc.SystemInfo.verify|verify} messages.
         * @param message SystemInfo message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: pruntime_rpc.ISystemInfo, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified SystemInfo message, length delimited. Does not implicitly {@link pruntime_rpc.SystemInfo.verify|verify} messages.
         * @param message SystemInfo message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: pruntime_rpc.ISystemInfo, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a SystemInfo message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns SystemInfo
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): pruntime_rpc.SystemInfo;

        /**
         * Decodes a SystemInfo message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns SystemInfo
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): pruntime_rpc.SystemInfo;

        /**
         * Verifies a SystemInfo message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a SystemInfo message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns SystemInfo
         */
        public static fromObject(object: { [k: string]: any }): pruntime_rpc.SystemInfo;

        /**
         * Creates a plain object from a SystemInfo message. Also converts values to other types if specified.
         * @param message SystemInfo
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: pruntime_rpc.SystemInfo, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this SystemInfo to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for SystemInfo
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** GatekeeperRole enum. */
    enum GatekeeperRole {
        None = 0,
        Dummy = 1,
        Active = 2
    }

    /** Properties of a GatekeeperStatus. */
    interface IGatekeeperStatus {

        /** GatekeeperStatus role */
        role?: (pruntime_rpc.GatekeeperRole|null);

        /** GatekeeperStatus masterPublicKey */
        masterPublicKey?: (string|null);
    }

    /** Represents a GatekeeperStatus. */
    class GatekeeperStatus implements IGatekeeperStatus {

        /**
         * Constructs a new GatekeeperStatus.
         * @param [properties] Properties to set
         */
        constructor(properties?: pruntime_rpc.IGatekeeperStatus);

        /** GatekeeperStatus role. */
        public role: pruntime_rpc.GatekeeperRole;

        /** GatekeeperStatus masterPublicKey. */
        public masterPublicKey: string;

        /**
         * Creates a new GatekeeperStatus instance using the specified properties.
         * @param [properties] Properties to set
         * @returns GatekeeperStatus instance
         */
        public static create(properties?: pruntime_rpc.IGatekeeperStatus): pruntime_rpc.GatekeeperStatus;

        /**
         * Encodes the specified GatekeeperStatus message. Does not implicitly {@link pruntime_rpc.GatekeeperStatus.verify|verify} messages.
         * @param message GatekeeperStatus message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: pruntime_rpc.IGatekeeperStatus, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified GatekeeperStatus message, length delimited. Does not implicitly {@link pruntime_rpc.GatekeeperStatus.verify|verify} messages.
         * @param message GatekeeperStatus message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: pruntime_rpc.IGatekeeperStatus, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a GatekeeperStatus message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns GatekeeperStatus
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): pruntime_rpc.GatekeeperStatus;

        /**
         * Decodes a GatekeeperStatus message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns GatekeeperStatus
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): pruntime_rpc.GatekeeperStatus;

        /**
         * Verifies a GatekeeperStatus message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a GatekeeperStatus message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns GatekeeperStatus
         */
        public static fromObject(object: { [k: string]: any }): pruntime_rpc.GatekeeperStatus;

        /**
         * Creates a plain object from a GatekeeperStatus message. Also converts values to other types if specified.
         * @param message GatekeeperStatus
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: pruntime_rpc.GatekeeperStatus, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this GatekeeperStatus to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for GatekeeperStatus
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a MemoryUsage. */
    interface IMemoryUsage {

        /** MemoryUsage rustUsed */
        rustUsed?: (number|Long|null);

        /** MemoryUsage rustPeakUsed */
        rustPeakUsed?: (number|Long|null);

        /** MemoryUsage totalPeakUsed */
        totalPeakUsed?: (number|Long|null);

        /** MemoryUsage free */
        free?: (number|Long|null);

        /** MemoryUsage rustSpike */
        rustSpike?: (number|Long|null);
    }

    /** Represents a MemoryUsage. */
    class MemoryUsage implements IMemoryUsage {

        /**
         * Constructs a new MemoryUsage.
         * @param [properties] Properties to set
         */
        constructor(properties?: pruntime_rpc.IMemoryUsage);

        /** MemoryUsage rustUsed. */
        public rustUsed: (number|Long);

        /** MemoryUsage rustPeakUsed. */
        public rustPeakUsed: (number|Long);

        /** MemoryUsage totalPeakUsed. */
        public totalPeakUsed: (number|Long);

        /** MemoryUsage free. */
        public free: (number|Long);

        /** MemoryUsage rustSpike. */
        public rustSpike: (number|Long);

        /**
         * Creates a new MemoryUsage instance using the specified properties.
         * @param [properties] Properties to set
         * @returns MemoryUsage instance
         */
        public static create(properties?: pruntime_rpc.IMemoryUsage): pruntime_rpc.MemoryUsage;

        /**
         * Encodes the specified MemoryUsage message. Does not implicitly {@link pruntime_rpc.MemoryUsage.verify|verify} messages.
         * @param message MemoryUsage message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: pruntime_rpc.IMemoryUsage, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified MemoryUsage message, length delimited. Does not implicitly {@link pruntime_rpc.MemoryUsage.verify|verify} messages.
         * @param message MemoryUsage message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: pruntime_rpc.IMemoryUsage, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a MemoryUsage message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns MemoryUsage
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): pruntime_rpc.MemoryUsage;

        /**
         * Decodes a MemoryUsage message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns MemoryUsage
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): pruntime_rpc.MemoryUsage;

        /**
         * Verifies a MemoryUsage message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a MemoryUsage message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns MemoryUsage
         */
        public static fromObject(object: { [k: string]: any }): pruntime_rpc.MemoryUsage;

        /**
         * Creates a plain object from a MemoryUsage message. Also converts values to other types if specified.
         * @param message MemoryUsage
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: pruntime_rpc.MemoryUsage, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this MemoryUsage to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for MemoryUsage
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a GetRuntimeInfoRequest. */
    interface IGetRuntimeInfoRequest {

        /** GetRuntimeInfoRequest forceRefreshRa */
        forceRefreshRa?: (boolean|null);

        /** GetRuntimeInfoRequest encodedOperator */
        encodedOperator?: (Uint8Array|null);
    }

    /** Represents a GetRuntimeInfoRequest. */
    class GetRuntimeInfoRequest implements IGetRuntimeInfoRequest {

        /**
         * Constructs a new GetRuntimeInfoRequest.
         * @param [properties] Properties to set
         */
        constructor(properties?: pruntime_rpc.IGetRuntimeInfoRequest);

        /** GetRuntimeInfoRequest forceRefreshRa. */
        public forceRefreshRa: boolean;

        /** GetRuntimeInfoRequest encodedOperator. */
        public encodedOperator?: (Uint8Array|null);

        /** GetRuntimeInfoRequest _encodedOperator. */
        public _encodedOperator?: "encodedOperator";

        /**
         * Creates a new GetRuntimeInfoRequest instance using the specified properties.
         * @param [properties] Properties to set
         * @returns GetRuntimeInfoRequest instance
         */
        public static create(properties?: pruntime_rpc.IGetRuntimeInfoRequest): pruntime_rpc.GetRuntimeInfoRequest;

        /**
         * Encodes the specified GetRuntimeInfoRequest message. Does not implicitly {@link pruntime_rpc.GetRuntimeInfoRequest.verify|verify} messages.
         * @param message GetRuntimeInfoRequest message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: pruntime_rpc.IGetRuntimeInfoRequest, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified GetRuntimeInfoRequest message, length delimited. Does not implicitly {@link pruntime_rpc.GetRuntimeInfoRequest.verify|verify} messages.
         * @param message GetRuntimeInfoRequest message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: pruntime_rpc.IGetRuntimeInfoRequest, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a GetRuntimeInfoRequest message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns GetRuntimeInfoRequest
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): pruntime_rpc.GetRuntimeInfoRequest;

        /**
         * Decodes a GetRuntimeInfoRequest message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns GetRuntimeInfoRequest
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): pruntime_rpc.GetRuntimeInfoRequest;

        /**
         * Verifies a GetRuntimeInfoRequest message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a GetRuntimeInfoRequest message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns GetRuntimeInfoRequest
         */
        public static fromObject(object: { [k: string]: any }): pruntime_rpc.GetRuntimeInfoRequest;

        /**
         * Creates a plain object from a GetRuntimeInfoRequest message. Also converts values to other types if specified.
         * @param message GetRuntimeInfoRequest
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: pruntime_rpc.GetRuntimeInfoRequest, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this GetRuntimeInfoRequest to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for GetRuntimeInfoRequest
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of an InitRuntimeResponse. */
    interface IInitRuntimeResponse {

        /** InitRuntimeResponse encodedRuntimeInfo */
        encodedRuntimeInfo?: (Uint8Array|null);

        /** InitRuntimeResponse encodedGenesisBlockHash */
        encodedGenesisBlockHash?: (Uint8Array|null);

        /** InitRuntimeResponse encodedPublicKey */
        encodedPublicKey?: (Uint8Array|null);

        /** InitRuntimeResponse encodedEcdhPublicKey */
        encodedEcdhPublicKey?: (Uint8Array|null);

        /** InitRuntimeResponse attestation */
        attestation?: (pruntime_rpc.IAttestation|null);
    }

    /** Represents an InitRuntimeResponse. */
    class InitRuntimeResponse implements IInitRuntimeResponse {

        /**
         * Constructs a new InitRuntimeResponse.
         * @param [properties] Properties to set
         */
        constructor(properties?: pruntime_rpc.IInitRuntimeResponse);

        /** InitRuntimeResponse encodedRuntimeInfo. */
        public encodedRuntimeInfo: Uint8Array;

        /** InitRuntimeResponse encodedGenesisBlockHash. */
        public encodedGenesisBlockHash: Uint8Array;

        /** InitRuntimeResponse encodedPublicKey. */
        public encodedPublicKey: Uint8Array;

        /** InitRuntimeResponse encodedEcdhPublicKey. */
        public encodedEcdhPublicKey: Uint8Array;

        /** InitRuntimeResponse attestation. */
        public attestation?: (pruntime_rpc.IAttestation|null);

        /** InitRuntimeResponse _attestation. */
        public _attestation?: "attestation";

        /**
         * Creates a new InitRuntimeResponse instance using the specified properties.
         * @param [properties] Properties to set
         * @returns InitRuntimeResponse instance
         */
        public static create(properties?: pruntime_rpc.IInitRuntimeResponse): pruntime_rpc.InitRuntimeResponse;

        /**
         * Encodes the specified InitRuntimeResponse message. Does not implicitly {@link pruntime_rpc.InitRuntimeResponse.verify|verify} messages.
         * @param message InitRuntimeResponse message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: pruntime_rpc.IInitRuntimeResponse, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified InitRuntimeResponse message, length delimited. Does not implicitly {@link pruntime_rpc.InitRuntimeResponse.verify|verify} messages.
         * @param message InitRuntimeResponse message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: pruntime_rpc.IInitRuntimeResponse, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes an InitRuntimeResponse message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns InitRuntimeResponse
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): pruntime_rpc.InitRuntimeResponse;

        /**
         * Decodes an InitRuntimeResponse message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns InitRuntimeResponse
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): pruntime_rpc.InitRuntimeResponse;

        /**
         * Verifies an InitRuntimeResponse message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates an InitRuntimeResponse message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns InitRuntimeResponse
         */
        public static fromObject(object: { [k: string]: any }): pruntime_rpc.InitRuntimeResponse;

        /**
         * Creates a plain object from an InitRuntimeResponse message. Also converts values to other types if specified.
         * @param message InitRuntimeResponse
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: pruntime_rpc.InitRuntimeResponse, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this InitRuntimeResponse to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for InitRuntimeResponse
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of an Attestation. */
    interface IAttestation {

        /** Attestation version */
        version?: (number|null);

        /** Attestation provider */
        provider?: (string|null);

        /** Attestation payload */
        payload?: (pruntime_rpc.IAttestationReport|null);

        /** Attestation encodedReport */
        encodedReport?: (Uint8Array|null);

        /** Attestation timestamp */
        timestamp?: (number|Long|null);
    }

    /** Represents an Attestation. */
    class Attestation implements IAttestation {

        /**
         * Constructs a new Attestation.
         * @param [properties] Properties to set
         */
        constructor(properties?: pruntime_rpc.IAttestation);

        /** Attestation version. */
        public version: number;

        /** Attestation provider. */
        public provider: string;

        /** Attestation payload. */
        public payload?: (pruntime_rpc.IAttestationReport|null);

        /** Attestation encodedReport. */
        public encodedReport: Uint8Array;

        /** Attestation timestamp. */
        public timestamp: (number|Long);

        /**
         * Creates a new Attestation instance using the specified properties.
         * @param [properties] Properties to set
         * @returns Attestation instance
         */
        public static create(properties?: pruntime_rpc.IAttestation): pruntime_rpc.Attestation;

        /**
         * Encodes the specified Attestation message. Does not implicitly {@link pruntime_rpc.Attestation.verify|verify} messages.
         * @param message Attestation message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: pruntime_rpc.IAttestation, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified Attestation message, length delimited. Does not implicitly {@link pruntime_rpc.Attestation.verify|verify} messages.
         * @param message Attestation message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: pruntime_rpc.IAttestation, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes an Attestation message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns Attestation
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): pruntime_rpc.Attestation;

        /**
         * Decodes an Attestation message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns Attestation
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): pruntime_rpc.Attestation;

        /**
         * Verifies an Attestation message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates an Attestation message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns Attestation
         */
        public static fromObject(object: { [k: string]: any }): pruntime_rpc.Attestation;

        /**
         * Creates a plain object from an Attestation message. Also converts values to other types if specified.
         * @param message Attestation
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: pruntime_rpc.Attestation, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this Attestation to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for Attestation
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of an AttestationReport. */
    interface IAttestationReport {

        /** AttestationReport report */
        report?: (string|null);

        /** AttestationReport signature */
        signature?: (Uint8Array|null);

        /** AttestationReport signingCert */
        signingCert?: (Uint8Array|null);
    }

    /** Represents an AttestationReport. */
    class AttestationReport implements IAttestationReport {

        /**
         * Constructs a new AttestationReport.
         * @param [properties] Properties to set
         */
        constructor(properties?: pruntime_rpc.IAttestationReport);

        /** AttestationReport report. */
        public report: string;

        /** AttestationReport signature. */
        public signature: Uint8Array;

        /** AttestationReport signingCert. */
        public signingCert: Uint8Array;

        /**
         * Creates a new AttestationReport instance using the specified properties.
         * @param [properties] Properties to set
         * @returns AttestationReport instance
         */
        public static create(properties?: pruntime_rpc.IAttestationReport): pruntime_rpc.AttestationReport;

        /**
         * Encodes the specified AttestationReport message. Does not implicitly {@link pruntime_rpc.AttestationReport.verify|verify} messages.
         * @param message AttestationReport message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: pruntime_rpc.IAttestationReport, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified AttestationReport message, length delimited. Does not implicitly {@link pruntime_rpc.AttestationReport.verify|verify} messages.
         * @param message AttestationReport message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: pruntime_rpc.IAttestationReport, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes an AttestationReport message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns AttestationReport
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): pruntime_rpc.AttestationReport;

        /**
         * Decodes an AttestationReport message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns AttestationReport
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): pruntime_rpc.AttestationReport;

        /**
         * Verifies an AttestationReport message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates an AttestationReport message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns AttestationReport
         */
        public static fromObject(object: { [k: string]: any }): pruntime_rpc.AttestationReport;

        /**
         * Creates a plain object from an AttestationReport message. Also converts values to other types if specified.
         * @param message AttestationReport
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: pruntime_rpc.AttestationReport, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this AttestationReport to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for AttestationReport
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a ContractQueryRequest. */
    interface IContractQueryRequest {

        /** ContractQueryRequest encodedEncryptedData */
        encodedEncryptedData?: (Uint8Array|null);

        /** ContractQueryRequest signature */
        signature?: (pruntime_rpc.ISignature|null);
    }

    /** Represents a ContractQueryRequest. */
    class ContractQueryRequest implements IContractQueryRequest {

        /**
         * Constructs a new ContractQueryRequest.
         * @param [properties] Properties to set
         */
        constructor(properties?: pruntime_rpc.IContractQueryRequest);

        /** ContractQueryRequest encodedEncryptedData. */
        public encodedEncryptedData: Uint8Array;

        /** ContractQueryRequest signature. */
        public signature?: (pruntime_rpc.ISignature|null);

        /**
         * Creates a new ContractQueryRequest instance using the specified properties.
         * @param [properties] Properties to set
         * @returns ContractQueryRequest instance
         */
        public static create(properties?: pruntime_rpc.IContractQueryRequest): pruntime_rpc.ContractQueryRequest;

        /**
         * Encodes the specified ContractQueryRequest message. Does not implicitly {@link pruntime_rpc.ContractQueryRequest.verify|verify} messages.
         * @param message ContractQueryRequest message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: pruntime_rpc.IContractQueryRequest, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified ContractQueryRequest message, length delimited. Does not implicitly {@link pruntime_rpc.ContractQueryRequest.verify|verify} messages.
         * @param message ContractQueryRequest message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: pruntime_rpc.IContractQueryRequest, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a ContractQueryRequest message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns ContractQueryRequest
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): pruntime_rpc.ContractQueryRequest;

        /**
         * Decodes a ContractQueryRequest message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns ContractQueryRequest
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): pruntime_rpc.ContractQueryRequest;

        /**
         * Verifies a ContractQueryRequest message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a ContractQueryRequest message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns ContractQueryRequest
         */
        public static fromObject(object: { [k: string]: any }): pruntime_rpc.ContractQueryRequest;

        /**
         * Creates a plain object from a ContractQueryRequest message. Also converts values to other types if specified.
         * @param message ContractQueryRequest
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: pruntime_rpc.ContractQueryRequest, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this ContractQueryRequest to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for ContractQueryRequest
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a Signature. */
    interface ISignature {

        /** Signature signedBy */
        signedBy?: (pruntime_rpc.ICertificate|null);

        /** Signature signatureType */
        signatureType?: (pruntime_rpc.SignatureType|null);

        /** Signature signature */
        signature?: (Uint8Array|null);
    }

    /** Represents a Signature. */
    class Signature implements ISignature {

        /**
         * Constructs a new Signature.
         * @param [properties] Properties to set
         */
        constructor(properties?: pruntime_rpc.ISignature);

        /** Signature signedBy. */
        public signedBy?: (pruntime_rpc.ICertificate|null);

        /** Signature signatureType. */
        public signatureType: pruntime_rpc.SignatureType;

        /** Signature signature. */
        public signature: Uint8Array;

        /**
         * Creates a new Signature instance using the specified properties.
         * @param [properties] Properties to set
         * @returns Signature instance
         */
        public static create(properties?: pruntime_rpc.ISignature): pruntime_rpc.Signature;

        /**
         * Encodes the specified Signature message. Does not implicitly {@link pruntime_rpc.Signature.verify|verify} messages.
         * @param message Signature message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: pruntime_rpc.ISignature, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified Signature message, length delimited. Does not implicitly {@link pruntime_rpc.Signature.verify|verify} messages.
         * @param message Signature message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: pruntime_rpc.ISignature, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a Signature message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns Signature
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): pruntime_rpc.Signature;

        /**
         * Decodes a Signature message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns Signature
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): pruntime_rpc.Signature;

        /**
         * Verifies a Signature message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a Signature message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns Signature
         */
        public static fromObject(object: { [k: string]: any }): pruntime_rpc.Signature;

        /**
         * Creates a plain object from a Signature message. Also converts values to other types if specified.
         * @param message Signature
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: pruntime_rpc.Signature, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this Signature to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for Signature
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a Certificate. */
    interface ICertificate {

        /** Certificate encodedBody */
        encodedBody?: (Uint8Array|null);

        /** Certificate signature */
        signature?: (pruntime_rpc.ISignature|null);
    }

    /** Represents a Certificate. */
    class Certificate implements ICertificate {

        /**
         * Constructs a new Certificate.
         * @param [properties] Properties to set
         */
        constructor(properties?: pruntime_rpc.ICertificate);

        /** Certificate encodedBody. */
        public encodedBody: Uint8Array;

        /** Certificate signature. */
        public signature?: (pruntime_rpc.ISignature|null);

        /**
         * Creates a new Certificate instance using the specified properties.
         * @param [properties] Properties to set
         * @returns Certificate instance
         */
        public static create(properties?: pruntime_rpc.ICertificate): pruntime_rpc.Certificate;

        /**
         * Encodes the specified Certificate message. Does not implicitly {@link pruntime_rpc.Certificate.verify|verify} messages.
         * @param message Certificate message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: pruntime_rpc.ICertificate, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified Certificate message, length delimited. Does not implicitly {@link pruntime_rpc.Certificate.verify|verify} messages.
         * @param message Certificate message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: pruntime_rpc.ICertificate, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a Certificate message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns Certificate
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): pruntime_rpc.Certificate;

        /**
         * Decodes a Certificate message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns Certificate
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): pruntime_rpc.Certificate;

        /**
         * Verifies a Certificate message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a Certificate message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns Certificate
         */
        public static fromObject(object: { [k: string]: any }): pruntime_rpc.Certificate;

        /**
         * Creates a plain object from a Certificate message. Also converts values to other types if specified.
         * @param message Certificate
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: pruntime_rpc.Certificate, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this Certificate to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for Certificate
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** SignatureType enum. */
    enum SignatureType {
        Ed25519 = 0,
        Sr25519 = 1,
        Ecdsa = 2,
        Ed25519WrapBytes = 3,
        Sr25519WrapBytes = 4,
        EcdsaWrapBytes = 5,
        Eip712 = 6
    }

    /** Properties of a ContractQueryResponse. */
    interface IContractQueryResponse {

        /** ContractQueryResponse encodedEncryptedData */
        encodedEncryptedData?: (Uint8Array|null);
    }

    /** Represents a ContractQueryResponse. */
    class ContractQueryResponse implements IContractQueryResponse {

        /**
         * Constructs a new ContractQueryResponse.
         * @param [properties] Properties to set
         */
        constructor(properties?: pruntime_rpc.IContractQueryResponse);

        /** ContractQueryResponse encodedEncryptedData. */
        public encodedEncryptedData: Uint8Array;

        /**
         * Creates a new ContractQueryResponse instance using the specified properties.
         * @param [properties] Properties to set
         * @returns ContractQueryResponse instance
         */
        public static create(properties?: pruntime_rpc.IContractQueryResponse): pruntime_rpc.ContractQueryResponse;

        /**
         * Encodes the specified ContractQueryResponse message. Does not implicitly {@link pruntime_rpc.ContractQueryResponse.verify|verify} messages.
         * @param message ContractQueryResponse message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: pruntime_rpc.IContractQueryResponse, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified ContractQueryResponse message, length delimited. Does not implicitly {@link pruntime_rpc.ContractQueryResponse.verify|verify} messages.
         * @param message ContractQueryResponse message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: pruntime_rpc.IContractQueryResponse, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a ContractQueryResponse message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns ContractQueryResponse
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): pruntime_rpc.ContractQueryResponse;

        /**
         * Decodes a ContractQueryResponse message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns ContractQueryResponse
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): pruntime_rpc.ContractQueryResponse;

        /**
         * Verifies a ContractQueryResponse message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a ContractQueryResponse message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns ContractQueryResponse
         */
        public static fromObject(object: { [k: string]: any }): pruntime_rpc.ContractQueryResponse;

        /**
         * Creates a plain object from a ContractQueryResponse message. Also converts values to other types if specified.
         * @param message ContractQueryResponse
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: pruntime_rpc.ContractQueryResponse, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this ContractQueryResponse to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for ContractQueryResponse
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of an AddEndpointRequest. */
    interface IAddEndpointRequest {

        /** AddEndpointRequest encodedEndpointType */
        encodedEndpointType?: (Uint8Array|null);

        /** AddEndpointRequest endpoint */
        endpoint?: (string|null);
    }

    /** Represents an AddEndpointRequest. */
    class AddEndpointRequest implements IAddEndpointRequest {

        /**
         * Constructs a new AddEndpointRequest.
         * @param [properties] Properties to set
         */
        constructor(properties?: pruntime_rpc.IAddEndpointRequest);

        /** AddEndpointRequest encodedEndpointType. */
        public encodedEndpointType: Uint8Array;

        /** AddEndpointRequest endpoint. */
        public endpoint: string;

        /**
         * Creates a new AddEndpointRequest instance using the specified properties.
         * @param [properties] Properties to set
         * @returns AddEndpointRequest instance
         */
        public static create(properties?: pruntime_rpc.IAddEndpointRequest): pruntime_rpc.AddEndpointRequest;

        /**
         * Encodes the specified AddEndpointRequest message. Does not implicitly {@link pruntime_rpc.AddEndpointRequest.verify|verify} messages.
         * @param message AddEndpointRequest message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: pruntime_rpc.IAddEndpointRequest, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified AddEndpointRequest message, length delimited. Does not implicitly {@link pruntime_rpc.AddEndpointRequest.verify|verify} messages.
         * @param message AddEndpointRequest message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: pruntime_rpc.IAddEndpointRequest, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes an AddEndpointRequest message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns AddEndpointRequest
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): pruntime_rpc.AddEndpointRequest;

        /**
         * Decodes an AddEndpointRequest message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns AddEndpointRequest
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): pruntime_rpc.AddEndpointRequest;

        /**
         * Verifies an AddEndpointRequest message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates an AddEndpointRequest message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns AddEndpointRequest
         */
        public static fromObject(object: { [k: string]: any }): pruntime_rpc.AddEndpointRequest;

        /**
         * Creates a plain object from an AddEndpointRequest message. Also converts values to other types if specified.
         * @param message AddEndpointRequest
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: pruntime_rpc.AddEndpointRequest, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this AddEndpointRequest to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for AddEndpointRequest
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a GetEndpointResponse. */
    interface IGetEndpointResponse {

        /** GetEndpointResponse encodedEndpointPayload */
        encodedEndpointPayload?: (Uint8Array|null);

        /** GetEndpointResponse signature */
        signature?: (Uint8Array|null);
    }

    /** Represents a GetEndpointResponse. */
    class GetEndpointResponse implements IGetEndpointResponse {

        /**
         * Constructs a new GetEndpointResponse.
         * @param [properties] Properties to set
         */
        constructor(properties?: pruntime_rpc.IGetEndpointResponse);

        /** GetEndpointResponse encodedEndpointPayload. */
        public encodedEndpointPayload?: (Uint8Array|null);

        /** GetEndpointResponse signature. */
        public signature?: (Uint8Array|null);

        /** GetEndpointResponse _encodedEndpointPayload. */
        public _encodedEndpointPayload?: "encodedEndpointPayload";

        /** GetEndpointResponse _signature. */
        public _signature?: "signature";

        /**
         * Creates a new GetEndpointResponse instance using the specified properties.
         * @param [properties] Properties to set
         * @returns GetEndpointResponse instance
         */
        public static create(properties?: pruntime_rpc.IGetEndpointResponse): pruntime_rpc.GetEndpointResponse;

        /**
         * Encodes the specified GetEndpointResponse message. Does not implicitly {@link pruntime_rpc.GetEndpointResponse.verify|verify} messages.
         * @param message GetEndpointResponse message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: pruntime_rpc.IGetEndpointResponse, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified GetEndpointResponse message, length delimited. Does not implicitly {@link pruntime_rpc.GetEndpointResponse.verify|verify} messages.
         * @param message GetEndpointResponse message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: pruntime_rpc.IGetEndpointResponse, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a GetEndpointResponse message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns GetEndpointResponse
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): pruntime_rpc.GetEndpointResponse;

        /**
         * Decodes a GetEndpointResponse message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns GetEndpointResponse
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): pruntime_rpc.GetEndpointResponse;

        /**
         * Verifies a GetEndpointResponse message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a GetEndpointResponse message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns GetEndpointResponse
         */
        public static fromObject(object: { [k: string]: any }): pruntime_rpc.GetEndpointResponse;

        /**
         * Creates a plain object from a GetEndpointResponse message. Also converts values to other types if specified.
         * @param message GetEndpointResponse
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: pruntime_rpc.GetEndpointResponse, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this GetEndpointResponse to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for GetEndpointResponse
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a GetContractInfoRequest. */
    interface IGetContractInfoRequest {

        /** GetContractInfoRequest contracts */
        contracts?: (string[]|null);
    }

    /** Represents a GetContractInfoRequest. */
    class GetContractInfoRequest implements IGetContractInfoRequest {

        /**
         * Constructs a new GetContractInfoRequest.
         * @param [properties] Properties to set
         */
        constructor(properties?: pruntime_rpc.IGetContractInfoRequest);

        /** GetContractInfoRequest contracts. */
        public contracts: string[];

        /**
         * Creates a new GetContractInfoRequest instance using the specified properties.
         * @param [properties] Properties to set
         * @returns GetContractInfoRequest instance
         */
        public static create(properties?: pruntime_rpc.IGetContractInfoRequest): pruntime_rpc.GetContractInfoRequest;

        /**
         * Encodes the specified GetContractInfoRequest message. Does not implicitly {@link pruntime_rpc.GetContractInfoRequest.verify|verify} messages.
         * @param message GetContractInfoRequest message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: pruntime_rpc.IGetContractInfoRequest, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified GetContractInfoRequest message, length delimited. Does not implicitly {@link pruntime_rpc.GetContractInfoRequest.verify|verify} messages.
         * @param message GetContractInfoRequest message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: pruntime_rpc.IGetContractInfoRequest, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a GetContractInfoRequest message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns GetContractInfoRequest
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): pruntime_rpc.GetContractInfoRequest;

        /**
         * Decodes a GetContractInfoRequest message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns GetContractInfoRequest
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): pruntime_rpc.GetContractInfoRequest;

        /**
         * Verifies a GetContractInfoRequest message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a GetContractInfoRequest message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns GetContractInfoRequest
         */
        public static fromObject(object: { [k: string]: any }): pruntime_rpc.GetContractInfoRequest;

        /**
         * Creates a plain object from a GetContractInfoRequest message. Also converts values to other types if specified.
         * @param message GetContractInfoRequest
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: pruntime_rpc.GetContractInfoRequest, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this GetContractInfoRequest to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for GetContractInfoRequest
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a GetContractInfoResponse. */
    interface IGetContractInfoResponse {

        /** GetContractInfoResponse contracts */
        contracts?: (pruntime_rpc.IContractInfo[]|null);
    }

    /** Represents a GetContractInfoResponse. */
    class GetContractInfoResponse implements IGetContractInfoResponse {

        /**
         * Constructs a new GetContractInfoResponse.
         * @param [properties] Properties to set
         */
        constructor(properties?: pruntime_rpc.IGetContractInfoResponse);

        /** GetContractInfoResponse contracts. */
        public contracts: pruntime_rpc.IContractInfo[];

        /**
         * Creates a new GetContractInfoResponse instance using the specified properties.
         * @param [properties] Properties to set
         * @returns GetContractInfoResponse instance
         */
        public static create(properties?: pruntime_rpc.IGetContractInfoResponse): pruntime_rpc.GetContractInfoResponse;

        /**
         * Encodes the specified GetContractInfoResponse message. Does not implicitly {@link pruntime_rpc.GetContractInfoResponse.verify|verify} messages.
         * @param message GetContractInfoResponse message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: pruntime_rpc.IGetContractInfoResponse, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified GetContractInfoResponse message, length delimited. Does not implicitly {@link pruntime_rpc.GetContractInfoResponse.verify|verify} messages.
         * @param message GetContractInfoResponse message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: pruntime_rpc.IGetContractInfoResponse, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a GetContractInfoResponse message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns GetContractInfoResponse
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): pruntime_rpc.GetContractInfoResponse;

        /**
         * Decodes a GetContractInfoResponse message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns GetContractInfoResponse
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): pruntime_rpc.GetContractInfoResponse;

        /**
         * Verifies a GetContractInfoResponse message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a GetContractInfoResponse message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns GetContractInfoResponse
         */
        public static fromObject(object: { [k: string]: any }): pruntime_rpc.GetContractInfoResponse;

        /**
         * Creates a plain object from a GetContractInfoResponse message. Also converts values to other types if specified.
         * @param message GetContractInfoResponse
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: pruntime_rpc.GetContractInfoResponse, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this GetContractInfoResponse to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for GetContractInfoResponse
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a ContractInfo. */
    interface IContractInfo {

        /** ContractInfo id */
        id?: (string|null);

        /** ContractInfo codeHash */
        codeHash?: (string|null);

        /** ContractInfo weight */
        weight?: (number|null);

        /** ContractInfo sidevm */
        sidevm?: (pruntime_rpc.ISidevmInfo|null);
    }

    /** Represents a ContractInfo. */
    class ContractInfo implements IContractInfo {

        /**
         * Constructs a new ContractInfo.
         * @param [properties] Properties to set
         */
        constructor(properties?: pruntime_rpc.IContractInfo);

        /** ContractInfo id. */
        public id: string;

        /** ContractInfo codeHash. */
        public codeHash: string;

        /** ContractInfo weight. */
        public weight: number;

        /** ContractInfo sidevm. */
        public sidevm?: (pruntime_rpc.ISidevmInfo|null);

        /**
         * Creates a new ContractInfo instance using the specified properties.
         * @param [properties] Properties to set
         * @returns ContractInfo instance
         */
        public static create(properties?: pruntime_rpc.IContractInfo): pruntime_rpc.ContractInfo;

        /**
         * Encodes the specified ContractInfo message. Does not implicitly {@link pruntime_rpc.ContractInfo.verify|verify} messages.
         * @param message ContractInfo message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: pruntime_rpc.IContractInfo, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified ContractInfo message, length delimited. Does not implicitly {@link pruntime_rpc.ContractInfo.verify|verify} messages.
         * @param message ContractInfo message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: pruntime_rpc.IContractInfo, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a ContractInfo message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns ContractInfo
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): pruntime_rpc.ContractInfo;

        /**
         * Decodes a ContractInfo message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns ContractInfo
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): pruntime_rpc.ContractInfo;

        /**
         * Verifies a ContractInfo message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a ContractInfo message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns ContractInfo
         */
        public static fromObject(object: { [k: string]: any }): pruntime_rpc.ContractInfo;

        /**
         * Creates a plain object from a ContractInfo message. Also converts values to other types if specified.
         * @param message ContractInfo
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: pruntime_rpc.ContractInfo, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this ContractInfo to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for ContractInfo
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a SidevmInfo. */
    interface ISidevmInfo {

        /** SidevmInfo state */
        state?: (string|null);

        /** SidevmInfo codeHash */
        codeHash?: (string|null);

        /** SidevmInfo startTime */
        startTime?: (string|null);

        /** SidevmInfo stopReason */
        stopReason?: (string|null);
    }

    /** Represents a SidevmInfo. */
    class SidevmInfo implements ISidevmInfo {

        /**
         * Constructs a new SidevmInfo.
         * @param [properties] Properties to set
         */
        constructor(properties?: pruntime_rpc.ISidevmInfo);

        /** SidevmInfo state. */
        public state: string;

        /** SidevmInfo codeHash. */
        public codeHash: string;

        /** SidevmInfo startTime. */
        public startTime: string;

        /** SidevmInfo stopReason. */
        public stopReason: string;

        /**
         * Creates a new SidevmInfo instance using the specified properties.
         * @param [properties] Properties to set
         * @returns SidevmInfo instance
         */
        public static create(properties?: pruntime_rpc.ISidevmInfo): pruntime_rpc.SidevmInfo;

        /**
         * Encodes the specified SidevmInfo message. Does not implicitly {@link pruntime_rpc.SidevmInfo.verify|verify} messages.
         * @param message SidevmInfo message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: pruntime_rpc.ISidevmInfo, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified SidevmInfo message, length delimited. Does not implicitly {@link pruntime_rpc.SidevmInfo.verify|verify} messages.
         * @param message SidevmInfo message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: pruntime_rpc.ISidevmInfo, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a SidevmInfo message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns SidevmInfo
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): pruntime_rpc.SidevmInfo;

        /**
         * Decodes a SidevmInfo message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns SidevmInfo
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): pruntime_rpc.SidevmInfo;

        /**
         * Verifies a SidevmInfo message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a SidevmInfo message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns SidevmInfo
         */
        public static fromObject(object: { [k: string]: any }): pruntime_rpc.SidevmInfo;

        /**
         * Creates a plain object from a SidevmInfo message. Also converts values to other types if specified.
         * @param message SidevmInfo
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: pruntime_rpc.SidevmInfo, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this SidevmInfo to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for SidevmInfo
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a SidevmCode. */
    interface ISidevmCode {

        /** SidevmCode contract */
        contract?: (Uint8Array|null);

        /** SidevmCode code */
        code?: (Uint8Array|null);
    }

    /** Represents a SidevmCode. */
    class SidevmCode implements ISidevmCode {

        /**
         * Constructs a new SidevmCode.
         * @param [properties] Properties to set
         */
        constructor(properties?: pruntime_rpc.ISidevmCode);

        /** SidevmCode contract. */
        public contract: Uint8Array;

        /** SidevmCode code. */
        public code: Uint8Array;

        /**
         * Creates a new SidevmCode instance using the specified properties.
         * @param [properties] Properties to set
         * @returns SidevmCode instance
         */
        public static create(properties?: pruntime_rpc.ISidevmCode): pruntime_rpc.SidevmCode;

        /**
         * Encodes the specified SidevmCode message. Does not implicitly {@link pruntime_rpc.SidevmCode.verify|verify} messages.
         * @param message SidevmCode message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: pruntime_rpc.ISidevmCode, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified SidevmCode message, length delimited. Does not implicitly {@link pruntime_rpc.SidevmCode.verify|verify} messages.
         * @param message SidevmCode message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: pruntime_rpc.ISidevmCode, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a SidevmCode message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns SidevmCode
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): pruntime_rpc.SidevmCode;

        /**
         * Decodes a SidevmCode message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns SidevmCode
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): pruntime_rpc.SidevmCode;

        /**
         * Verifies a SidevmCode message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a SidevmCode message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns SidevmCode
         */
        public static fromObject(object: { [k: string]: any }): pruntime_rpc.SidevmCode;

        /**
         * Creates a plain object from a SidevmCode message. Also converts values to other types if specified.
         * @param message SidevmCode
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: pruntime_rpc.SidevmCode, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this SidevmCode to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for SidevmCode
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a ContractParameters. */
    interface IContractParameters {

        /** ContractParameters deployer */
        deployer?: (string|null);

        /** ContractParameters clusterId */
        clusterId?: (string|null);

        /** ContractParameters codeHash */
        codeHash?: (string|null);

        /** ContractParameters salt */
        salt?: (string|null);
    }

    /** Represents a ContractParameters. */
    class ContractParameters implements IContractParameters {

        /**
         * Constructs a new ContractParameters.
         * @param [properties] Properties to set
         */
        constructor(properties?: pruntime_rpc.IContractParameters);

        /** ContractParameters deployer. */
        public deployer: string;

        /** ContractParameters clusterId. */
        public clusterId: string;

        /** ContractParameters codeHash. */
        public codeHash: string;

        /** ContractParameters salt. */
        public salt: string;

        /**
         * Creates a new ContractParameters instance using the specified properties.
         * @param [properties] Properties to set
         * @returns ContractParameters instance
         */
        public static create(properties?: pruntime_rpc.IContractParameters): pruntime_rpc.ContractParameters;

        /**
         * Encodes the specified ContractParameters message. Does not implicitly {@link pruntime_rpc.ContractParameters.verify|verify} messages.
         * @param message ContractParameters message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: pruntime_rpc.IContractParameters, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified ContractParameters message, length delimited. Does not implicitly {@link pruntime_rpc.ContractParameters.verify|verify} messages.
         * @param message ContractParameters message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: pruntime_rpc.IContractParameters, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a ContractParameters message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns ContractParameters
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): pruntime_rpc.ContractParameters;

        /**
         * Decodes a ContractParameters message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns ContractParameters
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): pruntime_rpc.ContractParameters;

        /**
         * Verifies a ContractParameters message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a ContractParameters message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns ContractParameters
         */
        public static fromObject(object: { [k: string]: any }): pruntime_rpc.ContractParameters;

        /**
         * Creates a plain object from a ContractParameters message. Also converts values to other types if specified.
         * @param message ContractParameters
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: pruntime_rpc.ContractParameters, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this ContractParameters to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for ContractParameters
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a ContractId. */
    interface IContractId {

        /** ContractId id */
        id?: (string|null);
    }

    /** Represents a ContractId. */
    class ContractId implements IContractId {

        /**
         * Constructs a new ContractId.
         * @param [properties] Properties to set
         */
        constructor(properties?: pruntime_rpc.IContractId);

        /** ContractId id. */
        public id: string;

        /**
         * Creates a new ContractId instance using the specified properties.
         * @param [properties] Properties to set
         * @returns ContractId instance
         */
        public static create(properties?: pruntime_rpc.IContractId): pruntime_rpc.ContractId;

        /**
         * Encodes the specified ContractId message. Does not implicitly {@link pruntime_rpc.ContractId.verify|verify} messages.
         * @param message ContractId message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: pruntime_rpc.IContractId, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified ContractId message, length delimited. Does not implicitly {@link pruntime_rpc.ContractId.verify|verify} messages.
         * @param message ContractId message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: pruntime_rpc.IContractId, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a ContractId message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns ContractId
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): pruntime_rpc.ContractId;

        /**
         * Decodes a ContractId message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns ContractId
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): pruntime_rpc.ContractId;

        /**
         * Verifies a ContractId message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a ContractId message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns ContractId
         */
        public static fromObject(object: { [k: string]: any }): pruntime_rpc.ContractId;

        /**
         * Creates a plain object from a ContractId message. Also converts values to other types if specified.
         * @param message ContractId
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: pruntime_rpc.ContractId, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this ContractId to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for ContractId
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }
}

/** Namespace google. */
export namespace google {

    /** Namespace protobuf. */
    namespace protobuf {

        /** Properties of an Empty. */
        interface IEmpty {
        }

        /** Represents an Empty. */
        class Empty implements IEmpty {

            /**
             * Constructs a new Empty.
             * @param [properties] Properties to set
             */
            constructor(properties?: google.protobuf.IEmpty);

            /**
             * Creates a new Empty instance using the specified properties.
             * @param [properties] Properties to set
             * @returns Empty instance
             */
            public static create(properties?: google.protobuf.IEmpty): google.protobuf.Empty;

            /**
             * Encodes the specified Empty message. Does not implicitly {@link google.protobuf.Empty.verify|verify} messages.
             * @param message Empty message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encode(message: google.protobuf.IEmpty, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Encodes the specified Empty message, length delimited. Does not implicitly {@link google.protobuf.Empty.verify|verify} messages.
             * @param message Empty message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encodeDelimited(message: google.protobuf.IEmpty, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Decodes an Empty message from the specified reader or buffer.
             * @param reader Reader or buffer to decode from
             * @param [length] Message length if known beforehand
             * @returns Empty
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): google.protobuf.Empty;

            /**
             * Decodes an Empty message from the specified reader or buffer, length delimited.
             * @param reader Reader or buffer to decode from
             * @returns Empty
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): google.protobuf.Empty;

            /**
             * Verifies an Empty message.
             * @param message Plain object to verify
             * @returns `null` if valid, otherwise the reason why it is not
             */
            public static verify(message: { [k: string]: any }): (string|null);

            /**
             * Creates an Empty message from a plain object. Also converts values to their respective internal types.
             * @param object Plain object
             * @returns Empty
             */
            public static fromObject(object: { [k: string]: any }): google.protobuf.Empty;

            /**
             * Creates a plain object from an Empty message. Also converts values to other types if specified.
             * @param message Empty
             * @param [options] Conversion options
             * @returns Plain object
             */
            public static toObject(message: google.protobuf.Empty, options?: $protobuf.IConversionOptions): { [k: string]: any };

            /**
             * Converts this Empty to JSON.
             * @returns JSON object
             */
            public toJSON(): { [k: string]: any };

            /**
             * Gets the default type url for Empty
             * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns The default type url
             */
            public static getTypeUrl(typeUrlPrefix?: string): string;
        }
    }
}

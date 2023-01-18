import { RegistryTypes, Signer, Codec } from '@polkadot/types/types';
import { ApiPromise } from '@polkadot/api';
import { Signer as Signer$1, SubmittableExtrinsic } from '@polkadot/api/types';
import { Bytes, Compact, u64 } from '@polkadot/types-codec';
import { AccountId } from '@polkadot/types/interfaces';
import { BN } from '@polkadot/util';
import { InjectedAccountWithMeta } from '@polkadot/extension-inject/types';
import { KeyringPair } from '@polkadot/keyring/types';
import * as $protobuf from 'protobufjs';

declare const types: RegistryTypes;

declare const randomHex: (size?: number) => string;

/** Namespace pruntime_rpc. */
declare namespace pruntime_rpc {
  /** Represents a PhactoryAPI */
  class PhactoryAPI extends $protobuf.rpc.Service {
    /**
     * Constructs a new PhactoryAPI service.
     * @param rpcImpl RPC implementation
     * @param [requestDelimited=false] Whether requests are length-delimited
     * @param [responseDelimited=false] Whether responses are length-delimited
     */
    constructor(
      rpcImpl: $protobuf.RPCImpl,
      requestDelimited?: boolean,
      responseDelimited?: boolean
    );

    /**
     * Creates new PhactoryAPI service using the specified rpc implementation.
     * @param rpcImpl RPC implementation
     * @param [requestDelimited=false] Whether requests are length-delimited
     * @param [responseDelimited=false] Whether responses are length-delimited
     * @returns RPC service. Useful where requests and/or responses are streamed.
     */
    public static create(
      rpcImpl: $protobuf.RPCImpl,
      requestDelimited?: boolean,
      responseDelimited?: boolean
    ): PhactoryAPI;

    /**
     * Calls GetInfo.
     * @param request Empty message or plain object
     * @param callback Node-style callback called with the error, if any, and PhactoryInfo
     */
    public getInfo(
      request: google.protobuf.IEmpty,
      callback: pruntime_rpc.PhactoryAPI.GetInfoCallback
    ): void;

    /**
     * Calls GetInfo.
     * @param request Empty message or plain object
     * @returns Promise
     */
    public getInfo(
      request: google.protobuf.IEmpty
    ): Promise<pruntime_rpc.PhactoryInfo>;

    /**
     * Calls SyncHeader.
     * @param request HeadersToSync message or plain object
     * @param callback Node-style callback called with the error, if any, and SyncedTo
     */
    public syncHeader(
      request: pruntime_rpc.IHeadersToSync,
      callback: pruntime_rpc.PhactoryAPI.SyncHeaderCallback
    ): void;

    /**
     * Calls SyncHeader.
     * @param request HeadersToSync message or plain object
     * @returns Promise
     */
    public syncHeader(
      request: pruntime_rpc.IHeadersToSync
    ): Promise<pruntime_rpc.SyncedTo>;

    /**
     * Calls SyncParaHeader.
     * @param request ParaHeadersToSync message or plain object
     * @param callback Node-style callback called with the error, if any, and SyncedTo
     */
    public syncParaHeader(
      request: pruntime_rpc.IParaHeadersToSync,
      callback: pruntime_rpc.PhactoryAPI.SyncParaHeaderCallback
    ): void;

    /**
     * Calls SyncParaHeader.
     * @param request ParaHeadersToSync message or plain object
     * @returns Promise
     */
    public syncParaHeader(
      request: pruntime_rpc.IParaHeadersToSync
    ): Promise<pruntime_rpc.SyncedTo>;

    /**
     * Calls SyncCombinedHeaders.
     * @param request CombinedHeadersToSync message or plain object
     * @param callback Node-style callback called with the error, if any, and HeadersSyncedTo
     */
    public syncCombinedHeaders(
      request: pruntime_rpc.ICombinedHeadersToSync,
      callback: pruntime_rpc.PhactoryAPI.SyncCombinedHeadersCallback
    ): void;

    /**
     * Calls SyncCombinedHeaders.
     * @param request CombinedHeadersToSync message or plain object
     * @returns Promise
     */
    public syncCombinedHeaders(
      request: pruntime_rpc.ICombinedHeadersToSync
    ): Promise<pruntime_rpc.HeadersSyncedTo>;

    /**
     * Calls DispatchBlocks.
     * @param request Blocks message or plain object
     * @param callback Node-style callback called with the error, if any, and SyncedTo
     */
    public dispatchBlocks(
      request: pruntime_rpc.IBlocks,
      callback: pruntime_rpc.PhactoryAPI.DispatchBlocksCallback
    ): void;

    /**
     * Calls DispatchBlocks.
     * @param request Blocks message or plain object
     * @returns Promise
     */
    public dispatchBlocks(
      request: pruntime_rpc.IBlocks
    ): Promise<pruntime_rpc.SyncedTo>;

    /**
     * Calls InitRuntime.
     * @param request InitRuntimeRequest message or plain object
     * @param callback Node-style callback called with the error, if any, and InitRuntimeResponse
     */
    public initRuntime(
      request: pruntime_rpc.IInitRuntimeRequest,
      callback: pruntime_rpc.PhactoryAPI.InitRuntimeCallback
    ): void;

    /**
     * Calls InitRuntime.
     * @param request InitRuntimeRequest message or plain object
     * @returns Promise
     */
    public initRuntime(
      request: pruntime_rpc.IInitRuntimeRequest
    ): Promise<pruntime_rpc.InitRuntimeResponse>;

    /**
     * Calls GetRuntimeInfo.
     * @param request GetRuntimeInfoRequest message or plain object
     * @param callback Node-style callback called with the error, if any, and InitRuntimeResponse
     */
    public getRuntimeInfo(
      request: pruntime_rpc.IGetRuntimeInfoRequest,
      callback: pruntime_rpc.PhactoryAPI.GetRuntimeInfoCallback
    ): void;

    /**
     * Calls GetRuntimeInfo.
     * @param request GetRuntimeInfoRequest message or plain object
     * @returns Promise
     */
    public getRuntimeInfo(
      request: pruntime_rpc.IGetRuntimeInfoRequest
    ): Promise<pruntime_rpc.InitRuntimeResponse>;

    /**
     * Calls GetEgressMessages.
     * @param request Empty message or plain object
     * @param callback Node-style callback called with the error, if any, and GetEgressMessagesResponse
     */
    public getEgressMessages(
      request: google.protobuf.IEmpty,
      callback: pruntime_rpc.PhactoryAPI.GetEgressMessagesCallback
    ): void;

    /**
     * Calls GetEgressMessages.
     * @param request Empty message or plain object
     * @returns Promise
     */
    public getEgressMessages(
      request: google.protobuf.IEmpty
    ): Promise<pruntime_rpc.GetEgressMessagesResponse>;

    /**
     * Calls ContractQuery.
     * @param request ContractQueryRequest message or plain object
     * @param callback Node-style callback called with the error, if any, and ContractQueryResponse
     */
    public contractQuery(
      request: pruntime_rpc.IContractQueryRequest,
      callback: pruntime_rpc.PhactoryAPI.ContractQueryCallback
    ): void;

    /**
     * Calls ContractQuery.
     * @param request ContractQueryRequest message or plain object
     * @returns Promise
     */
    public contractQuery(
      request: pruntime_rpc.IContractQueryRequest
    ): Promise<pruntime_rpc.ContractQueryResponse>;

    /**
     * Calls GetWorkerState.
     * @param request GetWorkerStateRequest message or plain object
     * @param callback Node-style callback called with the error, if any, and WorkerState
     */
    public getWorkerState(
      request: pruntime_rpc.IGetWorkerStateRequest,
      callback: pruntime_rpc.PhactoryAPI.GetWorkerStateCallback
    ): void;

    /**
     * Calls GetWorkerState.
     * @param request GetWorkerStateRequest message or plain object
     * @returns Promise
     */
    public getWorkerState(
      request: pruntime_rpc.IGetWorkerStateRequest
    ): Promise<pruntime_rpc.WorkerState>;

    /**
     * Calls AddEndpoint.
     * @param request AddEndpointRequest message or plain object
     * @param callback Node-style callback called with the error, if any, and GetEndpointResponse
     */
    public addEndpoint(
      request: pruntime_rpc.IAddEndpointRequest,
      callback: pruntime_rpc.PhactoryAPI.AddEndpointCallback
    ): void;

    /**
     * Calls AddEndpoint.
     * @param request AddEndpointRequest message or plain object
     * @returns Promise
     */
    public addEndpoint(
      request: pruntime_rpc.IAddEndpointRequest
    ): Promise<pruntime_rpc.GetEndpointResponse>;

    /**
     * Calls RefreshEndpointSigningTime.
     * @param request Empty message or plain object
     * @param callback Node-style callback called with the error, if any, and GetEndpointResponse
     */
    public refreshEndpointSigningTime(
      request: google.protobuf.IEmpty,
      callback: pruntime_rpc.PhactoryAPI.RefreshEndpointSigningTimeCallback
    ): void;

    /**
     * Calls RefreshEndpointSigningTime.
     * @param request Empty message or plain object
     * @returns Promise
     */
    public refreshEndpointSigningTime(
      request: google.protobuf.IEmpty
    ): Promise<pruntime_rpc.GetEndpointResponse>;

    /**
     * Calls GetEndpointInfo.
     * @param request Empty message or plain object
     * @param callback Node-style callback called with the error, if any, and GetEndpointResponse
     */
    public getEndpointInfo(
      request: google.protobuf.IEmpty,
      callback: pruntime_rpc.PhactoryAPI.GetEndpointInfoCallback
    ): void;

    /**
     * Calls GetEndpointInfo.
     * @param request Empty message or plain object
     * @returns Promise
     */
    public getEndpointInfo(
      request: google.protobuf.IEmpty
    ): Promise<pruntime_rpc.GetEndpointResponse>;

    /**
     * Calls SignEndpointInfo.
     * @param request SignEndpointsRequest message or plain object
     * @param callback Node-style callback called with the error, if any, and GetEndpointResponse
     */
    public signEndpointInfo(
      request: pruntime_rpc.ISignEndpointsRequest,
      callback: pruntime_rpc.PhactoryAPI.SignEndpointInfoCallback
    ): void;

    /**
     * Calls SignEndpointInfo.
     * @param request SignEndpointsRequest message or plain object
     * @returns Promise
     */
    public signEndpointInfo(
      request: pruntime_rpc.ISignEndpointsRequest
    ): Promise<pruntime_rpc.GetEndpointResponse>;

    /**
     * Calls DerivePhalaI2pKey.
     * @param request Empty message or plain object
     * @param callback Node-style callback called with the error, if any, and DerivePhalaI2pKeyResponse
     */
    public derivePhalaI2pKey(
      request: google.protobuf.IEmpty,
      callback: pruntime_rpc.PhactoryAPI.DerivePhalaI2pKeyCallback
    ): void;

    /**
     * Calls DerivePhalaI2pKey.
     * @param request Empty message or plain object
     * @returns Promise
     */
    public derivePhalaI2pKey(
      request: google.protobuf.IEmpty
    ): Promise<pruntime_rpc.DerivePhalaI2pKeyResponse>;

    /**
     * Calls Echo.
     * @param request EchoMessage message or plain object
     * @param callback Node-style callback called with the error, if any, and EchoMessage
     */
    public echo(
      request: pruntime_rpc.IEchoMessage,
      callback: pruntime_rpc.PhactoryAPI.EchoCallback
    ): void;

    /**
     * Calls Echo.
     * @param request EchoMessage message or plain object
     * @returns Promise
     */
    public echo(
      request: pruntime_rpc.IEchoMessage
    ): Promise<pruntime_rpc.EchoMessage>;

    /**
     * Calls HandoverCreateChallenge.
     * @param request Empty message or plain object
     * @param callback Node-style callback called with the error, if any, and HandoverChallenge
     */
    public handoverCreateChallenge(
      request: google.protobuf.IEmpty,
      callback: pruntime_rpc.PhactoryAPI.HandoverCreateChallengeCallback
    ): void;

    /**
     * Calls HandoverCreateChallenge.
     * @param request Empty message or plain object
     * @returns Promise
     */
    public handoverCreateChallenge(
      request: google.protobuf.IEmpty
    ): Promise<pruntime_rpc.HandoverChallenge>;

    /**
     * Calls HandoverStart.
     * @param request HandoverChallengeResponse message or plain object
     * @param callback Node-style callback called with the error, if any, and HandoverWorkerKey
     */
    public handoverStart(
      request: pruntime_rpc.IHandoverChallengeResponse,
      callback: pruntime_rpc.PhactoryAPI.HandoverStartCallback
    ): void;

    /**
     * Calls HandoverStart.
     * @param request HandoverChallengeResponse message or plain object
     * @returns Promise
     */
    public handoverStart(
      request: pruntime_rpc.IHandoverChallengeResponse
    ): Promise<pruntime_rpc.HandoverWorkerKey>;

    /**
     * Calls HandoverAcceptChallenge.
     * @param request HandoverChallenge message or plain object
     * @param callback Node-style callback called with the error, if any, and HandoverChallengeResponse
     */
    public handoverAcceptChallenge(
      request: pruntime_rpc.IHandoverChallenge,
      callback: pruntime_rpc.PhactoryAPI.HandoverAcceptChallengeCallback
    ): void;

    /**
     * Calls HandoverAcceptChallenge.
     * @param request HandoverChallenge message or plain object
     * @returns Promise
     */
    public handoverAcceptChallenge(
      request: pruntime_rpc.IHandoverChallenge
    ): Promise<pruntime_rpc.HandoverChallengeResponse>;

    /**
     * Calls HandoverReceive.
     * @param request HandoverWorkerKey message or plain object
     * @param callback Node-style callback called with the error, if any, and Empty
     */
    public handoverReceive(
      request: pruntime_rpc.IHandoverWorkerKey,
      callback: pruntime_rpc.PhactoryAPI.HandoverReceiveCallback
    ): void;

    /**
     * Calls HandoverReceive.
     * @param request HandoverWorkerKey message or plain object
     * @returns Promise
     */
    public handoverReceive(
      request: pruntime_rpc.IHandoverWorkerKey
    ): Promise<google.protobuf.Empty>;

    /**
     * Calls ConfigNetwork.
     * @param request NetworkConfig message or plain object
     * @param callback Node-style callback called with the error, if any, and Empty
     */
    public configNetwork(
      request: pruntime_rpc.INetworkConfig,
      callback: pruntime_rpc.PhactoryAPI.ConfigNetworkCallback
    ): void;

    /**
     * Calls ConfigNetwork.
     * @param request NetworkConfig message or plain object
     * @returns Promise
     */
    public configNetwork(
      request: pruntime_rpc.INetworkConfig
    ): Promise<google.protobuf.Empty>;

    /**
     * Calls HttpFetch.
     * @param request HttpRequest message or plain object
     * @param callback Node-style callback called with the error, if any, and HttpResponse
     */
    public httpFetch(
      request: pruntime_rpc.IHttpRequest,
      callback: pruntime_rpc.PhactoryAPI.HttpFetchCallback
    ): void;

    /**
     * Calls HttpFetch.
     * @param request HttpRequest message or plain object
     * @returns Promise
     */
    public httpFetch(
      request: pruntime_rpc.IHttpRequest
    ): Promise<pruntime_rpc.HttpResponse>;
  }

  namespace PhactoryAPI {
    /**
     * Callback as used by {@link pruntime_rpc.PhactoryAPI#getInfo}.
     * @param error Error, if any
     * @param [response] PhactoryInfo
     */
    type GetInfoCallback = (
      error: Error | null,
      response?: pruntime_rpc.PhactoryInfo
    ) => void;

    /**
     * Callback as used by {@link pruntime_rpc.PhactoryAPI#syncHeader}.
     * @param error Error, if any
     * @param [response] SyncedTo
     */
    type SyncHeaderCallback = (
      error: Error | null,
      response?: pruntime_rpc.SyncedTo
    ) => void;

    /**
     * Callback as used by {@link pruntime_rpc.PhactoryAPI#syncParaHeader}.
     * @param error Error, if any
     * @param [response] SyncedTo
     */
    type SyncParaHeaderCallback = (
      error: Error | null,
      response?: pruntime_rpc.SyncedTo
    ) => void;

    /**
     * Callback as used by {@link pruntime_rpc.PhactoryAPI#syncCombinedHeaders}.
     * @param error Error, if any
     * @param [response] HeadersSyncedTo
     */
    type SyncCombinedHeadersCallback = (
      error: Error | null,
      response?: pruntime_rpc.HeadersSyncedTo
    ) => void;

    /**
     * Callback as used by {@link pruntime_rpc.PhactoryAPI#dispatchBlocks}.
     * @param error Error, if any
     * @param [response] SyncedTo
     */
    type DispatchBlocksCallback = (
      error: Error | null,
      response?: pruntime_rpc.SyncedTo
    ) => void;

    /**
     * Callback as used by {@link pruntime_rpc.PhactoryAPI#initRuntime}.
     * @param error Error, if any
     * @param [response] InitRuntimeResponse
     */
    type InitRuntimeCallback = (
      error: Error | null,
      response?: pruntime_rpc.InitRuntimeResponse
    ) => void;

    /**
     * Callback as used by {@link pruntime_rpc.PhactoryAPI#getRuntimeInfo}.
     * @param error Error, if any
     * @param [response] InitRuntimeResponse
     */
    type GetRuntimeInfoCallback = (
      error: Error | null,
      response?: pruntime_rpc.InitRuntimeResponse
    ) => void;

    /**
     * Callback as used by {@link pruntime_rpc.PhactoryAPI#getEgressMessages}.
     * @param error Error, if any
     * @param [response] GetEgressMessagesResponse
     */
    type GetEgressMessagesCallback = (
      error: Error | null,
      response?: pruntime_rpc.GetEgressMessagesResponse
    ) => void;

    /**
     * Callback as used by {@link pruntime_rpc.PhactoryAPI#contractQuery}.
     * @param error Error, if any
     * @param [response] ContractQueryResponse
     */
    type ContractQueryCallback = (
      error: Error | null,
      response?: pruntime_rpc.ContractQueryResponse
    ) => void;

    /**
     * Callback as used by {@link pruntime_rpc.PhactoryAPI#getWorkerState}.
     * @param error Error, if any
     * @param [response] WorkerState
     */
    type GetWorkerStateCallback = (
      error: Error | null,
      response?: pruntime_rpc.WorkerState
    ) => void;

    /**
     * Callback as used by {@link pruntime_rpc.PhactoryAPI#addEndpoint}.
     * @param error Error, if any
     * @param [response] GetEndpointResponse
     */
    type AddEndpointCallback = (
      error: Error | null,
      response?: pruntime_rpc.GetEndpointResponse
    ) => void;

    /**
     * Callback as used by {@link pruntime_rpc.PhactoryAPI#refreshEndpointSigningTime}.
     * @param error Error, if any
     * @param [response] GetEndpointResponse
     */
    type RefreshEndpointSigningTimeCallback = (
      error: Error | null,
      response?: pruntime_rpc.GetEndpointResponse
    ) => void;

    /**
     * Callback as used by {@link pruntime_rpc.PhactoryAPI#getEndpointInfo}.
     * @param error Error, if any
     * @param [response] GetEndpointResponse
     */
    type GetEndpointInfoCallback = (
      error: Error | null,
      response?: pruntime_rpc.GetEndpointResponse
    ) => void;

    /**
     * Callback as used by {@link pruntime_rpc.PhactoryAPI#signEndpointInfo}.
     * @param error Error, if any
     * @param [response] GetEndpointResponse
     */
    type SignEndpointInfoCallback = (
      error: Error | null,
      response?: pruntime_rpc.GetEndpointResponse
    ) => void;

    /**
     * Callback as used by {@link pruntime_rpc.PhactoryAPI#derivePhalaI2pKey}.
     * @param error Error, if any
     * @param [response] DerivePhalaI2pKeyResponse
     */
    type DerivePhalaI2pKeyCallback = (
      error: Error | null,
      response?: pruntime_rpc.DerivePhalaI2pKeyResponse
    ) => void;

    /**
     * Callback as used by {@link pruntime_rpc.PhactoryAPI#echo}.
     * @param error Error, if any
     * @param [response] EchoMessage
     */
    type EchoCallback = (
      error: Error | null,
      response?: pruntime_rpc.EchoMessage
    ) => void;

    /**
     * Callback as used by {@link pruntime_rpc.PhactoryAPI#handoverCreateChallenge}.
     * @param error Error, if any
     * @param [response] HandoverChallenge
     */
    type HandoverCreateChallengeCallback = (
      error: Error | null,
      response?: pruntime_rpc.HandoverChallenge
    ) => void;

    /**
     * Callback as used by {@link pruntime_rpc.PhactoryAPI#handoverStart}.
     * @param error Error, if any
     * @param [response] HandoverWorkerKey
     */
    type HandoverStartCallback = (
      error: Error | null,
      response?: pruntime_rpc.HandoverWorkerKey
    ) => void;

    /**
     * Callback as used by {@link pruntime_rpc.PhactoryAPI#handoverAcceptChallenge}.
     * @param error Error, if any
     * @param [response] HandoverChallengeResponse
     */
    type HandoverAcceptChallengeCallback = (
      error: Error | null,
      response?: pruntime_rpc.HandoverChallengeResponse
    ) => void;

    /**
     * Callback as used by {@link pruntime_rpc.PhactoryAPI#handoverReceive}.
     * @param error Error, if any
     * @param [response] Empty
     */
    type HandoverReceiveCallback = (
      error: Error | null,
      response?: google.protobuf.Empty
    ) => void;

    /**
     * Callback as used by {@link pruntime_rpc.PhactoryAPI#configNetwork}.
     * @param error Error, if any
     * @param [response] Empty
     */
    type ConfigNetworkCallback = (
      error: Error | null,
      response?: google.protobuf.Empty
    ) => void;

    /**
     * Callback as used by {@link pruntime_rpc.PhactoryAPI#httpFetch}.
     * @param error Error, if any
     * @param [response] HttpResponse
     */
    type HttpFetchCallback = (
      error: Error | null,
      response?: pruntime_rpc.HttpResponse
    ) => void;
  }

  /** Properties of a PhactoryInfo. */
  interface IPhactoryInfo {
    /** PhactoryInfo initialized */
    initialized?: boolean | null;

    /** PhactoryInfo registered */
    registered?: boolean | null;

    /** PhactoryInfo genesisBlockHash */
    genesisBlockHash?: string | null;

    /** PhactoryInfo publicKey */
    publicKey?: string | null;

    /** PhactoryInfo ecdhPublicKey */
    ecdhPublicKey?: string | null;

    /** PhactoryInfo headernum */
    headernum?: number | null;

    /** PhactoryInfo paraHeadernum */
    paraHeadernum?: number | null;

    /** PhactoryInfo blocknum */
    blocknum?: number | null;

    /** PhactoryInfo stateRoot */
    stateRoot?: string | null;

    /** PhactoryInfo devMode */
    devMode?: boolean | null;

    /** PhactoryInfo pendingMessages */
    pendingMessages?: number | Long | null;

    /** PhactoryInfo score */
    score?: number | Long | null;

    /** PhactoryInfo gatekeeper */
    gatekeeper?: pruntime_rpc.IGatekeeperStatus | null;

    /** PhactoryInfo version */
    version?: string | null;

    /** PhactoryInfo gitRevision */
    gitRevision?: string | null;

    /** PhactoryInfo runningSideTasks */
    runningSideTasks?: number | Long | null;

    /** PhactoryInfo memoryUsage */
    memoryUsage?: pruntime_rpc.IMemoryUsage | null;

    /** PhactoryInfo waitingForParaheaders */
    waitingForParaheaders?: boolean | null;

    /** PhactoryInfo networkStatus */
    networkStatus?: pruntime_rpc.INetworkStatus | null;

    /** PhactoryInfo system */
    system?: pruntime_rpc.ISystemInfo | null;
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
    public genesisBlockHash?: string | null;

    /** PhactoryInfo publicKey. */
    public publicKey?: string | null;

    /** PhactoryInfo ecdhPublicKey. */
    public ecdhPublicKey?: string | null;

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
    public pendingMessages: number | Long;

    /** PhactoryInfo score. */
    public score: number | Long;

    /** PhactoryInfo gatekeeper. */
    public gatekeeper?: pruntime_rpc.IGatekeeperStatus | null;

    /** PhactoryInfo version. */
    public version: string;

    /** PhactoryInfo gitRevision. */
    public gitRevision: string;

    /** PhactoryInfo runningSideTasks. */
    public runningSideTasks: number | Long;

    /** PhactoryInfo memoryUsage. */
    public memoryUsage?: pruntime_rpc.IMemoryUsage | null;

    /** PhactoryInfo waitingForParaheaders. */
    public waitingForParaheaders: boolean;

    /** PhactoryInfo networkStatus. */
    public networkStatus?: pruntime_rpc.INetworkStatus | null;

    /** PhactoryInfo system. */
    public system?: pruntime_rpc.ISystemInfo | null;

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
    public static create(
      properties?: pruntime_rpc.IPhactoryInfo
    ): pruntime_rpc.PhactoryInfo;

    /**
     * Encodes the specified PhactoryInfo message. Does not implicitly {@link pruntime_rpc.PhactoryInfo.verify|verify} messages.
     * @param message PhactoryInfo message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encode(
      message: pruntime_rpc.IPhactoryInfo,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Encodes the specified PhactoryInfo message, length delimited. Does not implicitly {@link pruntime_rpc.PhactoryInfo.verify|verify} messages.
     * @param message PhactoryInfo message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encodeDelimited(
      message: pruntime_rpc.IPhactoryInfo,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Decodes a PhactoryInfo message from the specified reader or buffer.
     * @param reader Reader or buffer to decode from
     * @param [length] Message length if known beforehand
     * @returns PhactoryInfo
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decode(
      reader: $protobuf.Reader | Uint8Array,
      length?: number
    ): pruntime_rpc.PhactoryInfo;

    /**
     * Decodes a PhactoryInfo message from the specified reader or buffer, length delimited.
     * @param reader Reader or buffer to decode from
     * @returns PhactoryInfo
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decodeDelimited(
      reader: $protobuf.Reader | Uint8Array
    ): pruntime_rpc.PhactoryInfo;

    /**
     * Verifies a PhactoryInfo message.
     * @param message Plain object to verify
     * @returns `null` if valid, otherwise the reason why it is not
     */
    public static verify(message: { [k: string]: any }): string | null;

    /**
     * Creates a PhactoryInfo message from a plain object. Also converts values to their respective internal types.
     * @param object Plain object
     * @returns PhactoryInfo
     */
    public static fromObject(object: {
      [k: string]: any;
    }): pruntime_rpc.PhactoryInfo;

    /**
     * Creates a plain object from a PhactoryInfo message. Also converts values to other types if specified.
     * @param message PhactoryInfo
     * @param [options] Conversion options
     * @returns Plain object
     */
    public static toObject(
      message: pruntime_rpc.PhactoryInfo,
      options?: $protobuf.IConversionOptions
    ): { [k: string]: any };

    /**
     * Converts this PhactoryInfo to JSON.
     * @returns JSON object
     */
    public toJSON(): { [k: string]: any };
  }

  /** Properties of a SystemInfo. */
  interface ISystemInfo {
    /** SystemInfo registered */
    registered?: boolean | null;

    /** SystemInfo publicKey */
    publicKey?: string | null;

    /** SystemInfo ecdhPublicKey */
    ecdhPublicKey?: string | null;

    /** SystemInfo gatekeeper */
    gatekeeper?: pruntime_rpc.IGatekeeperStatus | null;

    /** SystemInfo numberOfClusters */
    numberOfClusters?: number | Long | null;

    /** SystemInfo numberOfContracts */
    numberOfContracts?: number | Long | null;

    /** SystemInfo consensusVersion */
    consensusVersion?: number | null;

    /** SystemInfo maxSupportedConsensusVersion */
    maxSupportedConsensusVersion?: number | null;
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
    public gatekeeper?: pruntime_rpc.IGatekeeperStatus | null;

    /** SystemInfo numberOfClusters. */
    public numberOfClusters: number | Long;

    /** SystemInfo numberOfContracts. */
    public numberOfContracts: number | Long;

    /** SystemInfo consensusVersion. */
    public consensusVersion: number;

    /** SystemInfo maxSupportedConsensusVersion. */
    public maxSupportedConsensusVersion: number;

    /**
     * Creates a new SystemInfo instance using the specified properties.
     * @param [properties] Properties to set
     * @returns SystemInfo instance
     */
    public static create(
      properties?: pruntime_rpc.ISystemInfo
    ): pruntime_rpc.SystemInfo;

    /**
     * Encodes the specified SystemInfo message. Does not implicitly {@link pruntime_rpc.SystemInfo.verify|verify} messages.
     * @param message SystemInfo message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encode(
      message: pruntime_rpc.ISystemInfo,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Encodes the specified SystemInfo message, length delimited. Does not implicitly {@link pruntime_rpc.SystemInfo.verify|verify} messages.
     * @param message SystemInfo message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encodeDelimited(
      message: pruntime_rpc.ISystemInfo,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Decodes a SystemInfo message from the specified reader or buffer.
     * @param reader Reader or buffer to decode from
     * @param [length] Message length if known beforehand
     * @returns SystemInfo
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decode(
      reader: $protobuf.Reader | Uint8Array,
      length?: number
    ): pruntime_rpc.SystemInfo;

    /**
     * Decodes a SystemInfo message from the specified reader or buffer, length delimited.
     * @param reader Reader or buffer to decode from
     * @returns SystemInfo
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decodeDelimited(
      reader: $protobuf.Reader | Uint8Array
    ): pruntime_rpc.SystemInfo;

    /**
     * Verifies a SystemInfo message.
     * @param message Plain object to verify
     * @returns `null` if valid, otherwise the reason why it is not
     */
    public static verify(message: { [k: string]: any }): string | null;

    /**
     * Creates a SystemInfo message from a plain object. Also converts values to their respective internal types.
     * @param object Plain object
     * @returns SystemInfo
     */
    public static fromObject(object: {
      [k: string]: any;
    }): pruntime_rpc.SystemInfo;

    /**
     * Creates a plain object from a SystemInfo message. Also converts values to other types if specified.
     * @param message SystemInfo
     * @param [options] Conversion options
     * @returns Plain object
     */
    public static toObject(
      message: pruntime_rpc.SystemInfo,
      options?: $protobuf.IConversionOptions
    ): { [k: string]: any };

    /**
     * Converts this SystemInfo to JSON.
     * @returns JSON object
     */
    public toJSON(): { [k: string]: any };
  }

  /** GatekeeperRole enum. */
  enum GatekeeperRole {
    None = 0,
    Dummy = 1,
    Active = 2,
  }

  /** Properties of a GatekeeperStatus. */
  interface IGatekeeperStatus {
    /** GatekeeperStatus role */
    role?: pruntime_rpc.GatekeeperRole | null;

    /** GatekeeperStatus masterPublicKey */
    masterPublicKey?: string | null;
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
    public static create(
      properties?: pruntime_rpc.IGatekeeperStatus
    ): pruntime_rpc.GatekeeperStatus;

    /**
     * Encodes the specified GatekeeperStatus message. Does not implicitly {@link pruntime_rpc.GatekeeperStatus.verify|verify} messages.
     * @param message GatekeeperStatus message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encode(
      message: pruntime_rpc.IGatekeeperStatus,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Encodes the specified GatekeeperStatus message, length delimited. Does not implicitly {@link pruntime_rpc.GatekeeperStatus.verify|verify} messages.
     * @param message GatekeeperStatus message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encodeDelimited(
      message: pruntime_rpc.IGatekeeperStatus,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Decodes a GatekeeperStatus message from the specified reader or buffer.
     * @param reader Reader or buffer to decode from
     * @param [length] Message length if known beforehand
     * @returns GatekeeperStatus
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decode(
      reader: $protobuf.Reader | Uint8Array,
      length?: number
    ): pruntime_rpc.GatekeeperStatus;

    /**
     * Decodes a GatekeeperStatus message from the specified reader or buffer, length delimited.
     * @param reader Reader or buffer to decode from
     * @returns GatekeeperStatus
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decodeDelimited(
      reader: $protobuf.Reader | Uint8Array
    ): pruntime_rpc.GatekeeperStatus;

    /**
     * Verifies a GatekeeperStatus message.
     * @param message Plain object to verify
     * @returns `null` if valid, otherwise the reason why it is not
     */
    public static verify(message: { [k: string]: any }): string | null;

    /**
     * Creates a GatekeeperStatus message from a plain object. Also converts values to their respective internal types.
     * @param object Plain object
     * @returns GatekeeperStatus
     */
    public static fromObject(object: {
      [k: string]: any;
    }): pruntime_rpc.GatekeeperStatus;

    /**
     * Creates a plain object from a GatekeeperStatus message. Also converts values to other types if specified.
     * @param message GatekeeperStatus
     * @param [options] Conversion options
     * @returns Plain object
     */
    public static toObject(
      message: pruntime_rpc.GatekeeperStatus,
      options?: $protobuf.IConversionOptions
    ): { [k: string]: any };

    /**
     * Converts this GatekeeperStatus to JSON.
     * @returns JSON object
     */
    public toJSON(): { [k: string]: any };
  }

  /** Properties of a MemoryUsage. */
  interface IMemoryUsage {
    /** MemoryUsage rustUsed */
    rustUsed?: number | Long | null;

    /** MemoryUsage rustPeakUsed */
    rustPeakUsed?: number | Long | null;

    /** MemoryUsage totalPeakUsed */
    totalPeakUsed?: number | Long | null;
  }

  /** Represents a MemoryUsage. */
  class MemoryUsage implements IMemoryUsage {
    /**
     * Constructs a new MemoryUsage.
     * @param [properties] Properties to set
     */
    constructor(properties?: pruntime_rpc.IMemoryUsage);

    /** MemoryUsage rustUsed. */
    public rustUsed: number | Long;

    /** MemoryUsage rustPeakUsed. */
    public rustPeakUsed: number | Long;

    /** MemoryUsage totalPeakUsed. */
    public totalPeakUsed: number | Long;

    /**
     * Creates a new MemoryUsage instance using the specified properties.
     * @param [properties] Properties to set
     * @returns MemoryUsage instance
     */
    public static create(
      properties?: pruntime_rpc.IMemoryUsage
    ): pruntime_rpc.MemoryUsage;

    /**
     * Encodes the specified MemoryUsage message. Does not implicitly {@link pruntime_rpc.MemoryUsage.verify|verify} messages.
     * @param message MemoryUsage message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encode(
      message: pruntime_rpc.IMemoryUsage,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Encodes the specified MemoryUsage message, length delimited. Does not implicitly {@link pruntime_rpc.MemoryUsage.verify|verify} messages.
     * @param message MemoryUsage message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encodeDelimited(
      message: pruntime_rpc.IMemoryUsage,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Decodes a MemoryUsage message from the specified reader or buffer.
     * @param reader Reader or buffer to decode from
     * @param [length] Message length if known beforehand
     * @returns MemoryUsage
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decode(
      reader: $protobuf.Reader | Uint8Array,
      length?: number
    ): pruntime_rpc.MemoryUsage;

    /**
     * Decodes a MemoryUsage message from the specified reader or buffer, length delimited.
     * @param reader Reader or buffer to decode from
     * @returns MemoryUsage
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decodeDelimited(
      reader: $protobuf.Reader | Uint8Array
    ): pruntime_rpc.MemoryUsage;

    /**
     * Verifies a MemoryUsage message.
     * @param message Plain object to verify
     * @returns `null` if valid, otherwise the reason why it is not
     */
    public static verify(message: { [k: string]: any }): string | null;

    /**
     * Creates a MemoryUsage message from a plain object. Also converts values to their respective internal types.
     * @param object Plain object
     * @returns MemoryUsage
     */
    public static fromObject(object: {
      [k: string]: any;
    }): pruntime_rpc.MemoryUsage;

    /**
     * Creates a plain object from a MemoryUsage message. Also converts values to other types if specified.
     * @param message MemoryUsage
     * @param [options] Conversion options
     * @returns Plain object
     */
    public static toObject(
      message: pruntime_rpc.MemoryUsage,
      options?: $protobuf.IConversionOptions
    ): { [k: string]: any };

    /**
     * Converts this MemoryUsage to JSON.
     * @returns JSON object
     */
    public toJSON(): { [k: string]: any };
  }

  /** Properties of a SyncedTo. */
  interface ISyncedTo {
    /** SyncedTo syncedTo */
    syncedTo?: number | null;
  }

  /** Represents a SyncedTo. */
  class SyncedTo implements ISyncedTo {
    /**
     * Constructs a new SyncedTo.
     * @param [properties] Properties to set
     */
    constructor(properties?: pruntime_rpc.ISyncedTo);

    /** SyncedTo syncedTo. */
    public syncedTo: number;

    /**
     * Creates a new SyncedTo instance using the specified properties.
     * @param [properties] Properties to set
     * @returns SyncedTo instance
     */
    public static create(
      properties?: pruntime_rpc.ISyncedTo
    ): pruntime_rpc.SyncedTo;

    /**
     * Encodes the specified SyncedTo message. Does not implicitly {@link pruntime_rpc.SyncedTo.verify|verify} messages.
     * @param message SyncedTo message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encode(
      message: pruntime_rpc.ISyncedTo,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Encodes the specified SyncedTo message, length delimited. Does not implicitly {@link pruntime_rpc.SyncedTo.verify|verify} messages.
     * @param message SyncedTo message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encodeDelimited(
      message: pruntime_rpc.ISyncedTo,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Decodes a SyncedTo message from the specified reader or buffer.
     * @param reader Reader or buffer to decode from
     * @param [length] Message length if known beforehand
     * @returns SyncedTo
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decode(
      reader: $protobuf.Reader | Uint8Array,
      length?: number
    ): pruntime_rpc.SyncedTo;

    /**
     * Decodes a SyncedTo message from the specified reader or buffer, length delimited.
     * @param reader Reader or buffer to decode from
     * @returns SyncedTo
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decodeDelimited(
      reader: $protobuf.Reader | Uint8Array
    ): pruntime_rpc.SyncedTo;

    /**
     * Verifies a SyncedTo message.
     * @param message Plain object to verify
     * @returns `null` if valid, otherwise the reason why it is not
     */
    public static verify(message: { [k: string]: any }): string | null;

    /**
     * Creates a SyncedTo message from a plain object. Also converts values to their respective internal types.
     * @param object Plain object
     * @returns SyncedTo
     */
    public static fromObject(object: {
      [k: string]: any;
    }): pruntime_rpc.SyncedTo;

    /**
     * Creates a plain object from a SyncedTo message. Also converts values to other types if specified.
     * @param message SyncedTo
     * @param [options] Conversion options
     * @returns Plain object
     */
    public static toObject(
      message: pruntime_rpc.SyncedTo,
      options?: $protobuf.IConversionOptions
    ): { [k: string]: any };

    /**
     * Converts this SyncedTo to JSON.
     * @returns JSON object
     */
    public toJSON(): { [k: string]: any };
  }

  /** Properties of a HeadersToSync. */
  interface IHeadersToSync {
    /** HeadersToSync encodedHeaders */
    encodedHeaders?: Uint8Array | null;

    /** HeadersToSync encodedAuthoritySetChange */
    encodedAuthoritySetChange?: Uint8Array | null;
  }

  /** Represents a HeadersToSync. */
  class HeadersToSync implements IHeadersToSync {
    /**
     * Constructs a new HeadersToSync.
     * @param [properties] Properties to set
     */
    constructor(properties?: pruntime_rpc.IHeadersToSync);

    /** HeadersToSync encodedHeaders. */
    public encodedHeaders: Uint8Array;

    /** HeadersToSync encodedAuthoritySetChange. */
    public encodedAuthoritySetChange?: Uint8Array | null;

    /** HeadersToSync _encodedAuthoritySetChange. */
    public _encodedAuthoritySetChange?: "encodedAuthoritySetChange";

    /**
     * Creates a new HeadersToSync instance using the specified properties.
     * @param [properties] Properties to set
     * @returns HeadersToSync instance
     */
    public static create(
      properties?: pruntime_rpc.IHeadersToSync
    ): pruntime_rpc.HeadersToSync;

    /**
     * Encodes the specified HeadersToSync message. Does not implicitly {@link pruntime_rpc.HeadersToSync.verify|verify} messages.
     * @param message HeadersToSync message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encode(
      message: pruntime_rpc.IHeadersToSync,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Encodes the specified HeadersToSync message, length delimited. Does not implicitly {@link pruntime_rpc.HeadersToSync.verify|verify} messages.
     * @param message HeadersToSync message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encodeDelimited(
      message: pruntime_rpc.IHeadersToSync,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Decodes a HeadersToSync message from the specified reader or buffer.
     * @param reader Reader or buffer to decode from
     * @param [length] Message length if known beforehand
     * @returns HeadersToSync
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decode(
      reader: $protobuf.Reader | Uint8Array,
      length?: number
    ): pruntime_rpc.HeadersToSync;

    /**
     * Decodes a HeadersToSync message from the specified reader or buffer, length delimited.
     * @param reader Reader or buffer to decode from
     * @returns HeadersToSync
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decodeDelimited(
      reader: $protobuf.Reader | Uint8Array
    ): pruntime_rpc.HeadersToSync;

    /**
     * Verifies a HeadersToSync message.
     * @param message Plain object to verify
     * @returns `null` if valid, otherwise the reason why it is not
     */
    public static verify(message: { [k: string]: any }): string | null;

    /**
     * Creates a HeadersToSync message from a plain object. Also converts values to their respective internal types.
     * @param object Plain object
     * @returns HeadersToSync
     */
    public static fromObject(object: {
      [k: string]: any;
    }): pruntime_rpc.HeadersToSync;

    /**
     * Creates a plain object from a HeadersToSync message. Also converts values to other types if specified.
     * @param message HeadersToSync
     * @param [options] Conversion options
     * @returns Plain object
     */
    public static toObject(
      message: pruntime_rpc.HeadersToSync,
      options?: $protobuf.IConversionOptions
    ): { [k: string]: any };

    /**
     * Converts this HeadersToSync to JSON.
     * @returns JSON object
     */
    public toJSON(): { [k: string]: any };
  }

  /** Properties of a ParaHeadersToSync. */
  interface IParaHeadersToSync {
    /** ParaHeadersToSync encodedHeaders */
    encodedHeaders?: Uint8Array | null;

    /** ParaHeadersToSync proof */
    proof?: Uint8Array[] | null;
  }

  /** Represents a ParaHeadersToSync. */
  class ParaHeadersToSync implements IParaHeadersToSync {
    /**
     * Constructs a new ParaHeadersToSync.
     * @param [properties] Properties to set
     */
    constructor(properties?: pruntime_rpc.IParaHeadersToSync);

    /** ParaHeadersToSync encodedHeaders. */
    public encodedHeaders: Uint8Array;

    /** ParaHeadersToSync proof. */
    public proof: Uint8Array[];

    /**
     * Creates a new ParaHeadersToSync instance using the specified properties.
     * @param [properties] Properties to set
     * @returns ParaHeadersToSync instance
     */
    public static create(
      properties?: pruntime_rpc.IParaHeadersToSync
    ): pruntime_rpc.ParaHeadersToSync;

    /**
     * Encodes the specified ParaHeadersToSync message. Does not implicitly {@link pruntime_rpc.ParaHeadersToSync.verify|verify} messages.
     * @param message ParaHeadersToSync message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encode(
      message: pruntime_rpc.IParaHeadersToSync,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Encodes the specified ParaHeadersToSync message, length delimited. Does not implicitly {@link pruntime_rpc.ParaHeadersToSync.verify|verify} messages.
     * @param message ParaHeadersToSync message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encodeDelimited(
      message: pruntime_rpc.IParaHeadersToSync,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Decodes a ParaHeadersToSync message from the specified reader or buffer.
     * @param reader Reader or buffer to decode from
     * @param [length] Message length if known beforehand
     * @returns ParaHeadersToSync
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decode(
      reader: $protobuf.Reader | Uint8Array,
      length?: number
    ): pruntime_rpc.ParaHeadersToSync;

    /**
     * Decodes a ParaHeadersToSync message from the specified reader or buffer, length delimited.
     * @param reader Reader or buffer to decode from
     * @returns ParaHeadersToSync
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decodeDelimited(
      reader: $protobuf.Reader | Uint8Array
    ): pruntime_rpc.ParaHeadersToSync;

    /**
     * Verifies a ParaHeadersToSync message.
     * @param message Plain object to verify
     * @returns `null` if valid, otherwise the reason why it is not
     */
    public static verify(message: { [k: string]: any }): string | null;

    /**
     * Creates a ParaHeadersToSync message from a plain object. Also converts values to their respective internal types.
     * @param object Plain object
     * @returns ParaHeadersToSync
     */
    public static fromObject(object: {
      [k: string]: any;
    }): pruntime_rpc.ParaHeadersToSync;

    /**
     * Creates a plain object from a ParaHeadersToSync message. Also converts values to other types if specified.
     * @param message ParaHeadersToSync
     * @param [options] Conversion options
     * @returns Plain object
     */
    public static toObject(
      message: pruntime_rpc.ParaHeadersToSync,
      options?: $protobuf.IConversionOptions
    ): { [k: string]: any };

    /**
     * Converts this ParaHeadersToSync to JSON.
     * @returns JSON object
     */
    public toJSON(): { [k: string]: any };
  }

  /** Properties of a CombinedHeadersToSync. */
  interface ICombinedHeadersToSync {
    /** CombinedHeadersToSync encodedRelaychainHeaders */
    encodedRelaychainHeaders?: Uint8Array | null;

    /** CombinedHeadersToSync authoritySetChange */
    authoritySetChange?: Uint8Array | null;

    /** CombinedHeadersToSync encodedParachainHeaders */
    encodedParachainHeaders?: Uint8Array | null;

    /** CombinedHeadersToSync proof */
    proof?: Uint8Array[] | null;
  }

  /** Represents a CombinedHeadersToSync. */
  class CombinedHeadersToSync implements ICombinedHeadersToSync {
    /**
     * Constructs a new CombinedHeadersToSync.
     * @param [properties] Properties to set
     */
    constructor(properties?: pruntime_rpc.ICombinedHeadersToSync);

    /** CombinedHeadersToSync encodedRelaychainHeaders. */
    public encodedRelaychainHeaders: Uint8Array;

    /** CombinedHeadersToSync authoritySetChange. */
    public authoritySetChange?: Uint8Array | null;

    /** CombinedHeadersToSync encodedParachainHeaders. */
    public encodedParachainHeaders: Uint8Array;

    /** CombinedHeadersToSync proof. */
    public proof: Uint8Array[];

    /** CombinedHeadersToSync _authoritySetChange. */
    public _authoritySetChange?: "authoritySetChange";

    /**
     * Creates a new CombinedHeadersToSync instance using the specified properties.
     * @param [properties] Properties to set
     * @returns CombinedHeadersToSync instance
     */
    public static create(
      properties?: pruntime_rpc.ICombinedHeadersToSync
    ): pruntime_rpc.CombinedHeadersToSync;

    /**
     * Encodes the specified CombinedHeadersToSync message. Does not implicitly {@link pruntime_rpc.CombinedHeadersToSync.verify|verify} messages.
     * @param message CombinedHeadersToSync message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encode(
      message: pruntime_rpc.ICombinedHeadersToSync,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Encodes the specified CombinedHeadersToSync message, length delimited. Does not implicitly {@link pruntime_rpc.CombinedHeadersToSync.verify|verify} messages.
     * @param message CombinedHeadersToSync message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encodeDelimited(
      message: pruntime_rpc.ICombinedHeadersToSync,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Decodes a CombinedHeadersToSync message from the specified reader or buffer.
     * @param reader Reader or buffer to decode from
     * @param [length] Message length if known beforehand
     * @returns CombinedHeadersToSync
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decode(
      reader: $protobuf.Reader | Uint8Array,
      length?: number
    ): pruntime_rpc.CombinedHeadersToSync;

    /**
     * Decodes a CombinedHeadersToSync message from the specified reader or buffer, length delimited.
     * @param reader Reader or buffer to decode from
     * @returns CombinedHeadersToSync
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decodeDelimited(
      reader: $protobuf.Reader | Uint8Array
    ): pruntime_rpc.CombinedHeadersToSync;

    /**
     * Verifies a CombinedHeadersToSync message.
     * @param message Plain object to verify
     * @returns `null` if valid, otherwise the reason why it is not
     */
    public static verify(message: { [k: string]: any }): string | null;

    /**
     * Creates a CombinedHeadersToSync message from a plain object. Also converts values to their respective internal types.
     * @param object Plain object
     * @returns CombinedHeadersToSync
     */
    public static fromObject(object: {
      [k: string]: any;
    }): pruntime_rpc.CombinedHeadersToSync;

    /**
     * Creates a plain object from a CombinedHeadersToSync message. Also converts values to other types if specified.
     * @param message CombinedHeadersToSync
     * @param [options] Conversion options
     * @returns Plain object
     */
    public static toObject(
      message: pruntime_rpc.CombinedHeadersToSync,
      options?: $protobuf.IConversionOptions
    ): { [k: string]: any };

    /**
     * Converts this CombinedHeadersToSync to JSON.
     * @returns JSON object
     */
    public toJSON(): { [k: string]: any };
  }

  /** Properties of a HeadersSyncedTo. */
  interface IHeadersSyncedTo {
    /** HeadersSyncedTo relaychainSyncedTo */
    relaychainSyncedTo?: number | null;

    /** HeadersSyncedTo parachainSyncedTo */
    parachainSyncedTo?: number | null;
  }

  /** Represents a HeadersSyncedTo. */
  class HeadersSyncedTo implements IHeadersSyncedTo {
    /**
     * Constructs a new HeadersSyncedTo.
     * @param [properties] Properties to set
     */
    constructor(properties?: pruntime_rpc.IHeadersSyncedTo);

    /** HeadersSyncedTo relaychainSyncedTo. */
    public relaychainSyncedTo: number;

    /** HeadersSyncedTo parachainSyncedTo. */
    public parachainSyncedTo: number;

    /**
     * Creates a new HeadersSyncedTo instance using the specified properties.
     * @param [properties] Properties to set
     * @returns HeadersSyncedTo instance
     */
    public static create(
      properties?: pruntime_rpc.IHeadersSyncedTo
    ): pruntime_rpc.HeadersSyncedTo;

    /**
     * Encodes the specified HeadersSyncedTo message. Does not implicitly {@link pruntime_rpc.HeadersSyncedTo.verify|verify} messages.
     * @param message HeadersSyncedTo message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encode(
      message: pruntime_rpc.IHeadersSyncedTo,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Encodes the specified HeadersSyncedTo message, length delimited. Does not implicitly {@link pruntime_rpc.HeadersSyncedTo.verify|verify} messages.
     * @param message HeadersSyncedTo message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encodeDelimited(
      message: pruntime_rpc.IHeadersSyncedTo,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Decodes a HeadersSyncedTo message from the specified reader or buffer.
     * @param reader Reader or buffer to decode from
     * @param [length] Message length if known beforehand
     * @returns HeadersSyncedTo
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decode(
      reader: $protobuf.Reader | Uint8Array,
      length?: number
    ): pruntime_rpc.HeadersSyncedTo;

    /**
     * Decodes a HeadersSyncedTo message from the specified reader or buffer, length delimited.
     * @param reader Reader or buffer to decode from
     * @returns HeadersSyncedTo
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decodeDelimited(
      reader: $protobuf.Reader | Uint8Array
    ): pruntime_rpc.HeadersSyncedTo;

    /**
     * Verifies a HeadersSyncedTo message.
     * @param message Plain object to verify
     * @returns `null` if valid, otherwise the reason why it is not
     */
    public static verify(message: { [k: string]: any }): string | null;

    /**
     * Creates a HeadersSyncedTo message from a plain object. Also converts values to their respective internal types.
     * @param object Plain object
     * @returns HeadersSyncedTo
     */
    public static fromObject(object: {
      [k: string]: any;
    }): pruntime_rpc.HeadersSyncedTo;

    /**
     * Creates a plain object from a HeadersSyncedTo message. Also converts values to other types if specified.
     * @param message HeadersSyncedTo
     * @param [options] Conversion options
     * @returns Plain object
     */
    public static toObject(
      message: pruntime_rpc.HeadersSyncedTo,
      options?: $protobuf.IConversionOptions
    ): { [k: string]: any };

    /**
     * Converts this HeadersSyncedTo to JSON.
     * @returns JSON object
     */
    public toJSON(): { [k: string]: any };
  }

  /** Properties of a Blocks. */
  interface IBlocks {
    /** Blocks encodedBlocks */
    encodedBlocks?: Uint8Array | null;
  }

  /** Represents a Blocks. */
  class Blocks implements IBlocks {
    /**
     * Constructs a new Blocks.
     * @param [properties] Properties to set
     */
    constructor(properties?: pruntime_rpc.IBlocks);

    /** Blocks encodedBlocks. */
    public encodedBlocks: Uint8Array;

    /**
     * Creates a new Blocks instance using the specified properties.
     * @param [properties] Properties to set
     * @returns Blocks instance
     */
    public static create(
      properties?: pruntime_rpc.IBlocks
    ): pruntime_rpc.Blocks;

    /**
     * Encodes the specified Blocks message. Does not implicitly {@link pruntime_rpc.Blocks.verify|verify} messages.
     * @param message Blocks message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encode(
      message: pruntime_rpc.IBlocks,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Encodes the specified Blocks message, length delimited. Does not implicitly {@link pruntime_rpc.Blocks.verify|verify} messages.
     * @param message Blocks message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encodeDelimited(
      message: pruntime_rpc.IBlocks,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Decodes a Blocks message from the specified reader or buffer.
     * @param reader Reader or buffer to decode from
     * @param [length] Message length if known beforehand
     * @returns Blocks
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decode(
      reader: $protobuf.Reader | Uint8Array,
      length?: number
    ): pruntime_rpc.Blocks;

    /**
     * Decodes a Blocks message from the specified reader or buffer, length delimited.
     * @param reader Reader or buffer to decode from
     * @returns Blocks
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decodeDelimited(
      reader: $protobuf.Reader | Uint8Array
    ): pruntime_rpc.Blocks;

    /**
     * Verifies a Blocks message.
     * @param message Plain object to verify
     * @returns `null` if valid, otherwise the reason why it is not
     */
    public static verify(message: { [k: string]: any }): string | null;

    /**
     * Creates a Blocks message from a plain object. Also converts values to their respective internal types.
     * @param object Plain object
     * @returns Blocks
     */
    public static fromObject(object: { [k: string]: any }): pruntime_rpc.Blocks;

    /**
     * Creates a plain object from a Blocks message. Also converts values to other types if specified.
     * @param message Blocks
     * @param [options] Conversion options
     * @returns Plain object
     */
    public static toObject(
      message: pruntime_rpc.Blocks,
      options?: $protobuf.IConversionOptions
    ): { [k: string]: any };

    /**
     * Converts this Blocks to JSON.
     * @returns JSON object
     */
    public toJSON(): { [k: string]: any };
  }

  /** Properties of an InitRuntimeRequest. */
  interface IInitRuntimeRequest {
    /** InitRuntimeRequest skipRa */
    skipRa?: boolean | null;

    /** InitRuntimeRequest encodedGenesisInfo */
    encodedGenesisInfo?: Uint8Array | null;

    /** InitRuntimeRequest debugSetKey */
    debugSetKey?: Uint8Array | null;

    /** InitRuntimeRequest encodedGenesisState */
    encodedGenesisState?: Uint8Array | null;

    /** InitRuntimeRequest encodedOperator */
    encodedOperator?: Uint8Array | null;

    /** InitRuntimeRequest isParachain */
    isParachain?: boolean | null;
  }

  /** Represents an InitRuntimeRequest. */
  class InitRuntimeRequest implements IInitRuntimeRequest {
    /**
     * Constructs a new InitRuntimeRequest.
     * @param [properties] Properties to set
     */
    constructor(properties?: pruntime_rpc.IInitRuntimeRequest);

    /** InitRuntimeRequest skipRa. */
    public skipRa: boolean;

    /** InitRuntimeRequest encodedGenesisInfo. */
    public encodedGenesisInfo: Uint8Array;

    /** InitRuntimeRequest debugSetKey. */
    public debugSetKey?: Uint8Array | null;

    /** InitRuntimeRequest encodedGenesisState. */
    public encodedGenesisState: Uint8Array;

    /** InitRuntimeRequest encodedOperator. */
    public encodedOperator?: Uint8Array | null;

    /** InitRuntimeRequest isParachain. */
    public isParachain: boolean;

    /** InitRuntimeRequest _debugSetKey. */
    public _debugSetKey?: "debugSetKey";

    /** InitRuntimeRequest _encodedOperator. */
    public _encodedOperator?: "encodedOperator";

    /**
     * Creates a new InitRuntimeRequest instance using the specified properties.
     * @param [properties] Properties to set
     * @returns InitRuntimeRequest instance
     */
    public static create(
      properties?: pruntime_rpc.IInitRuntimeRequest
    ): pruntime_rpc.InitRuntimeRequest;

    /**
     * Encodes the specified InitRuntimeRequest message. Does not implicitly {@link pruntime_rpc.InitRuntimeRequest.verify|verify} messages.
     * @param message InitRuntimeRequest message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encode(
      message: pruntime_rpc.IInitRuntimeRequest,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Encodes the specified InitRuntimeRequest message, length delimited. Does not implicitly {@link pruntime_rpc.InitRuntimeRequest.verify|verify} messages.
     * @param message InitRuntimeRequest message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encodeDelimited(
      message: pruntime_rpc.IInitRuntimeRequest,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Decodes an InitRuntimeRequest message from the specified reader or buffer.
     * @param reader Reader or buffer to decode from
     * @param [length] Message length if known beforehand
     * @returns InitRuntimeRequest
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decode(
      reader: $protobuf.Reader | Uint8Array,
      length?: number
    ): pruntime_rpc.InitRuntimeRequest;

    /**
     * Decodes an InitRuntimeRequest message from the specified reader or buffer, length delimited.
     * @param reader Reader or buffer to decode from
     * @returns InitRuntimeRequest
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decodeDelimited(
      reader: $protobuf.Reader | Uint8Array
    ): pruntime_rpc.InitRuntimeRequest;

    /**
     * Verifies an InitRuntimeRequest message.
     * @param message Plain object to verify
     * @returns `null` if valid, otherwise the reason why it is not
     */
    public static verify(message: { [k: string]: any }): string | null;

    /**
     * Creates an InitRuntimeRequest message from a plain object. Also converts values to their respective internal types.
     * @param object Plain object
     * @returns InitRuntimeRequest
     */
    public static fromObject(object: {
      [k: string]: any;
    }): pruntime_rpc.InitRuntimeRequest;

    /**
     * Creates a plain object from an InitRuntimeRequest message. Also converts values to other types if specified.
     * @param message InitRuntimeRequest
     * @param [options] Conversion options
     * @returns Plain object
     */
    public static toObject(
      message: pruntime_rpc.InitRuntimeRequest,
      options?: $protobuf.IConversionOptions
    ): { [k: string]: any };

    /**
     * Converts this InitRuntimeRequest to JSON.
     * @returns JSON object
     */
    public toJSON(): { [k: string]: any };
  }

  /** Properties of a GetRuntimeInfoRequest. */
  interface IGetRuntimeInfoRequest {
    /** GetRuntimeInfoRequest forceRefreshRa */
    forceRefreshRa?: boolean | null;

    /** GetRuntimeInfoRequest encodedOperator */
    encodedOperator?: Uint8Array | null;
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
    public encodedOperator?: Uint8Array | null;

    /** GetRuntimeInfoRequest _encodedOperator. */
    public _encodedOperator?: "encodedOperator";

    /**
     * Creates a new GetRuntimeInfoRequest instance using the specified properties.
     * @param [properties] Properties to set
     * @returns GetRuntimeInfoRequest instance
     */
    public static create(
      properties?: pruntime_rpc.IGetRuntimeInfoRequest
    ): pruntime_rpc.GetRuntimeInfoRequest;

    /**
     * Encodes the specified GetRuntimeInfoRequest message. Does not implicitly {@link pruntime_rpc.GetRuntimeInfoRequest.verify|verify} messages.
     * @param message GetRuntimeInfoRequest message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encode(
      message: pruntime_rpc.IGetRuntimeInfoRequest,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Encodes the specified GetRuntimeInfoRequest message, length delimited. Does not implicitly {@link pruntime_rpc.GetRuntimeInfoRequest.verify|verify} messages.
     * @param message GetRuntimeInfoRequest message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encodeDelimited(
      message: pruntime_rpc.IGetRuntimeInfoRequest,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Decodes a GetRuntimeInfoRequest message from the specified reader or buffer.
     * @param reader Reader or buffer to decode from
     * @param [length] Message length if known beforehand
     * @returns GetRuntimeInfoRequest
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decode(
      reader: $protobuf.Reader | Uint8Array,
      length?: number
    ): pruntime_rpc.GetRuntimeInfoRequest;

    /**
     * Decodes a GetRuntimeInfoRequest message from the specified reader or buffer, length delimited.
     * @param reader Reader or buffer to decode from
     * @returns GetRuntimeInfoRequest
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decodeDelimited(
      reader: $protobuf.Reader | Uint8Array
    ): pruntime_rpc.GetRuntimeInfoRequest;

    /**
     * Verifies a GetRuntimeInfoRequest message.
     * @param message Plain object to verify
     * @returns `null` if valid, otherwise the reason why it is not
     */
    public static verify(message: { [k: string]: any }): string | null;

    /**
     * Creates a GetRuntimeInfoRequest message from a plain object. Also converts values to their respective internal types.
     * @param object Plain object
     * @returns GetRuntimeInfoRequest
     */
    public static fromObject(object: {
      [k: string]: any;
    }): pruntime_rpc.GetRuntimeInfoRequest;

    /**
     * Creates a plain object from a GetRuntimeInfoRequest message. Also converts values to other types if specified.
     * @param message GetRuntimeInfoRequest
     * @param [options] Conversion options
     * @returns Plain object
     */
    public static toObject(
      message: pruntime_rpc.GetRuntimeInfoRequest,
      options?: $protobuf.IConversionOptions
    ): { [k: string]: any };

    /**
     * Converts this GetRuntimeInfoRequest to JSON.
     * @returns JSON object
     */
    public toJSON(): { [k: string]: any };
  }

  /** Properties of an InitRuntimeResponse. */
  interface IInitRuntimeResponse {
    /** InitRuntimeResponse encodedRuntimeInfo */
    encodedRuntimeInfo?: Uint8Array | null;

    /** InitRuntimeResponse encodedGenesisBlockHash */
    encodedGenesisBlockHash?: Uint8Array | null;

    /** InitRuntimeResponse encodedPublicKey */
    encodedPublicKey?: Uint8Array | null;

    /** InitRuntimeResponse encodedEcdhPublicKey */
    encodedEcdhPublicKey?: Uint8Array | null;

    /** InitRuntimeResponse attestation */
    attestation?: pruntime_rpc.IAttestation | null;
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
    public attestation?: pruntime_rpc.IAttestation | null;

    /** InitRuntimeResponse _attestation. */
    public _attestation?: "attestation";

    /**
     * Creates a new InitRuntimeResponse instance using the specified properties.
     * @param [properties] Properties to set
     * @returns InitRuntimeResponse instance
     */
    public static create(
      properties?: pruntime_rpc.IInitRuntimeResponse
    ): pruntime_rpc.InitRuntimeResponse;

    /**
     * Encodes the specified InitRuntimeResponse message. Does not implicitly {@link pruntime_rpc.InitRuntimeResponse.verify|verify} messages.
     * @param message InitRuntimeResponse message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encode(
      message: pruntime_rpc.IInitRuntimeResponse,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Encodes the specified InitRuntimeResponse message, length delimited. Does not implicitly {@link pruntime_rpc.InitRuntimeResponse.verify|verify} messages.
     * @param message InitRuntimeResponse message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encodeDelimited(
      message: pruntime_rpc.IInitRuntimeResponse,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Decodes an InitRuntimeResponse message from the specified reader or buffer.
     * @param reader Reader or buffer to decode from
     * @param [length] Message length if known beforehand
     * @returns InitRuntimeResponse
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decode(
      reader: $protobuf.Reader | Uint8Array,
      length?: number
    ): pruntime_rpc.InitRuntimeResponse;

    /**
     * Decodes an InitRuntimeResponse message from the specified reader or buffer, length delimited.
     * @param reader Reader or buffer to decode from
     * @returns InitRuntimeResponse
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decodeDelimited(
      reader: $protobuf.Reader | Uint8Array
    ): pruntime_rpc.InitRuntimeResponse;

    /**
     * Verifies an InitRuntimeResponse message.
     * @param message Plain object to verify
     * @returns `null` if valid, otherwise the reason why it is not
     */
    public static verify(message: { [k: string]: any }): string | null;

    /**
     * Creates an InitRuntimeResponse message from a plain object. Also converts values to their respective internal types.
     * @param object Plain object
     * @returns InitRuntimeResponse
     */
    public static fromObject(object: {
      [k: string]: any;
    }): pruntime_rpc.InitRuntimeResponse;

    /**
     * Creates a plain object from an InitRuntimeResponse message. Also converts values to other types if specified.
     * @param message InitRuntimeResponse
     * @param [options] Conversion options
     * @returns Plain object
     */
    public static toObject(
      message: pruntime_rpc.InitRuntimeResponse,
      options?: $protobuf.IConversionOptions
    ): { [k: string]: any };

    /**
     * Converts this InitRuntimeResponse to JSON.
     * @returns JSON object
     */
    public toJSON(): { [k: string]: any };
  }

  /** Properties of an Attestation. */
  interface IAttestation {
    /** Attestation version */
    version?: number | null;

    /** Attestation provider */
    provider?: string | null;

    /** Attestation payload */
    payload?: pruntime_rpc.IAttestationReport | null;

    /** Attestation timestamp */
    timestamp?: number | Long | null;
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
    public payload?: pruntime_rpc.IAttestationReport | null;

    /** Attestation timestamp. */
    public timestamp: number | Long;

    /**
     * Creates a new Attestation instance using the specified properties.
     * @param [properties] Properties to set
     * @returns Attestation instance
     */
    public static create(
      properties?: pruntime_rpc.IAttestation
    ): pruntime_rpc.Attestation;

    /**
     * Encodes the specified Attestation message. Does not implicitly {@link pruntime_rpc.Attestation.verify|verify} messages.
     * @param message Attestation message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encode(
      message: pruntime_rpc.IAttestation,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Encodes the specified Attestation message, length delimited. Does not implicitly {@link pruntime_rpc.Attestation.verify|verify} messages.
     * @param message Attestation message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encodeDelimited(
      message: pruntime_rpc.IAttestation,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Decodes an Attestation message from the specified reader or buffer.
     * @param reader Reader or buffer to decode from
     * @param [length] Message length if known beforehand
     * @returns Attestation
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decode(
      reader: $protobuf.Reader | Uint8Array,
      length?: number
    ): pruntime_rpc.Attestation;

    /**
     * Decodes an Attestation message from the specified reader or buffer, length delimited.
     * @param reader Reader or buffer to decode from
     * @returns Attestation
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decodeDelimited(
      reader: $protobuf.Reader | Uint8Array
    ): pruntime_rpc.Attestation;

    /**
     * Verifies an Attestation message.
     * @param message Plain object to verify
     * @returns `null` if valid, otherwise the reason why it is not
     */
    public static verify(message: { [k: string]: any }): string | null;

    /**
     * Creates an Attestation message from a plain object. Also converts values to their respective internal types.
     * @param object Plain object
     * @returns Attestation
     */
    public static fromObject(object: {
      [k: string]: any;
    }): pruntime_rpc.Attestation;

    /**
     * Creates a plain object from an Attestation message. Also converts values to other types if specified.
     * @param message Attestation
     * @param [options] Conversion options
     * @returns Plain object
     */
    public static toObject(
      message: pruntime_rpc.Attestation,
      options?: $protobuf.IConversionOptions
    ): { [k: string]: any };

    /**
     * Converts this Attestation to JSON.
     * @returns JSON object
     */
    public toJSON(): { [k: string]: any };
  }

  /** Properties of an AttestationReport. */
  interface IAttestationReport {
    /** AttestationReport report */
    report?: string | null;

    /** AttestationReport signature */
    signature?: Uint8Array | null;

    /** AttestationReport signingCert */
    signingCert?: Uint8Array | null;
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
    public static create(
      properties?: pruntime_rpc.IAttestationReport
    ): pruntime_rpc.AttestationReport;

    /**
     * Encodes the specified AttestationReport message. Does not implicitly {@link pruntime_rpc.AttestationReport.verify|verify} messages.
     * @param message AttestationReport message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encode(
      message: pruntime_rpc.IAttestationReport,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Encodes the specified AttestationReport message, length delimited. Does not implicitly {@link pruntime_rpc.AttestationReport.verify|verify} messages.
     * @param message AttestationReport message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encodeDelimited(
      message: pruntime_rpc.IAttestationReport,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Decodes an AttestationReport message from the specified reader or buffer.
     * @param reader Reader or buffer to decode from
     * @param [length] Message length if known beforehand
     * @returns AttestationReport
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decode(
      reader: $protobuf.Reader | Uint8Array,
      length?: number
    ): pruntime_rpc.AttestationReport;

    /**
     * Decodes an AttestationReport message from the specified reader or buffer, length delimited.
     * @param reader Reader or buffer to decode from
     * @returns AttestationReport
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decodeDelimited(
      reader: $protobuf.Reader | Uint8Array
    ): pruntime_rpc.AttestationReport;

    /**
     * Verifies an AttestationReport message.
     * @param message Plain object to verify
     * @returns `null` if valid, otherwise the reason why it is not
     */
    public static verify(message: { [k: string]: any }): string | null;

    /**
     * Creates an AttestationReport message from a plain object. Also converts values to their respective internal types.
     * @param object Plain object
     * @returns AttestationReport
     */
    public static fromObject(object: {
      [k: string]: any;
    }): pruntime_rpc.AttestationReport;

    /**
     * Creates a plain object from an AttestationReport message. Also converts values to other types if specified.
     * @param message AttestationReport
     * @param [options] Conversion options
     * @returns Plain object
     */
    public static toObject(
      message: pruntime_rpc.AttestationReport,
      options?: $protobuf.IConversionOptions
    ): { [k: string]: any };

    /**
     * Converts this AttestationReport to JSON.
     * @returns JSON object
     */
    public toJSON(): { [k: string]: any };
  }

  /** Properties of a GetEgressMessagesResponse. */
  interface IGetEgressMessagesResponse {
    /** GetEgressMessagesResponse encodedMessages */
    encodedMessages?: Uint8Array | null;
  }

  /** Represents a GetEgressMessagesResponse. */
  class GetEgressMessagesResponse implements IGetEgressMessagesResponse {
    /**
     * Constructs a new GetEgressMessagesResponse.
     * @param [properties] Properties to set
     */
    constructor(properties?: pruntime_rpc.IGetEgressMessagesResponse);

    /** GetEgressMessagesResponse encodedMessages. */
    public encodedMessages: Uint8Array;

    /**
     * Creates a new GetEgressMessagesResponse instance using the specified properties.
     * @param [properties] Properties to set
     * @returns GetEgressMessagesResponse instance
     */
    public static create(
      properties?: pruntime_rpc.IGetEgressMessagesResponse
    ): pruntime_rpc.GetEgressMessagesResponse;

    /**
     * Encodes the specified GetEgressMessagesResponse message. Does not implicitly {@link pruntime_rpc.GetEgressMessagesResponse.verify|verify} messages.
     * @param message GetEgressMessagesResponse message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encode(
      message: pruntime_rpc.IGetEgressMessagesResponse,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Encodes the specified GetEgressMessagesResponse message, length delimited. Does not implicitly {@link pruntime_rpc.GetEgressMessagesResponse.verify|verify} messages.
     * @param message GetEgressMessagesResponse message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encodeDelimited(
      message: pruntime_rpc.IGetEgressMessagesResponse,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Decodes a GetEgressMessagesResponse message from the specified reader or buffer.
     * @param reader Reader or buffer to decode from
     * @param [length] Message length if known beforehand
     * @returns GetEgressMessagesResponse
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decode(
      reader: $protobuf.Reader | Uint8Array,
      length?: number
    ): pruntime_rpc.GetEgressMessagesResponse;

    /**
     * Decodes a GetEgressMessagesResponse message from the specified reader or buffer, length delimited.
     * @param reader Reader or buffer to decode from
     * @returns GetEgressMessagesResponse
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decodeDelimited(
      reader: $protobuf.Reader | Uint8Array
    ): pruntime_rpc.GetEgressMessagesResponse;

    /**
     * Verifies a GetEgressMessagesResponse message.
     * @param message Plain object to verify
     * @returns `null` if valid, otherwise the reason why it is not
     */
    public static verify(message: { [k: string]: any }): string | null;

    /**
     * Creates a GetEgressMessagesResponse message from a plain object. Also converts values to their respective internal types.
     * @param object Plain object
     * @returns GetEgressMessagesResponse
     */
    public static fromObject(object: {
      [k: string]: any;
    }): pruntime_rpc.GetEgressMessagesResponse;

    /**
     * Creates a plain object from a GetEgressMessagesResponse message. Also converts values to other types if specified.
     * @param message GetEgressMessagesResponse
     * @param [options] Conversion options
     * @returns Plain object
     */
    public static toObject(
      message: pruntime_rpc.GetEgressMessagesResponse,
      options?: $protobuf.IConversionOptions
    ): { [k: string]: any };

    /**
     * Converts this GetEgressMessagesResponse to JSON.
     * @returns JSON object
     */
    public toJSON(): { [k: string]: any };
  }

  /** Properties of a ContractQueryRequest. */
  interface IContractQueryRequest {
    /** ContractQueryRequest encodedEncryptedData */
    encodedEncryptedData?: Uint8Array | null;

    /** ContractQueryRequest signature */
    signature?: pruntime_rpc.ISignature | null;
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
    public signature?: pruntime_rpc.ISignature | null;

    /**
     * Creates a new ContractQueryRequest instance using the specified properties.
     * @param [properties] Properties to set
     * @returns ContractQueryRequest instance
     */
    public static create(
      properties?: pruntime_rpc.IContractQueryRequest
    ): pruntime_rpc.ContractQueryRequest;

    /**
     * Encodes the specified ContractQueryRequest message. Does not implicitly {@link pruntime_rpc.ContractQueryRequest.verify|verify} messages.
     * @param message ContractQueryRequest message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encode(
      message: pruntime_rpc.IContractQueryRequest,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Encodes the specified ContractQueryRequest message, length delimited. Does not implicitly {@link pruntime_rpc.ContractQueryRequest.verify|verify} messages.
     * @param message ContractQueryRequest message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encodeDelimited(
      message: pruntime_rpc.IContractQueryRequest,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Decodes a ContractQueryRequest message from the specified reader or buffer.
     * @param reader Reader or buffer to decode from
     * @param [length] Message length if known beforehand
     * @returns ContractQueryRequest
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decode(
      reader: $protobuf.Reader | Uint8Array,
      length?: number
    ): pruntime_rpc.ContractQueryRequest;

    /**
     * Decodes a ContractQueryRequest message from the specified reader or buffer, length delimited.
     * @param reader Reader or buffer to decode from
     * @returns ContractQueryRequest
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decodeDelimited(
      reader: $protobuf.Reader | Uint8Array
    ): pruntime_rpc.ContractQueryRequest;

    /**
     * Verifies a ContractQueryRequest message.
     * @param message Plain object to verify
     * @returns `null` if valid, otherwise the reason why it is not
     */
    public static verify(message: { [k: string]: any }): string | null;

    /**
     * Creates a ContractQueryRequest message from a plain object. Also converts values to their respective internal types.
     * @param object Plain object
     * @returns ContractQueryRequest
     */
    public static fromObject(object: {
      [k: string]: any;
    }): pruntime_rpc.ContractQueryRequest;

    /**
     * Creates a plain object from a ContractQueryRequest message. Also converts values to other types if specified.
     * @param message ContractQueryRequest
     * @param [options] Conversion options
     * @returns Plain object
     */
    public static toObject(
      message: pruntime_rpc.ContractQueryRequest,
      options?: $protobuf.IConversionOptions
    ): { [k: string]: any };

    /**
     * Converts this ContractQueryRequest to JSON.
     * @returns JSON object
     */
    public toJSON(): { [k: string]: any };
  }

  /** Properties of a Signature. */
  interface ISignature {
    /** Signature signedBy */
    signedBy?: pruntime_rpc.ICertificate | null;

    /** Signature signatureType */
    signatureType?: pruntime_rpc.SignatureType | null;

    /** Signature signature */
    signature?: Uint8Array | null;
  }

  /** Represents a Signature. */
  class Signature implements ISignature {
    /**
     * Constructs a new Signature.
     * @param [properties] Properties to set
     */
    constructor(properties?: pruntime_rpc.ISignature);

    /** Signature signedBy. */
    public signedBy?: pruntime_rpc.ICertificate | null;

    /** Signature signatureType. */
    public signatureType: pruntime_rpc.SignatureType;

    /** Signature signature. */
    public signature: Uint8Array;

    /**
     * Creates a new Signature instance using the specified properties.
     * @param [properties] Properties to set
     * @returns Signature instance
     */
    public static create(
      properties?: pruntime_rpc.ISignature
    ): pruntime_rpc.Signature;

    /**
     * Encodes the specified Signature message. Does not implicitly {@link pruntime_rpc.Signature.verify|verify} messages.
     * @param message Signature message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encode(
      message: pruntime_rpc.ISignature,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Encodes the specified Signature message, length delimited. Does not implicitly {@link pruntime_rpc.Signature.verify|verify} messages.
     * @param message Signature message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encodeDelimited(
      message: pruntime_rpc.ISignature,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Decodes a Signature message from the specified reader or buffer.
     * @param reader Reader or buffer to decode from
     * @param [length] Message length if known beforehand
     * @returns Signature
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decode(
      reader: $protobuf.Reader | Uint8Array,
      length?: number
    ): pruntime_rpc.Signature;

    /**
     * Decodes a Signature message from the specified reader or buffer, length delimited.
     * @param reader Reader or buffer to decode from
     * @returns Signature
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decodeDelimited(
      reader: $protobuf.Reader | Uint8Array
    ): pruntime_rpc.Signature;

    /**
     * Verifies a Signature message.
     * @param message Plain object to verify
     * @returns `null` if valid, otherwise the reason why it is not
     */
    public static verify(message: { [k: string]: any }): string | null;

    /**
     * Creates a Signature message from a plain object. Also converts values to their respective internal types.
     * @param object Plain object
     * @returns Signature
     */
    public static fromObject(object: {
      [k: string]: any;
    }): pruntime_rpc.Signature;

    /**
     * Creates a plain object from a Signature message. Also converts values to other types if specified.
     * @param message Signature
     * @param [options] Conversion options
     * @returns Plain object
     */
    public static toObject(
      message: pruntime_rpc.Signature,
      options?: $protobuf.IConversionOptions
    ): { [k: string]: any };

    /**
     * Converts this Signature to JSON.
     * @returns JSON object
     */
    public toJSON(): { [k: string]: any };
  }

  /** Properties of a Certificate. */
  interface ICertificate {
    /** Certificate encodedBody */
    encodedBody?: Uint8Array | null;

    /** Certificate signature */
    signature?: pruntime_rpc.ISignature | null;
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
    public signature?: pruntime_rpc.ISignature | null;

    /**
     * Creates a new Certificate instance using the specified properties.
     * @param [properties] Properties to set
     * @returns Certificate instance
     */
    public static create(
      properties?: pruntime_rpc.ICertificate
    ): pruntime_rpc.Certificate;

    /**
     * Encodes the specified Certificate message. Does not implicitly {@link pruntime_rpc.Certificate.verify|verify} messages.
     * @param message Certificate message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encode(
      message: pruntime_rpc.ICertificate,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Encodes the specified Certificate message, length delimited. Does not implicitly {@link pruntime_rpc.Certificate.verify|verify} messages.
     * @param message Certificate message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encodeDelimited(
      message: pruntime_rpc.ICertificate,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Decodes a Certificate message from the specified reader or buffer.
     * @param reader Reader or buffer to decode from
     * @param [length] Message length if known beforehand
     * @returns Certificate
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decode(
      reader: $protobuf.Reader | Uint8Array,
      length?: number
    ): pruntime_rpc.Certificate;

    /**
     * Decodes a Certificate message from the specified reader or buffer, length delimited.
     * @param reader Reader or buffer to decode from
     * @returns Certificate
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decodeDelimited(
      reader: $protobuf.Reader | Uint8Array
    ): pruntime_rpc.Certificate;

    /**
     * Verifies a Certificate message.
     * @param message Plain object to verify
     * @returns `null` if valid, otherwise the reason why it is not
     */
    public static verify(message: { [k: string]: any }): string | null;

    /**
     * Creates a Certificate message from a plain object. Also converts values to their respective internal types.
     * @param object Plain object
     * @returns Certificate
     */
    public static fromObject(object: {
      [k: string]: any;
    }): pruntime_rpc.Certificate;

    /**
     * Creates a plain object from a Certificate message. Also converts values to other types if specified.
     * @param message Certificate
     * @param [options] Conversion options
     * @returns Plain object
     */
    public static toObject(
      message: pruntime_rpc.Certificate,
      options?: $protobuf.IConversionOptions
    ): { [k: string]: any };

    /**
     * Converts this Certificate to JSON.
     * @returns JSON object
     */
    public toJSON(): { [k: string]: any };
  }

  /** SignatureType enum. */
  enum SignatureType {
    Ed25519 = 0,
    Sr25519 = 1,
    Ecdsa = 2,
    Ed25519WrapBytes = 3,
    Sr25519WrapBytes = 4,
    EcdsaWrapBytes = 5,
  }

  /** Properties of a ContractQueryResponse. */
  interface IContractQueryResponse {
    /** ContractQueryResponse encodedEncryptedData */
    encodedEncryptedData?: Uint8Array | null;
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
    public static create(
      properties?: pruntime_rpc.IContractQueryResponse
    ): pruntime_rpc.ContractQueryResponse;

    /**
     * Encodes the specified ContractQueryResponse message. Does not implicitly {@link pruntime_rpc.ContractQueryResponse.verify|verify} messages.
     * @param message ContractQueryResponse message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encode(
      message: pruntime_rpc.IContractQueryResponse,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Encodes the specified ContractQueryResponse message, length delimited. Does not implicitly {@link pruntime_rpc.ContractQueryResponse.verify|verify} messages.
     * @param message ContractQueryResponse message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encodeDelimited(
      message: pruntime_rpc.IContractQueryResponse,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Decodes a ContractQueryResponse message from the specified reader or buffer.
     * @param reader Reader or buffer to decode from
     * @param [length] Message length if known beforehand
     * @returns ContractQueryResponse
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decode(
      reader: $protobuf.Reader | Uint8Array,
      length?: number
    ): pruntime_rpc.ContractQueryResponse;

    /**
     * Decodes a ContractQueryResponse message from the specified reader or buffer, length delimited.
     * @param reader Reader or buffer to decode from
     * @returns ContractQueryResponse
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decodeDelimited(
      reader: $protobuf.Reader | Uint8Array
    ): pruntime_rpc.ContractQueryResponse;

    /**
     * Verifies a ContractQueryResponse message.
     * @param message Plain object to verify
     * @returns `null` if valid, otherwise the reason why it is not
     */
    public static verify(message: { [k: string]: any }): string | null;

    /**
     * Creates a ContractQueryResponse message from a plain object. Also converts values to their respective internal types.
     * @param object Plain object
     * @returns ContractQueryResponse
     */
    public static fromObject(object: {
      [k: string]: any;
    }): pruntime_rpc.ContractQueryResponse;

    /**
     * Creates a plain object from a ContractQueryResponse message. Also converts values to other types if specified.
     * @param message ContractQueryResponse
     * @param [options] Conversion options
     * @returns Plain object
     */
    public static toObject(
      message: pruntime_rpc.ContractQueryResponse,
      options?: $protobuf.IConversionOptions
    ): { [k: string]: any };

    /**
     * Converts this ContractQueryResponse to JSON.
     * @returns JSON object
     */
    public toJSON(): { [k: string]: any };
  }

  /** Properties of a GetWorkerStateRequest. */
  interface IGetWorkerStateRequest {
    /** GetWorkerStateRequest publicKey */
    publicKey?: Uint8Array | null;
  }

  /** Represents a GetWorkerStateRequest. */
  class GetWorkerStateRequest implements IGetWorkerStateRequest {
    /**
     * Constructs a new GetWorkerStateRequest.
     * @param [properties] Properties to set
     */
    constructor(properties?: pruntime_rpc.IGetWorkerStateRequest);

    /** GetWorkerStateRequest publicKey. */
    public publicKey: Uint8Array;

    /**
     * Creates a new GetWorkerStateRequest instance using the specified properties.
     * @param [properties] Properties to set
     * @returns GetWorkerStateRequest instance
     */
    public static create(
      properties?: pruntime_rpc.IGetWorkerStateRequest
    ): pruntime_rpc.GetWorkerStateRequest;

    /**
     * Encodes the specified GetWorkerStateRequest message. Does not implicitly {@link pruntime_rpc.GetWorkerStateRequest.verify|verify} messages.
     * @param message GetWorkerStateRequest message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encode(
      message: pruntime_rpc.IGetWorkerStateRequest,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Encodes the specified GetWorkerStateRequest message, length delimited. Does not implicitly {@link pruntime_rpc.GetWorkerStateRequest.verify|verify} messages.
     * @param message GetWorkerStateRequest message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encodeDelimited(
      message: pruntime_rpc.IGetWorkerStateRequest,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Decodes a GetWorkerStateRequest message from the specified reader or buffer.
     * @param reader Reader or buffer to decode from
     * @param [length] Message length if known beforehand
     * @returns GetWorkerStateRequest
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decode(
      reader: $protobuf.Reader | Uint8Array,
      length?: number
    ): pruntime_rpc.GetWorkerStateRequest;

    /**
     * Decodes a GetWorkerStateRequest message from the specified reader or buffer, length delimited.
     * @param reader Reader or buffer to decode from
     * @returns GetWorkerStateRequest
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decodeDelimited(
      reader: $protobuf.Reader | Uint8Array
    ): pruntime_rpc.GetWorkerStateRequest;

    /**
     * Verifies a GetWorkerStateRequest message.
     * @param message Plain object to verify
     * @returns `null` if valid, otherwise the reason why it is not
     */
    public static verify(message: { [k: string]: any }): string | null;

    /**
     * Creates a GetWorkerStateRequest message from a plain object. Also converts values to their respective internal types.
     * @param object Plain object
     * @returns GetWorkerStateRequest
     */
    public static fromObject(object: {
      [k: string]: any;
    }): pruntime_rpc.GetWorkerStateRequest;

    /**
     * Creates a plain object from a GetWorkerStateRequest message. Also converts values to other types if specified.
     * @param message GetWorkerStateRequest
     * @param [options] Conversion options
     * @returns Plain object
     */
    public static toObject(
      message: pruntime_rpc.GetWorkerStateRequest,
      options?: $protobuf.IConversionOptions
    ): { [k: string]: any };

    /**
     * Converts this GetWorkerStateRequest to JSON.
     * @returns JSON object
     */
    public toJSON(): { [k: string]: any };
  }

  /** Properties of a WorkerStat. */
  interface IWorkerStat {
    /** WorkerStat lastHeartbeatForBlock */
    lastHeartbeatForBlock?: number | null;

    /** WorkerStat lastHeartbeatAtBlock */
    lastHeartbeatAtBlock?: number | null;

    /** WorkerStat lastGkResponsiveEvent */
    lastGkResponsiveEvent?: pruntime_rpc.ResponsiveEvent | null;

    /** WorkerStat lastGkResponsiveEventAtBlock */
    lastGkResponsiveEventAtBlock?: number | null;
  }

  /** Represents a WorkerStat. */
  class WorkerStat implements IWorkerStat {
    /**
     * Constructs a new WorkerStat.
     * @param [properties] Properties to set
     */
    constructor(properties?: pruntime_rpc.IWorkerStat);

    /** WorkerStat lastHeartbeatForBlock. */
    public lastHeartbeatForBlock: number;

    /** WorkerStat lastHeartbeatAtBlock. */
    public lastHeartbeatAtBlock: number;

    /** WorkerStat lastGkResponsiveEvent. */
    public lastGkResponsiveEvent: pruntime_rpc.ResponsiveEvent;

    /** WorkerStat lastGkResponsiveEventAtBlock. */
    public lastGkResponsiveEventAtBlock: number;

    /**
     * Creates a new WorkerStat instance using the specified properties.
     * @param [properties] Properties to set
     * @returns WorkerStat instance
     */
    public static create(
      properties?: pruntime_rpc.IWorkerStat
    ): pruntime_rpc.WorkerStat;

    /**
     * Encodes the specified WorkerStat message. Does not implicitly {@link pruntime_rpc.WorkerStat.verify|verify} messages.
     * @param message WorkerStat message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encode(
      message: pruntime_rpc.IWorkerStat,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Encodes the specified WorkerStat message, length delimited. Does not implicitly {@link pruntime_rpc.WorkerStat.verify|verify} messages.
     * @param message WorkerStat message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encodeDelimited(
      message: pruntime_rpc.IWorkerStat,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Decodes a WorkerStat message from the specified reader or buffer.
     * @param reader Reader or buffer to decode from
     * @param [length] Message length if known beforehand
     * @returns WorkerStat
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decode(
      reader: $protobuf.Reader | Uint8Array,
      length?: number
    ): pruntime_rpc.WorkerStat;

    /**
     * Decodes a WorkerStat message from the specified reader or buffer, length delimited.
     * @param reader Reader or buffer to decode from
     * @returns WorkerStat
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decodeDelimited(
      reader: $protobuf.Reader | Uint8Array
    ): pruntime_rpc.WorkerStat;

    /**
     * Verifies a WorkerStat message.
     * @param message Plain object to verify
     * @returns `null` if valid, otherwise the reason why it is not
     */
    public static verify(message: { [k: string]: any }): string | null;

    /**
     * Creates a WorkerStat message from a plain object. Also converts values to their respective internal types.
     * @param object Plain object
     * @returns WorkerStat
     */
    public static fromObject(object: {
      [k: string]: any;
    }): pruntime_rpc.WorkerStat;

    /**
     * Creates a plain object from a WorkerStat message. Also converts values to other types if specified.
     * @param message WorkerStat
     * @param [options] Conversion options
     * @returns Plain object
     */
    public static toObject(
      message: pruntime_rpc.WorkerStat,
      options?: $protobuf.IConversionOptions
    ): { [k: string]: any };

    /**
     * Converts this WorkerStat to JSON.
     * @returns JSON object
     */
    public toJSON(): { [k: string]: any };
  }

  /** Properties of a WorkerState. */
  interface IWorkerState {
    /** WorkerState registered */
    registered?: boolean | null;

    /** WorkerState unresponsive */
    unresponsive?: boolean | null;

    /** WorkerState benchState */
    benchState?: pruntime_rpc.IBenchState | null;

    /** WorkerState miningState */
    miningState?: pruntime_rpc.IMiningState | null;

    /** WorkerState waitingHeartbeats */
    waitingHeartbeats?: number[] | null;

    /** WorkerState tokenomicInfo */
    tokenomicInfo?: pruntime_rpc.ITokenomicInfo | null;

    /** WorkerState stat */
    stat?: pruntime_rpc.IWorkerStat | null;
  }

  /** Represents a WorkerState. */
  class WorkerState implements IWorkerState {
    /**
     * Constructs a new WorkerState.
     * @param [properties] Properties to set
     */
    constructor(properties?: pruntime_rpc.IWorkerState);

    /** WorkerState registered. */
    public registered: boolean;

    /** WorkerState unresponsive. */
    public unresponsive: boolean;

    /** WorkerState benchState. */
    public benchState?: pruntime_rpc.IBenchState | null;

    /** WorkerState miningState. */
    public miningState?: pruntime_rpc.IMiningState | null;

    /** WorkerState waitingHeartbeats. */
    public waitingHeartbeats: number[];

    /** WorkerState tokenomicInfo. */
    public tokenomicInfo?: pruntime_rpc.ITokenomicInfo | null;

    /** WorkerState stat. */
    public stat?: pruntime_rpc.IWorkerStat | null;

    /**
     * Creates a new WorkerState instance using the specified properties.
     * @param [properties] Properties to set
     * @returns WorkerState instance
     */
    public static create(
      properties?: pruntime_rpc.IWorkerState
    ): pruntime_rpc.WorkerState;

    /**
     * Encodes the specified WorkerState message. Does not implicitly {@link pruntime_rpc.WorkerState.verify|verify} messages.
     * @param message WorkerState message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encode(
      message: pruntime_rpc.IWorkerState,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Encodes the specified WorkerState message, length delimited. Does not implicitly {@link pruntime_rpc.WorkerState.verify|verify} messages.
     * @param message WorkerState message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encodeDelimited(
      message: pruntime_rpc.IWorkerState,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Decodes a WorkerState message from the specified reader or buffer.
     * @param reader Reader or buffer to decode from
     * @param [length] Message length if known beforehand
     * @returns WorkerState
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decode(
      reader: $protobuf.Reader | Uint8Array,
      length?: number
    ): pruntime_rpc.WorkerState;

    /**
     * Decodes a WorkerState message from the specified reader or buffer, length delimited.
     * @param reader Reader or buffer to decode from
     * @returns WorkerState
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decodeDelimited(
      reader: $protobuf.Reader | Uint8Array
    ): pruntime_rpc.WorkerState;

    /**
     * Verifies a WorkerState message.
     * @param message Plain object to verify
     * @returns `null` if valid, otherwise the reason why it is not
     */
    public static verify(message: { [k: string]: any }): string | null;

    /**
     * Creates a WorkerState message from a plain object. Also converts values to their respective internal types.
     * @param object Plain object
     * @returns WorkerState
     */
    public static fromObject(object: {
      [k: string]: any;
    }): pruntime_rpc.WorkerState;

    /**
     * Creates a plain object from a WorkerState message. Also converts values to other types if specified.
     * @param message WorkerState
     * @param [options] Conversion options
     * @returns Plain object
     */
    public static toObject(
      message: pruntime_rpc.WorkerState,
      options?: $protobuf.IConversionOptions
    ): { [k: string]: any };

    /**
     * Converts this WorkerState to JSON.
     * @returns JSON object
     */
    public toJSON(): { [k: string]: any };
  }

  /** Properties of a HandoverChallenge. */
  interface IHandoverChallenge {
    /** HandoverChallenge encodedChallenge */
    encodedChallenge?: Uint8Array | null;
  }

  /** Represents a HandoverChallenge. */
  class HandoverChallenge implements IHandoverChallenge {
    /**
     * Constructs a new HandoverChallenge.
     * @param [properties] Properties to set
     */
    constructor(properties?: pruntime_rpc.IHandoverChallenge);

    /** HandoverChallenge encodedChallenge. */
    public encodedChallenge: Uint8Array;

    /**
     * Creates a new HandoverChallenge instance using the specified properties.
     * @param [properties] Properties to set
     * @returns HandoverChallenge instance
     */
    public static create(
      properties?: pruntime_rpc.IHandoverChallenge
    ): pruntime_rpc.HandoverChallenge;

    /**
     * Encodes the specified HandoverChallenge message. Does not implicitly {@link pruntime_rpc.HandoverChallenge.verify|verify} messages.
     * @param message HandoverChallenge message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encode(
      message: pruntime_rpc.IHandoverChallenge,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Encodes the specified HandoverChallenge message, length delimited. Does not implicitly {@link pruntime_rpc.HandoverChallenge.verify|verify} messages.
     * @param message HandoverChallenge message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encodeDelimited(
      message: pruntime_rpc.IHandoverChallenge,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Decodes a HandoverChallenge message from the specified reader or buffer.
     * @param reader Reader or buffer to decode from
     * @param [length] Message length if known beforehand
     * @returns HandoverChallenge
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decode(
      reader: $protobuf.Reader | Uint8Array,
      length?: number
    ): pruntime_rpc.HandoverChallenge;

    /**
     * Decodes a HandoverChallenge message from the specified reader or buffer, length delimited.
     * @param reader Reader or buffer to decode from
     * @returns HandoverChallenge
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decodeDelimited(
      reader: $protobuf.Reader | Uint8Array
    ): pruntime_rpc.HandoverChallenge;

    /**
     * Verifies a HandoverChallenge message.
     * @param message Plain object to verify
     * @returns `null` if valid, otherwise the reason why it is not
     */
    public static verify(message: { [k: string]: any }): string | null;

    /**
     * Creates a HandoverChallenge message from a plain object. Also converts values to their respective internal types.
     * @param object Plain object
     * @returns HandoverChallenge
     */
    public static fromObject(object: {
      [k: string]: any;
    }): pruntime_rpc.HandoverChallenge;

    /**
     * Creates a plain object from a HandoverChallenge message. Also converts values to other types if specified.
     * @param message HandoverChallenge
     * @param [options] Conversion options
     * @returns Plain object
     */
    public static toObject(
      message: pruntime_rpc.HandoverChallenge,
      options?: $protobuf.IConversionOptions
    ): { [k: string]: any };

    /**
     * Converts this HandoverChallenge to JSON.
     * @returns JSON object
     */
    public toJSON(): { [k: string]: any };
  }

  /** Properties of a HandoverChallengeResponse. */
  interface IHandoverChallengeResponse {
    /** HandoverChallengeResponse encodedChallengeHandler */
    encodedChallengeHandler?: Uint8Array | null;

    /** HandoverChallengeResponse attestation */
    attestation?: pruntime_rpc.IAttestation | null;
  }

  /** Represents a HandoverChallengeResponse. */
  class HandoverChallengeResponse implements IHandoverChallengeResponse {
    /**
     * Constructs a new HandoverChallengeResponse.
     * @param [properties] Properties to set
     */
    constructor(properties?: pruntime_rpc.IHandoverChallengeResponse);

    /** HandoverChallengeResponse encodedChallengeHandler. */
    public encodedChallengeHandler: Uint8Array;

    /** HandoverChallengeResponse attestation. */
    public attestation?: pruntime_rpc.IAttestation | null;

    /**
     * Creates a new HandoverChallengeResponse instance using the specified properties.
     * @param [properties] Properties to set
     * @returns HandoverChallengeResponse instance
     */
    public static create(
      properties?: pruntime_rpc.IHandoverChallengeResponse
    ): pruntime_rpc.HandoverChallengeResponse;

    /**
     * Encodes the specified HandoverChallengeResponse message. Does not implicitly {@link pruntime_rpc.HandoverChallengeResponse.verify|verify} messages.
     * @param message HandoverChallengeResponse message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encode(
      message: pruntime_rpc.IHandoverChallengeResponse,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Encodes the specified HandoverChallengeResponse message, length delimited. Does not implicitly {@link pruntime_rpc.HandoverChallengeResponse.verify|verify} messages.
     * @param message HandoverChallengeResponse message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encodeDelimited(
      message: pruntime_rpc.IHandoverChallengeResponse,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Decodes a HandoverChallengeResponse message from the specified reader or buffer.
     * @param reader Reader or buffer to decode from
     * @param [length] Message length if known beforehand
     * @returns HandoverChallengeResponse
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decode(
      reader: $protobuf.Reader | Uint8Array,
      length?: number
    ): pruntime_rpc.HandoverChallengeResponse;

    /**
     * Decodes a HandoverChallengeResponse message from the specified reader or buffer, length delimited.
     * @param reader Reader or buffer to decode from
     * @returns HandoverChallengeResponse
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decodeDelimited(
      reader: $protobuf.Reader | Uint8Array
    ): pruntime_rpc.HandoverChallengeResponse;

    /**
     * Verifies a HandoverChallengeResponse message.
     * @param message Plain object to verify
     * @returns `null` if valid, otherwise the reason why it is not
     */
    public static verify(message: { [k: string]: any }): string | null;

    /**
     * Creates a HandoverChallengeResponse message from a plain object. Also converts values to their respective internal types.
     * @param object Plain object
     * @returns HandoverChallengeResponse
     */
    public static fromObject(object: {
      [k: string]: any;
    }): pruntime_rpc.HandoverChallengeResponse;

    /**
     * Creates a plain object from a HandoverChallengeResponse message. Also converts values to other types if specified.
     * @param message HandoverChallengeResponse
     * @param [options] Conversion options
     * @returns Plain object
     */
    public static toObject(
      message: pruntime_rpc.HandoverChallengeResponse,
      options?: $protobuf.IConversionOptions
    ): { [k: string]: any };

    /**
     * Converts this HandoverChallengeResponse to JSON.
     * @returns JSON object
     */
    public toJSON(): { [k: string]: any };
  }

  /** Properties of a HandoverWorkerKey. */
  interface IHandoverWorkerKey {
    /** HandoverWorkerKey encodedWorkerKey */
    encodedWorkerKey?: Uint8Array | null;

    /** HandoverWorkerKey attestation */
    attestation?: pruntime_rpc.IAttestation | null;
  }

  /** Represents a HandoverWorkerKey. */
  class HandoverWorkerKey implements IHandoverWorkerKey {
    /**
     * Constructs a new HandoverWorkerKey.
     * @param [properties] Properties to set
     */
    constructor(properties?: pruntime_rpc.IHandoverWorkerKey);

    /** HandoverWorkerKey encodedWorkerKey. */
    public encodedWorkerKey: Uint8Array;

    /** HandoverWorkerKey attestation. */
    public attestation?: pruntime_rpc.IAttestation | null;

    /**
     * Creates a new HandoverWorkerKey instance using the specified properties.
     * @param [properties] Properties to set
     * @returns HandoverWorkerKey instance
     */
    public static create(
      properties?: pruntime_rpc.IHandoverWorkerKey
    ): pruntime_rpc.HandoverWorkerKey;

    /**
     * Encodes the specified HandoverWorkerKey message. Does not implicitly {@link pruntime_rpc.HandoverWorkerKey.verify|verify} messages.
     * @param message HandoverWorkerKey message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encode(
      message: pruntime_rpc.IHandoverWorkerKey,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Encodes the specified HandoverWorkerKey message, length delimited. Does not implicitly {@link pruntime_rpc.HandoverWorkerKey.verify|verify} messages.
     * @param message HandoverWorkerKey message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encodeDelimited(
      message: pruntime_rpc.IHandoverWorkerKey,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Decodes a HandoverWorkerKey message from the specified reader or buffer.
     * @param reader Reader or buffer to decode from
     * @param [length] Message length if known beforehand
     * @returns HandoverWorkerKey
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decode(
      reader: $protobuf.Reader | Uint8Array,
      length?: number
    ): pruntime_rpc.HandoverWorkerKey;

    /**
     * Decodes a HandoverWorkerKey message from the specified reader or buffer, length delimited.
     * @param reader Reader or buffer to decode from
     * @returns HandoverWorkerKey
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decodeDelimited(
      reader: $protobuf.Reader | Uint8Array
    ): pruntime_rpc.HandoverWorkerKey;

    /**
     * Verifies a HandoverWorkerKey message.
     * @param message Plain object to verify
     * @returns `null` if valid, otherwise the reason why it is not
     */
    public static verify(message: { [k: string]: any }): string | null;

    /**
     * Creates a HandoverWorkerKey message from a plain object. Also converts values to their respective internal types.
     * @param object Plain object
     * @returns HandoverWorkerKey
     */
    public static fromObject(object: {
      [k: string]: any;
    }): pruntime_rpc.HandoverWorkerKey;

    /**
     * Creates a plain object from a HandoverWorkerKey message. Also converts values to other types if specified.
     * @param message HandoverWorkerKey
     * @param [options] Conversion options
     * @returns Plain object
     */
    public static toObject(
      message: pruntime_rpc.HandoverWorkerKey,
      options?: $protobuf.IConversionOptions
    ): { [k: string]: any };

    /**
     * Converts this HandoverWorkerKey to JSON.
     * @returns JSON object
     */
    public toJSON(): { [k: string]: any };
  }

  /** Properties of a BenchState. */
  interface IBenchState {
    /** BenchState startBlock */
    startBlock?: number | null;

    /** BenchState startTime */
    startTime?: number | Long | null;

    /** BenchState duration */
    duration?: number | null;
  }

  /** Represents a BenchState. */
  class BenchState implements IBenchState {
    /**
     * Constructs a new BenchState.
     * @param [properties] Properties to set
     */
    constructor(properties?: pruntime_rpc.IBenchState);

    /** BenchState startBlock. */
    public startBlock: number;

    /** BenchState startTime. */
    public startTime: number | Long;

    /** BenchState duration. */
    public duration: number;

    /**
     * Creates a new BenchState instance using the specified properties.
     * @param [properties] Properties to set
     * @returns BenchState instance
     */
    public static create(
      properties?: pruntime_rpc.IBenchState
    ): pruntime_rpc.BenchState;

    /**
     * Encodes the specified BenchState message. Does not implicitly {@link pruntime_rpc.BenchState.verify|verify} messages.
     * @param message BenchState message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encode(
      message: pruntime_rpc.IBenchState,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Encodes the specified BenchState message, length delimited. Does not implicitly {@link pruntime_rpc.BenchState.verify|verify} messages.
     * @param message BenchState message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encodeDelimited(
      message: pruntime_rpc.IBenchState,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Decodes a BenchState message from the specified reader or buffer.
     * @param reader Reader or buffer to decode from
     * @param [length] Message length if known beforehand
     * @returns BenchState
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decode(
      reader: $protobuf.Reader | Uint8Array,
      length?: number
    ): pruntime_rpc.BenchState;

    /**
     * Decodes a BenchState message from the specified reader or buffer, length delimited.
     * @param reader Reader or buffer to decode from
     * @returns BenchState
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decodeDelimited(
      reader: $protobuf.Reader | Uint8Array
    ): pruntime_rpc.BenchState;

    /**
     * Verifies a BenchState message.
     * @param message Plain object to verify
     * @returns `null` if valid, otherwise the reason why it is not
     */
    public static verify(message: { [k: string]: any }): string | null;

    /**
     * Creates a BenchState message from a plain object. Also converts values to their respective internal types.
     * @param object Plain object
     * @returns BenchState
     */
    public static fromObject(object: {
      [k: string]: any;
    }): pruntime_rpc.BenchState;

    /**
     * Creates a plain object from a BenchState message. Also converts values to other types if specified.
     * @param message BenchState
     * @param [options] Conversion options
     * @returns Plain object
     */
    public static toObject(
      message: pruntime_rpc.BenchState,
      options?: $protobuf.IConversionOptions
    ): { [k: string]: any };

    /**
     * Converts this BenchState to JSON.
     * @returns JSON object
     */
    public toJSON(): { [k: string]: any };
  }

  /** Properties of a MiningState. */
  interface IMiningState {
    /** MiningState sessionId */
    sessionId?: number | null;

    /** MiningState paused */
    paused?: boolean | null;

    /** MiningState startTime */
    startTime?: number | Long | null;
  }

  /** Represents a MiningState. */
  class MiningState implements IMiningState {
    /**
     * Constructs a new MiningState.
     * @param [properties] Properties to set
     */
    constructor(properties?: pruntime_rpc.IMiningState);

    /** MiningState sessionId. */
    public sessionId: number;

    /** MiningState paused. */
    public paused: boolean;

    /** MiningState startTime. */
    public startTime: number | Long;

    /**
     * Creates a new MiningState instance using the specified properties.
     * @param [properties] Properties to set
     * @returns MiningState instance
     */
    public static create(
      properties?: pruntime_rpc.IMiningState
    ): pruntime_rpc.MiningState;

    /**
     * Encodes the specified MiningState message. Does not implicitly {@link pruntime_rpc.MiningState.verify|verify} messages.
     * @param message MiningState message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encode(
      message: pruntime_rpc.IMiningState,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Encodes the specified MiningState message, length delimited. Does not implicitly {@link pruntime_rpc.MiningState.verify|verify} messages.
     * @param message MiningState message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encodeDelimited(
      message: pruntime_rpc.IMiningState,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Decodes a MiningState message from the specified reader or buffer.
     * @param reader Reader or buffer to decode from
     * @param [length] Message length if known beforehand
     * @returns MiningState
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decode(
      reader: $protobuf.Reader | Uint8Array,
      length?: number
    ): pruntime_rpc.MiningState;

    /**
     * Decodes a MiningState message from the specified reader or buffer, length delimited.
     * @param reader Reader or buffer to decode from
     * @returns MiningState
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decodeDelimited(
      reader: $protobuf.Reader | Uint8Array
    ): pruntime_rpc.MiningState;

    /**
     * Verifies a MiningState message.
     * @param message Plain object to verify
     * @returns `null` if valid, otherwise the reason why it is not
     */
    public static verify(message: { [k: string]: any }): string | null;

    /**
     * Creates a MiningState message from a plain object. Also converts values to their respective internal types.
     * @param object Plain object
     * @returns MiningState
     */
    public static fromObject(object: {
      [k: string]: any;
    }): pruntime_rpc.MiningState;

    /**
     * Creates a plain object from a MiningState message. Also converts values to other types if specified.
     * @param message MiningState
     * @param [options] Conversion options
     * @returns Plain object
     */
    public static toObject(
      message: pruntime_rpc.MiningState,
      options?: $protobuf.IConversionOptions
    ): { [k: string]: any };

    /**
     * Converts this MiningState to JSON.
     * @returns JSON object
     */
    public toJSON(): { [k: string]: any };
  }

  /** Properties of an EchoMessage. */
  interface IEchoMessage {
    /** EchoMessage echoMsg */
    echoMsg?: Uint8Array | null;
  }

  /** Represents an EchoMessage. */
  class EchoMessage implements IEchoMessage {
    /**
     * Constructs a new EchoMessage.
     * @param [properties] Properties to set
     */
    constructor(properties?: pruntime_rpc.IEchoMessage);

    /** EchoMessage echoMsg. */
    public echoMsg: Uint8Array;

    /**
     * Creates a new EchoMessage instance using the specified properties.
     * @param [properties] Properties to set
     * @returns EchoMessage instance
     */
    public static create(
      properties?: pruntime_rpc.IEchoMessage
    ): pruntime_rpc.EchoMessage;

    /**
     * Encodes the specified EchoMessage message. Does not implicitly {@link pruntime_rpc.EchoMessage.verify|verify} messages.
     * @param message EchoMessage message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encode(
      message: pruntime_rpc.IEchoMessage,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Encodes the specified EchoMessage message, length delimited. Does not implicitly {@link pruntime_rpc.EchoMessage.verify|verify} messages.
     * @param message EchoMessage message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encodeDelimited(
      message: pruntime_rpc.IEchoMessage,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Decodes an EchoMessage message from the specified reader or buffer.
     * @param reader Reader or buffer to decode from
     * @param [length] Message length if known beforehand
     * @returns EchoMessage
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decode(
      reader: $protobuf.Reader | Uint8Array,
      length?: number
    ): pruntime_rpc.EchoMessage;

    /**
     * Decodes an EchoMessage message from the specified reader or buffer, length delimited.
     * @param reader Reader or buffer to decode from
     * @returns EchoMessage
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decodeDelimited(
      reader: $protobuf.Reader | Uint8Array
    ): pruntime_rpc.EchoMessage;

    /**
     * Verifies an EchoMessage message.
     * @param message Plain object to verify
     * @returns `null` if valid, otherwise the reason why it is not
     */
    public static verify(message: { [k: string]: any }): string | null;

    /**
     * Creates an EchoMessage message from a plain object. Also converts values to their respective internal types.
     * @param object Plain object
     * @returns EchoMessage
     */
    public static fromObject(object: {
      [k: string]: any;
    }): pruntime_rpc.EchoMessage;

    /**
     * Creates a plain object from an EchoMessage message. Also converts values to other types if specified.
     * @param message EchoMessage
     * @param [options] Conversion options
     * @returns Plain object
     */
    public static toObject(
      message: pruntime_rpc.EchoMessage,
      options?: $protobuf.IConversionOptions
    ): { [k: string]: any };

    /**
     * Converts this EchoMessage to JSON.
     * @returns JSON object
     */
    public toJSON(): { [k: string]: any };
  }

  /** ResponsiveEvent enum. */
  enum ResponsiveEvent {
    NoEvent = 0,
    EnterUnresponsive = 1,
    ExitUnresponsive = 2,
  }

  /** Properties of an AddEndpointRequest. */
  interface IAddEndpointRequest {
    /** AddEndpointRequest encodedEndpointType */
    encodedEndpointType?: Uint8Array | null;

    /** AddEndpointRequest endpoint */
    endpoint?: string | null;
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
    public static create(
      properties?: pruntime_rpc.IAddEndpointRequest
    ): pruntime_rpc.AddEndpointRequest;

    /**
     * Encodes the specified AddEndpointRequest message. Does not implicitly {@link pruntime_rpc.AddEndpointRequest.verify|verify} messages.
     * @param message AddEndpointRequest message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encode(
      message: pruntime_rpc.IAddEndpointRequest,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Encodes the specified AddEndpointRequest message, length delimited. Does not implicitly {@link pruntime_rpc.AddEndpointRequest.verify|verify} messages.
     * @param message AddEndpointRequest message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encodeDelimited(
      message: pruntime_rpc.IAddEndpointRequest,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Decodes an AddEndpointRequest message from the specified reader or buffer.
     * @param reader Reader or buffer to decode from
     * @param [length] Message length if known beforehand
     * @returns AddEndpointRequest
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decode(
      reader: $protobuf.Reader | Uint8Array,
      length?: number
    ): pruntime_rpc.AddEndpointRequest;

    /**
     * Decodes an AddEndpointRequest message from the specified reader or buffer, length delimited.
     * @param reader Reader or buffer to decode from
     * @returns AddEndpointRequest
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decodeDelimited(
      reader: $protobuf.Reader | Uint8Array
    ): pruntime_rpc.AddEndpointRequest;

    /**
     * Verifies an AddEndpointRequest message.
     * @param message Plain object to verify
     * @returns `null` if valid, otherwise the reason why it is not
     */
    public static verify(message: { [k: string]: any }): string | null;

    /**
     * Creates an AddEndpointRequest message from a plain object. Also converts values to their respective internal types.
     * @param object Plain object
     * @returns AddEndpointRequest
     */
    public static fromObject(object: {
      [k: string]: any;
    }): pruntime_rpc.AddEndpointRequest;

    /**
     * Creates a plain object from an AddEndpointRequest message. Also converts values to other types if specified.
     * @param message AddEndpointRequest
     * @param [options] Conversion options
     * @returns Plain object
     */
    public static toObject(
      message: pruntime_rpc.AddEndpointRequest,
      options?: $protobuf.IConversionOptions
    ): { [k: string]: any };

    /**
     * Converts this AddEndpointRequest to JSON.
     * @returns JSON object
     */
    public toJSON(): { [k: string]: any };
  }

  /** Properties of a GetEndpointResponse. */
  interface IGetEndpointResponse {
    /** GetEndpointResponse encodedEndpointPayload */
    encodedEndpointPayload?: Uint8Array | null;

    /** GetEndpointResponse signature */
    signature?: Uint8Array | null;
  }

  /** Represents a GetEndpointResponse. */
  class GetEndpointResponse implements IGetEndpointResponse {
    /**
     * Constructs a new GetEndpointResponse.
     * @param [properties] Properties to set
     */
    constructor(properties?: pruntime_rpc.IGetEndpointResponse);

    /** GetEndpointResponse encodedEndpointPayload. */
    public encodedEndpointPayload?: Uint8Array | null;

    /** GetEndpointResponse signature. */
    public signature?: Uint8Array | null;

    /** GetEndpointResponse _encodedEndpointPayload. */
    public _encodedEndpointPayload?: "encodedEndpointPayload";

    /** GetEndpointResponse _signature. */
    public _signature?: "signature";

    /**
     * Creates a new GetEndpointResponse instance using the specified properties.
     * @param [properties] Properties to set
     * @returns GetEndpointResponse instance
     */
    public static create(
      properties?: pruntime_rpc.IGetEndpointResponse
    ): pruntime_rpc.GetEndpointResponse;

    /**
     * Encodes the specified GetEndpointResponse message. Does not implicitly {@link pruntime_rpc.GetEndpointResponse.verify|verify} messages.
     * @param message GetEndpointResponse message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encode(
      message: pruntime_rpc.IGetEndpointResponse,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Encodes the specified GetEndpointResponse message, length delimited. Does not implicitly {@link pruntime_rpc.GetEndpointResponse.verify|verify} messages.
     * @param message GetEndpointResponse message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encodeDelimited(
      message: pruntime_rpc.IGetEndpointResponse,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Decodes a GetEndpointResponse message from the specified reader or buffer.
     * @param reader Reader or buffer to decode from
     * @param [length] Message length if known beforehand
     * @returns GetEndpointResponse
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decode(
      reader: $protobuf.Reader | Uint8Array,
      length?: number
    ): pruntime_rpc.GetEndpointResponse;

    /**
     * Decodes a GetEndpointResponse message from the specified reader or buffer, length delimited.
     * @param reader Reader or buffer to decode from
     * @returns GetEndpointResponse
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decodeDelimited(
      reader: $protobuf.Reader | Uint8Array
    ): pruntime_rpc.GetEndpointResponse;

    /**
     * Verifies a GetEndpointResponse message.
     * @param message Plain object to verify
     * @returns `null` if valid, otherwise the reason why it is not
     */
    public static verify(message: { [k: string]: any }): string | null;

    /**
     * Creates a GetEndpointResponse message from a plain object. Also converts values to their respective internal types.
     * @param object Plain object
     * @returns GetEndpointResponse
     */
    public static fromObject(object: {
      [k: string]: any;
    }): pruntime_rpc.GetEndpointResponse;

    /**
     * Creates a plain object from a GetEndpointResponse message. Also converts values to other types if specified.
     * @param message GetEndpointResponse
     * @param [options] Conversion options
     * @returns Plain object
     */
    public static toObject(
      message: pruntime_rpc.GetEndpointResponse,
      options?: $protobuf.IConversionOptions
    ): { [k: string]: any };

    /**
     * Converts this GetEndpointResponse to JSON.
     * @returns JSON object
     */
    public toJSON(): { [k: string]: any };
  }

  /** Properties of a SignEndpointsRequest. */
  interface ISignEndpointsRequest {
    /** SignEndpointsRequest encodedEndpoints */
    encodedEndpoints?: Uint8Array | null;
  }

  /** Represents a SignEndpointsRequest. */
  class SignEndpointsRequest implements ISignEndpointsRequest {
    /**
     * Constructs a new SignEndpointsRequest.
     * @param [properties] Properties to set
     */
    constructor(properties?: pruntime_rpc.ISignEndpointsRequest);

    /** SignEndpointsRequest encodedEndpoints. */
    public encodedEndpoints: Uint8Array;

    /**
     * Creates a new SignEndpointsRequest instance using the specified properties.
     * @param [properties] Properties to set
     * @returns SignEndpointsRequest instance
     */
    public static create(
      properties?: pruntime_rpc.ISignEndpointsRequest
    ): pruntime_rpc.SignEndpointsRequest;

    /**
     * Encodes the specified SignEndpointsRequest message. Does not implicitly {@link pruntime_rpc.SignEndpointsRequest.verify|verify} messages.
     * @param message SignEndpointsRequest message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encode(
      message: pruntime_rpc.ISignEndpointsRequest,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Encodes the specified SignEndpointsRequest message, length delimited. Does not implicitly {@link pruntime_rpc.SignEndpointsRequest.verify|verify} messages.
     * @param message SignEndpointsRequest message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encodeDelimited(
      message: pruntime_rpc.ISignEndpointsRequest,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Decodes a SignEndpointsRequest message from the specified reader or buffer.
     * @param reader Reader or buffer to decode from
     * @param [length] Message length if known beforehand
     * @returns SignEndpointsRequest
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decode(
      reader: $protobuf.Reader | Uint8Array,
      length?: number
    ): pruntime_rpc.SignEndpointsRequest;

    /**
     * Decodes a SignEndpointsRequest message from the specified reader or buffer, length delimited.
     * @param reader Reader or buffer to decode from
     * @returns SignEndpointsRequest
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decodeDelimited(
      reader: $protobuf.Reader | Uint8Array
    ): pruntime_rpc.SignEndpointsRequest;

    /**
     * Verifies a SignEndpointsRequest message.
     * @param message Plain object to verify
     * @returns `null` if valid, otherwise the reason why it is not
     */
    public static verify(message: { [k: string]: any }): string | null;

    /**
     * Creates a SignEndpointsRequest message from a plain object. Also converts values to their respective internal types.
     * @param object Plain object
     * @returns SignEndpointsRequest
     */
    public static fromObject(object: {
      [k: string]: any;
    }): pruntime_rpc.SignEndpointsRequest;

    /**
     * Creates a plain object from a SignEndpointsRequest message. Also converts values to other types if specified.
     * @param message SignEndpointsRequest
     * @param [options] Conversion options
     * @returns Plain object
     */
    public static toObject(
      message: pruntime_rpc.SignEndpointsRequest,
      options?: $protobuf.IConversionOptions
    ): { [k: string]: any };

    /**
     * Converts this SignEndpointsRequest to JSON.
     * @returns JSON object
     */
    public toJSON(): { [k: string]: any };
  }

  /** Properties of a DerivePhalaI2pKeyResponse. */
  interface IDerivePhalaI2pKeyResponse {
    /** DerivePhalaI2pKeyResponse phalaI2pKey */
    phalaI2pKey?: Uint8Array | null;
  }

  /** Represents a DerivePhalaI2pKeyResponse. */
  class DerivePhalaI2pKeyResponse implements IDerivePhalaI2pKeyResponse {
    /**
     * Constructs a new DerivePhalaI2pKeyResponse.
     * @param [properties] Properties to set
     */
    constructor(properties?: pruntime_rpc.IDerivePhalaI2pKeyResponse);

    /** DerivePhalaI2pKeyResponse phalaI2pKey. */
    public phalaI2pKey: Uint8Array;

    /**
     * Creates a new DerivePhalaI2pKeyResponse instance using the specified properties.
     * @param [properties] Properties to set
     * @returns DerivePhalaI2pKeyResponse instance
     */
    public static create(
      properties?: pruntime_rpc.IDerivePhalaI2pKeyResponse
    ): pruntime_rpc.DerivePhalaI2pKeyResponse;

    /**
     * Encodes the specified DerivePhalaI2pKeyResponse message. Does not implicitly {@link pruntime_rpc.DerivePhalaI2pKeyResponse.verify|verify} messages.
     * @param message DerivePhalaI2pKeyResponse message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encode(
      message: pruntime_rpc.IDerivePhalaI2pKeyResponse,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Encodes the specified DerivePhalaI2pKeyResponse message, length delimited. Does not implicitly {@link pruntime_rpc.DerivePhalaI2pKeyResponse.verify|verify} messages.
     * @param message DerivePhalaI2pKeyResponse message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encodeDelimited(
      message: pruntime_rpc.IDerivePhalaI2pKeyResponse,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Decodes a DerivePhalaI2pKeyResponse message from the specified reader or buffer.
     * @param reader Reader or buffer to decode from
     * @param [length] Message length if known beforehand
     * @returns DerivePhalaI2pKeyResponse
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decode(
      reader: $protobuf.Reader | Uint8Array,
      length?: number
    ): pruntime_rpc.DerivePhalaI2pKeyResponse;

    /**
     * Decodes a DerivePhalaI2pKeyResponse message from the specified reader or buffer, length delimited.
     * @param reader Reader or buffer to decode from
     * @returns DerivePhalaI2pKeyResponse
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decodeDelimited(
      reader: $protobuf.Reader | Uint8Array
    ): pruntime_rpc.DerivePhalaI2pKeyResponse;

    /**
     * Verifies a DerivePhalaI2pKeyResponse message.
     * @param message Plain object to verify
     * @returns `null` if valid, otherwise the reason why it is not
     */
    public static verify(message: { [k: string]: any }): string | null;

    /**
     * Creates a DerivePhalaI2pKeyResponse message from a plain object. Also converts values to their respective internal types.
     * @param object Plain object
     * @returns DerivePhalaI2pKeyResponse
     */
    public static fromObject(object: {
      [k: string]: any;
    }): pruntime_rpc.DerivePhalaI2pKeyResponse;

    /**
     * Creates a plain object from a DerivePhalaI2pKeyResponse message. Also converts values to other types if specified.
     * @param message DerivePhalaI2pKeyResponse
     * @param [options] Conversion options
     * @returns Plain object
     */
    public static toObject(
      message: pruntime_rpc.DerivePhalaI2pKeyResponse,
      options?: $protobuf.IConversionOptions
    ): { [k: string]: any };

    /**
     * Converts this DerivePhalaI2pKeyResponse to JSON.
     * @returns JSON object
     */
    public toJSON(): { [k: string]: any };
  }

  /** Properties of a TokenomicStat. */
  interface ITokenomicStat {
    /** TokenomicStat lastPayout */
    lastPayout?: string | null;

    /** TokenomicStat lastPayoutAtBlock */
    lastPayoutAtBlock?: number | null;

    /** TokenomicStat totalPayout */
    totalPayout?: string | null;

    /** TokenomicStat totalPayoutCount */
    totalPayoutCount?: number | null;

    /** TokenomicStat lastSlash */
    lastSlash?: string | null;

    /** TokenomicStat lastSlashAtBlock */
    lastSlashAtBlock?: number | null;

    /** TokenomicStat totalSlash */
    totalSlash?: string | null;

    /** TokenomicStat totalSlashCount */
    totalSlashCount?: number | null;
  }

  /** Represents a TokenomicStat. */
  class TokenomicStat implements ITokenomicStat {
    /**
     * Constructs a new TokenomicStat.
     * @param [properties] Properties to set
     */
    constructor(properties?: pruntime_rpc.ITokenomicStat);

    /** TokenomicStat lastPayout. */
    public lastPayout: string;

    /** TokenomicStat lastPayoutAtBlock. */
    public lastPayoutAtBlock: number;

    /** TokenomicStat totalPayout. */
    public totalPayout: string;

    /** TokenomicStat totalPayoutCount. */
    public totalPayoutCount: number;

    /** TokenomicStat lastSlash. */
    public lastSlash: string;

    /** TokenomicStat lastSlashAtBlock. */
    public lastSlashAtBlock: number;

    /** TokenomicStat totalSlash. */
    public totalSlash: string;

    /** TokenomicStat totalSlashCount. */
    public totalSlashCount: number;

    /**
     * Creates a new TokenomicStat instance using the specified properties.
     * @param [properties] Properties to set
     * @returns TokenomicStat instance
     */
    public static create(
      properties?: pruntime_rpc.ITokenomicStat
    ): pruntime_rpc.TokenomicStat;

    /**
     * Encodes the specified TokenomicStat message. Does not implicitly {@link pruntime_rpc.TokenomicStat.verify|verify} messages.
     * @param message TokenomicStat message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encode(
      message: pruntime_rpc.ITokenomicStat,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Encodes the specified TokenomicStat message, length delimited. Does not implicitly {@link pruntime_rpc.TokenomicStat.verify|verify} messages.
     * @param message TokenomicStat message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encodeDelimited(
      message: pruntime_rpc.ITokenomicStat,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Decodes a TokenomicStat message from the specified reader or buffer.
     * @param reader Reader or buffer to decode from
     * @param [length] Message length if known beforehand
     * @returns TokenomicStat
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decode(
      reader: $protobuf.Reader | Uint8Array,
      length?: number
    ): pruntime_rpc.TokenomicStat;

    /**
     * Decodes a TokenomicStat message from the specified reader or buffer, length delimited.
     * @param reader Reader or buffer to decode from
     * @returns TokenomicStat
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decodeDelimited(
      reader: $protobuf.Reader | Uint8Array
    ): pruntime_rpc.TokenomicStat;

    /**
     * Verifies a TokenomicStat message.
     * @param message Plain object to verify
     * @returns `null` if valid, otherwise the reason why it is not
     */
    public static verify(message: { [k: string]: any }): string | null;

    /**
     * Creates a TokenomicStat message from a plain object. Also converts values to their respective internal types.
     * @param object Plain object
     * @returns TokenomicStat
     */
    public static fromObject(object: {
      [k: string]: any;
    }): pruntime_rpc.TokenomicStat;

    /**
     * Creates a plain object from a TokenomicStat message. Also converts values to other types if specified.
     * @param message TokenomicStat
     * @param [options] Conversion options
     * @returns Plain object
     */
    public static toObject(
      message: pruntime_rpc.TokenomicStat,
      options?: $protobuf.IConversionOptions
    ): { [k: string]: any };

    /**
     * Converts this TokenomicStat to JSON.
     * @returns JSON object
     */
    public toJSON(): { [k: string]: any };
  }

  /** Properties of a TokenomicInfo. */
  interface ITokenomicInfo {
    /** TokenomicInfo v */
    v?: string | null;

    /** TokenomicInfo vInit */
    vInit?: string | null;

    /** TokenomicInfo vDeductible */
    vDeductible?: string | null;

    /** TokenomicInfo share */
    share?: string | null;

    /** TokenomicInfo vUpdateAt */
    vUpdateAt?: number | Long | null;

    /** TokenomicInfo vUpdateBlock */
    vUpdateBlock?: number | null;

    /** TokenomicInfo iterationLast */
    iterationLast?: number | Long | null;

    /** TokenomicInfo challengeTimeLast */
    challengeTimeLast?: number | Long | null;

    /** TokenomicInfo pBench */
    pBench?: string | null;

    /** TokenomicInfo pInstant */
    pInstant?: string | null;

    /** TokenomicInfo confidenceLevel */
    confidenceLevel?: number | null;

    /** TokenomicInfo stat */
    stat?: pruntime_rpc.ITokenomicStat | null;
  }

  /** Represents a TokenomicInfo. */
  class TokenomicInfo implements ITokenomicInfo {
    /**
     * Constructs a new TokenomicInfo.
     * @param [properties] Properties to set
     */
    constructor(properties?: pruntime_rpc.ITokenomicInfo);

    /** TokenomicInfo v. */
    public v: string;

    /** TokenomicInfo vInit. */
    public vInit: string;

    /** TokenomicInfo vDeductible. */
    public vDeductible: string;

    /** TokenomicInfo share. */
    public share: string;

    /** TokenomicInfo vUpdateAt. */
    public vUpdateAt: number | Long;

    /** TokenomicInfo vUpdateBlock. */
    public vUpdateBlock: number;

    /** TokenomicInfo iterationLast. */
    public iterationLast: number | Long;

    /** TokenomicInfo challengeTimeLast. */
    public challengeTimeLast: number | Long;

    /** TokenomicInfo pBench. */
    public pBench: string;

    /** TokenomicInfo pInstant. */
    public pInstant: string;

    /** TokenomicInfo confidenceLevel. */
    public confidenceLevel: number;

    /** TokenomicInfo stat. */
    public stat?: pruntime_rpc.ITokenomicStat | null;

    /**
     * Creates a new TokenomicInfo instance using the specified properties.
     * @param [properties] Properties to set
     * @returns TokenomicInfo instance
     */
    public static create(
      properties?: pruntime_rpc.ITokenomicInfo
    ): pruntime_rpc.TokenomicInfo;

    /**
     * Encodes the specified TokenomicInfo message. Does not implicitly {@link pruntime_rpc.TokenomicInfo.verify|verify} messages.
     * @param message TokenomicInfo message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encode(
      message: pruntime_rpc.ITokenomicInfo,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Encodes the specified TokenomicInfo message, length delimited. Does not implicitly {@link pruntime_rpc.TokenomicInfo.verify|verify} messages.
     * @param message TokenomicInfo message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encodeDelimited(
      message: pruntime_rpc.ITokenomicInfo,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Decodes a TokenomicInfo message from the specified reader or buffer.
     * @param reader Reader or buffer to decode from
     * @param [length] Message length if known beforehand
     * @returns TokenomicInfo
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decode(
      reader: $protobuf.Reader | Uint8Array,
      length?: number
    ): pruntime_rpc.TokenomicInfo;

    /**
     * Decodes a TokenomicInfo message from the specified reader or buffer, length delimited.
     * @param reader Reader or buffer to decode from
     * @returns TokenomicInfo
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decodeDelimited(
      reader: $protobuf.Reader | Uint8Array
    ): pruntime_rpc.TokenomicInfo;

    /**
     * Verifies a TokenomicInfo message.
     * @param message Plain object to verify
     * @returns `null` if valid, otherwise the reason why it is not
     */
    public static verify(message: { [k: string]: any }): string | null;

    /**
     * Creates a TokenomicInfo message from a plain object. Also converts values to their respective internal types.
     * @param object Plain object
     * @returns TokenomicInfo
     */
    public static fromObject(object: {
      [k: string]: any;
    }): pruntime_rpc.TokenomicInfo;

    /**
     * Creates a plain object from a TokenomicInfo message. Also converts values to other types if specified.
     * @param message TokenomicInfo
     * @param [options] Conversion options
     * @returns Plain object
     */
    public static toObject(
      message: pruntime_rpc.TokenomicInfo,
      options?: $protobuf.IConversionOptions
    ): { [k: string]: any };

    /**
     * Converts this TokenomicInfo to JSON.
     * @returns JSON object
     */
    public toJSON(): { [k: string]: any };
  }

  /** Properties of a NetworkStatus. */
  interface INetworkStatus {
    /** NetworkStatus publicRpcPort */
    publicRpcPort?: number | null;

    /** NetworkStatus config */
    config?: pruntime_rpc.INetworkConfig | null;
  }

  /** Represents a NetworkStatus. */
  class NetworkStatus implements INetworkStatus {
    /**
     * Constructs a new NetworkStatus.
     * @param [properties] Properties to set
     */
    constructor(properties?: pruntime_rpc.INetworkStatus);

    /** NetworkStatus publicRpcPort. */
    public publicRpcPort?: number | null;

    /** NetworkStatus config. */
    public config?: pruntime_rpc.INetworkConfig | null;

    /** NetworkStatus _publicRpcPort. */
    public _publicRpcPort?: "publicRpcPort";

    /** NetworkStatus _config. */
    public _config?: "config";

    /**
     * Creates a new NetworkStatus instance using the specified properties.
     * @param [properties] Properties to set
     * @returns NetworkStatus instance
     */
    public static create(
      properties?: pruntime_rpc.INetworkStatus
    ): pruntime_rpc.NetworkStatus;

    /**
     * Encodes the specified NetworkStatus message. Does not implicitly {@link pruntime_rpc.NetworkStatus.verify|verify} messages.
     * @param message NetworkStatus message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encode(
      message: pruntime_rpc.INetworkStatus,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Encodes the specified NetworkStatus message, length delimited. Does not implicitly {@link pruntime_rpc.NetworkStatus.verify|verify} messages.
     * @param message NetworkStatus message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encodeDelimited(
      message: pruntime_rpc.INetworkStatus,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Decodes a NetworkStatus message from the specified reader or buffer.
     * @param reader Reader or buffer to decode from
     * @param [length] Message length if known beforehand
     * @returns NetworkStatus
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decode(
      reader: $protobuf.Reader | Uint8Array,
      length?: number
    ): pruntime_rpc.NetworkStatus;

    /**
     * Decodes a NetworkStatus message from the specified reader or buffer, length delimited.
     * @param reader Reader or buffer to decode from
     * @returns NetworkStatus
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decodeDelimited(
      reader: $protobuf.Reader | Uint8Array
    ): pruntime_rpc.NetworkStatus;

    /**
     * Verifies a NetworkStatus message.
     * @param message Plain object to verify
     * @returns `null` if valid, otherwise the reason why it is not
     */
    public static verify(message: { [k: string]: any }): string | null;

    /**
     * Creates a NetworkStatus message from a plain object. Also converts values to their respective internal types.
     * @param object Plain object
     * @returns NetworkStatus
     */
    public static fromObject(object: {
      [k: string]: any;
    }): pruntime_rpc.NetworkStatus;

    /**
     * Creates a plain object from a NetworkStatus message. Also converts values to other types if specified.
     * @param message NetworkStatus
     * @param [options] Conversion options
     * @returns Plain object
     */
    public static toObject(
      message: pruntime_rpc.NetworkStatus,
      options?: $protobuf.IConversionOptions
    ): { [k: string]: any };

    /**
     * Converts this NetworkStatus to JSON.
     * @returns JSON object
     */
    public toJSON(): { [k: string]: any };
  }

  /** Properties of a NetworkConfig. */
  interface INetworkConfig {
    /** NetworkConfig allProxy */
    allProxy?: string | null;

    /** NetworkConfig i2pProxy */
    i2pProxy?: string | null;
  }

  /** Represents a NetworkConfig. */
  class NetworkConfig implements INetworkConfig {
    /**
     * Constructs a new NetworkConfig.
     * @param [properties] Properties to set
     */
    constructor(properties?: pruntime_rpc.INetworkConfig);

    /** NetworkConfig allProxy. */
    public allProxy: string;

    /** NetworkConfig i2pProxy. */
    public i2pProxy: string;

    /**
     * Creates a new NetworkConfig instance using the specified properties.
     * @param [properties] Properties to set
     * @returns NetworkConfig instance
     */
    public static create(
      properties?: pruntime_rpc.INetworkConfig
    ): pruntime_rpc.NetworkConfig;

    /**
     * Encodes the specified NetworkConfig message. Does not implicitly {@link pruntime_rpc.NetworkConfig.verify|verify} messages.
     * @param message NetworkConfig message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encode(
      message: pruntime_rpc.INetworkConfig,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Encodes the specified NetworkConfig message, length delimited. Does not implicitly {@link pruntime_rpc.NetworkConfig.verify|verify} messages.
     * @param message NetworkConfig message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encodeDelimited(
      message: pruntime_rpc.INetworkConfig,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Decodes a NetworkConfig message from the specified reader or buffer.
     * @param reader Reader or buffer to decode from
     * @param [length] Message length if known beforehand
     * @returns NetworkConfig
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decode(
      reader: $protobuf.Reader | Uint8Array,
      length?: number
    ): pruntime_rpc.NetworkConfig;

    /**
     * Decodes a NetworkConfig message from the specified reader or buffer, length delimited.
     * @param reader Reader or buffer to decode from
     * @returns NetworkConfig
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decodeDelimited(
      reader: $protobuf.Reader | Uint8Array
    ): pruntime_rpc.NetworkConfig;

    /**
     * Verifies a NetworkConfig message.
     * @param message Plain object to verify
     * @returns `null` if valid, otherwise the reason why it is not
     */
    public static verify(message: { [k: string]: any }): string | null;

    /**
     * Creates a NetworkConfig message from a plain object. Also converts values to their respective internal types.
     * @param object Plain object
     * @returns NetworkConfig
     */
    public static fromObject(object: {
      [k: string]: any;
    }): pruntime_rpc.NetworkConfig;

    /**
     * Creates a plain object from a NetworkConfig message. Also converts values to other types if specified.
     * @param message NetworkConfig
     * @param [options] Conversion options
     * @returns Plain object
     */
    public static toObject(
      message: pruntime_rpc.NetworkConfig,
      options?: $protobuf.IConversionOptions
    ): { [k: string]: any };

    /**
     * Converts this NetworkConfig to JSON.
     * @returns JSON object
     */
    public toJSON(): { [k: string]: any };
  }

  /** Properties of a HttpHeader. */
  interface IHttpHeader {
    /** HttpHeader name */
    name?: string | null;

    /** HttpHeader value */
    value?: string | null;
  }

  /** Represents a HttpHeader. */
  class HttpHeader implements IHttpHeader {
    /**
     * Constructs a new HttpHeader.
     * @param [properties] Properties to set
     */
    constructor(properties?: pruntime_rpc.IHttpHeader);

    /** HttpHeader name. */
    public name: string;

    /** HttpHeader value. */
    public value: string;

    /**
     * Creates a new HttpHeader instance using the specified properties.
     * @param [properties] Properties to set
     * @returns HttpHeader instance
     */
    public static create(
      properties?: pruntime_rpc.IHttpHeader
    ): pruntime_rpc.HttpHeader;

    /**
     * Encodes the specified HttpHeader message. Does not implicitly {@link pruntime_rpc.HttpHeader.verify|verify} messages.
     * @param message HttpHeader message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encode(
      message: pruntime_rpc.IHttpHeader,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Encodes the specified HttpHeader message, length delimited. Does not implicitly {@link pruntime_rpc.HttpHeader.verify|verify} messages.
     * @param message HttpHeader message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encodeDelimited(
      message: pruntime_rpc.IHttpHeader,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Decodes a HttpHeader message from the specified reader or buffer.
     * @param reader Reader or buffer to decode from
     * @param [length] Message length if known beforehand
     * @returns HttpHeader
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decode(
      reader: $protobuf.Reader | Uint8Array,
      length?: number
    ): pruntime_rpc.HttpHeader;

    /**
     * Decodes a HttpHeader message from the specified reader or buffer, length delimited.
     * @param reader Reader or buffer to decode from
     * @returns HttpHeader
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decodeDelimited(
      reader: $protobuf.Reader | Uint8Array
    ): pruntime_rpc.HttpHeader;

    /**
     * Verifies a HttpHeader message.
     * @param message Plain object to verify
     * @returns `null` if valid, otherwise the reason why it is not
     */
    public static verify(message: { [k: string]: any }): string | null;

    /**
     * Creates a HttpHeader message from a plain object. Also converts values to their respective internal types.
     * @param object Plain object
     * @returns HttpHeader
     */
    public static fromObject(object: {
      [k: string]: any;
    }): pruntime_rpc.HttpHeader;

    /**
     * Creates a plain object from a HttpHeader message. Also converts values to other types if specified.
     * @param message HttpHeader
     * @param [options] Conversion options
     * @returns Plain object
     */
    public static toObject(
      message: pruntime_rpc.HttpHeader,
      options?: $protobuf.IConversionOptions
    ): { [k: string]: any };

    /**
     * Converts this HttpHeader to JSON.
     * @returns JSON object
     */
    public toJSON(): { [k: string]: any };
  }

  /** Properties of a HttpRequest. */
  interface IHttpRequest {
    /** HttpRequest url */
    url?: string | null;

    /** HttpRequest method */
    method?: string | null;

    /** HttpRequest headers */
    headers?: pruntime_rpc.IHttpHeader[] | null;

    /** HttpRequest body */
    body?: Uint8Array | null;
  }

  /** Represents a HttpRequest. */
  class HttpRequest implements IHttpRequest {
    /**
     * Constructs a new HttpRequest.
     * @param [properties] Properties to set
     */
    constructor(properties?: pruntime_rpc.IHttpRequest);

    /** HttpRequest url. */
    public url: string;

    /** HttpRequest method. */
    public method: string;

    /** HttpRequest headers. */
    public headers: pruntime_rpc.IHttpHeader[];

    /** HttpRequest body. */
    public body: Uint8Array;

    /**
     * Creates a new HttpRequest instance using the specified properties.
     * @param [properties] Properties to set
     * @returns HttpRequest instance
     */
    public static create(
      properties?: pruntime_rpc.IHttpRequest
    ): pruntime_rpc.HttpRequest;

    /**
     * Encodes the specified HttpRequest message. Does not implicitly {@link pruntime_rpc.HttpRequest.verify|verify} messages.
     * @param message HttpRequest message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encode(
      message: pruntime_rpc.IHttpRequest,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Encodes the specified HttpRequest message, length delimited. Does not implicitly {@link pruntime_rpc.HttpRequest.verify|verify} messages.
     * @param message HttpRequest message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encodeDelimited(
      message: pruntime_rpc.IHttpRequest,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Decodes a HttpRequest message from the specified reader or buffer.
     * @param reader Reader or buffer to decode from
     * @param [length] Message length if known beforehand
     * @returns HttpRequest
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decode(
      reader: $protobuf.Reader | Uint8Array,
      length?: number
    ): pruntime_rpc.HttpRequest;

    /**
     * Decodes a HttpRequest message from the specified reader or buffer, length delimited.
     * @param reader Reader or buffer to decode from
     * @returns HttpRequest
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decodeDelimited(
      reader: $protobuf.Reader | Uint8Array
    ): pruntime_rpc.HttpRequest;

    /**
     * Verifies a HttpRequest message.
     * @param message Plain object to verify
     * @returns `null` if valid, otherwise the reason why it is not
     */
    public static verify(message: { [k: string]: any }): string | null;

    /**
     * Creates a HttpRequest message from a plain object. Also converts values to their respective internal types.
     * @param object Plain object
     * @returns HttpRequest
     */
    public static fromObject(object: {
      [k: string]: any;
    }): pruntime_rpc.HttpRequest;

    /**
     * Creates a plain object from a HttpRequest message. Also converts values to other types if specified.
     * @param message HttpRequest
     * @param [options] Conversion options
     * @returns Plain object
     */
    public static toObject(
      message: pruntime_rpc.HttpRequest,
      options?: $protobuf.IConversionOptions
    ): { [k: string]: any };

    /**
     * Converts this HttpRequest to JSON.
     * @returns JSON object
     */
    public toJSON(): { [k: string]: any };
  }

  /** Properties of a HttpResponse. */
  interface IHttpResponse {
    /** HttpResponse statusCode */
    statusCode?: number | null;

    /** HttpResponse headers */
    headers?: pruntime_rpc.IHttpHeader[] | null;

    /** HttpResponse body */
    body?: Uint8Array | null;
  }

  /** Represents a HttpResponse. */
  class HttpResponse implements IHttpResponse {
    /**
     * Constructs a new HttpResponse.
     * @param [properties] Properties to set
     */
    constructor(properties?: pruntime_rpc.IHttpResponse);

    /** HttpResponse statusCode. */
    public statusCode: number;

    /** HttpResponse headers. */
    public headers: pruntime_rpc.IHttpHeader[];

    /** HttpResponse body. */
    public body: Uint8Array;

    /**
     * Creates a new HttpResponse instance using the specified properties.
     * @param [properties] Properties to set
     * @returns HttpResponse instance
     */
    public static create(
      properties?: pruntime_rpc.IHttpResponse
    ): pruntime_rpc.HttpResponse;

    /**
     * Encodes the specified HttpResponse message. Does not implicitly {@link pruntime_rpc.HttpResponse.verify|verify} messages.
     * @param message HttpResponse message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encode(
      message: pruntime_rpc.IHttpResponse,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Encodes the specified HttpResponse message, length delimited. Does not implicitly {@link pruntime_rpc.HttpResponse.verify|verify} messages.
     * @param message HttpResponse message or plain object to encode
     * @param [writer] Writer to encode to
     * @returns Writer
     */
    public static encodeDelimited(
      message: pruntime_rpc.IHttpResponse,
      writer?: $protobuf.Writer
    ): $protobuf.Writer;

    /**
     * Decodes a HttpResponse message from the specified reader or buffer.
     * @param reader Reader or buffer to decode from
     * @param [length] Message length if known beforehand
     * @returns HttpResponse
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decode(
      reader: $protobuf.Reader | Uint8Array,
      length?: number
    ): pruntime_rpc.HttpResponse;

    /**
     * Decodes a HttpResponse message from the specified reader or buffer, length delimited.
     * @param reader Reader or buffer to decode from
     * @returns HttpResponse
     * @throws {Error} If the payload is not a reader or valid buffer
     * @throws {$protobuf.util.ProtocolError} If required fields are missing
     */
    public static decodeDelimited(
      reader: $protobuf.Reader | Uint8Array
    ): pruntime_rpc.HttpResponse;

    /**
     * Verifies a HttpResponse message.
     * @param message Plain object to verify
     * @returns `null` if valid, otherwise the reason why it is not
     */
    public static verify(message: { [k: string]: any }): string | null;

    /**
     * Creates a HttpResponse message from a plain object. Also converts values to their respective internal types.
     * @param object Plain object
     * @returns HttpResponse
     */
    public static fromObject(object: {
      [k: string]: any;
    }): pruntime_rpc.HttpResponse;

    /**
     * Creates a plain object from a HttpResponse message. Also converts values to other types if specified.
     * @param message HttpResponse
     * @param [options] Conversion options
     * @returns Plain object
     */
    public static toObject(
      message: pruntime_rpc.HttpResponse,
      options?: $protobuf.IConversionOptions
    ): { [k: string]: any };

    /**
     * Converts this HttpResponse to JSON.
     * @returns JSON object
     */
    public toJSON(): { [k: string]: any };
  }
}

/** Namespace google. */
declare namespace google {
  /** Namespace protobuf. */
  namespace protobuf {
    /** Properties of an Empty. */
    interface IEmpty {}

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
      public static create(
        properties?: google.protobuf.IEmpty
      ): google.protobuf.Empty;

      /**
       * Encodes the specified Empty message. Does not implicitly {@link google.protobuf.Empty.verify|verify} messages.
       * @param message Empty message or plain object to encode
       * @param [writer] Writer to encode to
       * @returns Writer
       */
      public static encode(
        message: google.protobuf.IEmpty,
        writer?: $protobuf.Writer
      ): $protobuf.Writer;

      /**
       * Encodes the specified Empty message, length delimited. Does not implicitly {@link google.protobuf.Empty.verify|verify} messages.
       * @param message Empty message or plain object to encode
       * @param [writer] Writer to encode to
       * @returns Writer
       */
      public static encodeDelimited(
        message: google.protobuf.IEmpty,
        writer?: $protobuf.Writer
      ): $protobuf.Writer;

      /**
       * Decodes an Empty message from the specified reader or buffer.
       * @param reader Reader or buffer to decode from
       * @param [length] Message length if known beforehand
       * @returns Empty
       * @throws {Error} If the payload is not a reader or valid buffer
       * @throws {$protobuf.util.ProtocolError} If required fields are missing
       */
      public static decode(
        reader: $protobuf.Reader | Uint8Array,
        length?: number
      ): google.protobuf.Empty;

      /**
       * Decodes an Empty message from the specified reader or buffer, length delimited.
       * @param reader Reader or buffer to decode from
       * @returns Empty
       * @throws {Error} If the payload is not a reader or valid buffer
       * @throws {$protobuf.util.ProtocolError} If required fields are missing
       */
      public static decodeDelimited(
        reader: $protobuf.Reader | Uint8Array
      ): google.protobuf.Empty;

      /**
       * Verifies an Empty message.
       * @param message Plain object to verify
       * @returns `null` if valid, otherwise the reason why it is not
       */
      public static verify(message: { [k: string]: any }): string | null;

      /**
       * Creates an Empty message from a plain object. Also converts values to their respective internal types.
       * @param object Plain object
       * @returns Empty
       */
      public static fromObject(object: {
        [k: string]: any;
      }): google.protobuf.Empty;

      /**
       * Creates a plain object from an Empty message. Also converts values to other types if specified.
       * @param message Empty
       * @param [options] Conversion options
       * @returns Plain object
       */
      public static toObject(
        message: google.protobuf.Empty,
        options?: $protobuf.IConversionOptions
      ): { [k: string]: any };

      /**
       * Converts this Empty to JSON.
       * @returns JSON object
       */
      public toJSON(): { [k: string]: any };
    }
  }
}

declare type CertificateData = {
    certificate: pruntime_rpc.ICertificate;
    pubkey: Uint8Array;
    secret: Uint8Array;
};
interface CertificateBaseParams {
    api: ApiPromise;
    signatureType?: pruntime_rpc.SignatureType;
}
interface CertificateParamsWithSigner extends CertificateBaseParams {
    signer: Signer | Signer$1;
    account: InjectedAccountWithMeta;
}
interface CertificateParamsWithPair extends CertificateBaseParams {
    pair: KeyringPair;
}
declare type CertificateParams = CertificateParamsWithSigner | CertificateParamsWithPair;
declare const signCertificate: (params: CertificateParams) => Promise<CertificateData>;

declare type QueryFn = (encodedQuery: string, certificateData: CertificateData) => Promise<string>;
declare type SidevmQuery = (bytes: Bytes, certificateData: CertificateData) => Promise<string>;
declare type CommandFn = (params: {
    contractId: string;
    payload: string;
    deposit: BN;
}) => SubmittableExtrinsic<"promise">;
interface PhalaInstance {
    query: QueryFn;
    command: CommandFn;
}
interface ContractExecResultWeightV2 extends Codec {
    gasConsumedV2?: {
        refTime: Compact<u64>;
        proofSize: Compact<u64>;
    };
    gasConsumed?: u64;
    gasRequiredV2?: {
        refTime: Compact<u64>;
        proofSize: Compact<u64>;
    };
    gasRequired?: u64;
}
interface CreateFnOptions {
    api: ApiPromise;
    baseURL: string;
    contractId: string;
    remotePubkey?: string;
    autoDeposit?: boolean;
}
interface CreateFnResult {
    api: ApiPromise;
    sidevmQuery: SidevmQuery;
    instantiate: SidevmQuery;
}
interface ContractInfo {
    cluster: string;
    codeIndex: {
        wasmCode: string;
    };
    deployer: AccountId;
    pubkey: string;
}
interface ClusterInfo {
    owner: AccountId;
    permission: "Public" | string;
    systemContract?: string;
    workers: string[];
    gasPrice: BN;
}
declare const createPruntimeApi: (baseURL: string) => pruntime_rpc.PhactoryAPI;
declare function create({ api, baseURL, contractId, remotePubkey, autoDeposit, }: CreateFnOptions): Promise<CreateFnResult>;

export { CertificateData, ClusterInfo, CommandFn, ContractExecResultWeightV2, ContractInfo, CreateFnOptions, CreateFnResult, PhalaInstance, QueryFn, SidevmQuery, create, createPruntimeApi, randomHex, signCertificate, types };

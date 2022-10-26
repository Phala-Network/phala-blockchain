/*eslint-disable block-scoped-var, id-length, no-control-regex, no-magic-numbers, no-prototype-builtins, no-redeclare, no-shadow, no-var, sort-vars*/
"use strict";

var $protobuf = require("protobufjs/minimal");

// Common aliases
var $Reader = $protobuf.Reader, $Writer = $protobuf.Writer, $util = $protobuf.util;

// Exported root namespace
var $root = $protobuf.roots["default"] || ($protobuf.roots["default"] = {});

$root.pruntime_rpc = (function() {

    /**
     * Namespace pruntime_rpc.
     * @exports pruntime_rpc
     * @namespace
     */
    var pruntime_rpc = {};

    pruntime_rpc.PhactoryAPI = (function() {

        /**
         * Constructs a new PhactoryAPI service.
         * @memberof pruntime_rpc
         * @classdesc Represents a PhactoryAPI
         * @extends $protobuf.rpc.Service
         * @constructor
         * @param {$protobuf.RPCImpl} rpcImpl RPC implementation
         * @param {boolean} [requestDelimited=false] Whether requests are length-delimited
         * @param {boolean} [responseDelimited=false] Whether responses are length-delimited
         */
        function PhactoryAPI(rpcImpl, requestDelimited, responseDelimited) {
            $protobuf.rpc.Service.call(this, rpcImpl, requestDelimited, responseDelimited);
        }

        (PhactoryAPI.prototype = Object.create($protobuf.rpc.Service.prototype)).constructor = PhactoryAPI;

        /**
         * Creates new PhactoryAPI service using the specified rpc implementation.
         * @function create
         * @memberof pruntime_rpc.PhactoryAPI
         * @static
         * @param {$protobuf.RPCImpl} rpcImpl RPC implementation
         * @param {boolean} [requestDelimited=false] Whether requests are length-delimited
         * @param {boolean} [responseDelimited=false] Whether responses are length-delimited
         * @returns {PhactoryAPI} RPC service. Useful where requests and/or responses are streamed.
         */
        PhactoryAPI.create = function create(rpcImpl, requestDelimited, responseDelimited) {
            return new this(rpcImpl, requestDelimited, responseDelimited);
        };

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#getInfo}.
         * @memberof pruntime_rpc.PhactoryAPI
         * @typedef GetInfoCallback
         * @type {function}
         * @param {Error|null} error Error, if any
         * @param {pruntime_rpc.PhactoryInfo} [response] PhactoryInfo
         */

        /**
         * Calls GetInfo.
         * @function getInfo
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {google.protobuf.IEmpty} request Empty message or plain object
         * @param {pruntime_rpc.PhactoryAPI.GetInfoCallback} callback Node-style callback called with the error, if any, and PhactoryInfo
         * @returns {undefined}
         * @variation 1
         */
        Object.defineProperty(PhactoryAPI.prototype.getInfo = function getInfo(request, callback) {
            return this.rpcCall(getInfo, $root.google.protobuf.Empty, $root.pruntime_rpc.PhactoryInfo, request, callback);
        }, "name", { value: "GetInfo" });

        /**
         * Calls GetInfo.
         * @function getInfo
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {google.protobuf.IEmpty} request Empty message or plain object
         * @returns {Promise<pruntime_rpc.PhactoryInfo>} Promise
         * @variation 2
         */

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#syncHeader}.
         * @memberof pruntime_rpc.PhactoryAPI
         * @typedef SyncHeaderCallback
         * @type {function}
         * @param {Error|null} error Error, if any
         * @param {pruntime_rpc.SyncedTo} [response] SyncedTo
         */

        /**
         * Calls SyncHeader.
         * @function syncHeader
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {pruntime_rpc.IHeadersToSync} request HeadersToSync message or plain object
         * @param {pruntime_rpc.PhactoryAPI.SyncHeaderCallback} callback Node-style callback called with the error, if any, and SyncedTo
         * @returns {undefined}
         * @variation 1
         */
        Object.defineProperty(PhactoryAPI.prototype.syncHeader = function syncHeader(request, callback) {
            return this.rpcCall(syncHeader, $root.pruntime_rpc.HeadersToSync, $root.pruntime_rpc.SyncedTo, request, callback);
        }, "name", { value: "SyncHeader" });

        /**
         * Calls SyncHeader.
         * @function syncHeader
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {pruntime_rpc.IHeadersToSync} request HeadersToSync message or plain object
         * @returns {Promise<pruntime_rpc.SyncedTo>} Promise
         * @variation 2
         */

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#syncParaHeader}.
         * @memberof pruntime_rpc.PhactoryAPI
         * @typedef SyncParaHeaderCallback
         * @type {function}
         * @param {Error|null} error Error, if any
         * @param {pruntime_rpc.SyncedTo} [response] SyncedTo
         */

        /**
         * Calls SyncParaHeader.
         * @function syncParaHeader
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {pruntime_rpc.IParaHeadersToSync} request ParaHeadersToSync message or plain object
         * @param {pruntime_rpc.PhactoryAPI.SyncParaHeaderCallback} callback Node-style callback called with the error, if any, and SyncedTo
         * @returns {undefined}
         * @variation 1
         */
        Object.defineProperty(PhactoryAPI.prototype.syncParaHeader = function syncParaHeader(request, callback) {
            return this.rpcCall(syncParaHeader, $root.pruntime_rpc.ParaHeadersToSync, $root.pruntime_rpc.SyncedTo, request, callback);
        }, "name", { value: "SyncParaHeader" });

        /**
         * Calls SyncParaHeader.
         * @function syncParaHeader
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {pruntime_rpc.IParaHeadersToSync} request ParaHeadersToSync message or plain object
         * @returns {Promise<pruntime_rpc.SyncedTo>} Promise
         * @variation 2
         */

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#syncCombinedHeaders}.
         * @memberof pruntime_rpc.PhactoryAPI
         * @typedef SyncCombinedHeadersCallback
         * @type {function}
         * @param {Error|null} error Error, if any
         * @param {pruntime_rpc.HeadersSyncedTo} [response] HeadersSyncedTo
         */

        /**
         * Calls SyncCombinedHeaders.
         * @function syncCombinedHeaders
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {pruntime_rpc.ICombinedHeadersToSync} request CombinedHeadersToSync message or plain object
         * @param {pruntime_rpc.PhactoryAPI.SyncCombinedHeadersCallback} callback Node-style callback called with the error, if any, and HeadersSyncedTo
         * @returns {undefined}
         * @variation 1
         */
        Object.defineProperty(PhactoryAPI.prototype.syncCombinedHeaders = function syncCombinedHeaders(request, callback) {
            return this.rpcCall(syncCombinedHeaders, $root.pruntime_rpc.CombinedHeadersToSync, $root.pruntime_rpc.HeadersSyncedTo, request, callback);
        }, "name", { value: "SyncCombinedHeaders" });

        /**
         * Calls SyncCombinedHeaders.
         * @function syncCombinedHeaders
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {pruntime_rpc.ICombinedHeadersToSync} request CombinedHeadersToSync message or plain object
         * @returns {Promise<pruntime_rpc.HeadersSyncedTo>} Promise
         * @variation 2
         */

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#dispatchBlocks}.
         * @memberof pruntime_rpc.PhactoryAPI
         * @typedef DispatchBlocksCallback
         * @type {function}
         * @param {Error|null} error Error, if any
         * @param {pruntime_rpc.SyncedTo} [response] SyncedTo
         */

        /**
         * Calls DispatchBlocks.
         * @function dispatchBlocks
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {pruntime_rpc.IBlocks} request Blocks message or plain object
         * @param {pruntime_rpc.PhactoryAPI.DispatchBlocksCallback} callback Node-style callback called with the error, if any, and SyncedTo
         * @returns {undefined}
         * @variation 1
         */
        Object.defineProperty(PhactoryAPI.prototype.dispatchBlocks = function dispatchBlocks(request, callback) {
            return this.rpcCall(dispatchBlocks, $root.pruntime_rpc.Blocks, $root.pruntime_rpc.SyncedTo, request, callback);
        }, "name", { value: "DispatchBlocks" });

        /**
         * Calls DispatchBlocks.
         * @function dispatchBlocks
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {pruntime_rpc.IBlocks} request Blocks message or plain object
         * @returns {Promise<pruntime_rpc.SyncedTo>} Promise
         * @variation 2
         */

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#initRuntime}.
         * @memberof pruntime_rpc.PhactoryAPI
         * @typedef InitRuntimeCallback
         * @type {function}
         * @param {Error|null} error Error, if any
         * @param {pruntime_rpc.InitRuntimeResponse} [response] InitRuntimeResponse
         */

        /**
         * Calls InitRuntime.
         * @function initRuntime
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {pruntime_rpc.IInitRuntimeRequest} request InitRuntimeRequest message or plain object
         * @param {pruntime_rpc.PhactoryAPI.InitRuntimeCallback} callback Node-style callback called with the error, if any, and InitRuntimeResponse
         * @returns {undefined}
         * @variation 1
         */
        Object.defineProperty(PhactoryAPI.prototype.initRuntime = function initRuntime(request, callback) {
            return this.rpcCall(initRuntime, $root.pruntime_rpc.InitRuntimeRequest, $root.pruntime_rpc.InitRuntimeResponse, request, callback);
        }, "name", { value: "InitRuntime" });

        /**
         * Calls InitRuntime.
         * @function initRuntime
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {pruntime_rpc.IInitRuntimeRequest} request InitRuntimeRequest message or plain object
         * @returns {Promise<pruntime_rpc.InitRuntimeResponse>} Promise
         * @variation 2
         */

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#getRuntimeInfo}.
         * @memberof pruntime_rpc.PhactoryAPI
         * @typedef GetRuntimeInfoCallback
         * @type {function}
         * @param {Error|null} error Error, if any
         * @param {pruntime_rpc.InitRuntimeResponse} [response] InitRuntimeResponse
         */

        /**
         * Calls GetRuntimeInfo.
         * @function getRuntimeInfo
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {pruntime_rpc.IGetRuntimeInfoRequest} request GetRuntimeInfoRequest message or plain object
         * @param {pruntime_rpc.PhactoryAPI.GetRuntimeInfoCallback} callback Node-style callback called with the error, if any, and InitRuntimeResponse
         * @returns {undefined}
         * @variation 1
         */
        Object.defineProperty(PhactoryAPI.prototype.getRuntimeInfo = function getRuntimeInfo(request, callback) {
            return this.rpcCall(getRuntimeInfo, $root.pruntime_rpc.GetRuntimeInfoRequest, $root.pruntime_rpc.InitRuntimeResponse, request, callback);
        }, "name", { value: "GetRuntimeInfo" });

        /**
         * Calls GetRuntimeInfo.
         * @function getRuntimeInfo
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {pruntime_rpc.IGetRuntimeInfoRequest} request GetRuntimeInfoRequest message or plain object
         * @returns {Promise<pruntime_rpc.InitRuntimeResponse>} Promise
         * @variation 2
         */

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#getEgressMessages}.
         * @memberof pruntime_rpc.PhactoryAPI
         * @typedef GetEgressMessagesCallback
         * @type {function}
         * @param {Error|null} error Error, if any
         * @param {pruntime_rpc.GetEgressMessagesResponse} [response] GetEgressMessagesResponse
         */

        /**
         * Calls GetEgressMessages.
         * @function getEgressMessages
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {google.protobuf.IEmpty} request Empty message or plain object
         * @param {pruntime_rpc.PhactoryAPI.GetEgressMessagesCallback} callback Node-style callback called with the error, if any, and GetEgressMessagesResponse
         * @returns {undefined}
         * @variation 1
         */
        Object.defineProperty(PhactoryAPI.prototype.getEgressMessages = function getEgressMessages(request, callback) {
            return this.rpcCall(getEgressMessages, $root.google.protobuf.Empty, $root.pruntime_rpc.GetEgressMessagesResponse, request, callback);
        }, "name", { value: "GetEgressMessages" });

        /**
         * Calls GetEgressMessages.
         * @function getEgressMessages
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {google.protobuf.IEmpty} request Empty message or plain object
         * @returns {Promise<pruntime_rpc.GetEgressMessagesResponse>} Promise
         * @variation 2
         */

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#contractQuery}.
         * @memberof pruntime_rpc.PhactoryAPI
         * @typedef ContractQueryCallback
         * @type {function}
         * @param {Error|null} error Error, if any
         * @param {pruntime_rpc.ContractQueryResponse} [response] ContractQueryResponse
         */

        /**
         * Calls ContractQuery.
         * @function contractQuery
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {pruntime_rpc.IContractQueryRequest} request ContractQueryRequest message or plain object
         * @param {pruntime_rpc.PhactoryAPI.ContractQueryCallback} callback Node-style callback called with the error, if any, and ContractQueryResponse
         * @returns {undefined}
         * @variation 1
         */
        Object.defineProperty(PhactoryAPI.prototype.contractQuery = function contractQuery(request, callback) {
            return this.rpcCall(contractQuery, $root.pruntime_rpc.ContractQueryRequest, $root.pruntime_rpc.ContractQueryResponse, request, callback);
        }, "name", { value: "ContractQuery" });

        /**
         * Calls ContractQuery.
         * @function contractQuery
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {pruntime_rpc.IContractQueryRequest} request ContractQueryRequest message or plain object
         * @returns {Promise<pruntime_rpc.ContractQueryResponse>} Promise
         * @variation 2
         */

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#getWorkerState}.
         * @memberof pruntime_rpc.PhactoryAPI
         * @typedef GetWorkerStateCallback
         * @type {function}
         * @param {Error|null} error Error, if any
         * @param {pruntime_rpc.WorkerState} [response] WorkerState
         */

        /**
         * Calls GetWorkerState.
         * @function getWorkerState
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {pruntime_rpc.IGetWorkerStateRequest} request GetWorkerStateRequest message or plain object
         * @param {pruntime_rpc.PhactoryAPI.GetWorkerStateCallback} callback Node-style callback called with the error, if any, and WorkerState
         * @returns {undefined}
         * @variation 1
         */
        Object.defineProperty(PhactoryAPI.prototype.getWorkerState = function getWorkerState(request, callback) {
            return this.rpcCall(getWorkerState, $root.pruntime_rpc.GetWorkerStateRequest, $root.pruntime_rpc.WorkerState, request, callback);
        }, "name", { value: "GetWorkerState" });

        /**
         * Calls GetWorkerState.
         * @function getWorkerState
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {pruntime_rpc.IGetWorkerStateRequest} request GetWorkerStateRequest message or plain object
         * @returns {Promise<pruntime_rpc.WorkerState>} Promise
         * @variation 2
         */

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#addEndpoint}.
         * @memberof pruntime_rpc.PhactoryAPI
         * @typedef AddEndpointCallback
         * @type {function}
         * @param {Error|null} error Error, if any
         * @param {pruntime_rpc.GetEndpointResponse} [response] GetEndpointResponse
         */

        /**
         * Calls AddEndpoint.
         * @function addEndpoint
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {pruntime_rpc.IAddEndpointRequest} request AddEndpointRequest message or plain object
         * @param {pruntime_rpc.PhactoryAPI.AddEndpointCallback} callback Node-style callback called with the error, if any, and GetEndpointResponse
         * @returns {undefined}
         * @variation 1
         */
        Object.defineProperty(PhactoryAPI.prototype.addEndpoint = function addEndpoint(request, callback) {
            return this.rpcCall(addEndpoint, $root.pruntime_rpc.AddEndpointRequest, $root.pruntime_rpc.GetEndpointResponse, request, callback);
        }, "name", { value: "AddEndpoint" });

        /**
         * Calls AddEndpoint.
         * @function addEndpoint
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {pruntime_rpc.IAddEndpointRequest} request AddEndpointRequest message or plain object
         * @returns {Promise<pruntime_rpc.GetEndpointResponse>} Promise
         * @variation 2
         */

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#refreshEndpointSigningTime}.
         * @memberof pruntime_rpc.PhactoryAPI
         * @typedef RefreshEndpointSigningTimeCallback
         * @type {function}
         * @param {Error|null} error Error, if any
         * @param {pruntime_rpc.GetEndpointResponse} [response] GetEndpointResponse
         */

        /**
         * Calls RefreshEndpointSigningTime.
         * @function refreshEndpointSigningTime
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {google.protobuf.IEmpty} request Empty message or plain object
         * @param {pruntime_rpc.PhactoryAPI.RefreshEndpointSigningTimeCallback} callback Node-style callback called with the error, if any, and GetEndpointResponse
         * @returns {undefined}
         * @variation 1
         */
        Object.defineProperty(PhactoryAPI.prototype.refreshEndpointSigningTime = function refreshEndpointSigningTime(request, callback) {
            return this.rpcCall(refreshEndpointSigningTime, $root.google.protobuf.Empty, $root.pruntime_rpc.GetEndpointResponse, request, callback);
        }, "name", { value: "RefreshEndpointSigningTime" });

        /**
         * Calls RefreshEndpointSigningTime.
         * @function refreshEndpointSigningTime
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {google.protobuf.IEmpty} request Empty message or plain object
         * @returns {Promise<pruntime_rpc.GetEndpointResponse>} Promise
         * @variation 2
         */

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#getEndpointInfo}.
         * @memberof pruntime_rpc.PhactoryAPI
         * @typedef GetEndpointInfoCallback
         * @type {function}
         * @param {Error|null} error Error, if any
         * @param {pruntime_rpc.GetEndpointResponse} [response] GetEndpointResponse
         */

        /**
         * Calls GetEndpointInfo.
         * @function getEndpointInfo
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {google.protobuf.IEmpty} request Empty message or plain object
         * @param {pruntime_rpc.PhactoryAPI.GetEndpointInfoCallback} callback Node-style callback called with the error, if any, and GetEndpointResponse
         * @returns {undefined}
         * @variation 1
         */
        Object.defineProperty(PhactoryAPI.prototype.getEndpointInfo = function getEndpointInfo(request, callback) {
            return this.rpcCall(getEndpointInfo, $root.google.protobuf.Empty, $root.pruntime_rpc.GetEndpointResponse, request, callback);
        }, "name", { value: "GetEndpointInfo" });

        /**
         * Calls GetEndpointInfo.
         * @function getEndpointInfo
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {google.protobuf.IEmpty} request Empty message or plain object
         * @returns {Promise<pruntime_rpc.GetEndpointResponse>} Promise
         * @variation 2
         */

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#signEndpointInfo}.
         * @memberof pruntime_rpc.PhactoryAPI
         * @typedef SignEndpointInfoCallback
         * @type {function}
         * @param {Error|null} error Error, if any
         * @param {pruntime_rpc.GetEndpointResponse} [response] GetEndpointResponse
         */

        /**
         * Calls SignEndpointInfo.
         * @function signEndpointInfo
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {pruntime_rpc.ISignEndpointsRequest} request SignEndpointsRequest message or plain object
         * @param {pruntime_rpc.PhactoryAPI.SignEndpointInfoCallback} callback Node-style callback called with the error, if any, and GetEndpointResponse
         * @returns {undefined}
         * @variation 1
         */
        Object.defineProperty(PhactoryAPI.prototype.signEndpointInfo = function signEndpointInfo(request, callback) {
            return this.rpcCall(signEndpointInfo, $root.pruntime_rpc.SignEndpointsRequest, $root.pruntime_rpc.GetEndpointResponse, request, callback);
        }, "name", { value: "SignEndpointInfo" });

        /**
         * Calls SignEndpointInfo.
         * @function signEndpointInfo
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {pruntime_rpc.ISignEndpointsRequest} request SignEndpointsRequest message or plain object
         * @returns {Promise<pruntime_rpc.GetEndpointResponse>} Promise
         * @variation 2
         */

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#derivePhalaI2pKey}.
         * @memberof pruntime_rpc.PhactoryAPI
         * @typedef DerivePhalaI2pKeyCallback
         * @type {function}
         * @param {Error|null} error Error, if any
         * @param {pruntime_rpc.DerivePhalaI2pKeyResponse} [response] DerivePhalaI2pKeyResponse
         */

        /**
         * Calls DerivePhalaI2pKey.
         * @function derivePhalaI2pKey
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {google.protobuf.IEmpty} request Empty message or plain object
         * @param {pruntime_rpc.PhactoryAPI.DerivePhalaI2pKeyCallback} callback Node-style callback called with the error, if any, and DerivePhalaI2pKeyResponse
         * @returns {undefined}
         * @variation 1
         */
        Object.defineProperty(PhactoryAPI.prototype.derivePhalaI2pKey = function derivePhalaI2pKey(request, callback) {
            return this.rpcCall(derivePhalaI2pKey, $root.google.protobuf.Empty, $root.pruntime_rpc.DerivePhalaI2pKeyResponse, request, callback);
        }, "name", { value: "DerivePhalaI2pKey" });

        /**
         * Calls DerivePhalaI2pKey.
         * @function derivePhalaI2pKey
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {google.protobuf.IEmpty} request Empty message or plain object
         * @returns {Promise<pruntime_rpc.DerivePhalaI2pKeyResponse>} Promise
         * @variation 2
         */

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#echo}.
         * @memberof pruntime_rpc.PhactoryAPI
         * @typedef EchoCallback
         * @type {function}
         * @param {Error|null} error Error, if any
         * @param {pruntime_rpc.EchoMessage} [response] EchoMessage
         */

        /**
         * Calls Echo.
         * @function echo
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {pruntime_rpc.IEchoMessage} request EchoMessage message or plain object
         * @param {pruntime_rpc.PhactoryAPI.EchoCallback} callback Node-style callback called with the error, if any, and EchoMessage
         * @returns {undefined}
         * @variation 1
         */
        Object.defineProperty(PhactoryAPI.prototype.echo = function echo(request, callback) {
            return this.rpcCall(echo, $root.pruntime_rpc.EchoMessage, $root.pruntime_rpc.EchoMessage, request, callback);
        }, "name", { value: "Echo" });

        /**
         * Calls Echo.
         * @function echo
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {pruntime_rpc.IEchoMessage} request EchoMessage message or plain object
         * @returns {Promise<pruntime_rpc.EchoMessage>} Promise
         * @variation 2
         */

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#handoverCreateChallenge}.
         * @memberof pruntime_rpc.PhactoryAPI
         * @typedef HandoverCreateChallengeCallback
         * @type {function}
         * @param {Error|null} error Error, if any
         * @param {pruntime_rpc.HandoverChallenge} [response] HandoverChallenge
         */

        /**
         * Calls HandoverCreateChallenge.
         * @function handoverCreateChallenge
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {google.protobuf.IEmpty} request Empty message or plain object
         * @param {pruntime_rpc.PhactoryAPI.HandoverCreateChallengeCallback} callback Node-style callback called with the error, if any, and HandoverChallenge
         * @returns {undefined}
         * @variation 1
         */
        Object.defineProperty(PhactoryAPI.prototype.handoverCreateChallenge = function handoverCreateChallenge(request, callback) {
            return this.rpcCall(handoverCreateChallenge, $root.google.protobuf.Empty, $root.pruntime_rpc.HandoverChallenge, request, callback);
        }, "name", { value: "HandoverCreateChallenge" });

        /**
         * Calls HandoverCreateChallenge.
         * @function handoverCreateChallenge
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {google.protobuf.IEmpty} request Empty message or plain object
         * @returns {Promise<pruntime_rpc.HandoverChallenge>} Promise
         * @variation 2
         */

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#handoverStart}.
         * @memberof pruntime_rpc.PhactoryAPI
         * @typedef HandoverStartCallback
         * @type {function}
         * @param {Error|null} error Error, if any
         * @param {pruntime_rpc.HandoverWorkerKey} [response] HandoverWorkerKey
         */

        /**
         * Calls HandoverStart.
         * @function handoverStart
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {pruntime_rpc.IHandoverChallengeResponse} request HandoverChallengeResponse message or plain object
         * @param {pruntime_rpc.PhactoryAPI.HandoverStartCallback} callback Node-style callback called with the error, if any, and HandoverWorkerKey
         * @returns {undefined}
         * @variation 1
         */
        Object.defineProperty(PhactoryAPI.prototype.handoverStart = function handoverStart(request, callback) {
            return this.rpcCall(handoverStart, $root.pruntime_rpc.HandoverChallengeResponse, $root.pruntime_rpc.HandoverWorkerKey, request, callback);
        }, "name", { value: "HandoverStart" });

        /**
         * Calls HandoverStart.
         * @function handoverStart
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {pruntime_rpc.IHandoverChallengeResponse} request HandoverChallengeResponse message or plain object
         * @returns {Promise<pruntime_rpc.HandoverWorkerKey>} Promise
         * @variation 2
         */

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#handoverAcceptChallenge}.
         * @memberof pruntime_rpc.PhactoryAPI
         * @typedef HandoverAcceptChallengeCallback
         * @type {function}
         * @param {Error|null} error Error, if any
         * @param {pruntime_rpc.HandoverChallengeResponse} [response] HandoverChallengeResponse
         */

        /**
         * Calls HandoverAcceptChallenge.
         * @function handoverAcceptChallenge
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {pruntime_rpc.IHandoverChallenge} request HandoverChallenge message or plain object
         * @param {pruntime_rpc.PhactoryAPI.HandoverAcceptChallengeCallback} callback Node-style callback called with the error, if any, and HandoverChallengeResponse
         * @returns {undefined}
         * @variation 1
         */
        Object.defineProperty(PhactoryAPI.prototype.handoverAcceptChallenge = function handoverAcceptChallenge(request, callback) {
            return this.rpcCall(handoverAcceptChallenge, $root.pruntime_rpc.HandoverChallenge, $root.pruntime_rpc.HandoverChallengeResponse, request, callback);
        }, "name", { value: "HandoverAcceptChallenge" });

        /**
         * Calls HandoverAcceptChallenge.
         * @function handoverAcceptChallenge
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {pruntime_rpc.IHandoverChallenge} request HandoverChallenge message or plain object
         * @returns {Promise<pruntime_rpc.HandoverChallengeResponse>} Promise
         * @variation 2
         */

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#handoverReceive}.
         * @memberof pruntime_rpc.PhactoryAPI
         * @typedef HandoverReceiveCallback
         * @type {function}
         * @param {Error|null} error Error, if any
         * @param {google.protobuf.Empty} [response] Empty
         */

        /**
         * Calls HandoverReceive.
         * @function handoverReceive
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {pruntime_rpc.IHandoverWorkerKey} request HandoverWorkerKey message or plain object
         * @param {pruntime_rpc.PhactoryAPI.HandoverReceiveCallback} callback Node-style callback called with the error, if any, and Empty
         * @returns {undefined}
         * @variation 1
         */
        Object.defineProperty(PhactoryAPI.prototype.handoverReceive = function handoverReceive(request, callback) {
            return this.rpcCall(handoverReceive, $root.pruntime_rpc.HandoverWorkerKey, $root.google.protobuf.Empty, request, callback);
        }, "name", { value: "HandoverReceive" });

        /**
         * Calls HandoverReceive.
         * @function handoverReceive
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {pruntime_rpc.IHandoverWorkerKey} request HandoverWorkerKey message or plain object
         * @returns {Promise<google.protobuf.Empty>} Promise
         * @variation 2
         */

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#configNetwork}.
         * @memberof pruntime_rpc.PhactoryAPI
         * @typedef ConfigNetworkCallback
         * @type {function}
         * @param {Error|null} error Error, if any
         * @param {google.protobuf.Empty} [response] Empty
         */

        /**
         * Calls ConfigNetwork.
         * @function configNetwork
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {pruntime_rpc.INetworkConfig} request NetworkConfig message or plain object
         * @param {pruntime_rpc.PhactoryAPI.ConfigNetworkCallback} callback Node-style callback called with the error, if any, and Empty
         * @returns {undefined}
         * @variation 1
         */
        Object.defineProperty(PhactoryAPI.prototype.configNetwork = function configNetwork(request, callback) {
            return this.rpcCall(configNetwork, $root.pruntime_rpc.NetworkConfig, $root.google.protobuf.Empty, request, callback);
        }, "name", { value: "ConfigNetwork" });

        /**
         * Calls ConfigNetwork.
         * @function configNetwork
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {pruntime_rpc.INetworkConfig} request NetworkConfig message or plain object
         * @returns {Promise<google.protobuf.Empty>} Promise
         * @variation 2
         */

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#httpFetch}.
         * @memberof pruntime_rpc.PhactoryAPI
         * @typedef HttpFetchCallback
         * @type {function}
         * @param {Error|null} error Error, if any
         * @param {pruntime_rpc.HttpResponse} [response] HttpResponse
         */

        /**
         * Calls HttpFetch.
         * @function httpFetch
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {pruntime_rpc.IHttpRequest} request HttpRequest message or plain object
         * @param {pruntime_rpc.PhactoryAPI.HttpFetchCallback} callback Node-style callback called with the error, if any, and HttpResponse
         * @returns {undefined}
         * @variation 1
         */
        Object.defineProperty(PhactoryAPI.prototype.httpFetch = function httpFetch(request, callback) {
            return this.rpcCall(httpFetch, $root.pruntime_rpc.HttpRequest, $root.pruntime_rpc.HttpResponse, request, callback);
        }, "name", { value: "HttpFetch" });

        /**
         * Calls HttpFetch.
         * @function httpFetch
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {pruntime_rpc.IHttpRequest} request HttpRequest message or plain object
         * @returns {Promise<pruntime_rpc.HttpResponse>} Promise
         * @variation 2
         */

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#getContractInfo}.
         * @memberof pruntime_rpc.PhactoryAPI
         * @typedef GetContractInfoCallback
         * @type {function}
         * @param {Error|null} error Error, if any
         * @param {pruntime_rpc.GetContractInfoResponse} [response] GetContractInfoResponse
         */

        /**
         * Calls GetContractInfo.
         * @function getContractInfo
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {pruntime_rpc.IGetContractInfoRequest} request GetContractInfoRequest message or plain object
         * @param {pruntime_rpc.PhactoryAPI.GetContractInfoCallback} callback Node-style callback called with the error, if any, and GetContractInfoResponse
         * @returns {undefined}
         * @variation 1
         */
        Object.defineProperty(PhactoryAPI.prototype.getContractInfo = function getContractInfo(request, callback) {
            return this.rpcCall(getContractInfo, $root.pruntime_rpc.GetContractInfoRequest, $root.pruntime_rpc.GetContractInfoResponse, request, callback);
        }, "name", { value: "GetContractInfo" });

        /**
         * Calls GetContractInfo.
         * @function getContractInfo
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {pruntime_rpc.IGetContractInfoRequest} request GetContractInfoRequest message or plain object
         * @returns {Promise<pruntime_rpc.GetContractInfoResponse>} Promise
         * @variation 2
         */

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#getClusterInfo}.
         * @memberof pruntime_rpc.PhactoryAPI
         * @typedef GetClusterInfoCallback
         * @type {function}
         * @param {Error|null} error Error, if any
         * @param {pruntime_rpc.GetClusterInfoResponse} [response] GetClusterInfoResponse
         */

        /**
         * Calls GetClusterInfo.
         * @function getClusterInfo
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {google.protobuf.IEmpty} request Empty message or plain object
         * @param {pruntime_rpc.PhactoryAPI.GetClusterInfoCallback} callback Node-style callback called with the error, if any, and GetClusterInfoResponse
         * @returns {undefined}
         * @variation 1
         */
        Object.defineProperty(PhactoryAPI.prototype.getClusterInfo = function getClusterInfo(request, callback) {
            return this.rpcCall(getClusterInfo, $root.google.protobuf.Empty, $root.pruntime_rpc.GetClusterInfoResponse, request, callback);
        }, "name", { value: "GetClusterInfo" });

        /**
         * Calls GetClusterInfo.
         * @function getClusterInfo
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {google.protobuf.IEmpty} request Empty message or plain object
         * @returns {Promise<pruntime_rpc.GetClusterInfoResponse>} Promise
         * @variation 2
         */

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#uploadSidevmCode}.
         * @memberof pruntime_rpc.PhactoryAPI
         * @typedef UploadSidevmCodeCallback
         * @type {function}
         * @param {Error|null} error Error, if any
         * @param {google.protobuf.Empty} [response] Empty
         */

        /**
         * Calls UploadSidevmCode.
         * @function uploadSidevmCode
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {pruntime_rpc.ISidevmCode} request SidevmCode message or plain object
         * @param {pruntime_rpc.PhactoryAPI.UploadSidevmCodeCallback} callback Node-style callback called with the error, if any, and Empty
         * @returns {undefined}
         * @variation 1
         */
        Object.defineProperty(PhactoryAPI.prototype.uploadSidevmCode = function uploadSidevmCode(request, callback) {
            return this.rpcCall(uploadSidevmCode, $root.pruntime_rpc.SidevmCode, $root.google.protobuf.Empty, request, callback);
        }, "name", { value: "UploadSidevmCode" });

        /**
         * Calls UploadSidevmCode.
         * @function uploadSidevmCode
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {pruntime_rpc.ISidevmCode} request SidevmCode message or plain object
         * @returns {Promise<google.protobuf.Empty>} Promise
         * @variation 2
         */

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#calculateContractId}.
         * @memberof pruntime_rpc.PhactoryAPI
         * @typedef CalculateContractIdCallback
         * @type {function}
         * @param {Error|null} error Error, if any
         * @param {pruntime_rpc.ContractId} [response] ContractId
         */

        /**
         * Calls CalculateContractId.
         * @function calculateContractId
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {pruntime_rpc.IContractParameters} request ContractParameters message or plain object
         * @param {pruntime_rpc.PhactoryAPI.CalculateContractIdCallback} callback Node-style callback called with the error, if any, and ContractId
         * @returns {undefined}
         * @variation 1
         */
        Object.defineProperty(PhactoryAPI.prototype.calculateContractId = function calculateContractId(request, callback) {
            return this.rpcCall(calculateContractId, $root.pruntime_rpc.ContractParameters, $root.pruntime_rpc.ContractId, request, callback);
        }, "name", { value: "CalculateContractId" });

        /**
         * Calls CalculateContractId.
         * @function calculateContractId
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {pruntime_rpc.IContractParameters} request ContractParameters message or plain object
         * @returns {Promise<pruntime_rpc.ContractId>} Promise
         * @variation 2
         */

        return PhactoryAPI;
    })();

    pruntime_rpc.PhactoryInfo = (function() {

        /**
         * Properties of a PhactoryInfo.
         * @memberof pruntime_rpc
         * @interface IPhactoryInfo
         * @property {boolean|null} [initialized] PhactoryInfo initialized
         * @property {boolean|null} [registered] PhactoryInfo registered
         * @property {string|null} [genesisBlockHash] PhactoryInfo genesisBlockHash
         * @property {string|null} [publicKey] PhactoryInfo publicKey
         * @property {string|null} [ecdhPublicKey] PhactoryInfo ecdhPublicKey
         * @property {number|null} [headernum] PhactoryInfo headernum
         * @property {number|null} [paraHeadernum] PhactoryInfo paraHeadernum
         * @property {number|null} [blocknum] PhactoryInfo blocknum
         * @property {string|null} [stateRoot] PhactoryInfo stateRoot
         * @property {boolean|null} [devMode] PhactoryInfo devMode
         * @property {number|Long|null} [pendingMessages] PhactoryInfo pendingMessages
         * @property {number|Long|null} [score] PhactoryInfo score
         * @property {pruntime_rpc.IGatekeeperStatus|null} [gatekeeper] PhactoryInfo gatekeeper
         * @property {string|null} [version] PhactoryInfo version
         * @property {string|null} [gitRevision] PhactoryInfo gitRevision
         * @property {pruntime_rpc.IMemoryUsage|null} [memoryUsage] PhactoryInfo memoryUsage
         * @property {boolean|null} [waitingForParaheaders] PhactoryInfo waitingForParaheaders
         * @property {pruntime_rpc.INetworkStatus|null} [networkStatus] PhactoryInfo networkStatus
         * @property {pruntime_rpc.ISystemInfo|null} [system] PhactoryInfo system
         */

        /**
         * Constructs a new PhactoryInfo.
         * @memberof pruntime_rpc
         * @classdesc Represents a PhactoryInfo.
         * @implements IPhactoryInfo
         * @constructor
         * @param {pruntime_rpc.IPhactoryInfo=} [properties] Properties to set
         */
        function PhactoryInfo(properties) {
            if (properties)
                for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * PhactoryInfo initialized.
         * @member {boolean} initialized
         * @memberof pruntime_rpc.PhactoryInfo
         * @instance
         */
        PhactoryInfo.prototype.initialized = false;

        /**
         * PhactoryInfo registered.
         * @member {boolean} registered
         * @memberof pruntime_rpc.PhactoryInfo
         * @instance
         */
        PhactoryInfo.prototype.registered = false;

        /**
         * PhactoryInfo genesisBlockHash.
         * @member {string|null|undefined} genesisBlockHash
         * @memberof pruntime_rpc.PhactoryInfo
         * @instance
         */
        PhactoryInfo.prototype.genesisBlockHash = null;

        /**
         * PhactoryInfo publicKey.
         * @member {string|null|undefined} publicKey
         * @memberof pruntime_rpc.PhactoryInfo
         * @instance
         */
        PhactoryInfo.prototype.publicKey = null;

        /**
         * PhactoryInfo ecdhPublicKey.
         * @member {string|null|undefined} ecdhPublicKey
         * @memberof pruntime_rpc.PhactoryInfo
         * @instance
         */
        PhactoryInfo.prototype.ecdhPublicKey = null;

        /**
         * PhactoryInfo headernum.
         * @member {number} headernum
         * @memberof pruntime_rpc.PhactoryInfo
         * @instance
         */
        PhactoryInfo.prototype.headernum = 0;

        /**
         * PhactoryInfo paraHeadernum.
         * @member {number} paraHeadernum
         * @memberof pruntime_rpc.PhactoryInfo
         * @instance
         */
        PhactoryInfo.prototype.paraHeadernum = 0;

        /**
         * PhactoryInfo blocknum.
         * @member {number} blocknum
         * @memberof pruntime_rpc.PhactoryInfo
         * @instance
         */
        PhactoryInfo.prototype.blocknum = 0;

        /**
         * PhactoryInfo stateRoot.
         * @member {string} stateRoot
         * @memberof pruntime_rpc.PhactoryInfo
         * @instance
         */
        PhactoryInfo.prototype.stateRoot = "";

        /**
         * PhactoryInfo devMode.
         * @member {boolean} devMode
         * @memberof pruntime_rpc.PhactoryInfo
         * @instance
         */
        PhactoryInfo.prototype.devMode = false;

        /**
         * PhactoryInfo pendingMessages.
         * @member {number|Long} pendingMessages
         * @memberof pruntime_rpc.PhactoryInfo
         * @instance
         */
        PhactoryInfo.prototype.pendingMessages = $util.Long ? $util.Long.fromBits(0,0,true) : 0;

        /**
         * PhactoryInfo score.
         * @member {number|Long} score
         * @memberof pruntime_rpc.PhactoryInfo
         * @instance
         */
        PhactoryInfo.prototype.score = $util.Long ? $util.Long.fromBits(0,0,true) : 0;

        /**
         * PhactoryInfo gatekeeper.
         * @member {pruntime_rpc.IGatekeeperStatus|null|undefined} gatekeeper
         * @memberof pruntime_rpc.PhactoryInfo
         * @instance
         */
        PhactoryInfo.prototype.gatekeeper = null;

        /**
         * PhactoryInfo version.
         * @member {string} version
         * @memberof pruntime_rpc.PhactoryInfo
         * @instance
         */
        PhactoryInfo.prototype.version = "";

        /**
         * PhactoryInfo gitRevision.
         * @member {string} gitRevision
         * @memberof pruntime_rpc.PhactoryInfo
         * @instance
         */
        PhactoryInfo.prototype.gitRevision = "";

        /**
         * PhactoryInfo memoryUsage.
         * @member {pruntime_rpc.IMemoryUsage|null|undefined} memoryUsage
         * @memberof pruntime_rpc.PhactoryInfo
         * @instance
         */
        PhactoryInfo.prototype.memoryUsage = null;

        /**
         * PhactoryInfo waitingForParaheaders.
         * @member {boolean} waitingForParaheaders
         * @memberof pruntime_rpc.PhactoryInfo
         * @instance
         */
        PhactoryInfo.prototype.waitingForParaheaders = false;

        /**
         * PhactoryInfo networkStatus.
         * @member {pruntime_rpc.INetworkStatus|null|undefined} networkStatus
         * @memberof pruntime_rpc.PhactoryInfo
         * @instance
         */
        PhactoryInfo.prototype.networkStatus = null;

        /**
         * PhactoryInfo system.
         * @member {pruntime_rpc.ISystemInfo|null|undefined} system
         * @memberof pruntime_rpc.PhactoryInfo
         * @instance
         */
        PhactoryInfo.prototype.system = null;

        // OneOf field names bound to virtual getters and setters
        var $oneOfFields;

        /**
         * PhactoryInfo _genesisBlockHash.
         * @member {"genesisBlockHash"|undefined} _genesisBlockHash
         * @memberof pruntime_rpc.PhactoryInfo
         * @instance
         */
        Object.defineProperty(PhactoryInfo.prototype, "_genesisBlockHash", {
            get: $util.oneOfGetter($oneOfFields = ["genesisBlockHash"]),
            set: $util.oneOfSetter($oneOfFields)
        });

        /**
         * PhactoryInfo _publicKey.
         * @member {"publicKey"|undefined} _publicKey
         * @memberof pruntime_rpc.PhactoryInfo
         * @instance
         */
        Object.defineProperty(PhactoryInfo.prototype, "_publicKey", {
            get: $util.oneOfGetter($oneOfFields = ["publicKey"]),
            set: $util.oneOfSetter($oneOfFields)
        });

        /**
         * PhactoryInfo _ecdhPublicKey.
         * @member {"ecdhPublicKey"|undefined} _ecdhPublicKey
         * @memberof pruntime_rpc.PhactoryInfo
         * @instance
         */
        Object.defineProperty(PhactoryInfo.prototype, "_ecdhPublicKey", {
            get: $util.oneOfGetter($oneOfFields = ["ecdhPublicKey"]),
            set: $util.oneOfSetter($oneOfFields)
        });

        /**
         * Creates a new PhactoryInfo instance using the specified properties.
         * @function create
         * @memberof pruntime_rpc.PhactoryInfo
         * @static
         * @param {pruntime_rpc.IPhactoryInfo=} [properties] Properties to set
         * @returns {pruntime_rpc.PhactoryInfo} PhactoryInfo instance
         */
        PhactoryInfo.create = function create(properties) {
            return new PhactoryInfo(properties);
        };

        /**
         * Encodes the specified PhactoryInfo message. Does not implicitly {@link pruntime_rpc.PhactoryInfo.verify|verify} messages.
         * @function encode
         * @memberof pruntime_rpc.PhactoryInfo
         * @static
         * @param {pruntime_rpc.IPhactoryInfo} message PhactoryInfo message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        PhactoryInfo.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.initialized != null && Object.hasOwnProperty.call(message, "initialized"))
                writer.uint32(/* id 1, wireType 0 =*/8).bool(message.initialized);
            if (message.registered != null && Object.hasOwnProperty.call(message, "registered"))
                writer.uint32(/* id 2, wireType 0 =*/16).bool(message.registered);
            if (message.genesisBlockHash != null && Object.hasOwnProperty.call(message, "genesisBlockHash"))
                writer.uint32(/* id 4, wireType 2 =*/34).string(message.genesisBlockHash);
            if (message.publicKey != null && Object.hasOwnProperty.call(message, "publicKey"))
                writer.uint32(/* id 5, wireType 2 =*/42).string(message.publicKey);
            if (message.ecdhPublicKey != null && Object.hasOwnProperty.call(message, "ecdhPublicKey"))
                writer.uint32(/* id 6, wireType 2 =*/50).string(message.ecdhPublicKey);
            if (message.headernum != null && Object.hasOwnProperty.call(message, "headernum"))
                writer.uint32(/* id 7, wireType 0 =*/56).uint32(message.headernum);
            if (message.paraHeadernum != null && Object.hasOwnProperty.call(message, "paraHeadernum"))
                writer.uint32(/* id 8, wireType 0 =*/64).uint32(message.paraHeadernum);
            if (message.blocknum != null && Object.hasOwnProperty.call(message, "blocknum"))
                writer.uint32(/* id 9, wireType 0 =*/72).uint32(message.blocknum);
            if (message.stateRoot != null && Object.hasOwnProperty.call(message, "stateRoot"))
                writer.uint32(/* id 10, wireType 2 =*/82).string(message.stateRoot);
            if (message.devMode != null && Object.hasOwnProperty.call(message, "devMode"))
                writer.uint32(/* id 11, wireType 0 =*/88).bool(message.devMode);
            if (message.pendingMessages != null && Object.hasOwnProperty.call(message, "pendingMessages"))
                writer.uint32(/* id 12, wireType 0 =*/96).uint64(message.pendingMessages);
            if (message.score != null && Object.hasOwnProperty.call(message, "score"))
                writer.uint32(/* id 13, wireType 0 =*/104).uint64(message.score);
            if (message.gatekeeper != null && Object.hasOwnProperty.call(message, "gatekeeper"))
                $root.pruntime_rpc.GatekeeperStatus.encode(message.gatekeeper, writer.uint32(/* id 14, wireType 2 =*/114).fork()).ldelim();
            if (message.version != null && Object.hasOwnProperty.call(message, "version"))
                writer.uint32(/* id 15, wireType 2 =*/122).string(message.version);
            if (message.gitRevision != null && Object.hasOwnProperty.call(message, "gitRevision"))
                writer.uint32(/* id 16, wireType 2 =*/130).string(message.gitRevision);
            if (message.memoryUsage != null && Object.hasOwnProperty.call(message, "memoryUsage"))
                $root.pruntime_rpc.MemoryUsage.encode(message.memoryUsage, writer.uint32(/* id 18, wireType 2 =*/146).fork()).ldelim();
            if (message.waitingForParaheaders != null && Object.hasOwnProperty.call(message, "waitingForParaheaders"))
                writer.uint32(/* id 21, wireType 0 =*/168).bool(message.waitingForParaheaders);
            if (message.networkStatus != null && Object.hasOwnProperty.call(message, "networkStatus"))
                $root.pruntime_rpc.NetworkStatus.encode(message.networkStatus, writer.uint32(/* id 22, wireType 2 =*/178).fork()).ldelim();
            if (message.system != null && Object.hasOwnProperty.call(message, "system"))
                $root.pruntime_rpc.SystemInfo.encode(message.system, writer.uint32(/* id 23, wireType 2 =*/186).fork()).ldelim();
            return writer;
        };

        /**
         * Encodes the specified PhactoryInfo message, length delimited. Does not implicitly {@link pruntime_rpc.PhactoryInfo.verify|verify} messages.
         * @function encodeDelimited
         * @memberof pruntime_rpc.PhactoryInfo
         * @static
         * @param {pruntime_rpc.IPhactoryInfo} message PhactoryInfo message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        PhactoryInfo.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes a PhactoryInfo message from the specified reader or buffer.
         * @function decode
         * @memberof pruntime_rpc.PhactoryInfo
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {pruntime_rpc.PhactoryInfo} PhactoryInfo
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        PhactoryInfo.decode = function decode(reader, length) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            var end = length === undefined ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.PhactoryInfo();
            while (reader.pos < end) {
                var tag = reader.uint32();
                switch (tag >>> 3) {
                case 1:
                    message.initialized = reader.bool();
                    break;
                case 2:
                    message.registered = reader.bool();
                    break;
                case 4:
                    message.genesisBlockHash = reader.string();
                    break;
                case 5:
                    message.publicKey = reader.string();
                    break;
                case 6:
                    message.ecdhPublicKey = reader.string();
                    break;
                case 7:
                    message.headernum = reader.uint32();
                    break;
                case 8:
                    message.paraHeadernum = reader.uint32();
                    break;
                case 9:
                    message.blocknum = reader.uint32();
                    break;
                case 10:
                    message.stateRoot = reader.string();
                    break;
                case 11:
                    message.devMode = reader.bool();
                    break;
                case 12:
                    message.pendingMessages = reader.uint64();
                    break;
                case 13:
                    message.score = reader.uint64();
                    break;
                case 14:
                    message.gatekeeper = $root.pruntime_rpc.GatekeeperStatus.decode(reader, reader.uint32());
                    break;
                case 15:
                    message.version = reader.string();
                    break;
                case 16:
                    message.gitRevision = reader.string();
                    break;
                case 18:
                    message.memoryUsage = $root.pruntime_rpc.MemoryUsage.decode(reader, reader.uint32());
                    break;
                case 21:
                    message.waitingForParaheaders = reader.bool();
                    break;
                case 22:
                    message.networkStatus = $root.pruntime_rpc.NetworkStatus.decode(reader, reader.uint32());
                    break;
                case 23:
                    message.system = $root.pruntime_rpc.SystemInfo.decode(reader, reader.uint32());
                    break;
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a PhactoryInfo message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof pruntime_rpc.PhactoryInfo
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {pruntime_rpc.PhactoryInfo} PhactoryInfo
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        PhactoryInfo.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies a PhactoryInfo message.
         * @function verify
         * @memberof pruntime_rpc.PhactoryInfo
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        PhactoryInfo.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            var properties = {};
            if (message.initialized != null && message.hasOwnProperty("initialized"))
                if (typeof message.initialized !== "boolean")
                    return "initialized: boolean expected";
            if (message.registered != null && message.hasOwnProperty("registered"))
                if (typeof message.registered !== "boolean")
                    return "registered: boolean expected";
            if (message.genesisBlockHash != null && message.hasOwnProperty("genesisBlockHash")) {
                properties._genesisBlockHash = 1;
                if (!$util.isString(message.genesisBlockHash))
                    return "genesisBlockHash: string expected";
            }
            if (message.publicKey != null && message.hasOwnProperty("publicKey")) {
                properties._publicKey = 1;
                if (!$util.isString(message.publicKey))
                    return "publicKey: string expected";
            }
            if (message.ecdhPublicKey != null && message.hasOwnProperty("ecdhPublicKey")) {
                properties._ecdhPublicKey = 1;
                if (!$util.isString(message.ecdhPublicKey))
                    return "ecdhPublicKey: string expected";
            }
            if (message.headernum != null && message.hasOwnProperty("headernum"))
                if (!$util.isInteger(message.headernum))
                    return "headernum: integer expected";
            if (message.paraHeadernum != null && message.hasOwnProperty("paraHeadernum"))
                if (!$util.isInteger(message.paraHeadernum))
                    return "paraHeadernum: integer expected";
            if (message.blocknum != null && message.hasOwnProperty("blocknum"))
                if (!$util.isInteger(message.blocknum))
                    return "blocknum: integer expected";
            if (message.stateRoot != null && message.hasOwnProperty("stateRoot"))
                if (!$util.isString(message.stateRoot))
                    return "stateRoot: string expected";
            if (message.devMode != null && message.hasOwnProperty("devMode"))
                if (typeof message.devMode !== "boolean")
                    return "devMode: boolean expected";
            if (message.pendingMessages != null && message.hasOwnProperty("pendingMessages"))
                if (!$util.isInteger(message.pendingMessages) && !(message.pendingMessages && $util.isInteger(message.pendingMessages.low) && $util.isInteger(message.pendingMessages.high)))
                    return "pendingMessages: integer|Long expected";
            if (message.score != null && message.hasOwnProperty("score"))
                if (!$util.isInteger(message.score) && !(message.score && $util.isInteger(message.score.low) && $util.isInteger(message.score.high)))
                    return "score: integer|Long expected";
            if (message.gatekeeper != null && message.hasOwnProperty("gatekeeper")) {
                var error = $root.pruntime_rpc.GatekeeperStatus.verify(message.gatekeeper);
                if (error)
                    return "gatekeeper." + error;
            }
            if (message.version != null && message.hasOwnProperty("version"))
                if (!$util.isString(message.version))
                    return "version: string expected";
            if (message.gitRevision != null && message.hasOwnProperty("gitRevision"))
                if (!$util.isString(message.gitRevision))
                    return "gitRevision: string expected";
            if (message.memoryUsage != null && message.hasOwnProperty("memoryUsage")) {
                var error = $root.pruntime_rpc.MemoryUsage.verify(message.memoryUsage);
                if (error)
                    return "memoryUsage." + error;
            }
            if (message.waitingForParaheaders != null && message.hasOwnProperty("waitingForParaheaders"))
                if (typeof message.waitingForParaheaders !== "boolean")
                    return "waitingForParaheaders: boolean expected";
            if (message.networkStatus != null && message.hasOwnProperty("networkStatus")) {
                var error = $root.pruntime_rpc.NetworkStatus.verify(message.networkStatus);
                if (error)
                    return "networkStatus." + error;
            }
            if (message.system != null && message.hasOwnProperty("system")) {
                var error = $root.pruntime_rpc.SystemInfo.verify(message.system);
                if (error)
                    return "system." + error;
            }
            return null;
        };

        /**
         * Creates a PhactoryInfo message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof pruntime_rpc.PhactoryInfo
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {pruntime_rpc.PhactoryInfo} PhactoryInfo
         */
        PhactoryInfo.fromObject = function fromObject(object) {
            if (object instanceof $root.pruntime_rpc.PhactoryInfo)
                return object;
            var message = new $root.pruntime_rpc.PhactoryInfo();
            if (object.initialized != null)
                message.initialized = Boolean(object.initialized);
            if (object.registered != null)
                message.registered = Boolean(object.registered);
            if (object.genesisBlockHash != null)
                message.genesisBlockHash = String(object.genesisBlockHash);
            if (object.publicKey != null)
                message.publicKey = String(object.publicKey);
            if (object.ecdhPublicKey != null)
                message.ecdhPublicKey = String(object.ecdhPublicKey);
            if (object.headernum != null)
                message.headernum = object.headernum >>> 0;
            if (object.paraHeadernum != null)
                message.paraHeadernum = object.paraHeadernum >>> 0;
            if (object.blocknum != null)
                message.blocknum = object.blocknum >>> 0;
            if (object.stateRoot != null)
                message.stateRoot = String(object.stateRoot);
            if (object.devMode != null)
                message.devMode = Boolean(object.devMode);
            if (object.pendingMessages != null)
                if ($util.Long)
                    (message.pendingMessages = $util.Long.fromValue(object.pendingMessages)).unsigned = true;
                else if (typeof object.pendingMessages === "string")
                    message.pendingMessages = parseInt(object.pendingMessages, 10);
                else if (typeof object.pendingMessages === "number")
                    message.pendingMessages = object.pendingMessages;
                else if (typeof object.pendingMessages === "object")
                    message.pendingMessages = new $util.LongBits(object.pendingMessages.low >>> 0, object.pendingMessages.high >>> 0).toNumber(true);
            if (object.score != null)
                if ($util.Long)
                    (message.score = $util.Long.fromValue(object.score)).unsigned = true;
                else if (typeof object.score === "string")
                    message.score = parseInt(object.score, 10);
                else if (typeof object.score === "number")
                    message.score = object.score;
                else if (typeof object.score === "object")
                    message.score = new $util.LongBits(object.score.low >>> 0, object.score.high >>> 0).toNumber(true);
            if (object.gatekeeper != null) {
                if (typeof object.gatekeeper !== "object")
                    throw TypeError(".pruntime_rpc.PhactoryInfo.gatekeeper: object expected");
                message.gatekeeper = $root.pruntime_rpc.GatekeeperStatus.fromObject(object.gatekeeper);
            }
            if (object.version != null)
                message.version = String(object.version);
            if (object.gitRevision != null)
                message.gitRevision = String(object.gitRevision);
            if (object.memoryUsage != null) {
                if (typeof object.memoryUsage !== "object")
                    throw TypeError(".pruntime_rpc.PhactoryInfo.memoryUsage: object expected");
                message.memoryUsage = $root.pruntime_rpc.MemoryUsage.fromObject(object.memoryUsage);
            }
            if (object.waitingForParaheaders != null)
                message.waitingForParaheaders = Boolean(object.waitingForParaheaders);
            if (object.networkStatus != null) {
                if (typeof object.networkStatus !== "object")
                    throw TypeError(".pruntime_rpc.PhactoryInfo.networkStatus: object expected");
                message.networkStatus = $root.pruntime_rpc.NetworkStatus.fromObject(object.networkStatus);
            }
            if (object.system != null) {
                if (typeof object.system !== "object")
                    throw TypeError(".pruntime_rpc.PhactoryInfo.system: object expected");
                message.system = $root.pruntime_rpc.SystemInfo.fromObject(object.system);
            }
            return message;
        };

        /**
         * Creates a plain object from a PhactoryInfo message. Also converts values to other types if specified.
         * @function toObject
         * @memberof pruntime_rpc.PhactoryInfo
         * @static
         * @param {pruntime_rpc.PhactoryInfo} message PhactoryInfo
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        PhactoryInfo.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            var object = {};
            if (options.defaults) {
                object.initialized = false;
                object.registered = false;
                object.headernum = 0;
                object.paraHeadernum = 0;
                object.blocknum = 0;
                object.stateRoot = "";
                object.devMode = false;
                if ($util.Long) {
                    var long = new $util.Long(0, 0, true);
                    object.pendingMessages = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : long;
                } else
                    object.pendingMessages = options.longs === String ? "0" : 0;
                if ($util.Long) {
                    var long = new $util.Long(0, 0, true);
                    object.score = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : long;
                } else
                    object.score = options.longs === String ? "0" : 0;
                object.gatekeeper = null;
                object.version = "";
                object.gitRevision = "";
                object.memoryUsage = null;
                object.waitingForParaheaders = false;
                object.networkStatus = null;
                object.system = null;
            }
            if (message.initialized != null && message.hasOwnProperty("initialized"))
                object.initialized = message.initialized;
            if (message.registered != null && message.hasOwnProperty("registered"))
                object.registered = message.registered;
            if (message.genesisBlockHash != null && message.hasOwnProperty("genesisBlockHash")) {
                object.genesisBlockHash = message.genesisBlockHash;
                if (options.oneofs)
                    object._genesisBlockHash = "genesisBlockHash";
            }
            if (message.publicKey != null && message.hasOwnProperty("publicKey")) {
                object.publicKey = message.publicKey;
                if (options.oneofs)
                    object._publicKey = "publicKey";
            }
            if (message.ecdhPublicKey != null && message.hasOwnProperty("ecdhPublicKey")) {
                object.ecdhPublicKey = message.ecdhPublicKey;
                if (options.oneofs)
                    object._ecdhPublicKey = "ecdhPublicKey";
            }
            if (message.headernum != null && message.hasOwnProperty("headernum"))
                object.headernum = message.headernum;
            if (message.paraHeadernum != null && message.hasOwnProperty("paraHeadernum"))
                object.paraHeadernum = message.paraHeadernum;
            if (message.blocknum != null && message.hasOwnProperty("blocknum"))
                object.blocknum = message.blocknum;
            if (message.stateRoot != null && message.hasOwnProperty("stateRoot"))
                object.stateRoot = message.stateRoot;
            if (message.devMode != null && message.hasOwnProperty("devMode"))
                object.devMode = message.devMode;
            if (message.pendingMessages != null && message.hasOwnProperty("pendingMessages"))
                if (typeof message.pendingMessages === "number")
                    object.pendingMessages = options.longs === String ? String(message.pendingMessages) : message.pendingMessages;
                else
                    object.pendingMessages = options.longs === String ? $util.Long.prototype.toString.call(message.pendingMessages) : options.longs === Number ? new $util.LongBits(message.pendingMessages.low >>> 0, message.pendingMessages.high >>> 0).toNumber(true) : message.pendingMessages;
            if (message.score != null && message.hasOwnProperty("score"))
                if (typeof message.score === "number")
                    object.score = options.longs === String ? String(message.score) : message.score;
                else
                    object.score = options.longs === String ? $util.Long.prototype.toString.call(message.score) : options.longs === Number ? new $util.LongBits(message.score.low >>> 0, message.score.high >>> 0).toNumber(true) : message.score;
            if (message.gatekeeper != null && message.hasOwnProperty("gatekeeper"))
                object.gatekeeper = $root.pruntime_rpc.GatekeeperStatus.toObject(message.gatekeeper, options);
            if (message.version != null && message.hasOwnProperty("version"))
                object.version = message.version;
            if (message.gitRevision != null && message.hasOwnProperty("gitRevision"))
                object.gitRevision = message.gitRevision;
            if (message.memoryUsage != null && message.hasOwnProperty("memoryUsage"))
                object.memoryUsage = $root.pruntime_rpc.MemoryUsage.toObject(message.memoryUsage, options);
            if (message.waitingForParaheaders != null && message.hasOwnProperty("waitingForParaheaders"))
                object.waitingForParaheaders = message.waitingForParaheaders;
            if (message.networkStatus != null && message.hasOwnProperty("networkStatus"))
                object.networkStatus = $root.pruntime_rpc.NetworkStatus.toObject(message.networkStatus, options);
            if (message.system != null && message.hasOwnProperty("system"))
                object.system = $root.pruntime_rpc.SystemInfo.toObject(message.system, options);
            return object;
        };

        /**
         * Converts this PhactoryInfo to JSON.
         * @function toJSON
         * @memberof pruntime_rpc.PhactoryInfo
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        PhactoryInfo.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        return PhactoryInfo;
    })();

    pruntime_rpc.SystemInfo = (function() {

        /**
         * Properties of a SystemInfo.
         * @memberof pruntime_rpc
         * @interface ISystemInfo
         * @property {boolean|null} [registered] SystemInfo registered
         * @property {string|null} [publicKey] SystemInfo publicKey
         * @property {string|null} [ecdhPublicKey] SystemInfo ecdhPublicKey
         * @property {pruntime_rpc.IGatekeeperStatus|null} [gatekeeper] SystemInfo gatekeeper
         * @property {number|Long|null} [numberOfClusters] SystemInfo numberOfClusters
         * @property {number|Long|null} [numberOfContracts] SystemInfo numberOfContracts
         * @property {number|null} [consensusVersion] SystemInfo consensusVersion
         * @property {number|null} [maxSupportedConsensusVersion] SystemInfo maxSupportedConsensusVersion
         */

        /**
         * Constructs a new SystemInfo.
         * @memberof pruntime_rpc
         * @classdesc Represents a SystemInfo.
         * @implements ISystemInfo
         * @constructor
         * @param {pruntime_rpc.ISystemInfo=} [properties] Properties to set
         */
        function SystemInfo(properties) {
            if (properties)
                for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * SystemInfo registered.
         * @member {boolean} registered
         * @memberof pruntime_rpc.SystemInfo
         * @instance
         */
        SystemInfo.prototype.registered = false;

        /**
         * SystemInfo publicKey.
         * @member {string} publicKey
         * @memberof pruntime_rpc.SystemInfo
         * @instance
         */
        SystemInfo.prototype.publicKey = "";

        /**
         * SystemInfo ecdhPublicKey.
         * @member {string} ecdhPublicKey
         * @memberof pruntime_rpc.SystemInfo
         * @instance
         */
        SystemInfo.prototype.ecdhPublicKey = "";

        /**
         * SystemInfo gatekeeper.
         * @member {pruntime_rpc.IGatekeeperStatus|null|undefined} gatekeeper
         * @memberof pruntime_rpc.SystemInfo
         * @instance
         */
        SystemInfo.prototype.gatekeeper = null;

        /**
         * SystemInfo numberOfClusters.
         * @member {number|Long} numberOfClusters
         * @memberof pruntime_rpc.SystemInfo
         * @instance
         */
        SystemInfo.prototype.numberOfClusters = $util.Long ? $util.Long.fromBits(0,0,true) : 0;

        /**
         * SystemInfo numberOfContracts.
         * @member {number|Long} numberOfContracts
         * @memberof pruntime_rpc.SystemInfo
         * @instance
         */
        SystemInfo.prototype.numberOfContracts = $util.Long ? $util.Long.fromBits(0,0,true) : 0;

        /**
         * SystemInfo consensusVersion.
         * @member {number} consensusVersion
         * @memberof pruntime_rpc.SystemInfo
         * @instance
         */
        SystemInfo.prototype.consensusVersion = 0;

        /**
         * SystemInfo maxSupportedConsensusVersion.
         * @member {number} maxSupportedConsensusVersion
         * @memberof pruntime_rpc.SystemInfo
         * @instance
         */
        SystemInfo.prototype.maxSupportedConsensusVersion = 0;

        /**
         * Creates a new SystemInfo instance using the specified properties.
         * @function create
         * @memberof pruntime_rpc.SystemInfo
         * @static
         * @param {pruntime_rpc.ISystemInfo=} [properties] Properties to set
         * @returns {pruntime_rpc.SystemInfo} SystemInfo instance
         */
        SystemInfo.create = function create(properties) {
            return new SystemInfo(properties);
        };

        /**
         * Encodes the specified SystemInfo message. Does not implicitly {@link pruntime_rpc.SystemInfo.verify|verify} messages.
         * @function encode
         * @memberof pruntime_rpc.SystemInfo
         * @static
         * @param {pruntime_rpc.ISystemInfo} message SystemInfo message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        SystemInfo.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.registered != null && Object.hasOwnProperty.call(message, "registered"))
                writer.uint32(/* id 1, wireType 0 =*/8).bool(message.registered);
            if (message.publicKey != null && Object.hasOwnProperty.call(message, "publicKey"))
                writer.uint32(/* id 2, wireType 2 =*/18).string(message.publicKey);
            if (message.ecdhPublicKey != null && Object.hasOwnProperty.call(message, "ecdhPublicKey"))
                writer.uint32(/* id 3, wireType 2 =*/26).string(message.ecdhPublicKey);
            if (message.gatekeeper != null && Object.hasOwnProperty.call(message, "gatekeeper"))
                $root.pruntime_rpc.GatekeeperStatus.encode(message.gatekeeper, writer.uint32(/* id 4, wireType 2 =*/34).fork()).ldelim();
            if (message.numberOfClusters != null && Object.hasOwnProperty.call(message, "numberOfClusters"))
                writer.uint32(/* id 5, wireType 0 =*/40).uint64(message.numberOfClusters);
            if (message.numberOfContracts != null && Object.hasOwnProperty.call(message, "numberOfContracts"))
                writer.uint32(/* id 6, wireType 0 =*/48).uint64(message.numberOfContracts);
            if (message.consensusVersion != null && Object.hasOwnProperty.call(message, "consensusVersion"))
                writer.uint32(/* id 7, wireType 0 =*/56).uint32(message.consensusVersion);
            if (message.maxSupportedConsensusVersion != null && Object.hasOwnProperty.call(message, "maxSupportedConsensusVersion"))
                writer.uint32(/* id 8, wireType 0 =*/64).uint32(message.maxSupportedConsensusVersion);
            return writer;
        };

        /**
         * Encodes the specified SystemInfo message, length delimited. Does not implicitly {@link pruntime_rpc.SystemInfo.verify|verify} messages.
         * @function encodeDelimited
         * @memberof pruntime_rpc.SystemInfo
         * @static
         * @param {pruntime_rpc.ISystemInfo} message SystemInfo message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        SystemInfo.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes a SystemInfo message from the specified reader or buffer.
         * @function decode
         * @memberof pruntime_rpc.SystemInfo
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {pruntime_rpc.SystemInfo} SystemInfo
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        SystemInfo.decode = function decode(reader, length) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            var end = length === undefined ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.SystemInfo();
            while (reader.pos < end) {
                var tag = reader.uint32();
                switch (tag >>> 3) {
                case 1:
                    message.registered = reader.bool();
                    break;
                case 2:
                    message.publicKey = reader.string();
                    break;
                case 3:
                    message.ecdhPublicKey = reader.string();
                    break;
                case 4:
                    message.gatekeeper = $root.pruntime_rpc.GatekeeperStatus.decode(reader, reader.uint32());
                    break;
                case 5:
                    message.numberOfClusters = reader.uint64();
                    break;
                case 6:
                    message.numberOfContracts = reader.uint64();
                    break;
                case 7:
                    message.consensusVersion = reader.uint32();
                    break;
                case 8:
                    message.maxSupportedConsensusVersion = reader.uint32();
                    break;
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a SystemInfo message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof pruntime_rpc.SystemInfo
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {pruntime_rpc.SystemInfo} SystemInfo
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        SystemInfo.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies a SystemInfo message.
         * @function verify
         * @memberof pruntime_rpc.SystemInfo
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        SystemInfo.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            if (message.registered != null && message.hasOwnProperty("registered"))
                if (typeof message.registered !== "boolean")
                    return "registered: boolean expected";
            if (message.publicKey != null && message.hasOwnProperty("publicKey"))
                if (!$util.isString(message.publicKey))
                    return "publicKey: string expected";
            if (message.ecdhPublicKey != null && message.hasOwnProperty("ecdhPublicKey"))
                if (!$util.isString(message.ecdhPublicKey))
                    return "ecdhPublicKey: string expected";
            if (message.gatekeeper != null && message.hasOwnProperty("gatekeeper")) {
                var error = $root.pruntime_rpc.GatekeeperStatus.verify(message.gatekeeper);
                if (error)
                    return "gatekeeper." + error;
            }
            if (message.numberOfClusters != null && message.hasOwnProperty("numberOfClusters"))
                if (!$util.isInteger(message.numberOfClusters) && !(message.numberOfClusters && $util.isInteger(message.numberOfClusters.low) && $util.isInteger(message.numberOfClusters.high)))
                    return "numberOfClusters: integer|Long expected";
            if (message.numberOfContracts != null && message.hasOwnProperty("numberOfContracts"))
                if (!$util.isInteger(message.numberOfContracts) && !(message.numberOfContracts && $util.isInteger(message.numberOfContracts.low) && $util.isInteger(message.numberOfContracts.high)))
                    return "numberOfContracts: integer|Long expected";
            if (message.consensusVersion != null && message.hasOwnProperty("consensusVersion"))
                if (!$util.isInteger(message.consensusVersion))
                    return "consensusVersion: integer expected";
            if (message.maxSupportedConsensusVersion != null && message.hasOwnProperty("maxSupportedConsensusVersion"))
                if (!$util.isInteger(message.maxSupportedConsensusVersion))
                    return "maxSupportedConsensusVersion: integer expected";
            return null;
        };

        /**
         * Creates a SystemInfo message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof pruntime_rpc.SystemInfo
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {pruntime_rpc.SystemInfo} SystemInfo
         */
        SystemInfo.fromObject = function fromObject(object) {
            if (object instanceof $root.pruntime_rpc.SystemInfo)
                return object;
            var message = new $root.pruntime_rpc.SystemInfo();
            if (object.registered != null)
                message.registered = Boolean(object.registered);
            if (object.publicKey != null)
                message.publicKey = String(object.publicKey);
            if (object.ecdhPublicKey != null)
                message.ecdhPublicKey = String(object.ecdhPublicKey);
            if (object.gatekeeper != null) {
                if (typeof object.gatekeeper !== "object")
                    throw TypeError(".pruntime_rpc.SystemInfo.gatekeeper: object expected");
                message.gatekeeper = $root.pruntime_rpc.GatekeeperStatus.fromObject(object.gatekeeper);
            }
            if (object.numberOfClusters != null)
                if ($util.Long)
                    (message.numberOfClusters = $util.Long.fromValue(object.numberOfClusters)).unsigned = true;
                else if (typeof object.numberOfClusters === "string")
                    message.numberOfClusters = parseInt(object.numberOfClusters, 10);
                else if (typeof object.numberOfClusters === "number")
                    message.numberOfClusters = object.numberOfClusters;
                else if (typeof object.numberOfClusters === "object")
                    message.numberOfClusters = new $util.LongBits(object.numberOfClusters.low >>> 0, object.numberOfClusters.high >>> 0).toNumber(true);
            if (object.numberOfContracts != null)
                if ($util.Long)
                    (message.numberOfContracts = $util.Long.fromValue(object.numberOfContracts)).unsigned = true;
                else if (typeof object.numberOfContracts === "string")
                    message.numberOfContracts = parseInt(object.numberOfContracts, 10);
                else if (typeof object.numberOfContracts === "number")
                    message.numberOfContracts = object.numberOfContracts;
                else if (typeof object.numberOfContracts === "object")
                    message.numberOfContracts = new $util.LongBits(object.numberOfContracts.low >>> 0, object.numberOfContracts.high >>> 0).toNumber(true);
            if (object.consensusVersion != null)
                message.consensusVersion = object.consensusVersion >>> 0;
            if (object.maxSupportedConsensusVersion != null)
                message.maxSupportedConsensusVersion = object.maxSupportedConsensusVersion >>> 0;
            return message;
        };

        /**
         * Creates a plain object from a SystemInfo message. Also converts values to other types if specified.
         * @function toObject
         * @memberof pruntime_rpc.SystemInfo
         * @static
         * @param {pruntime_rpc.SystemInfo} message SystemInfo
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        SystemInfo.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            var object = {};
            if (options.defaults) {
                object.registered = false;
                object.publicKey = "";
                object.ecdhPublicKey = "";
                object.gatekeeper = null;
                if ($util.Long) {
                    var long = new $util.Long(0, 0, true);
                    object.numberOfClusters = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : long;
                } else
                    object.numberOfClusters = options.longs === String ? "0" : 0;
                if ($util.Long) {
                    var long = new $util.Long(0, 0, true);
                    object.numberOfContracts = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : long;
                } else
                    object.numberOfContracts = options.longs === String ? "0" : 0;
                object.consensusVersion = 0;
                object.maxSupportedConsensusVersion = 0;
            }
            if (message.registered != null && message.hasOwnProperty("registered"))
                object.registered = message.registered;
            if (message.publicKey != null && message.hasOwnProperty("publicKey"))
                object.publicKey = message.publicKey;
            if (message.ecdhPublicKey != null && message.hasOwnProperty("ecdhPublicKey"))
                object.ecdhPublicKey = message.ecdhPublicKey;
            if (message.gatekeeper != null && message.hasOwnProperty("gatekeeper"))
                object.gatekeeper = $root.pruntime_rpc.GatekeeperStatus.toObject(message.gatekeeper, options);
            if (message.numberOfClusters != null && message.hasOwnProperty("numberOfClusters"))
                if (typeof message.numberOfClusters === "number")
                    object.numberOfClusters = options.longs === String ? String(message.numberOfClusters) : message.numberOfClusters;
                else
                    object.numberOfClusters = options.longs === String ? $util.Long.prototype.toString.call(message.numberOfClusters) : options.longs === Number ? new $util.LongBits(message.numberOfClusters.low >>> 0, message.numberOfClusters.high >>> 0).toNumber(true) : message.numberOfClusters;
            if (message.numberOfContracts != null && message.hasOwnProperty("numberOfContracts"))
                if (typeof message.numberOfContracts === "number")
                    object.numberOfContracts = options.longs === String ? String(message.numberOfContracts) : message.numberOfContracts;
                else
                    object.numberOfContracts = options.longs === String ? $util.Long.prototype.toString.call(message.numberOfContracts) : options.longs === Number ? new $util.LongBits(message.numberOfContracts.low >>> 0, message.numberOfContracts.high >>> 0).toNumber(true) : message.numberOfContracts;
            if (message.consensusVersion != null && message.hasOwnProperty("consensusVersion"))
                object.consensusVersion = message.consensusVersion;
            if (message.maxSupportedConsensusVersion != null && message.hasOwnProperty("maxSupportedConsensusVersion"))
                object.maxSupportedConsensusVersion = message.maxSupportedConsensusVersion;
            return object;
        };

        /**
         * Converts this SystemInfo to JSON.
         * @function toJSON
         * @memberof pruntime_rpc.SystemInfo
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        SystemInfo.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        return SystemInfo;
    })();

    /**
     * GatekeeperRole enum.
     * @name pruntime_rpc.GatekeeperRole
     * @enum {number}
     * @property {number} None=0 None value
     * @property {number} Dummy=1 Dummy value
     * @property {number} Active=2 Active value
     */
    pruntime_rpc.GatekeeperRole = (function() {
        var valuesById = {}, values = Object.create(valuesById);
        values[valuesById[0] = "None"] = 0;
        values[valuesById[1] = "Dummy"] = 1;
        values[valuesById[2] = "Active"] = 2;
        return values;
    })();

    pruntime_rpc.GatekeeperStatus = (function() {

        /**
         * Properties of a GatekeeperStatus.
         * @memberof pruntime_rpc
         * @interface IGatekeeperStatus
         * @property {pruntime_rpc.GatekeeperRole|null} [role] GatekeeperStatus role
         * @property {string|null} [masterPublicKey] GatekeeperStatus masterPublicKey
         */

        /**
         * Constructs a new GatekeeperStatus.
         * @memberof pruntime_rpc
         * @classdesc Represents a GatekeeperStatus.
         * @implements IGatekeeperStatus
         * @constructor
         * @param {pruntime_rpc.IGatekeeperStatus=} [properties] Properties to set
         */
        function GatekeeperStatus(properties) {
            if (properties)
                for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * GatekeeperStatus role.
         * @member {pruntime_rpc.GatekeeperRole} role
         * @memberof pruntime_rpc.GatekeeperStatus
         * @instance
         */
        GatekeeperStatus.prototype.role = 0;

        /**
         * GatekeeperStatus masterPublicKey.
         * @member {string} masterPublicKey
         * @memberof pruntime_rpc.GatekeeperStatus
         * @instance
         */
        GatekeeperStatus.prototype.masterPublicKey = "";

        /**
         * Creates a new GatekeeperStatus instance using the specified properties.
         * @function create
         * @memberof pruntime_rpc.GatekeeperStatus
         * @static
         * @param {pruntime_rpc.IGatekeeperStatus=} [properties] Properties to set
         * @returns {pruntime_rpc.GatekeeperStatus} GatekeeperStatus instance
         */
        GatekeeperStatus.create = function create(properties) {
            return new GatekeeperStatus(properties);
        };

        /**
         * Encodes the specified GatekeeperStatus message. Does not implicitly {@link pruntime_rpc.GatekeeperStatus.verify|verify} messages.
         * @function encode
         * @memberof pruntime_rpc.GatekeeperStatus
         * @static
         * @param {pruntime_rpc.IGatekeeperStatus} message GatekeeperStatus message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        GatekeeperStatus.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.role != null && Object.hasOwnProperty.call(message, "role"))
                writer.uint32(/* id 1, wireType 0 =*/8).int32(message.role);
            if (message.masterPublicKey != null && Object.hasOwnProperty.call(message, "masterPublicKey"))
                writer.uint32(/* id 2, wireType 2 =*/18).string(message.masterPublicKey);
            return writer;
        };

        /**
         * Encodes the specified GatekeeperStatus message, length delimited. Does not implicitly {@link pruntime_rpc.GatekeeperStatus.verify|verify} messages.
         * @function encodeDelimited
         * @memberof pruntime_rpc.GatekeeperStatus
         * @static
         * @param {pruntime_rpc.IGatekeeperStatus} message GatekeeperStatus message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        GatekeeperStatus.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes a GatekeeperStatus message from the specified reader or buffer.
         * @function decode
         * @memberof pruntime_rpc.GatekeeperStatus
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {pruntime_rpc.GatekeeperStatus} GatekeeperStatus
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        GatekeeperStatus.decode = function decode(reader, length) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            var end = length === undefined ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.GatekeeperStatus();
            while (reader.pos < end) {
                var tag = reader.uint32();
                switch (tag >>> 3) {
                case 1:
                    message.role = reader.int32();
                    break;
                case 2:
                    message.masterPublicKey = reader.string();
                    break;
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a GatekeeperStatus message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof pruntime_rpc.GatekeeperStatus
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {pruntime_rpc.GatekeeperStatus} GatekeeperStatus
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        GatekeeperStatus.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies a GatekeeperStatus message.
         * @function verify
         * @memberof pruntime_rpc.GatekeeperStatus
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        GatekeeperStatus.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            if (message.role != null && message.hasOwnProperty("role"))
                switch (message.role) {
                default:
                    return "role: enum value expected";
                case 0:
                case 1:
                case 2:
                    break;
                }
            if (message.masterPublicKey != null && message.hasOwnProperty("masterPublicKey"))
                if (!$util.isString(message.masterPublicKey))
                    return "masterPublicKey: string expected";
            return null;
        };

        /**
         * Creates a GatekeeperStatus message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof pruntime_rpc.GatekeeperStatus
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {pruntime_rpc.GatekeeperStatus} GatekeeperStatus
         */
        GatekeeperStatus.fromObject = function fromObject(object) {
            if (object instanceof $root.pruntime_rpc.GatekeeperStatus)
                return object;
            var message = new $root.pruntime_rpc.GatekeeperStatus();
            switch (object.role) {
            case "None":
            case 0:
                message.role = 0;
                break;
            case "Dummy":
            case 1:
                message.role = 1;
                break;
            case "Active":
            case 2:
                message.role = 2;
                break;
            }
            if (object.masterPublicKey != null)
                message.masterPublicKey = String(object.masterPublicKey);
            return message;
        };

        /**
         * Creates a plain object from a GatekeeperStatus message. Also converts values to other types if specified.
         * @function toObject
         * @memberof pruntime_rpc.GatekeeperStatus
         * @static
         * @param {pruntime_rpc.GatekeeperStatus} message GatekeeperStatus
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        GatekeeperStatus.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            var object = {};
            if (options.defaults) {
                object.role = options.enums === String ? "None" : 0;
                object.masterPublicKey = "";
            }
            if (message.role != null && message.hasOwnProperty("role"))
                object.role = options.enums === String ? $root.pruntime_rpc.GatekeeperRole[message.role] : message.role;
            if (message.masterPublicKey != null && message.hasOwnProperty("masterPublicKey"))
                object.masterPublicKey = message.masterPublicKey;
            return object;
        };

        /**
         * Converts this GatekeeperStatus to JSON.
         * @function toJSON
         * @memberof pruntime_rpc.GatekeeperStatus
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        GatekeeperStatus.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        return GatekeeperStatus;
    })();

    pruntime_rpc.MemoryUsage = (function() {

        /**
         * Properties of a MemoryUsage.
         * @memberof pruntime_rpc
         * @interface IMemoryUsage
         * @property {number|Long|null} [rustUsed] MemoryUsage rustUsed
         * @property {number|Long|null} [rustPeakUsed] MemoryUsage rustPeakUsed
         * @property {number|Long|null} [totalPeakUsed] MemoryUsage totalPeakUsed
         */

        /**
         * Constructs a new MemoryUsage.
         * @memberof pruntime_rpc
         * @classdesc Represents a MemoryUsage.
         * @implements IMemoryUsage
         * @constructor
         * @param {pruntime_rpc.IMemoryUsage=} [properties] Properties to set
         */
        function MemoryUsage(properties) {
            if (properties)
                for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * MemoryUsage rustUsed.
         * @member {number|Long} rustUsed
         * @memberof pruntime_rpc.MemoryUsage
         * @instance
         */
        MemoryUsage.prototype.rustUsed = $util.Long ? $util.Long.fromBits(0,0,true) : 0;

        /**
         * MemoryUsage rustPeakUsed.
         * @member {number|Long} rustPeakUsed
         * @memberof pruntime_rpc.MemoryUsage
         * @instance
         */
        MemoryUsage.prototype.rustPeakUsed = $util.Long ? $util.Long.fromBits(0,0,true) : 0;

        /**
         * MemoryUsage totalPeakUsed.
         * @member {number|Long} totalPeakUsed
         * @memberof pruntime_rpc.MemoryUsage
         * @instance
         */
        MemoryUsage.prototype.totalPeakUsed = $util.Long ? $util.Long.fromBits(0,0,true) : 0;

        /**
         * Creates a new MemoryUsage instance using the specified properties.
         * @function create
         * @memberof pruntime_rpc.MemoryUsage
         * @static
         * @param {pruntime_rpc.IMemoryUsage=} [properties] Properties to set
         * @returns {pruntime_rpc.MemoryUsage} MemoryUsage instance
         */
        MemoryUsage.create = function create(properties) {
            return new MemoryUsage(properties);
        };

        /**
         * Encodes the specified MemoryUsage message. Does not implicitly {@link pruntime_rpc.MemoryUsage.verify|verify} messages.
         * @function encode
         * @memberof pruntime_rpc.MemoryUsage
         * @static
         * @param {pruntime_rpc.IMemoryUsage} message MemoryUsage message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        MemoryUsage.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.rustUsed != null && Object.hasOwnProperty.call(message, "rustUsed"))
                writer.uint32(/* id 1, wireType 0 =*/8).uint64(message.rustUsed);
            if (message.rustPeakUsed != null && Object.hasOwnProperty.call(message, "rustPeakUsed"))
                writer.uint32(/* id 2, wireType 0 =*/16).uint64(message.rustPeakUsed);
            if (message.totalPeakUsed != null && Object.hasOwnProperty.call(message, "totalPeakUsed"))
                writer.uint32(/* id 3, wireType 0 =*/24).uint64(message.totalPeakUsed);
            return writer;
        };

        /**
         * Encodes the specified MemoryUsage message, length delimited. Does not implicitly {@link pruntime_rpc.MemoryUsage.verify|verify} messages.
         * @function encodeDelimited
         * @memberof pruntime_rpc.MemoryUsage
         * @static
         * @param {pruntime_rpc.IMemoryUsage} message MemoryUsage message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        MemoryUsage.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes a MemoryUsage message from the specified reader or buffer.
         * @function decode
         * @memberof pruntime_rpc.MemoryUsage
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {pruntime_rpc.MemoryUsage} MemoryUsage
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        MemoryUsage.decode = function decode(reader, length) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            var end = length === undefined ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.MemoryUsage();
            while (reader.pos < end) {
                var tag = reader.uint32();
                switch (tag >>> 3) {
                case 1:
                    message.rustUsed = reader.uint64();
                    break;
                case 2:
                    message.rustPeakUsed = reader.uint64();
                    break;
                case 3:
                    message.totalPeakUsed = reader.uint64();
                    break;
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a MemoryUsage message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof pruntime_rpc.MemoryUsage
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {pruntime_rpc.MemoryUsage} MemoryUsage
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        MemoryUsage.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies a MemoryUsage message.
         * @function verify
         * @memberof pruntime_rpc.MemoryUsage
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        MemoryUsage.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            if (message.rustUsed != null && message.hasOwnProperty("rustUsed"))
                if (!$util.isInteger(message.rustUsed) && !(message.rustUsed && $util.isInteger(message.rustUsed.low) && $util.isInteger(message.rustUsed.high)))
                    return "rustUsed: integer|Long expected";
            if (message.rustPeakUsed != null && message.hasOwnProperty("rustPeakUsed"))
                if (!$util.isInteger(message.rustPeakUsed) && !(message.rustPeakUsed && $util.isInteger(message.rustPeakUsed.low) && $util.isInteger(message.rustPeakUsed.high)))
                    return "rustPeakUsed: integer|Long expected";
            if (message.totalPeakUsed != null && message.hasOwnProperty("totalPeakUsed"))
                if (!$util.isInteger(message.totalPeakUsed) && !(message.totalPeakUsed && $util.isInteger(message.totalPeakUsed.low) && $util.isInteger(message.totalPeakUsed.high)))
                    return "totalPeakUsed: integer|Long expected";
            return null;
        };

        /**
         * Creates a MemoryUsage message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof pruntime_rpc.MemoryUsage
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {pruntime_rpc.MemoryUsage} MemoryUsage
         */
        MemoryUsage.fromObject = function fromObject(object) {
            if (object instanceof $root.pruntime_rpc.MemoryUsage)
                return object;
            var message = new $root.pruntime_rpc.MemoryUsage();
            if (object.rustUsed != null)
                if ($util.Long)
                    (message.rustUsed = $util.Long.fromValue(object.rustUsed)).unsigned = true;
                else if (typeof object.rustUsed === "string")
                    message.rustUsed = parseInt(object.rustUsed, 10);
                else if (typeof object.rustUsed === "number")
                    message.rustUsed = object.rustUsed;
                else if (typeof object.rustUsed === "object")
                    message.rustUsed = new $util.LongBits(object.rustUsed.low >>> 0, object.rustUsed.high >>> 0).toNumber(true);
            if (object.rustPeakUsed != null)
                if ($util.Long)
                    (message.rustPeakUsed = $util.Long.fromValue(object.rustPeakUsed)).unsigned = true;
                else if (typeof object.rustPeakUsed === "string")
                    message.rustPeakUsed = parseInt(object.rustPeakUsed, 10);
                else if (typeof object.rustPeakUsed === "number")
                    message.rustPeakUsed = object.rustPeakUsed;
                else if (typeof object.rustPeakUsed === "object")
                    message.rustPeakUsed = new $util.LongBits(object.rustPeakUsed.low >>> 0, object.rustPeakUsed.high >>> 0).toNumber(true);
            if (object.totalPeakUsed != null)
                if ($util.Long)
                    (message.totalPeakUsed = $util.Long.fromValue(object.totalPeakUsed)).unsigned = true;
                else if (typeof object.totalPeakUsed === "string")
                    message.totalPeakUsed = parseInt(object.totalPeakUsed, 10);
                else if (typeof object.totalPeakUsed === "number")
                    message.totalPeakUsed = object.totalPeakUsed;
                else if (typeof object.totalPeakUsed === "object")
                    message.totalPeakUsed = new $util.LongBits(object.totalPeakUsed.low >>> 0, object.totalPeakUsed.high >>> 0).toNumber(true);
            return message;
        };

        /**
         * Creates a plain object from a MemoryUsage message. Also converts values to other types if specified.
         * @function toObject
         * @memberof pruntime_rpc.MemoryUsage
         * @static
         * @param {pruntime_rpc.MemoryUsage} message MemoryUsage
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        MemoryUsage.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            var object = {};
            if (options.defaults) {
                if ($util.Long) {
                    var long = new $util.Long(0, 0, true);
                    object.rustUsed = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : long;
                } else
                    object.rustUsed = options.longs === String ? "0" : 0;
                if ($util.Long) {
                    var long = new $util.Long(0, 0, true);
                    object.rustPeakUsed = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : long;
                } else
                    object.rustPeakUsed = options.longs === String ? "0" : 0;
                if ($util.Long) {
                    var long = new $util.Long(0, 0, true);
                    object.totalPeakUsed = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : long;
                } else
                    object.totalPeakUsed = options.longs === String ? "0" : 0;
            }
            if (message.rustUsed != null && message.hasOwnProperty("rustUsed"))
                if (typeof message.rustUsed === "number")
                    object.rustUsed = options.longs === String ? String(message.rustUsed) : message.rustUsed;
                else
                    object.rustUsed = options.longs === String ? $util.Long.prototype.toString.call(message.rustUsed) : options.longs === Number ? new $util.LongBits(message.rustUsed.low >>> 0, message.rustUsed.high >>> 0).toNumber(true) : message.rustUsed;
            if (message.rustPeakUsed != null && message.hasOwnProperty("rustPeakUsed"))
                if (typeof message.rustPeakUsed === "number")
                    object.rustPeakUsed = options.longs === String ? String(message.rustPeakUsed) : message.rustPeakUsed;
                else
                    object.rustPeakUsed = options.longs === String ? $util.Long.prototype.toString.call(message.rustPeakUsed) : options.longs === Number ? new $util.LongBits(message.rustPeakUsed.low >>> 0, message.rustPeakUsed.high >>> 0).toNumber(true) : message.rustPeakUsed;
            if (message.totalPeakUsed != null && message.hasOwnProperty("totalPeakUsed"))
                if (typeof message.totalPeakUsed === "number")
                    object.totalPeakUsed = options.longs === String ? String(message.totalPeakUsed) : message.totalPeakUsed;
                else
                    object.totalPeakUsed = options.longs === String ? $util.Long.prototype.toString.call(message.totalPeakUsed) : options.longs === Number ? new $util.LongBits(message.totalPeakUsed.low >>> 0, message.totalPeakUsed.high >>> 0).toNumber(true) : message.totalPeakUsed;
            return object;
        };

        /**
         * Converts this MemoryUsage to JSON.
         * @function toJSON
         * @memberof pruntime_rpc.MemoryUsage
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        MemoryUsage.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        return MemoryUsage;
    })();

    pruntime_rpc.SyncedTo = (function() {

        /**
         * Properties of a SyncedTo.
         * @memberof pruntime_rpc
         * @interface ISyncedTo
         * @property {number|null} [syncedTo] SyncedTo syncedTo
         */

        /**
         * Constructs a new SyncedTo.
         * @memberof pruntime_rpc
         * @classdesc Represents a SyncedTo.
         * @implements ISyncedTo
         * @constructor
         * @param {pruntime_rpc.ISyncedTo=} [properties] Properties to set
         */
        function SyncedTo(properties) {
            if (properties)
                for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * SyncedTo syncedTo.
         * @member {number} syncedTo
         * @memberof pruntime_rpc.SyncedTo
         * @instance
         */
        SyncedTo.prototype.syncedTo = 0;

        /**
         * Creates a new SyncedTo instance using the specified properties.
         * @function create
         * @memberof pruntime_rpc.SyncedTo
         * @static
         * @param {pruntime_rpc.ISyncedTo=} [properties] Properties to set
         * @returns {pruntime_rpc.SyncedTo} SyncedTo instance
         */
        SyncedTo.create = function create(properties) {
            return new SyncedTo(properties);
        };

        /**
         * Encodes the specified SyncedTo message. Does not implicitly {@link pruntime_rpc.SyncedTo.verify|verify} messages.
         * @function encode
         * @memberof pruntime_rpc.SyncedTo
         * @static
         * @param {pruntime_rpc.ISyncedTo} message SyncedTo message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        SyncedTo.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.syncedTo != null && Object.hasOwnProperty.call(message, "syncedTo"))
                writer.uint32(/* id 1, wireType 0 =*/8).uint32(message.syncedTo);
            return writer;
        };

        /**
         * Encodes the specified SyncedTo message, length delimited. Does not implicitly {@link pruntime_rpc.SyncedTo.verify|verify} messages.
         * @function encodeDelimited
         * @memberof pruntime_rpc.SyncedTo
         * @static
         * @param {pruntime_rpc.ISyncedTo} message SyncedTo message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        SyncedTo.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes a SyncedTo message from the specified reader or buffer.
         * @function decode
         * @memberof pruntime_rpc.SyncedTo
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {pruntime_rpc.SyncedTo} SyncedTo
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        SyncedTo.decode = function decode(reader, length) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            var end = length === undefined ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.SyncedTo();
            while (reader.pos < end) {
                var tag = reader.uint32();
                switch (tag >>> 3) {
                case 1:
                    message.syncedTo = reader.uint32();
                    break;
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a SyncedTo message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof pruntime_rpc.SyncedTo
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {pruntime_rpc.SyncedTo} SyncedTo
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        SyncedTo.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies a SyncedTo message.
         * @function verify
         * @memberof pruntime_rpc.SyncedTo
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        SyncedTo.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            if (message.syncedTo != null && message.hasOwnProperty("syncedTo"))
                if (!$util.isInteger(message.syncedTo))
                    return "syncedTo: integer expected";
            return null;
        };

        /**
         * Creates a SyncedTo message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof pruntime_rpc.SyncedTo
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {pruntime_rpc.SyncedTo} SyncedTo
         */
        SyncedTo.fromObject = function fromObject(object) {
            if (object instanceof $root.pruntime_rpc.SyncedTo)
                return object;
            var message = new $root.pruntime_rpc.SyncedTo();
            if (object.syncedTo != null)
                message.syncedTo = object.syncedTo >>> 0;
            return message;
        };

        /**
         * Creates a plain object from a SyncedTo message. Also converts values to other types if specified.
         * @function toObject
         * @memberof pruntime_rpc.SyncedTo
         * @static
         * @param {pruntime_rpc.SyncedTo} message SyncedTo
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        SyncedTo.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            var object = {};
            if (options.defaults)
                object.syncedTo = 0;
            if (message.syncedTo != null && message.hasOwnProperty("syncedTo"))
                object.syncedTo = message.syncedTo;
            return object;
        };

        /**
         * Converts this SyncedTo to JSON.
         * @function toJSON
         * @memberof pruntime_rpc.SyncedTo
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        SyncedTo.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        return SyncedTo;
    })();

    pruntime_rpc.HeadersToSync = (function() {

        /**
         * Properties of a HeadersToSync.
         * @memberof pruntime_rpc
         * @interface IHeadersToSync
         * @property {Uint8Array|null} [encodedHeaders] HeadersToSync encodedHeaders
         * @property {Uint8Array|null} [encodedAuthoritySetChange] HeadersToSync encodedAuthoritySetChange
         */

        /**
         * Constructs a new HeadersToSync.
         * @memberof pruntime_rpc
         * @classdesc Represents a HeadersToSync.
         * @implements IHeadersToSync
         * @constructor
         * @param {pruntime_rpc.IHeadersToSync=} [properties] Properties to set
         */
        function HeadersToSync(properties) {
            if (properties)
                for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * HeadersToSync encodedHeaders.
         * @member {Uint8Array} encodedHeaders
         * @memberof pruntime_rpc.HeadersToSync
         * @instance
         */
        HeadersToSync.prototype.encodedHeaders = $util.newBuffer([]);

        /**
         * HeadersToSync encodedAuthoritySetChange.
         * @member {Uint8Array|null|undefined} encodedAuthoritySetChange
         * @memberof pruntime_rpc.HeadersToSync
         * @instance
         */
        HeadersToSync.prototype.encodedAuthoritySetChange = null;

        // OneOf field names bound to virtual getters and setters
        var $oneOfFields;

        /**
         * HeadersToSync _encodedAuthoritySetChange.
         * @member {"encodedAuthoritySetChange"|undefined} _encodedAuthoritySetChange
         * @memberof pruntime_rpc.HeadersToSync
         * @instance
         */
        Object.defineProperty(HeadersToSync.prototype, "_encodedAuthoritySetChange", {
            get: $util.oneOfGetter($oneOfFields = ["encodedAuthoritySetChange"]),
            set: $util.oneOfSetter($oneOfFields)
        });

        /**
         * Creates a new HeadersToSync instance using the specified properties.
         * @function create
         * @memberof pruntime_rpc.HeadersToSync
         * @static
         * @param {pruntime_rpc.IHeadersToSync=} [properties] Properties to set
         * @returns {pruntime_rpc.HeadersToSync} HeadersToSync instance
         */
        HeadersToSync.create = function create(properties) {
            return new HeadersToSync(properties);
        };

        /**
         * Encodes the specified HeadersToSync message. Does not implicitly {@link pruntime_rpc.HeadersToSync.verify|verify} messages.
         * @function encode
         * @memberof pruntime_rpc.HeadersToSync
         * @static
         * @param {pruntime_rpc.IHeadersToSync} message HeadersToSync message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        HeadersToSync.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.encodedHeaders != null && Object.hasOwnProperty.call(message, "encodedHeaders"))
                writer.uint32(/* id 1, wireType 2 =*/10).bytes(message.encodedHeaders);
            if (message.encodedAuthoritySetChange != null && Object.hasOwnProperty.call(message, "encodedAuthoritySetChange"))
                writer.uint32(/* id 2, wireType 2 =*/18).bytes(message.encodedAuthoritySetChange);
            return writer;
        };

        /**
         * Encodes the specified HeadersToSync message, length delimited. Does not implicitly {@link pruntime_rpc.HeadersToSync.verify|verify} messages.
         * @function encodeDelimited
         * @memberof pruntime_rpc.HeadersToSync
         * @static
         * @param {pruntime_rpc.IHeadersToSync} message HeadersToSync message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        HeadersToSync.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes a HeadersToSync message from the specified reader or buffer.
         * @function decode
         * @memberof pruntime_rpc.HeadersToSync
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {pruntime_rpc.HeadersToSync} HeadersToSync
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        HeadersToSync.decode = function decode(reader, length) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            var end = length === undefined ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.HeadersToSync();
            while (reader.pos < end) {
                var tag = reader.uint32();
                switch (tag >>> 3) {
                case 1:
                    message.encodedHeaders = reader.bytes();
                    break;
                case 2:
                    message.encodedAuthoritySetChange = reader.bytes();
                    break;
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a HeadersToSync message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof pruntime_rpc.HeadersToSync
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {pruntime_rpc.HeadersToSync} HeadersToSync
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        HeadersToSync.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies a HeadersToSync message.
         * @function verify
         * @memberof pruntime_rpc.HeadersToSync
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        HeadersToSync.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            var properties = {};
            if (message.encodedHeaders != null && message.hasOwnProperty("encodedHeaders"))
                if (!(message.encodedHeaders && typeof message.encodedHeaders.length === "number" || $util.isString(message.encodedHeaders)))
                    return "encodedHeaders: buffer expected";
            if (message.encodedAuthoritySetChange != null && message.hasOwnProperty("encodedAuthoritySetChange")) {
                properties._encodedAuthoritySetChange = 1;
                if (!(message.encodedAuthoritySetChange && typeof message.encodedAuthoritySetChange.length === "number" || $util.isString(message.encodedAuthoritySetChange)))
                    return "encodedAuthoritySetChange: buffer expected";
            }
            return null;
        };

        /**
         * Creates a HeadersToSync message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof pruntime_rpc.HeadersToSync
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {pruntime_rpc.HeadersToSync} HeadersToSync
         */
        HeadersToSync.fromObject = function fromObject(object) {
            if (object instanceof $root.pruntime_rpc.HeadersToSync)
                return object;
            var message = new $root.pruntime_rpc.HeadersToSync();
            if (object.encodedHeaders != null)
                if (typeof object.encodedHeaders === "string")
                    $util.base64.decode(object.encodedHeaders, message.encodedHeaders = $util.newBuffer($util.base64.length(object.encodedHeaders)), 0);
                else if (object.encodedHeaders.length)
                    message.encodedHeaders = object.encodedHeaders;
            if (object.encodedAuthoritySetChange != null)
                if (typeof object.encodedAuthoritySetChange === "string")
                    $util.base64.decode(object.encodedAuthoritySetChange, message.encodedAuthoritySetChange = $util.newBuffer($util.base64.length(object.encodedAuthoritySetChange)), 0);
                else if (object.encodedAuthoritySetChange.length)
                    message.encodedAuthoritySetChange = object.encodedAuthoritySetChange;
            return message;
        };

        /**
         * Creates a plain object from a HeadersToSync message. Also converts values to other types if specified.
         * @function toObject
         * @memberof pruntime_rpc.HeadersToSync
         * @static
         * @param {pruntime_rpc.HeadersToSync} message HeadersToSync
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        HeadersToSync.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            var object = {};
            if (options.defaults)
                if (options.bytes === String)
                    object.encodedHeaders = "";
                else {
                    object.encodedHeaders = [];
                    if (options.bytes !== Array)
                        object.encodedHeaders = $util.newBuffer(object.encodedHeaders);
                }
            if (message.encodedHeaders != null && message.hasOwnProperty("encodedHeaders"))
                object.encodedHeaders = options.bytes === String ? $util.base64.encode(message.encodedHeaders, 0, message.encodedHeaders.length) : options.bytes === Array ? Array.prototype.slice.call(message.encodedHeaders) : message.encodedHeaders;
            if (message.encodedAuthoritySetChange != null && message.hasOwnProperty("encodedAuthoritySetChange")) {
                object.encodedAuthoritySetChange = options.bytes === String ? $util.base64.encode(message.encodedAuthoritySetChange, 0, message.encodedAuthoritySetChange.length) : options.bytes === Array ? Array.prototype.slice.call(message.encodedAuthoritySetChange) : message.encodedAuthoritySetChange;
                if (options.oneofs)
                    object._encodedAuthoritySetChange = "encodedAuthoritySetChange";
            }
            return object;
        };

        /**
         * Converts this HeadersToSync to JSON.
         * @function toJSON
         * @memberof pruntime_rpc.HeadersToSync
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        HeadersToSync.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        return HeadersToSync;
    })();

    pruntime_rpc.ParaHeadersToSync = (function() {

        /**
         * Properties of a ParaHeadersToSync.
         * @memberof pruntime_rpc
         * @interface IParaHeadersToSync
         * @property {Uint8Array|null} [encodedHeaders] ParaHeadersToSync encodedHeaders
         * @property {Array.<Uint8Array>|null} [proof] ParaHeadersToSync proof
         */

        /**
         * Constructs a new ParaHeadersToSync.
         * @memberof pruntime_rpc
         * @classdesc Represents a ParaHeadersToSync.
         * @implements IParaHeadersToSync
         * @constructor
         * @param {pruntime_rpc.IParaHeadersToSync=} [properties] Properties to set
         */
        function ParaHeadersToSync(properties) {
            this.proof = [];
            if (properties)
                for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * ParaHeadersToSync encodedHeaders.
         * @member {Uint8Array} encodedHeaders
         * @memberof pruntime_rpc.ParaHeadersToSync
         * @instance
         */
        ParaHeadersToSync.prototype.encodedHeaders = $util.newBuffer([]);

        /**
         * ParaHeadersToSync proof.
         * @member {Array.<Uint8Array>} proof
         * @memberof pruntime_rpc.ParaHeadersToSync
         * @instance
         */
        ParaHeadersToSync.prototype.proof = $util.emptyArray;

        /**
         * Creates a new ParaHeadersToSync instance using the specified properties.
         * @function create
         * @memberof pruntime_rpc.ParaHeadersToSync
         * @static
         * @param {pruntime_rpc.IParaHeadersToSync=} [properties] Properties to set
         * @returns {pruntime_rpc.ParaHeadersToSync} ParaHeadersToSync instance
         */
        ParaHeadersToSync.create = function create(properties) {
            return new ParaHeadersToSync(properties);
        };

        /**
         * Encodes the specified ParaHeadersToSync message. Does not implicitly {@link pruntime_rpc.ParaHeadersToSync.verify|verify} messages.
         * @function encode
         * @memberof pruntime_rpc.ParaHeadersToSync
         * @static
         * @param {pruntime_rpc.IParaHeadersToSync} message ParaHeadersToSync message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        ParaHeadersToSync.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.encodedHeaders != null && Object.hasOwnProperty.call(message, "encodedHeaders"))
                writer.uint32(/* id 1, wireType 2 =*/10).bytes(message.encodedHeaders);
            if (message.proof != null && message.proof.length)
                for (var i = 0; i < message.proof.length; ++i)
                    writer.uint32(/* id 2, wireType 2 =*/18).bytes(message.proof[i]);
            return writer;
        };

        /**
         * Encodes the specified ParaHeadersToSync message, length delimited. Does not implicitly {@link pruntime_rpc.ParaHeadersToSync.verify|verify} messages.
         * @function encodeDelimited
         * @memberof pruntime_rpc.ParaHeadersToSync
         * @static
         * @param {pruntime_rpc.IParaHeadersToSync} message ParaHeadersToSync message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        ParaHeadersToSync.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes a ParaHeadersToSync message from the specified reader or buffer.
         * @function decode
         * @memberof pruntime_rpc.ParaHeadersToSync
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {pruntime_rpc.ParaHeadersToSync} ParaHeadersToSync
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        ParaHeadersToSync.decode = function decode(reader, length) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            var end = length === undefined ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.ParaHeadersToSync();
            while (reader.pos < end) {
                var tag = reader.uint32();
                switch (tag >>> 3) {
                case 1:
                    message.encodedHeaders = reader.bytes();
                    break;
                case 2:
                    if (!(message.proof && message.proof.length))
                        message.proof = [];
                    message.proof.push(reader.bytes());
                    break;
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a ParaHeadersToSync message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof pruntime_rpc.ParaHeadersToSync
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {pruntime_rpc.ParaHeadersToSync} ParaHeadersToSync
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        ParaHeadersToSync.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies a ParaHeadersToSync message.
         * @function verify
         * @memberof pruntime_rpc.ParaHeadersToSync
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        ParaHeadersToSync.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            if (message.encodedHeaders != null && message.hasOwnProperty("encodedHeaders"))
                if (!(message.encodedHeaders && typeof message.encodedHeaders.length === "number" || $util.isString(message.encodedHeaders)))
                    return "encodedHeaders: buffer expected";
            if (message.proof != null && message.hasOwnProperty("proof")) {
                if (!Array.isArray(message.proof))
                    return "proof: array expected";
                for (var i = 0; i < message.proof.length; ++i)
                    if (!(message.proof[i] && typeof message.proof[i].length === "number" || $util.isString(message.proof[i])))
                        return "proof: buffer[] expected";
            }
            return null;
        };

        /**
         * Creates a ParaHeadersToSync message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof pruntime_rpc.ParaHeadersToSync
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {pruntime_rpc.ParaHeadersToSync} ParaHeadersToSync
         */
        ParaHeadersToSync.fromObject = function fromObject(object) {
            if (object instanceof $root.pruntime_rpc.ParaHeadersToSync)
                return object;
            var message = new $root.pruntime_rpc.ParaHeadersToSync();
            if (object.encodedHeaders != null)
                if (typeof object.encodedHeaders === "string")
                    $util.base64.decode(object.encodedHeaders, message.encodedHeaders = $util.newBuffer($util.base64.length(object.encodedHeaders)), 0);
                else if (object.encodedHeaders.length)
                    message.encodedHeaders = object.encodedHeaders;
            if (object.proof) {
                if (!Array.isArray(object.proof))
                    throw TypeError(".pruntime_rpc.ParaHeadersToSync.proof: array expected");
                message.proof = [];
                for (var i = 0; i < object.proof.length; ++i)
                    if (typeof object.proof[i] === "string")
                        $util.base64.decode(object.proof[i], message.proof[i] = $util.newBuffer($util.base64.length(object.proof[i])), 0);
                    else if (object.proof[i].length)
                        message.proof[i] = object.proof[i];
            }
            return message;
        };

        /**
         * Creates a plain object from a ParaHeadersToSync message. Also converts values to other types if specified.
         * @function toObject
         * @memberof pruntime_rpc.ParaHeadersToSync
         * @static
         * @param {pruntime_rpc.ParaHeadersToSync} message ParaHeadersToSync
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        ParaHeadersToSync.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            var object = {};
            if (options.arrays || options.defaults)
                object.proof = [];
            if (options.defaults)
                if (options.bytes === String)
                    object.encodedHeaders = "";
                else {
                    object.encodedHeaders = [];
                    if (options.bytes !== Array)
                        object.encodedHeaders = $util.newBuffer(object.encodedHeaders);
                }
            if (message.encodedHeaders != null && message.hasOwnProperty("encodedHeaders"))
                object.encodedHeaders = options.bytes === String ? $util.base64.encode(message.encodedHeaders, 0, message.encodedHeaders.length) : options.bytes === Array ? Array.prototype.slice.call(message.encodedHeaders) : message.encodedHeaders;
            if (message.proof && message.proof.length) {
                object.proof = [];
                for (var j = 0; j < message.proof.length; ++j)
                    object.proof[j] = options.bytes === String ? $util.base64.encode(message.proof[j], 0, message.proof[j].length) : options.bytes === Array ? Array.prototype.slice.call(message.proof[j]) : message.proof[j];
            }
            return object;
        };

        /**
         * Converts this ParaHeadersToSync to JSON.
         * @function toJSON
         * @memberof pruntime_rpc.ParaHeadersToSync
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        ParaHeadersToSync.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        return ParaHeadersToSync;
    })();

    pruntime_rpc.CombinedHeadersToSync = (function() {

        /**
         * Properties of a CombinedHeadersToSync.
         * @memberof pruntime_rpc
         * @interface ICombinedHeadersToSync
         * @property {Uint8Array|null} [encodedRelaychainHeaders] CombinedHeadersToSync encodedRelaychainHeaders
         * @property {Uint8Array|null} [authoritySetChange] CombinedHeadersToSync authoritySetChange
         * @property {Uint8Array|null} [encodedParachainHeaders] CombinedHeadersToSync encodedParachainHeaders
         * @property {Array.<Uint8Array>|null} [proof] CombinedHeadersToSync proof
         */

        /**
         * Constructs a new CombinedHeadersToSync.
         * @memberof pruntime_rpc
         * @classdesc Represents a CombinedHeadersToSync.
         * @implements ICombinedHeadersToSync
         * @constructor
         * @param {pruntime_rpc.ICombinedHeadersToSync=} [properties] Properties to set
         */
        function CombinedHeadersToSync(properties) {
            this.proof = [];
            if (properties)
                for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * CombinedHeadersToSync encodedRelaychainHeaders.
         * @member {Uint8Array} encodedRelaychainHeaders
         * @memberof pruntime_rpc.CombinedHeadersToSync
         * @instance
         */
        CombinedHeadersToSync.prototype.encodedRelaychainHeaders = $util.newBuffer([]);

        /**
         * CombinedHeadersToSync authoritySetChange.
         * @member {Uint8Array|null|undefined} authoritySetChange
         * @memberof pruntime_rpc.CombinedHeadersToSync
         * @instance
         */
        CombinedHeadersToSync.prototype.authoritySetChange = null;

        /**
         * CombinedHeadersToSync encodedParachainHeaders.
         * @member {Uint8Array} encodedParachainHeaders
         * @memberof pruntime_rpc.CombinedHeadersToSync
         * @instance
         */
        CombinedHeadersToSync.prototype.encodedParachainHeaders = $util.newBuffer([]);

        /**
         * CombinedHeadersToSync proof.
         * @member {Array.<Uint8Array>} proof
         * @memberof pruntime_rpc.CombinedHeadersToSync
         * @instance
         */
        CombinedHeadersToSync.prototype.proof = $util.emptyArray;

        // OneOf field names bound to virtual getters and setters
        var $oneOfFields;

        /**
         * CombinedHeadersToSync _authoritySetChange.
         * @member {"authoritySetChange"|undefined} _authoritySetChange
         * @memberof pruntime_rpc.CombinedHeadersToSync
         * @instance
         */
        Object.defineProperty(CombinedHeadersToSync.prototype, "_authoritySetChange", {
            get: $util.oneOfGetter($oneOfFields = ["authoritySetChange"]),
            set: $util.oneOfSetter($oneOfFields)
        });

        /**
         * Creates a new CombinedHeadersToSync instance using the specified properties.
         * @function create
         * @memberof pruntime_rpc.CombinedHeadersToSync
         * @static
         * @param {pruntime_rpc.ICombinedHeadersToSync=} [properties] Properties to set
         * @returns {pruntime_rpc.CombinedHeadersToSync} CombinedHeadersToSync instance
         */
        CombinedHeadersToSync.create = function create(properties) {
            return new CombinedHeadersToSync(properties);
        };

        /**
         * Encodes the specified CombinedHeadersToSync message. Does not implicitly {@link pruntime_rpc.CombinedHeadersToSync.verify|verify} messages.
         * @function encode
         * @memberof pruntime_rpc.CombinedHeadersToSync
         * @static
         * @param {pruntime_rpc.ICombinedHeadersToSync} message CombinedHeadersToSync message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        CombinedHeadersToSync.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.encodedRelaychainHeaders != null && Object.hasOwnProperty.call(message, "encodedRelaychainHeaders"))
                writer.uint32(/* id 1, wireType 2 =*/10).bytes(message.encodedRelaychainHeaders);
            if (message.authoritySetChange != null && Object.hasOwnProperty.call(message, "authoritySetChange"))
                writer.uint32(/* id 2, wireType 2 =*/18).bytes(message.authoritySetChange);
            if (message.encodedParachainHeaders != null && Object.hasOwnProperty.call(message, "encodedParachainHeaders"))
                writer.uint32(/* id 3, wireType 2 =*/26).bytes(message.encodedParachainHeaders);
            if (message.proof != null && message.proof.length)
                for (var i = 0; i < message.proof.length; ++i)
                    writer.uint32(/* id 4, wireType 2 =*/34).bytes(message.proof[i]);
            return writer;
        };

        /**
         * Encodes the specified CombinedHeadersToSync message, length delimited. Does not implicitly {@link pruntime_rpc.CombinedHeadersToSync.verify|verify} messages.
         * @function encodeDelimited
         * @memberof pruntime_rpc.CombinedHeadersToSync
         * @static
         * @param {pruntime_rpc.ICombinedHeadersToSync} message CombinedHeadersToSync message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        CombinedHeadersToSync.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes a CombinedHeadersToSync message from the specified reader or buffer.
         * @function decode
         * @memberof pruntime_rpc.CombinedHeadersToSync
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {pruntime_rpc.CombinedHeadersToSync} CombinedHeadersToSync
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        CombinedHeadersToSync.decode = function decode(reader, length) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            var end = length === undefined ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.CombinedHeadersToSync();
            while (reader.pos < end) {
                var tag = reader.uint32();
                switch (tag >>> 3) {
                case 1:
                    message.encodedRelaychainHeaders = reader.bytes();
                    break;
                case 2:
                    message.authoritySetChange = reader.bytes();
                    break;
                case 3:
                    message.encodedParachainHeaders = reader.bytes();
                    break;
                case 4:
                    if (!(message.proof && message.proof.length))
                        message.proof = [];
                    message.proof.push(reader.bytes());
                    break;
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a CombinedHeadersToSync message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof pruntime_rpc.CombinedHeadersToSync
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {pruntime_rpc.CombinedHeadersToSync} CombinedHeadersToSync
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        CombinedHeadersToSync.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies a CombinedHeadersToSync message.
         * @function verify
         * @memberof pruntime_rpc.CombinedHeadersToSync
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        CombinedHeadersToSync.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            var properties = {};
            if (message.encodedRelaychainHeaders != null && message.hasOwnProperty("encodedRelaychainHeaders"))
                if (!(message.encodedRelaychainHeaders && typeof message.encodedRelaychainHeaders.length === "number" || $util.isString(message.encodedRelaychainHeaders)))
                    return "encodedRelaychainHeaders: buffer expected";
            if (message.authoritySetChange != null && message.hasOwnProperty("authoritySetChange")) {
                properties._authoritySetChange = 1;
                if (!(message.authoritySetChange && typeof message.authoritySetChange.length === "number" || $util.isString(message.authoritySetChange)))
                    return "authoritySetChange: buffer expected";
            }
            if (message.encodedParachainHeaders != null && message.hasOwnProperty("encodedParachainHeaders"))
                if (!(message.encodedParachainHeaders && typeof message.encodedParachainHeaders.length === "number" || $util.isString(message.encodedParachainHeaders)))
                    return "encodedParachainHeaders: buffer expected";
            if (message.proof != null && message.hasOwnProperty("proof")) {
                if (!Array.isArray(message.proof))
                    return "proof: array expected";
                for (var i = 0; i < message.proof.length; ++i)
                    if (!(message.proof[i] && typeof message.proof[i].length === "number" || $util.isString(message.proof[i])))
                        return "proof: buffer[] expected";
            }
            return null;
        };

        /**
         * Creates a CombinedHeadersToSync message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof pruntime_rpc.CombinedHeadersToSync
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {pruntime_rpc.CombinedHeadersToSync} CombinedHeadersToSync
         */
        CombinedHeadersToSync.fromObject = function fromObject(object) {
            if (object instanceof $root.pruntime_rpc.CombinedHeadersToSync)
                return object;
            var message = new $root.pruntime_rpc.CombinedHeadersToSync();
            if (object.encodedRelaychainHeaders != null)
                if (typeof object.encodedRelaychainHeaders === "string")
                    $util.base64.decode(object.encodedRelaychainHeaders, message.encodedRelaychainHeaders = $util.newBuffer($util.base64.length(object.encodedRelaychainHeaders)), 0);
                else if (object.encodedRelaychainHeaders.length)
                    message.encodedRelaychainHeaders = object.encodedRelaychainHeaders;
            if (object.authoritySetChange != null)
                if (typeof object.authoritySetChange === "string")
                    $util.base64.decode(object.authoritySetChange, message.authoritySetChange = $util.newBuffer($util.base64.length(object.authoritySetChange)), 0);
                else if (object.authoritySetChange.length)
                    message.authoritySetChange = object.authoritySetChange;
            if (object.encodedParachainHeaders != null)
                if (typeof object.encodedParachainHeaders === "string")
                    $util.base64.decode(object.encodedParachainHeaders, message.encodedParachainHeaders = $util.newBuffer($util.base64.length(object.encodedParachainHeaders)), 0);
                else if (object.encodedParachainHeaders.length)
                    message.encodedParachainHeaders = object.encodedParachainHeaders;
            if (object.proof) {
                if (!Array.isArray(object.proof))
                    throw TypeError(".pruntime_rpc.CombinedHeadersToSync.proof: array expected");
                message.proof = [];
                for (var i = 0; i < object.proof.length; ++i)
                    if (typeof object.proof[i] === "string")
                        $util.base64.decode(object.proof[i], message.proof[i] = $util.newBuffer($util.base64.length(object.proof[i])), 0);
                    else if (object.proof[i].length)
                        message.proof[i] = object.proof[i];
            }
            return message;
        };

        /**
         * Creates a plain object from a CombinedHeadersToSync message. Also converts values to other types if specified.
         * @function toObject
         * @memberof pruntime_rpc.CombinedHeadersToSync
         * @static
         * @param {pruntime_rpc.CombinedHeadersToSync} message CombinedHeadersToSync
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        CombinedHeadersToSync.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            var object = {};
            if (options.arrays || options.defaults)
                object.proof = [];
            if (options.defaults) {
                if (options.bytes === String)
                    object.encodedRelaychainHeaders = "";
                else {
                    object.encodedRelaychainHeaders = [];
                    if (options.bytes !== Array)
                        object.encodedRelaychainHeaders = $util.newBuffer(object.encodedRelaychainHeaders);
                }
                if (options.bytes === String)
                    object.encodedParachainHeaders = "";
                else {
                    object.encodedParachainHeaders = [];
                    if (options.bytes !== Array)
                        object.encodedParachainHeaders = $util.newBuffer(object.encodedParachainHeaders);
                }
            }
            if (message.encodedRelaychainHeaders != null && message.hasOwnProperty("encodedRelaychainHeaders"))
                object.encodedRelaychainHeaders = options.bytes === String ? $util.base64.encode(message.encodedRelaychainHeaders, 0, message.encodedRelaychainHeaders.length) : options.bytes === Array ? Array.prototype.slice.call(message.encodedRelaychainHeaders) : message.encodedRelaychainHeaders;
            if (message.authoritySetChange != null && message.hasOwnProperty("authoritySetChange")) {
                object.authoritySetChange = options.bytes === String ? $util.base64.encode(message.authoritySetChange, 0, message.authoritySetChange.length) : options.bytes === Array ? Array.prototype.slice.call(message.authoritySetChange) : message.authoritySetChange;
                if (options.oneofs)
                    object._authoritySetChange = "authoritySetChange";
            }
            if (message.encodedParachainHeaders != null && message.hasOwnProperty("encodedParachainHeaders"))
                object.encodedParachainHeaders = options.bytes === String ? $util.base64.encode(message.encodedParachainHeaders, 0, message.encodedParachainHeaders.length) : options.bytes === Array ? Array.prototype.slice.call(message.encodedParachainHeaders) : message.encodedParachainHeaders;
            if (message.proof && message.proof.length) {
                object.proof = [];
                for (var j = 0; j < message.proof.length; ++j)
                    object.proof[j] = options.bytes === String ? $util.base64.encode(message.proof[j], 0, message.proof[j].length) : options.bytes === Array ? Array.prototype.slice.call(message.proof[j]) : message.proof[j];
            }
            return object;
        };

        /**
         * Converts this CombinedHeadersToSync to JSON.
         * @function toJSON
         * @memberof pruntime_rpc.CombinedHeadersToSync
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        CombinedHeadersToSync.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        return CombinedHeadersToSync;
    })();

    pruntime_rpc.HeadersSyncedTo = (function() {

        /**
         * Properties of a HeadersSyncedTo.
         * @memberof pruntime_rpc
         * @interface IHeadersSyncedTo
         * @property {number|null} [relaychainSyncedTo] HeadersSyncedTo relaychainSyncedTo
         * @property {number|null} [parachainSyncedTo] HeadersSyncedTo parachainSyncedTo
         */

        /**
         * Constructs a new HeadersSyncedTo.
         * @memberof pruntime_rpc
         * @classdesc Represents a HeadersSyncedTo.
         * @implements IHeadersSyncedTo
         * @constructor
         * @param {pruntime_rpc.IHeadersSyncedTo=} [properties] Properties to set
         */
        function HeadersSyncedTo(properties) {
            if (properties)
                for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * HeadersSyncedTo relaychainSyncedTo.
         * @member {number} relaychainSyncedTo
         * @memberof pruntime_rpc.HeadersSyncedTo
         * @instance
         */
        HeadersSyncedTo.prototype.relaychainSyncedTo = 0;

        /**
         * HeadersSyncedTo parachainSyncedTo.
         * @member {number} parachainSyncedTo
         * @memberof pruntime_rpc.HeadersSyncedTo
         * @instance
         */
        HeadersSyncedTo.prototype.parachainSyncedTo = 0;

        /**
         * Creates a new HeadersSyncedTo instance using the specified properties.
         * @function create
         * @memberof pruntime_rpc.HeadersSyncedTo
         * @static
         * @param {pruntime_rpc.IHeadersSyncedTo=} [properties] Properties to set
         * @returns {pruntime_rpc.HeadersSyncedTo} HeadersSyncedTo instance
         */
        HeadersSyncedTo.create = function create(properties) {
            return new HeadersSyncedTo(properties);
        };

        /**
         * Encodes the specified HeadersSyncedTo message. Does not implicitly {@link pruntime_rpc.HeadersSyncedTo.verify|verify} messages.
         * @function encode
         * @memberof pruntime_rpc.HeadersSyncedTo
         * @static
         * @param {pruntime_rpc.IHeadersSyncedTo} message HeadersSyncedTo message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        HeadersSyncedTo.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.relaychainSyncedTo != null && Object.hasOwnProperty.call(message, "relaychainSyncedTo"))
                writer.uint32(/* id 1, wireType 0 =*/8).uint32(message.relaychainSyncedTo);
            if (message.parachainSyncedTo != null && Object.hasOwnProperty.call(message, "parachainSyncedTo"))
                writer.uint32(/* id 2, wireType 0 =*/16).uint32(message.parachainSyncedTo);
            return writer;
        };

        /**
         * Encodes the specified HeadersSyncedTo message, length delimited. Does not implicitly {@link pruntime_rpc.HeadersSyncedTo.verify|verify} messages.
         * @function encodeDelimited
         * @memberof pruntime_rpc.HeadersSyncedTo
         * @static
         * @param {pruntime_rpc.IHeadersSyncedTo} message HeadersSyncedTo message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        HeadersSyncedTo.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes a HeadersSyncedTo message from the specified reader or buffer.
         * @function decode
         * @memberof pruntime_rpc.HeadersSyncedTo
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {pruntime_rpc.HeadersSyncedTo} HeadersSyncedTo
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        HeadersSyncedTo.decode = function decode(reader, length) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            var end = length === undefined ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.HeadersSyncedTo();
            while (reader.pos < end) {
                var tag = reader.uint32();
                switch (tag >>> 3) {
                case 1:
                    message.relaychainSyncedTo = reader.uint32();
                    break;
                case 2:
                    message.parachainSyncedTo = reader.uint32();
                    break;
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a HeadersSyncedTo message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof pruntime_rpc.HeadersSyncedTo
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {pruntime_rpc.HeadersSyncedTo} HeadersSyncedTo
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        HeadersSyncedTo.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies a HeadersSyncedTo message.
         * @function verify
         * @memberof pruntime_rpc.HeadersSyncedTo
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        HeadersSyncedTo.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            if (message.relaychainSyncedTo != null && message.hasOwnProperty("relaychainSyncedTo"))
                if (!$util.isInteger(message.relaychainSyncedTo))
                    return "relaychainSyncedTo: integer expected";
            if (message.parachainSyncedTo != null && message.hasOwnProperty("parachainSyncedTo"))
                if (!$util.isInteger(message.parachainSyncedTo))
                    return "parachainSyncedTo: integer expected";
            return null;
        };

        /**
         * Creates a HeadersSyncedTo message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof pruntime_rpc.HeadersSyncedTo
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {pruntime_rpc.HeadersSyncedTo} HeadersSyncedTo
         */
        HeadersSyncedTo.fromObject = function fromObject(object) {
            if (object instanceof $root.pruntime_rpc.HeadersSyncedTo)
                return object;
            var message = new $root.pruntime_rpc.HeadersSyncedTo();
            if (object.relaychainSyncedTo != null)
                message.relaychainSyncedTo = object.relaychainSyncedTo >>> 0;
            if (object.parachainSyncedTo != null)
                message.parachainSyncedTo = object.parachainSyncedTo >>> 0;
            return message;
        };

        /**
         * Creates a plain object from a HeadersSyncedTo message. Also converts values to other types if specified.
         * @function toObject
         * @memberof pruntime_rpc.HeadersSyncedTo
         * @static
         * @param {pruntime_rpc.HeadersSyncedTo} message HeadersSyncedTo
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        HeadersSyncedTo.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            var object = {};
            if (options.defaults) {
                object.relaychainSyncedTo = 0;
                object.parachainSyncedTo = 0;
            }
            if (message.relaychainSyncedTo != null && message.hasOwnProperty("relaychainSyncedTo"))
                object.relaychainSyncedTo = message.relaychainSyncedTo;
            if (message.parachainSyncedTo != null && message.hasOwnProperty("parachainSyncedTo"))
                object.parachainSyncedTo = message.parachainSyncedTo;
            return object;
        };

        /**
         * Converts this HeadersSyncedTo to JSON.
         * @function toJSON
         * @memberof pruntime_rpc.HeadersSyncedTo
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        HeadersSyncedTo.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        return HeadersSyncedTo;
    })();

    pruntime_rpc.Blocks = (function() {

        /**
         * Properties of a Blocks.
         * @memberof pruntime_rpc
         * @interface IBlocks
         * @property {Uint8Array|null} [encodedBlocks] Blocks encodedBlocks
         */

        /**
         * Constructs a new Blocks.
         * @memberof pruntime_rpc
         * @classdesc Represents a Blocks.
         * @implements IBlocks
         * @constructor
         * @param {pruntime_rpc.IBlocks=} [properties] Properties to set
         */
        function Blocks(properties) {
            if (properties)
                for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * Blocks encodedBlocks.
         * @member {Uint8Array} encodedBlocks
         * @memberof pruntime_rpc.Blocks
         * @instance
         */
        Blocks.prototype.encodedBlocks = $util.newBuffer([]);

        /**
         * Creates a new Blocks instance using the specified properties.
         * @function create
         * @memberof pruntime_rpc.Blocks
         * @static
         * @param {pruntime_rpc.IBlocks=} [properties] Properties to set
         * @returns {pruntime_rpc.Blocks} Blocks instance
         */
        Blocks.create = function create(properties) {
            return new Blocks(properties);
        };

        /**
         * Encodes the specified Blocks message. Does not implicitly {@link pruntime_rpc.Blocks.verify|verify} messages.
         * @function encode
         * @memberof pruntime_rpc.Blocks
         * @static
         * @param {pruntime_rpc.IBlocks} message Blocks message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        Blocks.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.encodedBlocks != null && Object.hasOwnProperty.call(message, "encodedBlocks"))
                writer.uint32(/* id 1, wireType 2 =*/10).bytes(message.encodedBlocks);
            return writer;
        };

        /**
         * Encodes the specified Blocks message, length delimited. Does not implicitly {@link pruntime_rpc.Blocks.verify|verify} messages.
         * @function encodeDelimited
         * @memberof pruntime_rpc.Blocks
         * @static
         * @param {pruntime_rpc.IBlocks} message Blocks message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        Blocks.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes a Blocks message from the specified reader or buffer.
         * @function decode
         * @memberof pruntime_rpc.Blocks
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {pruntime_rpc.Blocks} Blocks
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        Blocks.decode = function decode(reader, length) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            var end = length === undefined ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.Blocks();
            while (reader.pos < end) {
                var tag = reader.uint32();
                switch (tag >>> 3) {
                case 1:
                    message.encodedBlocks = reader.bytes();
                    break;
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a Blocks message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof pruntime_rpc.Blocks
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {pruntime_rpc.Blocks} Blocks
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        Blocks.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies a Blocks message.
         * @function verify
         * @memberof pruntime_rpc.Blocks
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        Blocks.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            if (message.encodedBlocks != null && message.hasOwnProperty("encodedBlocks"))
                if (!(message.encodedBlocks && typeof message.encodedBlocks.length === "number" || $util.isString(message.encodedBlocks)))
                    return "encodedBlocks: buffer expected";
            return null;
        };

        /**
         * Creates a Blocks message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof pruntime_rpc.Blocks
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {pruntime_rpc.Blocks} Blocks
         */
        Blocks.fromObject = function fromObject(object) {
            if (object instanceof $root.pruntime_rpc.Blocks)
                return object;
            var message = new $root.pruntime_rpc.Blocks();
            if (object.encodedBlocks != null)
                if (typeof object.encodedBlocks === "string")
                    $util.base64.decode(object.encodedBlocks, message.encodedBlocks = $util.newBuffer($util.base64.length(object.encodedBlocks)), 0);
                else if (object.encodedBlocks.length)
                    message.encodedBlocks = object.encodedBlocks;
            return message;
        };

        /**
         * Creates a plain object from a Blocks message. Also converts values to other types if specified.
         * @function toObject
         * @memberof pruntime_rpc.Blocks
         * @static
         * @param {pruntime_rpc.Blocks} message Blocks
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        Blocks.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            var object = {};
            if (options.defaults)
                if (options.bytes === String)
                    object.encodedBlocks = "";
                else {
                    object.encodedBlocks = [];
                    if (options.bytes !== Array)
                        object.encodedBlocks = $util.newBuffer(object.encodedBlocks);
                }
            if (message.encodedBlocks != null && message.hasOwnProperty("encodedBlocks"))
                object.encodedBlocks = options.bytes === String ? $util.base64.encode(message.encodedBlocks, 0, message.encodedBlocks.length) : options.bytes === Array ? Array.prototype.slice.call(message.encodedBlocks) : message.encodedBlocks;
            return object;
        };

        /**
         * Converts this Blocks to JSON.
         * @function toJSON
         * @memberof pruntime_rpc.Blocks
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        Blocks.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        return Blocks;
    })();

    pruntime_rpc.InitRuntimeRequest = (function() {

        /**
         * Properties of an InitRuntimeRequest.
         * @memberof pruntime_rpc
         * @interface IInitRuntimeRequest
         * @property {boolean|null} [skipRa] InitRuntimeRequest skipRa
         * @property {Uint8Array|null} [encodedGenesisInfo] InitRuntimeRequest encodedGenesisInfo
         * @property {Uint8Array|null} [debugSetKey] InitRuntimeRequest debugSetKey
         * @property {Uint8Array|null} [encodedGenesisState] InitRuntimeRequest encodedGenesisState
         * @property {Uint8Array|null} [encodedOperator] InitRuntimeRequest encodedOperator
         * @property {boolean|null} [isParachain] InitRuntimeRequest isParachain
         * @property {Uint8Array|null} [attestationProvider] InitRuntimeRequest attestationProvider
         */

        /**
         * Constructs a new InitRuntimeRequest.
         * @memberof pruntime_rpc
         * @classdesc Represents an InitRuntimeRequest.
         * @implements IInitRuntimeRequest
         * @constructor
         * @param {pruntime_rpc.IInitRuntimeRequest=} [properties] Properties to set
         */
        function InitRuntimeRequest(properties) {
            if (properties)
                for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * InitRuntimeRequest skipRa.
         * @member {boolean} skipRa
         * @memberof pruntime_rpc.InitRuntimeRequest
         * @instance
         */
        InitRuntimeRequest.prototype.skipRa = false;

        /**
         * InitRuntimeRequest encodedGenesisInfo.
         * @member {Uint8Array} encodedGenesisInfo
         * @memberof pruntime_rpc.InitRuntimeRequest
         * @instance
         */
        InitRuntimeRequest.prototype.encodedGenesisInfo = $util.newBuffer([]);

        /**
         * InitRuntimeRequest debugSetKey.
         * @member {Uint8Array|null|undefined} debugSetKey
         * @memberof pruntime_rpc.InitRuntimeRequest
         * @instance
         */
        InitRuntimeRequest.prototype.debugSetKey = null;

        /**
         * InitRuntimeRequest encodedGenesisState.
         * @member {Uint8Array} encodedGenesisState
         * @memberof pruntime_rpc.InitRuntimeRequest
         * @instance
         */
        InitRuntimeRequest.prototype.encodedGenesisState = $util.newBuffer([]);

        /**
         * InitRuntimeRequest encodedOperator.
         * @member {Uint8Array|null|undefined} encodedOperator
         * @memberof pruntime_rpc.InitRuntimeRequest
         * @instance
         */
        InitRuntimeRequest.prototype.encodedOperator = null;

        /**
         * InitRuntimeRequest isParachain.
         * @member {boolean} isParachain
         * @memberof pruntime_rpc.InitRuntimeRequest
         * @instance
         */
        InitRuntimeRequest.prototype.isParachain = false;

        /**
         * InitRuntimeRequest attestationProvider.
         * @member {Uint8Array|null|undefined} attestationProvider
         * @memberof pruntime_rpc.InitRuntimeRequest
         * @instance
         */
        InitRuntimeRequest.prototype.attestationProvider = null;

        // OneOf field names bound to virtual getters and setters
        var $oneOfFields;

        /**
         * InitRuntimeRequest _debugSetKey.
         * @member {"debugSetKey"|undefined} _debugSetKey
         * @memberof pruntime_rpc.InitRuntimeRequest
         * @instance
         */
        Object.defineProperty(InitRuntimeRequest.prototype, "_debugSetKey", {
            get: $util.oneOfGetter($oneOfFields = ["debugSetKey"]),
            set: $util.oneOfSetter($oneOfFields)
        });

        /**
         * InitRuntimeRequest _encodedOperator.
         * @member {"encodedOperator"|undefined} _encodedOperator
         * @memberof pruntime_rpc.InitRuntimeRequest
         * @instance
         */
        Object.defineProperty(InitRuntimeRequest.prototype, "_encodedOperator", {
            get: $util.oneOfGetter($oneOfFields = ["encodedOperator"]),
            set: $util.oneOfSetter($oneOfFields)
        });

        /**
         * InitRuntimeRequest _attestationProvider.
         * @member {"attestationProvider"|undefined} _attestationProvider
         * @memberof pruntime_rpc.InitRuntimeRequest
         * @instance
         */
        Object.defineProperty(InitRuntimeRequest.prototype, "_attestationProvider", {
            get: $util.oneOfGetter($oneOfFields = ["attestationProvider"]),
            set: $util.oneOfSetter($oneOfFields)
        });

        /**
         * Creates a new InitRuntimeRequest instance using the specified properties.
         * @function create
         * @memberof pruntime_rpc.InitRuntimeRequest
         * @static
         * @param {pruntime_rpc.IInitRuntimeRequest=} [properties] Properties to set
         * @returns {pruntime_rpc.InitRuntimeRequest} InitRuntimeRequest instance
         */
        InitRuntimeRequest.create = function create(properties) {
            return new InitRuntimeRequest(properties);
        };

        /**
         * Encodes the specified InitRuntimeRequest message. Does not implicitly {@link pruntime_rpc.InitRuntimeRequest.verify|verify} messages.
         * @function encode
         * @memberof pruntime_rpc.InitRuntimeRequest
         * @static
         * @param {pruntime_rpc.IInitRuntimeRequest} message InitRuntimeRequest message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        InitRuntimeRequest.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.skipRa != null && Object.hasOwnProperty.call(message, "skipRa"))
                writer.uint32(/* id 1, wireType 0 =*/8).bool(message.skipRa);
            if (message.encodedGenesisInfo != null && Object.hasOwnProperty.call(message, "encodedGenesisInfo"))
                writer.uint32(/* id 2, wireType 2 =*/18).bytes(message.encodedGenesisInfo);
            if (message.debugSetKey != null && Object.hasOwnProperty.call(message, "debugSetKey"))
                writer.uint32(/* id 3, wireType 2 =*/26).bytes(message.debugSetKey);
            if (message.encodedGenesisState != null && Object.hasOwnProperty.call(message, "encodedGenesisState"))
                writer.uint32(/* id 4, wireType 2 =*/34).bytes(message.encodedGenesisState);
            if (message.encodedOperator != null && Object.hasOwnProperty.call(message, "encodedOperator"))
                writer.uint32(/* id 5, wireType 2 =*/42).bytes(message.encodedOperator);
            if (message.isParachain != null && Object.hasOwnProperty.call(message, "isParachain"))
                writer.uint32(/* id 6, wireType 0 =*/48).bool(message.isParachain);
            if (message.attestationProvider != null && Object.hasOwnProperty.call(message, "attestationProvider"))
                writer.uint32(/* id 7, wireType 2 =*/58).bytes(message.attestationProvider);
            return writer;
        };

        /**
         * Encodes the specified InitRuntimeRequest message, length delimited. Does not implicitly {@link pruntime_rpc.InitRuntimeRequest.verify|verify} messages.
         * @function encodeDelimited
         * @memberof pruntime_rpc.InitRuntimeRequest
         * @static
         * @param {pruntime_rpc.IInitRuntimeRequest} message InitRuntimeRequest message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        InitRuntimeRequest.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes an InitRuntimeRequest message from the specified reader or buffer.
         * @function decode
         * @memberof pruntime_rpc.InitRuntimeRequest
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {pruntime_rpc.InitRuntimeRequest} InitRuntimeRequest
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        InitRuntimeRequest.decode = function decode(reader, length) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            var end = length === undefined ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.InitRuntimeRequest();
            while (reader.pos < end) {
                var tag = reader.uint32();
                switch (tag >>> 3) {
                case 1:
                    message.skipRa = reader.bool();
                    break;
                case 2:
                    message.encodedGenesisInfo = reader.bytes();
                    break;
                case 3:
                    message.debugSetKey = reader.bytes();
                    break;
                case 4:
                    message.encodedGenesisState = reader.bytes();
                    break;
                case 5:
                    message.encodedOperator = reader.bytes();
                    break;
                case 6:
                    message.isParachain = reader.bool();
                    break;
                case 7:
                    message.attestationProvider = reader.bytes();
                    break;
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes an InitRuntimeRequest message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof pruntime_rpc.InitRuntimeRequest
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {pruntime_rpc.InitRuntimeRequest} InitRuntimeRequest
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        InitRuntimeRequest.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies an InitRuntimeRequest message.
         * @function verify
         * @memberof pruntime_rpc.InitRuntimeRequest
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        InitRuntimeRequest.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            var properties = {};
            if (message.skipRa != null && message.hasOwnProperty("skipRa"))
                if (typeof message.skipRa !== "boolean")
                    return "skipRa: boolean expected";
            if (message.encodedGenesisInfo != null && message.hasOwnProperty("encodedGenesisInfo"))
                if (!(message.encodedGenesisInfo && typeof message.encodedGenesisInfo.length === "number" || $util.isString(message.encodedGenesisInfo)))
                    return "encodedGenesisInfo: buffer expected";
            if (message.debugSetKey != null && message.hasOwnProperty("debugSetKey")) {
                properties._debugSetKey = 1;
                if (!(message.debugSetKey && typeof message.debugSetKey.length === "number" || $util.isString(message.debugSetKey)))
                    return "debugSetKey: buffer expected";
            }
            if (message.encodedGenesisState != null && message.hasOwnProperty("encodedGenesisState"))
                if (!(message.encodedGenesisState && typeof message.encodedGenesisState.length === "number" || $util.isString(message.encodedGenesisState)))
                    return "encodedGenesisState: buffer expected";
            if (message.encodedOperator != null && message.hasOwnProperty("encodedOperator")) {
                properties._encodedOperator = 1;
                if (!(message.encodedOperator && typeof message.encodedOperator.length === "number" || $util.isString(message.encodedOperator)))
                    return "encodedOperator: buffer expected";
            }
            if (message.isParachain != null && message.hasOwnProperty("isParachain"))
                if (typeof message.isParachain !== "boolean")
                    return "isParachain: boolean expected";
            if (message.attestationProvider != null && message.hasOwnProperty("attestationProvider")) {
                properties._attestationProvider = 1;
                if (!(message.attestationProvider && typeof message.attestationProvider.length === "number" || $util.isString(message.attestationProvider)))
                    return "attestationProvider: buffer expected";
            }
            return null;
        };

        /**
         * Creates an InitRuntimeRequest message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof pruntime_rpc.InitRuntimeRequest
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {pruntime_rpc.InitRuntimeRequest} InitRuntimeRequest
         */
        InitRuntimeRequest.fromObject = function fromObject(object) {
            if (object instanceof $root.pruntime_rpc.InitRuntimeRequest)
                return object;
            var message = new $root.pruntime_rpc.InitRuntimeRequest();
            if (object.skipRa != null)
                message.skipRa = Boolean(object.skipRa);
            if (object.encodedGenesisInfo != null)
                if (typeof object.encodedGenesisInfo === "string")
                    $util.base64.decode(object.encodedGenesisInfo, message.encodedGenesisInfo = $util.newBuffer($util.base64.length(object.encodedGenesisInfo)), 0);
                else if (object.encodedGenesisInfo.length)
                    message.encodedGenesisInfo = object.encodedGenesisInfo;
            if (object.debugSetKey != null)
                if (typeof object.debugSetKey === "string")
                    $util.base64.decode(object.debugSetKey, message.debugSetKey = $util.newBuffer($util.base64.length(object.debugSetKey)), 0);
                else if (object.debugSetKey.length)
                    message.debugSetKey = object.debugSetKey;
            if (object.encodedGenesisState != null)
                if (typeof object.encodedGenesisState === "string")
                    $util.base64.decode(object.encodedGenesisState, message.encodedGenesisState = $util.newBuffer($util.base64.length(object.encodedGenesisState)), 0);
                else if (object.encodedGenesisState.length)
                    message.encodedGenesisState = object.encodedGenesisState;
            if (object.encodedOperator != null)
                if (typeof object.encodedOperator === "string")
                    $util.base64.decode(object.encodedOperator, message.encodedOperator = $util.newBuffer($util.base64.length(object.encodedOperator)), 0);
                else if (object.encodedOperator.length)
                    message.encodedOperator = object.encodedOperator;
            if (object.isParachain != null)
                message.isParachain = Boolean(object.isParachain);
            if (object.attestationProvider != null)
                if (typeof object.attestationProvider === "string")
                    $util.base64.decode(object.attestationProvider, message.attestationProvider = $util.newBuffer($util.base64.length(object.attestationProvider)), 0);
                else if (object.attestationProvider.length)
                    message.attestationProvider = object.attestationProvider;
            return message;
        };

        /**
         * Creates a plain object from an InitRuntimeRequest message. Also converts values to other types if specified.
         * @function toObject
         * @memberof pruntime_rpc.InitRuntimeRequest
         * @static
         * @param {pruntime_rpc.InitRuntimeRequest} message InitRuntimeRequest
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        InitRuntimeRequest.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            var object = {};
            if (options.defaults) {
                object.skipRa = false;
                if (options.bytes === String)
                    object.encodedGenesisInfo = "";
                else {
                    object.encodedGenesisInfo = [];
                    if (options.bytes !== Array)
                        object.encodedGenesisInfo = $util.newBuffer(object.encodedGenesisInfo);
                }
                if (options.bytes === String)
                    object.encodedGenesisState = "";
                else {
                    object.encodedGenesisState = [];
                    if (options.bytes !== Array)
                        object.encodedGenesisState = $util.newBuffer(object.encodedGenesisState);
                }
                object.isParachain = false;
            }
            if (message.skipRa != null && message.hasOwnProperty("skipRa"))
                object.skipRa = message.skipRa;
            if (message.encodedGenesisInfo != null && message.hasOwnProperty("encodedGenesisInfo"))
                object.encodedGenesisInfo = options.bytes === String ? $util.base64.encode(message.encodedGenesisInfo, 0, message.encodedGenesisInfo.length) : options.bytes === Array ? Array.prototype.slice.call(message.encodedGenesisInfo) : message.encodedGenesisInfo;
            if (message.debugSetKey != null && message.hasOwnProperty("debugSetKey")) {
                object.debugSetKey = options.bytes === String ? $util.base64.encode(message.debugSetKey, 0, message.debugSetKey.length) : options.bytes === Array ? Array.prototype.slice.call(message.debugSetKey) : message.debugSetKey;
                if (options.oneofs)
                    object._debugSetKey = "debugSetKey";
            }
            if (message.encodedGenesisState != null && message.hasOwnProperty("encodedGenesisState"))
                object.encodedGenesisState = options.bytes === String ? $util.base64.encode(message.encodedGenesisState, 0, message.encodedGenesisState.length) : options.bytes === Array ? Array.prototype.slice.call(message.encodedGenesisState) : message.encodedGenesisState;
            if (message.encodedOperator != null && message.hasOwnProperty("encodedOperator")) {
                object.encodedOperator = options.bytes === String ? $util.base64.encode(message.encodedOperator, 0, message.encodedOperator.length) : options.bytes === Array ? Array.prototype.slice.call(message.encodedOperator) : message.encodedOperator;
                if (options.oneofs)
                    object._encodedOperator = "encodedOperator";
            }
            if (message.isParachain != null && message.hasOwnProperty("isParachain"))
                object.isParachain = message.isParachain;
            if (message.attestationProvider != null && message.hasOwnProperty("attestationProvider")) {
                object.attestationProvider = options.bytes === String ? $util.base64.encode(message.attestationProvider, 0, message.attestationProvider.length) : options.bytes === Array ? Array.prototype.slice.call(message.attestationProvider) : message.attestationProvider;
                if (options.oneofs)
                    object._attestationProvider = "attestationProvider";
            }
            return object;
        };

        /**
         * Converts this InitRuntimeRequest to JSON.
         * @function toJSON
         * @memberof pruntime_rpc.InitRuntimeRequest
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        InitRuntimeRequest.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        return InitRuntimeRequest;
    })();

    pruntime_rpc.GetRuntimeInfoRequest = (function() {

        /**
         * Properties of a GetRuntimeInfoRequest.
         * @memberof pruntime_rpc
         * @interface IGetRuntimeInfoRequest
         * @property {boolean|null} [forceRefreshRa] GetRuntimeInfoRequest forceRefreshRa
         * @property {Uint8Array|null} [encodedOperator] GetRuntimeInfoRequest encodedOperator
         */

        /**
         * Constructs a new GetRuntimeInfoRequest.
         * @memberof pruntime_rpc
         * @classdesc Represents a GetRuntimeInfoRequest.
         * @implements IGetRuntimeInfoRequest
         * @constructor
         * @param {pruntime_rpc.IGetRuntimeInfoRequest=} [properties] Properties to set
         */
        function GetRuntimeInfoRequest(properties) {
            if (properties)
                for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * GetRuntimeInfoRequest forceRefreshRa.
         * @member {boolean} forceRefreshRa
         * @memberof pruntime_rpc.GetRuntimeInfoRequest
         * @instance
         */
        GetRuntimeInfoRequest.prototype.forceRefreshRa = false;

        /**
         * GetRuntimeInfoRequest encodedOperator.
         * @member {Uint8Array|null|undefined} encodedOperator
         * @memberof pruntime_rpc.GetRuntimeInfoRequest
         * @instance
         */
        GetRuntimeInfoRequest.prototype.encodedOperator = null;

        // OneOf field names bound to virtual getters and setters
        var $oneOfFields;

        /**
         * GetRuntimeInfoRequest _encodedOperator.
         * @member {"encodedOperator"|undefined} _encodedOperator
         * @memberof pruntime_rpc.GetRuntimeInfoRequest
         * @instance
         */
        Object.defineProperty(GetRuntimeInfoRequest.prototype, "_encodedOperator", {
            get: $util.oneOfGetter($oneOfFields = ["encodedOperator"]),
            set: $util.oneOfSetter($oneOfFields)
        });

        /**
         * Creates a new GetRuntimeInfoRequest instance using the specified properties.
         * @function create
         * @memberof pruntime_rpc.GetRuntimeInfoRequest
         * @static
         * @param {pruntime_rpc.IGetRuntimeInfoRequest=} [properties] Properties to set
         * @returns {pruntime_rpc.GetRuntimeInfoRequest} GetRuntimeInfoRequest instance
         */
        GetRuntimeInfoRequest.create = function create(properties) {
            return new GetRuntimeInfoRequest(properties);
        };

        /**
         * Encodes the specified GetRuntimeInfoRequest message. Does not implicitly {@link pruntime_rpc.GetRuntimeInfoRequest.verify|verify} messages.
         * @function encode
         * @memberof pruntime_rpc.GetRuntimeInfoRequest
         * @static
         * @param {pruntime_rpc.IGetRuntimeInfoRequest} message GetRuntimeInfoRequest message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        GetRuntimeInfoRequest.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.forceRefreshRa != null && Object.hasOwnProperty.call(message, "forceRefreshRa"))
                writer.uint32(/* id 1, wireType 0 =*/8).bool(message.forceRefreshRa);
            if (message.encodedOperator != null && Object.hasOwnProperty.call(message, "encodedOperator"))
                writer.uint32(/* id 2, wireType 2 =*/18).bytes(message.encodedOperator);
            return writer;
        };

        /**
         * Encodes the specified GetRuntimeInfoRequest message, length delimited. Does not implicitly {@link pruntime_rpc.GetRuntimeInfoRequest.verify|verify} messages.
         * @function encodeDelimited
         * @memberof pruntime_rpc.GetRuntimeInfoRequest
         * @static
         * @param {pruntime_rpc.IGetRuntimeInfoRequest} message GetRuntimeInfoRequest message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        GetRuntimeInfoRequest.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes a GetRuntimeInfoRequest message from the specified reader or buffer.
         * @function decode
         * @memberof pruntime_rpc.GetRuntimeInfoRequest
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {pruntime_rpc.GetRuntimeInfoRequest} GetRuntimeInfoRequest
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        GetRuntimeInfoRequest.decode = function decode(reader, length) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            var end = length === undefined ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.GetRuntimeInfoRequest();
            while (reader.pos < end) {
                var tag = reader.uint32();
                switch (tag >>> 3) {
                case 1:
                    message.forceRefreshRa = reader.bool();
                    break;
                case 2:
                    message.encodedOperator = reader.bytes();
                    break;
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a GetRuntimeInfoRequest message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof pruntime_rpc.GetRuntimeInfoRequest
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {pruntime_rpc.GetRuntimeInfoRequest} GetRuntimeInfoRequest
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        GetRuntimeInfoRequest.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies a GetRuntimeInfoRequest message.
         * @function verify
         * @memberof pruntime_rpc.GetRuntimeInfoRequest
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        GetRuntimeInfoRequest.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            var properties = {};
            if (message.forceRefreshRa != null && message.hasOwnProperty("forceRefreshRa"))
                if (typeof message.forceRefreshRa !== "boolean")
                    return "forceRefreshRa: boolean expected";
            if (message.encodedOperator != null && message.hasOwnProperty("encodedOperator")) {
                properties._encodedOperator = 1;
                if (!(message.encodedOperator && typeof message.encodedOperator.length === "number" || $util.isString(message.encodedOperator)))
                    return "encodedOperator: buffer expected";
            }
            return null;
        };

        /**
         * Creates a GetRuntimeInfoRequest message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof pruntime_rpc.GetRuntimeInfoRequest
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {pruntime_rpc.GetRuntimeInfoRequest} GetRuntimeInfoRequest
         */
        GetRuntimeInfoRequest.fromObject = function fromObject(object) {
            if (object instanceof $root.pruntime_rpc.GetRuntimeInfoRequest)
                return object;
            var message = new $root.pruntime_rpc.GetRuntimeInfoRequest();
            if (object.forceRefreshRa != null)
                message.forceRefreshRa = Boolean(object.forceRefreshRa);
            if (object.encodedOperator != null)
                if (typeof object.encodedOperator === "string")
                    $util.base64.decode(object.encodedOperator, message.encodedOperator = $util.newBuffer($util.base64.length(object.encodedOperator)), 0);
                else if (object.encodedOperator.length)
                    message.encodedOperator = object.encodedOperator;
            return message;
        };

        /**
         * Creates a plain object from a GetRuntimeInfoRequest message. Also converts values to other types if specified.
         * @function toObject
         * @memberof pruntime_rpc.GetRuntimeInfoRequest
         * @static
         * @param {pruntime_rpc.GetRuntimeInfoRequest} message GetRuntimeInfoRequest
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        GetRuntimeInfoRequest.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            var object = {};
            if (options.defaults)
                object.forceRefreshRa = false;
            if (message.forceRefreshRa != null && message.hasOwnProperty("forceRefreshRa"))
                object.forceRefreshRa = message.forceRefreshRa;
            if (message.encodedOperator != null && message.hasOwnProperty("encodedOperator")) {
                object.encodedOperator = options.bytes === String ? $util.base64.encode(message.encodedOperator, 0, message.encodedOperator.length) : options.bytes === Array ? Array.prototype.slice.call(message.encodedOperator) : message.encodedOperator;
                if (options.oneofs)
                    object._encodedOperator = "encodedOperator";
            }
            return object;
        };

        /**
         * Converts this GetRuntimeInfoRequest to JSON.
         * @function toJSON
         * @memberof pruntime_rpc.GetRuntimeInfoRequest
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        GetRuntimeInfoRequest.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        return GetRuntimeInfoRequest;
    })();

    pruntime_rpc.InitRuntimeResponse = (function() {

        /**
         * Properties of an InitRuntimeResponse.
         * @memberof pruntime_rpc
         * @interface IInitRuntimeResponse
         * @property {Uint8Array|null} [encodedRuntimeInfo] InitRuntimeResponse encodedRuntimeInfo
         * @property {Uint8Array|null} [encodedGenesisBlockHash] InitRuntimeResponse encodedGenesisBlockHash
         * @property {Uint8Array|null} [encodedPublicKey] InitRuntimeResponse encodedPublicKey
         * @property {Uint8Array|null} [encodedEcdhPublicKey] InitRuntimeResponse encodedEcdhPublicKey
         * @property {pruntime_rpc.IAttestation|null} [attestation] InitRuntimeResponse attestation
         */

        /**
         * Constructs a new InitRuntimeResponse.
         * @memberof pruntime_rpc
         * @classdesc Represents an InitRuntimeResponse.
         * @implements IInitRuntimeResponse
         * @constructor
         * @param {pruntime_rpc.IInitRuntimeResponse=} [properties] Properties to set
         */
        function InitRuntimeResponse(properties) {
            if (properties)
                for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * InitRuntimeResponse encodedRuntimeInfo.
         * @member {Uint8Array} encodedRuntimeInfo
         * @memberof pruntime_rpc.InitRuntimeResponse
         * @instance
         */
        InitRuntimeResponse.prototype.encodedRuntimeInfo = $util.newBuffer([]);

        /**
         * InitRuntimeResponse encodedGenesisBlockHash.
         * @member {Uint8Array} encodedGenesisBlockHash
         * @memberof pruntime_rpc.InitRuntimeResponse
         * @instance
         */
        InitRuntimeResponse.prototype.encodedGenesisBlockHash = $util.newBuffer([]);

        /**
         * InitRuntimeResponse encodedPublicKey.
         * @member {Uint8Array} encodedPublicKey
         * @memberof pruntime_rpc.InitRuntimeResponse
         * @instance
         */
        InitRuntimeResponse.prototype.encodedPublicKey = $util.newBuffer([]);

        /**
         * InitRuntimeResponse encodedEcdhPublicKey.
         * @member {Uint8Array} encodedEcdhPublicKey
         * @memberof pruntime_rpc.InitRuntimeResponse
         * @instance
         */
        InitRuntimeResponse.prototype.encodedEcdhPublicKey = $util.newBuffer([]);

        /**
         * InitRuntimeResponse attestation.
         * @member {pruntime_rpc.IAttestation|null|undefined} attestation
         * @memberof pruntime_rpc.InitRuntimeResponse
         * @instance
         */
        InitRuntimeResponse.prototype.attestation = null;

        // OneOf field names bound to virtual getters and setters
        var $oneOfFields;

        /**
         * InitRuntimeResponse _attestation.
         * @member {"attestation"|undefined} _attestation
         * @memberof pruntime_rpc.InitRuntimeResponse
         * @instance
         */
        Object.defineProperty(InitRuntimeResponse.prototype, "_attestation", {
            get: $util.oneOfGetter($oneOfFields = ["attestation"]),
            set: $util.oneOfSetter($oneOfFields)
        });

        /**
         * Creates a new InitRuntimeResponse instance using the specified properties.
         * @function create
         * @memberof pruntime_rpc.InitRuntimeResponse
         * @static
         * @param {pruntime_rpc.IInitRuntimeResponse=} [properties] Properties to set
         * @returns {pruntime_rpc.InitRuntimeResponse} InitRuntimeResponse instance
         */
        InitRuntimeResponse.create = function create(properties) {
            return new InitRuntimeResponse(properties);
        };

        /**
         * Encodes the specified InitRuntimeResponse message. Does not implicitly {@link pruntime_rpc.InitRuntimeResponse.verify|verify} messages.
         * @function encode
         * @memberof pruntime_rpc.InitRuntimeResponse
         * @static
         * @param {pruntime_rpc.IInitRuntimeResponse} message InitRuntimeResponse message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        InitRuntimeResponse.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.encodedRuntimeInfo != null && Object.hasOwnProperty.call(message, "encodedRuntimeInfo"))
                writer.uint32(/* id 1, wireType 2 =*/10).bytes(message.encodedRuntimeInfo);
            if (message.encodedGenesisBlockHash != null && Object.hasOwnProperty.call(message, "encodedGenesisBlockHash"))
                writer.uint32(/* id 2, wireType 2 =*/18).bytes(message.encodedGenesisBlockHash);
            if (message.encodedPublicKey != null && Object.hasOwnProperty.call(message, "encodedPublicKey"))
                writer.uint32(/* id 3, wireType 2 =*/26).bytes(message.encodedPublicKey);
            if (message.encodedEcdhPublicKey != null && Object.hasOwnProperty.call(message, "encodedEcdhPublicKey"))
                writer.uint32(/* id 4, wireType 2 =*/34).bytes(message.encodedEcdhPublicKey);
            if (message.attestation != null && Object.hasOwnProperty.call(message, "attestation"))
                $root.pruntime_rpc.Attestation.encode(message.attestation, writer.uint32(/* id 5, wireType 2 =*/42).fork()).ldelim();
            return writer;
        };

        /**
         * Encodes the specified InitRuntimeResponse message, length delimited. Does not implicitly {@link pruntime_rpc.InitRuntimeResponse.verify|verify} messages.
         * @function encodeDelimited
         * @memberof pruntime_rpc.InitRuntimeResponse
         * @static
         * @param {pruntime_rpc.IInitRuntimeResponse} message InitRuntimeResponse message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        InitRuntimeResponse.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes an InitRuntimeResponse message from the specified reader or buffer.
         * @function decode
         * @memberof pruntime_rpc.InitRuntimeResponse
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {pruntime_rpc.InitRuntimeResponse} InitRuntimeResponse
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        InitRuntimeResponse.decode = function decode(reader, length) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            var end = length === undefined ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.InitRuntimeResponse();
            while (reader.pos < end) {
                var tag = reader.uint32();
                switch (tag >>> 3) {
                case 1:
                    message.encodedRuntimeInfo = reader.bytes();
                    break;
                case 2:
                    message.encodedGenesisBlockHash = reader.bytes();
                    break;
                case 3:
                    message.encodedPublicKey = reader.bytes();
                    break;
                case 4:
                    message.encodedEcdhPublicKey = reader.bytes();
                    break;
                case 5:
                    message.attestation = $root.pruntime_rpc.Attestation.decode(reader, reader.uint32());
                    break;
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes an InitRuntimeResponse message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof pruntime_rpc.InitRuntimeResponse
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {pruntime_rpc.InitRuntimeResponse} InitRuntimeResponse
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        InitRuntimeResponse.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies an InitRuntimeResponse message.
         * @function verify
         * @memberof pruntime_rpc.InitRuntimeResponse
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        InitRuntimeResponse.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            var properties = {};
            if (message.encodedRuntimeInfo != null && message.hasOwnProperty("encodedRuntimeInfo"))
                if (!(message.encodedRuntimeInfo && typeof message.encodedRuntimeInfo.length === "number" || $util.isString(message.encodedRuntimeInfo)))
                    return "encodedRuntimeInfo: buffer expected";
            if (message.encodedGenesisBlockHash != null && message.hasOwnProperty("encodedGenesisBlockHash"))
                if (!(message.encodedGenesisBlockHash && typeof message.encodedGenesisBlockHash.length === "number" || $util.isString(message.encodedGenesisBlockHash)))
                    return "encodedGenesisBlockHash: buffer expected";
            if (message.encodedPublicKey != null && message.hasOwnProperty("encodedPublicKey"))
                if (!(message.encodedPublicKey && typeof message.encodedPublicKey.length === "number" || $util.isString(message.encodedPublicKey)))
                    return "encodedPublicKey: buffer expected";
            if (message.encodedEcdhPublicKey != null && message.hasOwnProperty("encodedEcdhPublicKey"))
                if (!(message.encodedEcdhPublicKey && typeof message.encodedEcdhPublicKey.length === "number" || $util.isString(message.encodedEcdhPublicKey)))
                    return "encodedEcdhPublicKey: buffer expected";
            if (message.attestation != null && message.hasOwnProperty("attestation")) {
                properties._attestation = 1;
                {
                    var error = $root.pruntime_rpc.Attestation.verify(message.attestation);
                    if (error)
                        return "attestation." + error;
                }
            }
            return null;
        };

        /**
         * Creates an InitRuntimeResponse message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof pruntime_rpc.InitRuntimeResponse
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {pruntime_rpc.InitRuntimeResponse} InitRuntimeResponse
         */
        InitRuntimeResponse.fromObject = function fromObject(object) {
            if (object instanceof $root.pruntime_rpc.InitRuntimeResponse)
                return object;
            var message = new $root.pruntime_rpc.InitRuntimeResponse();
            if (object.encodedRuntimeInfo != null)
                if (typeof object.encodedRuntimeInfo === "string")
                    $util.base64.decode(object.encodedRuntimeInfo, message.encodedRuntimeInfo = $util.newBuffer($util.base64.length(object.encodedRuntimeInfo)), 0);
                else if (object.encodedRuntimeInfo.length)
                    message.encodedRuntimeInfo = object.encodedRuntimeInfo;
            if (object.encodedGenesisBlockHash != null)
                if (typeof object.encodedGenesisBlockHash === "string")
                    $util.base64.decode(object.encodedGenesisBlockHash, message.encodedGenesisBlockHash = $util.newBuffer($util.base64.length(object.encodedGenesisBlockHash)), 0);
                else if (object.encodedGenesisBlockHash.length)
                    message.encodedGenesisBlockHash = object.encodedGenesisBlockHash;
            if (object.encodedPublicKey != null)
                if (typeof object.encodedPublicKey === "string")
                    $util.base64.decode(object.encodedPublicKey, message.encodedPublicKey = $util.newBuffer($util.base64.length(object.encodedPublicKey)), 0);
                else if (object.encodedPublicKey.length)
                    message.encodedPublicKey = object.encodedPublicKey;
            if (object.encodedEcdhPublicKey != null)
                if (typeof object.encodedEcdhPublicKey === "string")
                    $util.base64.decode(object.encodedEcdhPublicKey, message.encodedEcdhPublicKey = $util.newBuffer($util.base64.length(object.encodedEcdhPublicKey)), 0);
                else if (object.encodedEcdhPublicKey.length)
                    message.encodedEcdhPublicKey = object.encodedEcdhPublicKey;
            if (object.attestation != null) {
                if (typeof object.attestation !== "object")
                    throw TypeError(".pruntime_rpc.InitRuntimeResponse.attestation: object expected");
                message.attestation = $root.pruntime_rpc.Attestation.fromObject(object.attestation);
            }
            return message;
        };

        /**
         * Creates a plain object from an InitRuntimeResponse message. Also converts values to other types if specified.
         * @function toObject
         * @memberof pruntime_rpc.InitRuntimeResponse
         * @static
         * @param {pruntime_rpc.InitRuntimeResponse} message InitRuntimeResponse
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        InitRuntimeResponse.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            var object = {};
            if (options.defaults) {
                if (options.bytes === String)
                    object.encodedRuntimeInfo = "";
                else {
                    object.encodedRuntimeInfo = [];
                    if (options.bytes !== Array)
                        object.encodedRuntimeInfo = $util.newBuffer(object.encodedRuntimeInfo);
                }
                if (options.bytes === String)
                    object.encodedGenesisBlockHash = "";
                else {
                    object.encodedGenesisBlockHash = [];
                    if (options.bytes !== Array)
                        object.encodedGenesisBlockHash = $util.newBuffer(object.encodedGenesisBlockHash);
                }
                if (options.bytes === String)
                    object.encodedPublicKey = "";
                else {
                    object.encodedPublicKey = [];
                    if (options.bytes !== Array)
                        object.encodedPublicKey = $util.newBuffer(object.encodedPublicKey);
                }
                if (options.bytes === String)
                    object.encodedEcdhPublicKey = "";
                else {
                    object.encodedEcdhPublicKey = [];
                    if (options.bytes !== Array)
                        object.encodedEcdhPublicKey = $util.newBuffer(object.encodedEcdhPublicKey);
                }
            }
            if (message.encodedRuntimeInfo != null && message.hasOwnProperty("encodedRuntimeInfo"))
                object.encodedRuntimeInfo = options.bytes === String ? $util.base64.encode(message.encodedRuntimeInfo, 0, message.encodedRuntimeInfo.length) : options.bytes === Array ? Array.prototype.slice.call(message.encodedRuntimeInfo) : message.encodedRuntimeInfo;
            if (message.encodedGenesisBlockHash != null && message.hasOwnProperty("encodedGenesisBlockHash"))
                object.encodedGenesisBlockHash = options.bytes === String ? $util.base64.encode(message.encodedGenesisBlockHash, 0, message.encodedGenesisBlockHash.length) : options.bytes === Array ? Array.prototype.slice.call(message.encodedGenesisBlockHash) : message.encodedGenesisBlockHash;
            if (message.encodedPublicKey != null && message.hasOwnProperty("encodedPublicKey"))
                object.encodedPublicKey = options.bytes === String ? $util.base64.encode(message.encodedPublicKey, 0, message.encodedPublicKey.length) : options.bytes === Array ? Array.prototype.slice.call(message.encodedPublicKey) : message.encodedPublicKey;
            if (message.encodedEcdhPublicKey != null && message.hasOwnProperty("encodedEcdhPublicKey"))
                object.encodedEcdhPublicKey = options.bytes === String ? $util.base64.encode(message.encodedEcdhPublicKey, 0, message.encodedEcdhPublicKey.length) : options.bytes === Array ? Array.prototype.slice.call(message.encodedEcdhPublicKey) : message.encodedEcdhPublicKey;
            if (message.attestation != null && message.hasOwnProperty("attestation")) {
                object.attestation = $root.pruntime_rpc.Attestation.toObject(message.attestation, options);
                if (options.oneofs)
                    object._attestation = "attestation";
            }
            return object;
        };

        /**
         * Converts this InitRuntimeResponse to JSON.
         * @function toJSON
         * @memberof pruntime_rpc.InitRuntimeResponse
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        InitRuntimeResponse.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        return InitRuntimeResponse;
    })();

    pruntime_rpc.Attestation = (function() {

        /**
         * Properties of an Attestation.
         * @memberof pruntime_rpc
         * @interface IAttestation
         * @property {number|null} [version] Attestation version
         * @property {string|null} [provider] Attestation provider
         * @property {pruntime_rpc.IAttestationReport|null} [payload] Attestation payload
         * @property {Uint8Array|null} [encodedReport] Attestation encodedReport
         * @property {number|Long|null} [timestamp] Attestation timestamp
         */

        /**
         * Constructs a new Attestation.
         * @memberof pruntime_rpc
         * @classdesc Represents an Attestation.
         * @implements IAttestation
         * @constructor
         * @param {pruntime_rpc.IAttestation=} [properties] Properties to set
         */
        function Attestation(properties) {
            if (properties)
                for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * Attestation version.
         * @member {number} version
         * @memberof pruntime_rpc.Attestation
         * @instance
         */
        Attestation.prototype.version = 0;

        /**
         * Attestation provider.
         * @member {string} provider
         * @memberof pruntime_rpc.Attestation
         * @instance
         */
        Attestation.prototype.provider = "";

        /**
         * Attestation payload.
         * @member {pruntime_rpc.IAttestationReport|null|undefined} payload
         * @memberof pruntime_rpc.Attestation
         * @instance
         */
        Attestation.prototype.payload = null;

        /**
         * Attestation encodedReport.
         * @member {Uint8Array} encodedReport
         * @memberof pruntime_rpc.Attestation
         * @instance
         */
        Attestation.prototype.encodedReport = $util.newBuffer([]);

        /**
         * Attestation timestamp.
         * @member {number|Long} timestamp
         * @memberof pruntime_rpc.Attestation
         * @instance
         */
        Attestation.prototype.timestamp = $util.Long ? $util.Long.fromBits(0,0,true) : 0;

        /**
         * Creates a new Attestation instance using the specified properties.
         * @function create
         * @memberof pruntime_rpc.Attestation
         * @static
         * @param {pruntime_rpc.IAttestation=} [properties] Properties to set
         * @returns {pruntime_rpc.Attestation} Attestation instance
         */
        Attestation.create = function create(properties) {
            return new Attestation(properties);
        };

        /**
         * Encodes the specified Attestation message. Does not implicitly {@link pruntime_rpc.Attestation.verify|verify} messages.
         * @function encode
         * @memberof pruntime_rpc.Attestation
         * @static
         * @param {pruntime_rpc.IAttestation} message Attestation message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        Attestation.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.version != null && Object.hasOwnProperty.call(message, "version"))
                writer.uint32(/* id 1, wireType 0 =*/8).int32(message.version);
            if (message.provider != null && Object.hasOwnProperty.call(message, "provider"))
                writer.uint32(/* id 2, wireType 2 =*/18).string(message.provider);
            if (message.payload != null && Object.hasOwnProperty.call(message, "payload"))
                $root.pruntime_rpc.AttestationReport.encode(message.payload, writer.uint32(/* id 3, wireType 2 =*/26).fork()).ldelim();
            if (message.timestamp != null && Object.hasOwnProperty.call(message, "timestamp"))
                writer.uint32(/* id 4, wireType 0 =*/32).uint64(message.timestamp);
            if (message.encodedReport != null && Object.hasOwnProperty.call(message, "encodedReport"))
                writer.uint32(/* id 5, wireType 2 =*/42).bytes(message.encodedReport);
            return writer;
        };

        /**
         * Encodes the specified Attestation message, length delimited. Does not implicitly {@link pruntime_rpc.Attestation.verify|verify} messages.
         * @function encodeDelimited
         * @memberof pruntime_rpc.Attestation
         * @static
         * @param {pruntime_rpc.IAttestation} message Attestation message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        Attestation.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes an Attestation message from the specified reader or buffer.
         * @function decode
         * @memberof pruntime_rpc.Attestation
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {pruntime_rpc.Attestation} Attestation
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        Attestation.decode = function decode(reader, length) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            var end = length === undefined ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.Attestation();
            while (reader.pos < end) {
                var tag = reader.uint32();
                switch (tag >>> 3) {
                case 1:
                    message.version = reader.int32();
                    break;
                case 2:
                    message.provider = reader.string();
                    break;
                case 3:
                    message.payload = $root.pruntime_rpc.AttestationReport.decode(reader, reader.uint32());
                    break;
                case 5:
                    message.encodedReport = reader.bytes();
                    break;
                case 4:
                    message.timestamp = reader.uint64();
                    break;
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes an Attestation message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof pruntime_rpc.Attestation
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {pruntime_rpc.Attestation} Attestation
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        Attestation.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies an Attestation message.
         * @function verify
         * @memberof pruntime_rpc.Attestation
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        Attestation.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            if (message.version != null && message.hasOwnProperty("version"))
                if (!$util.isInteger(message.version))
                    return "version: integer expected";
            if (message.provider != null && message.hasOwnProperty("provider"))
                if (!$util.isString(message.provider))
                    return "provider: string expected";
            if (message.payload != null && message.hasOwnProperty("payload")) {
                var error = $root.pruntime_rpc.AttestationReport.verify(message.payload);
                if (error)
                    return "payload." + error;
            }
            if (message.encodedReport != null && message.hasOwnProperty("encodedReport"))
                if (!(message.encodedReport && typeof message.encodedReport.length === "number" || $util.isString(message.encodedReport)))
                    return "encodedReport: buffer expected";
            if (message.timestamp != null && message.hasOwnProperty("timestamp"))
                if (!$util.isInteger(message.timestamp) && !(message.timestamp && $util.isInteger(message.timestamp.low) && $util.isInteger(message.timestamp.high)))
                    return "timestamp: integer|Long expected";
            return null;
        };

        /**
         * Creates an Attestation message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof pruntime_rpc.Attestation
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {pruntime_rpc.Attestation} Attestation
         */
        Attestation.fromObject = function fromObject(object) {
            if (object instanceof $root.pruntime_rpc.Attestation)
                return object;
            var message = new $root.pruntime_rpc.Attestation();
            if (object.version != null)
                message.version = object.version | 0;
            if (object.provider != null)
                message.provider = String(object.provider);
            if (object.payload != null) {
                if (typeof object.payload !== "object")
                    throw TypeError(".pruntime_rpc.Attestation.payload: object expected");
                message.payload = $root.pruntime_rpc.AttestationReport.fromObject(object.payload);
            }
            if (object.encodedReport != null)
                if (typeof object.encodedReport === "string")
                    $util.base64.decode(object.encodedReport, message.encodedReport = $util.newBuffer($util.base64.length(object.encodedReport)), 0);
                else if (object.encodedReport.length)
                    message.encodedReport = object.encodedReport;
            if (object.timestamp != null)
                if ($util.Long)
                    (message.timestamp = $util.Long.fromValue(object.timestamp)).unsigned = true;
                else if (typeof object.timestamp === "string")
                    message.timestamp = parseInt(object.timestamp, 10);
                else if (typeof object.timestamp === "number")
                    message.timestamp = object.timestamp;
                else if (typeof object.timestamp === "object")
                    message.timestamp = new $util.LongBits(object.timestamp.low >>> 0, object.timestamp.high >>> 0).toNumber(true);
            return message;
        };

        /**
         * Creates a plain object from an Attestation message. Also converts values to other types if specified.
         * @function toObject
         * @memberof pruntime_rpc.Attestation
         * @static
         * @param {pruntime_rpc.Attestation} message Attestation
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        Attestation.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            var object = {};
            if (options.defaults) {
                object.version = 0;
                object.provider = "";
                object.payload = null;
                if ($util.Long) {
                    var long = new $util.Long(0, 0, true);
                    object.timestamp = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : long;
                } else
                    object.timestamp = options.longs === String ? "0" : 0;
                if (options.bytes === String)
                    object.encodedReport = "";
                else {
                    object.encodedReport = [];
                    if (options.bytes !== Array)
                        object.encodedReport = $util.newBuffer(object.encodedReport);
                }
            }
            if (message.version != null && message.hasOwnProperty("version"))
                object.version = message.version;
            if (message.provider != null && message.hasOwnProperty("provider"))
                object.provider = message.provider;
            if (message.payload != null && message.hasOwnProperty("payload"))
                object.payload = $root.pruntime_rpc.AttestationReport.toObject(message.payload, options);
            if (message.timestamp != null && message.hasOwnProperty("timestamp"))
                if (typeof message.timestamp === "number")
                    object.timestamp = options.longs === String ? String(message.timestamp) : message.timestamp;
                else
                    object.timestamp = options.longs === String ? $util.Long.prototype.toString.call(message.timestamp) : options.longs === Number ? new $util.LongBits(message.timestamp.low >>> 0, message.timestamp.high >>> 0).toNumber(true) : message.timestamp;
            if (message.encodedReport != null && message.hasOwnProperty("encodedReport"))
                object.encodedReport = options.bytes === String ? $util.base64.encode(message.encodedReport, 0, message.encodedReport.length) : options.bytes === Array ? Array.prototype.slice.call(message.encodedReport) : message.encodedReport;
            return object;
        };

        /**
         * Converts this Attestation to JSON.
         * @function toJSON
         * @memberof pruntime_rpc.Attestation
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        Attestation.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        return Attestation;
    })();

    pruntime_rpc.AttestationReport = (function() {

        /**
         * Properties of an AttestationReport.
         * @memberof pruntime_rpc
         * @interface IAttestationReport
         * @property {string|null} [report] AttestationReport report
         * @property {Uint8Array|null} [signature] AttestationReport signature
         * @property {Uint8Array|null} [signingCert] AttestationReport signingCert
         */

        /**
         * Constructs a new AttestationReport.
         * @memberof pruntime_rpc
         * @classdesc Represents an AttestationReport.
         * @implements IAttestationReport
         * @constructor
         * @param {pruntime_rpc.IAttestationReport=} [properties] Properties to set
         */
        function AttestationReport(properties) {
            if (properties)
                for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * AttestationReport report.
         * @member {string} report
         * @memberof pruntime_rpc.AttestationReport
         * @instance
         */
        AttestationReport.prototype.report = "";

        /**
         * AttestationReport signature.
         * @member {Uint8Array} signature
         * @memberof pruntime_rpc.AttestationReport
         * @instance
         */
        AttestationReport.prototype.signature = $util.newBuffer([]);

        /**
         * AttestationReport signingCert.
         * @member {Uint8Array} signingCert
         * @memberof pruntime_rpc.AttestationReport
         * @instance
         */
        AttestationReport.prototype.signingCert = $util.newBuffer([]);

        /**
         * Creates a new AttestationReport instance using the specified properties.
         * @function create
         * @memberof pruntime_rpc.AttestationReport
         * @static
         * @param {pruntime_rpc.IAttestationReport=} [properties] Properties to set
         * @returns {pruntime_rpc.AttestationReport} AttestationReport instance
         */
        AttestationReport.create = function create(properties) {
            return new AttestationReport(properties);
        };

        /**
         * Encodes the specified AttestationReport message. Does not implicitly {@link pruntime_rpc.AttestationReport.verify|verify} messages.
         * @function encode
         * @memberof pruntime_rpc.AttestationReport
         * @static
         * @param {pruntime_rpc.IAttestationReport} message AttestationReport message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        AttestationReport.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.report != null && Object.hasOwnProperty.call(message, "report"))
                writer.uint32(/* id 1, wireType 2 =*/10).string(message.report);
            if (message.signature != null && Object.hasOwnProperty.call(message, "signature"))
                writer.uint32(/* id 2, wireType 2 =*/18).bytes(message.signature);
            if (message.signingCert != null && Object.hasOwnProperty.call(message, "signingCert"))
                writer.uint32(/* id 3, wireType 2 =*/26).bytes(message.signingCert);
            return writer;
        };

        /**
         * Encodes the specified AttestationReport message, length delimited. Does not implicitly {@link pruntime_rpc.AttestationReport.verify|verify} messages.
         * @function encodeDelimited
         * @memberof pruntime_rpc.AttestationReport
         * @static
         * @param {pruntime_rpc.IAttestationReport} message AttestationReport message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        AttestationReport.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes an AttestationReport message from the specified reader or buffer.
         * @function decode
         * @memberof pruntime_rpc.AttestationReport
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {pruntime_rpc.AttestationReport} AttestationReport
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        AttestationReport.decode = function decode(reader, length) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            var end = length === undefined ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.AttestationReport();
            while (reader.pos < end) {
                var tag = reader.uint32();
                switch (tag >>> 3) {
                case 1:
                    message.report = reader.string();
                    break;
                case 2:
                    message.signature = reader.bytes();
                    break;
                case 3:
                    message.signingCert = reader.bytes();
                    break;
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes an AttestationReport message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof pruntime_rpc.AttestationReport
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {pruntime_rpc.AttestationReport} AttestationReport
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        AttestationReport.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies an AttestationReport message.
         * @function verify
         * @memberof pruntime_rpc.AttestationReport
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        AttestationReport.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            if (message.report != null && message.hasOwnProperty("report"))
                if (!$util.isString(message.report))
                    return "report: string expected";
            if (message.signature != null && message.hasOwnProperty("signature"))
                if (!(message.signature && typeof message.signature.length === "number" || $util.isString(message.signature)))
                    return "signature: buffer expected";
            if (message.signingCert != null && message.hasOwnProperty("signingCert"))
                if (!(message.signingCert && typeof message.signingCert.length === "number" || $util.isString(message.signingCert)))
                    return "signingCert: buffer expected";
            return null;
        };

        /**
         * Creates an AttestationReport message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof pruntime_rpc.AttestationReport
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {pruntime_rpc.AttestationReport} AttestationReport
         */
        AttestationReport.fromObject = function fromObject(object) {
            if (object instanceof $root.pruntime_rpc.AttestationReport)
                return object;
            var message = new $root.pruntime_rpc.AttestationReport();
            if (object.report != null)
                message.report = String(object.report);
            if (object.signature != null)
                if (typeof object.signature === "string")
                    $util.base64.decode(object.signature, message.signature = $util.newBuffer($util.base64.length(object.signature)), 0);
                else if (object.signature.length)
                    message.signature = object.signature;
            if (object.signingCert != null)
                if (typeof object.signingCert === "string")
                    $util.base64.decode(object.signingCert, message.signingCert = $util.newBuffer($util.base64.length(object.signingCert)), 0);
                else if (object.signingCert.length)
                    message.signingCert = object.signingCert;
            return message;
        };

        /**
         * Creates a plain object from an AttestationReport message. Also converts values to other types if specified.
         * @function toObject
         * @memberof pruntime_rpc.AttestationReport
         * @static
         * @param {pruntime_rpc.AttestationReport} message AttestationReport
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        AttestationReport.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            var object = {};
            if (options.defaults) {
                object.report = "";
                if (options.bytes === String)
                    object.signature = "";
                else {
                    object.signature = [];
                    if (options.bytes !== Array)
                        object.signature = $util.newBuffer(object.signature);
                }
                if (options.bytes === String)
                    object.signingCert = "";
                else {
                    object.signingCert = [];
                    if (options.bytes !== Array)
                        object.signingCert = $util.newBuffer(object.signingCert);
                }
            }
            if (message.report != null && message.hasOwnProperty("report"))
                object.report = message.report;
            if (message.signature != null && message.hasOwnProperty("signature"))
                object.signature = options.bytes === String ? $util.base64.encode(message.signature, 0, message.signature.length) : options.bytes === Array ? Array.prototype.slice.call(message.signature) : message.signature;
            if (message.signingCert != null && message.hasOwnProperty("signingCert"))
                object.signingCert = options.bytes === String ? $util.base64.encode(message.signingCert, 0, message.signingCert.length) : options.bytes === Array ? Array.prototype.slice.call(message.signingCert) : message.signingCert;
            return object;
        };

        /**
         * Converts this AttestationReport to JSON.
         * @function toJSON
         * @memberof pruntime_rpc.AttestationReport
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        AttestationReport.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        return AttestationReport;
    })();

    pruntime_rpc.GetEgressMessagesResponse = (function() {

        /**
         * Properties of a GetEgressMessagesResponse.
         * @memberof pruntime_rpc
         * @interface IGetEgressMessagesResponse
         * @property {Uint8Array|null} [encodedMessages] GetEgressMessagesResponse encodedMessages
         */

        /**
         * Constructs a new GetEgressMessagesResponse.
         * @memberof pruntime_rpc
         * @classdesc Represents a GetEgressMessagesResponse.
         * @implements IGetEgressMessagesResponse
         * @constructor
         * @param {pruntime_rpc.IGetEgressMessagesResponse=} [properties] Properties to set
         */
        function GetEgressMessagesResponse(properties) {
            if (properties)
                for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * GetEgressMessagesResponse encodedMessages.
         * @member {Uint8Array} encodedMessages
         * @memberof pruntime_rpc.GetEgressMessagesResponse
         * @instance
         */
        GetEgressMessagesResponse.prototype.encodedMessages = $util.newBuffer([]);

        /**
         * Creates a new GetEgressMessagesResponse instance using the specified properties.
         * @function create
         * @memberof pruntime_rpc.GetEgressMessagesResponse
         * @static
         * @param {pruntime_rpc.IGetEgressMessagesResponse=} [properties] Properties to set
         * @returns {pruntime_rpc.GetEgressMessagesResponse} GetEgressMessagesResponse instance
         */
        GetEgressMessagesResponse.create = function create(properties) {
            return new GetEgressMessagesResponse(properties);
        };

        /**
         * Encodes the specified GetEgressMessagesResponse message. Does not implicitly {@link pruntime_rpc.GetEgressMessagesResponse.verify|verify} messages.
         * @function encode
         * @memberof pruntime_rpc.GetEgressMessagesResponse
         * @static
         * @param {pruntime_rpc.IGetEgressMessagesResponse} message GetEgressMessagesResponse message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        GetEgressMessagesResponse.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.encodedMessages != null && Object.hasOwnProperty.call(message, "encodedMessages"))
                writer.uint32(/* id 1, wireType 2 =*/10).bytes(message.encodedMessages);
            return writer;
        };

        /**
         * Encodes the specified GetEgressMessagesResponse message, length delimited. Does not implicitly {@link pruntime_rpc.GetEgressMessagesResponse.verify|verify} messages.
         * @function encodeDelimited
         * @memberof pruntime_rpc.GetEgressMessagesResponse
         * @static
         * @param {pruntime_rpc.IGetEgressMessagesResponse} message GetEgressMessagesResponse message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        GetEgressMessagesResponse.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes a GetEgressMessagesResponse message from the specified reader or buffer.
         * @function decode
         * @memberof pruntime_rpc.GetEgressMessagesResponse
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {pruntime_rpc.GetEgressMessagesResponse} GetEgressMessagesResponse
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        GetEgressMessagesResponse.decode = function decode(reader, length) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            var end = length === undefined ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.GetEgressMessagesResponse();
            while (reader.pos < end) {
                var tag = reader.uint32();
                switch (tag >>> 3) {
                case 1:
                    message.encodedMessages = reader.bytes();
                    break;
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a GetEgressMessagesResponse message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof pruntime_rpc.GetEgressMessagesResponse
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {pruntime_rpc.GetEgressMessagesResponse} GetEgressMessagesResponse
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        GetEgressMessagesResponse.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies a GetEgressMessagesResponse message.
         * @function verify
         * @memberof pruntime_rpc.GetEgressMessagesResponse
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        GetEgressMessagesResponse.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            if (message.encodedMessages != null && message.hasOwnProperty("encodedMessages"))
                if (!(message.encodedMessages && typeof message.encodedMessages.length === "number" || $util.isString(message.encodedMessages)))
                    return "encodedMessages: buffer expected";
            return null;
        };

        /**
         * Creates a GetEgressMessagesResponse message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof pruntime_rpc.GetEgressMessagesResponse
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {pruntime_rpc.GetEgressMessagesResponse} GetEgressMessagesResponse
         */
        GetEgressMessagesResponse.fromObject = function fromObject(object) {
            if (object instanceof $root.pruntime_rpc.GetEgressMessagesResponse)
                return object;
            var message = new $root.pruntime_rpc.GetEgressMessagesResponse();
            if (object.encodedMessages != null)
                if (typeof object.encodedMessages === "string")
                    $util.base64.decode(object.encodedMessages, message.encodedMessages = $util.newBuffer($util.base64.length(object.encodedMessages)), 0);
                else if (object.encodedMessages.length)
                    message.encodedMessages = object.encodedMessages;
            return message;
        };

        /**
         * Creates a plain object from a GetEgressMessagesResponse message. Also converts values to other types if specified.
         * @function toObject
         * @memberof pruntime_rpc.GetEgressMessagesResponse
         * @static
         * @param {pruntime_rpc.GetEgressMessagesResponse} message GetEgressMessagesResponse
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        GetEgressMessagesResponse.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            var object = {};
            if (options.defaults)
                if (options.bytes === String)
                    object.encodedMessages = "";
                else {
                    object.encodedMessages = [];
                    if (options.bytes !== Array)
                        object.encodedMessages = $util.newBuffer(object.encodedMessages);
                }
            if (message.encodedMessages != null && message.hasOwnProperty("encodedMessages"))
                object.encodedMessages = options.bytes === String ? $util.base64.encode(message.encodedMessages, 0, message.encodedMessages.length) : options.bytes === Array ? Array.prototype.slice.call(message.encodedMessages) : message.encodedMessages;
            return object;
        };

        /**
         * Converts this GetEgressMessagesResponse to JSON.
         * @function toJSON
         * @memberof pruntime_rpc.GetEgressMessagesResponse
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        GetEgressMessagesResponse.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        return GetEgressMessagesResponse;
    })();

    pruntime_rpc.ContractQueryRequest = (function() {

        /**
         * Properties of a ContractQueryRequest.
         * @memberof pruntime_rpc
         * @interface IContractQueryRequest
         * @property {Uint8Array|null} [encodedEncryptedData] ContractQueryRequest encodedEncryptedData
         * @property {pruntime_rpc.ISignature|null} [signature] ContractQueryRequest signature
         */

        /**
         * Constructs a new ContractQueryRequest.
         * @memberof pruntime_rpc
         * @classdesc Represents a ContractQueryRequest.
         * @implements IContractQueryRequest
         * @constructor
         * @param {pruntime_rpc.IContractQueryRequest=} [properties] Properties to set
         */
        function ContractQueryRequest(properties) {
            if (properties)
                for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * ContractQueryRequest encodedEncryptedData.
         * @member {Uint8Array} encodedEncryptedData
         * @memberof pruntime_rpc.ContractQueryRequest
         * @instance
         */
        ContractQueryRequest.prototype.encodedEncryptedData = $util.newBuffer([]);

        /**
         * ContractQueryRequest signature.
         * @member {pruntime_rpc.ISignature|null|undefined} signature
         * @memberof pruntime_rpc.ContractQueryRequest
         * @instance
         */
        ContractQueryRequest.prototype.signature = null;

        /**
         * Creates a new ContractQueryRequest instance using the specified properties.
         * @function create
         * @memberof pruntime_rpc.ContractQueryRequest
         * @static
         * @param {pruntime_rpc.IContractQueryRequest=} [properties] Properties to set
         * @returns {pruntime_rpc.ContractQueryRequest} ContractQueryRequest instance
         */
        ContractQueryRequest.create = function create(properties) {
            return new ContractQueryRequest(properties);
        };

        /**
         * Encodes the specified ContractQueryRequest message. Does not implicitly {@link pruntime_rpc.ContractQueryRequest.verify|verify} messages.
         * @function encode
         * @memberof pruntime_rpc.ContractQueryRequest
         * @static
         * @param {pruntime_rpc.IContractQueryRequest} message ContractQueryRequest message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        ContractQueryRequest.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.encodedEncryptedData != null && Object.hasOwnProperty.call(message, "encodedEncryptedData"))
                writer.uint32(/* id 1, wireType 2 =*/10).bytes(message.encodedEncryptedData);
            if (message.signature != null && Object.hasOwnProperty.call(message, "signature"))
                $root.pruntime_rpc.Signature.encode(message.signature, writer.uint32(/* id 2, wireType 2 =*/18).fork()).ldelim();
            return writer;
        };

        /**
         * Encodes the specified ContractQueryRequest message, length delimited. Does not implicitly {@link pruntime_rpc.ContractQueryRequest.verify|verify} messages.
         * @function encodeDelimited
         * @memberof pruntime_rpc.ContractQueryRequest
         * @static
         * @param {pruntime_rpc.IContractQueryRequest} message ContractQueryRequest message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        ContractQueryRequest.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes a ContractQueryRequest message from the specified reader or buffer.
         * @function decode
         * @memberof pruntime_rpc.ContractQueryRequest
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {pruntime_rpc.ContractQueryRequest} ContractQueryRequest
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        ContractQueryRequest.decode = function decode(reader, length) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            var end = length === undefined ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.ContractQueryRequest();
            while (reader.pos < end) {
                var tag = reader.uint32();
                switch (tag >>> 3) {
                case 1:
                    message.encodedEncryptedData = reader.bytes();
                    break;
                case 2:
                    message.signature = $root.pruntime_rpc.Signature.decode(reader, reader.uint32());
                    break;
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a ContractQueryRequest message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof pruntime_rpc.ContractQueryRequest
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {pruntime_rpc.ContractQueryRequest} ContractQueryRequest
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        ContractQueryRequest.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies a ContractQueryRequest message.
         * @function verify
         * @memberof pruntime_rpc.ContractQueryRequest
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        ContractQueryRequest.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            if (message.encodedEncryptedData != null && message.hasOwnProperty("encodedEncryptedData"))
                if (!(message.encodedEncryptedData && typeof message.encodedEncryptedData.length === "number" || $util.isString(message.encodedEncryptedData)))
                    return "encodedEncryptedData: buffer expected";
            if (message.signature != null && message.hasOwnProperty("signature")) {
                var error = $root.pruntime_rpc.Signature.verify(message.signature);
                if (error)
                    return "signature." + error;
            }
            return null;
        };

        /**
         * Creates a ContractQueryRequest message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof pruntime_rpc.ContractQueryRequest
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {pruntime_rpc.ContractQueryRequest} ContractQueryRequest
         */
        ContractQueryRequest.fromObject = function fromObject(object) {
            if (object instanceof $root.pruntime_rpc.ContractQueryRequest)
                return object;
            var message = new $root.pruntime_rpc.ContractQueryRequest();
            if (object.encodedEncryptedData != null)
                if (typeof object.encodedEncryptedData === "string")
                    $util.base64.decode(object.encodedEncryptedData, message.encodedEncryptedData = $util.newBuffer($util.base64.length(object.encodedEncryptedData)), 0);
                else if (object.encodedEncryptedData.length)
                    message.encodedEncryptedData = object.encodedEncryptedData;
            if (object.signature != null) {
                if (typeof object.signature !== "object")
                    throw TypeError(".pruntime_rpc.ContractQueryRequest.signature: object expected");
                message.signature = $root.pruntime_rpc.Signature.fromObject(object.signature);
            }
            return message;
        };

        /**
         * Creates a plain object from a ContractQueryRequest message. Also converts values to other types if specified.
         * @function toObject
         * @memberof pruntime_rpc.ContractQueryRequest
         * @static
         * @param {pruntime_rpc.ContractQueryRequest} message ContractQueryRequest
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        ContractQueryRequest.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            var object = {};
            if (options.defaults) {
                if (options.bytes === String)
                    object.encodedEncryptedData = "";
                else {
                    object.encodedEncryptedData = [];
                    if (options.bytes !== Array)
                        object.encodedEncryptedData = $util.newBuffer(object.encodedEncryptedData);
                }
                object.signature = null;
            }
            if (message.encodedEncryptedData != null && message.hasOwnProperty("encodedEncryptedData"))
                object.encodedEncryptedData = options.bytes === String ? $util.base64.encode(message.encodedEncryptedData, 0, message.encodedEncryptedData.length) : options.bytes === Array ? Array.prototype.slice.call(message.encodedEncryptedData) : message.encodedEncryptedData;
            if (message.signature != null && message.hasOwnProperty("signature"))
                object.signature = $root.pruntime_rpc.Signature.toObject(message.signature, options);
            return object;
        };

        /**
         * Converts this ContractQueryRequest to JSON.
         * @function toJSON
         * @memberof pruntime_rpc.ContractQueryRequest
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        ContractQueryRequest.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        return ContractQueryRequest;
    })();

    pruntime_rpc.Signature = (function() {

        /**
         * Properties of a Signature.
         * @memberof pruntime_rpc
         * @interface ISignature
         * @property {pruntime_rpc.ICertificate|null} [signedBy] Signature signedBy
         * @property {pruntime_rpc.SignatureType|null} [signatureType] Signature signatureType
         * @property {Uint8Array|null} [signature] Signature signature
         */

        /**
         * Constructs a new Signature.
         * @memberof pruntime_rpc
         * @classdesc Represents a Signature.
         * @implements ISignature
         * @constructor
         * @param {pruntime_rpc.ISignature=} [properties] Properties to set
         */
        function Signature(properties) {
            if (properties)
                for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * Signature signedBy.
         * @member {pruntime_rpc.ICertificate|null|undefined} signedBy
         * @memberof pruntime_rpc.Signature
         * @instance
         */
        Signature.prototype.signedBy = null;

        /**
         * Signature signatureType.
         * @member {pruntime_rpc.SignatureType} signatureType
         * @memberof pruntime_rpc.Signature
         * @instance
         */
        Signature.prototype.signatureType = 0;

        /**
         * Signature signature.
         * @member {Uint8Array} signature
         * @memberof pruntime_rpc.Signature
         * @instance
         */
        Signature.prototype.signature = $util.newBuffer([]);

        /**
         * Creates a new Signature instance using the specified properties.
         * @function create
         * @memberof pruntime_rpc.Signature
         * @static
         * @param {pruntime_rpc.ISignature=} [properties] Properties to set
         * @returns {pruntime_rpc.Signature} Signature instance
         */
        Signature.create = function create(properties) {
            return new Signature(properties);
        };

        /**
         * Encodes the specified Signature message. Does not implicitly {@link pruntime_rpc.Signature.verify|verify} messages.
         * @function encode
         * @memberof pruntime_rpc.Signature
         * @static
         * @param {pruntime_rpc.ISignature} message Signature message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        Signature.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.signedBy != null && Object.hasOwnProperty.call(message, "signedBy"))
                $root.pruntime_rpc.Certificate.encode(message.signedBy, writer.uint32(/* id 1, wireType 2 =*/10).fork()).ldelim();
            if (message.signatureType != null && Object.hasOwnProperty.call(message, "signatureType"))
                writer.uint32(/* id 2, wireType 0 =*/16).int32(message.signatureType);
            if (message.signature != null && Object.hasOwnProperty.call(message, "signature"))
                writer.uint32(/* id 3, wireType 2 =*/26).bytes(message.signature);
            return writer;
        };

        /**
         * Encodes the specified Signature message, length delimited. Does not implicitly {@link pruntime_rpc.Signature.verify|verify} messages.
         * @function encodeDelimited
         * @memberof pruntime_rpc.Signature
         * @static
         * @param {pruntime_rpc.ISignature} message Signature message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        Signature.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes a Signature message from the specified reader or buffer.
         * @function decode
         * @memberof pruntime_rpc.Signature
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {pruntime_rpc.Signature} Signature
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        Signature.decode = function decode(reader, length) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            var end = length === undefined ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.Signature();
            while (reader.pos < end) {
                var tag = reader.uint32();
                switch (tag >>> 3) {
                case 1:
                    message.signedBy = $root.pruntime_rpc.Certificate.decode(reader, reader.uint32());
                    break;
                case 2:
                    message.signatureType = reader.int32();
                    break;
                case 3:
                    message.signature = reader.bytes();
                    break;
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a Signature message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof pruntime_rpc.Signature
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {pruntime_rpc.Signature} Signature
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        Signature.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies a Signature message.
         * @function verify
         * @memberof pruntime_rpc.Signature
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        Signature.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            if (message.signedBy != null && message.hasOwnProperty("signedBy")) {
                var error = $root.pruntime_rpc.Certificate.verify(message.signedBy);
                if (error)
                    return "signedBy." + error;
            }
            if (message.signatureType != null && message.hasOwnProperty("signatureType"))
                switch (message.signatureType) {
                default:
                    return "signatureType: enum value expected";
                case 0:
                case 1:
                case 2:
                case 3:
                case 4:
                case 5:
                    break;
                }
            if (message.signature != null && message.hasOwnProperty("signature"))
                if (!(message.signature && typeof message.signature.length === "number" || $util.isString(message.signature)))
                    return "signature: buffer expected";
            return null;
        };

        /**
         * Creates a Signature message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof pruntime_rpc.Signature
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {pruntime_rpc.Signature} Signature
         */
        Signature.fromObject = function fromObject(object) {
            if (object instanceof $root.pruntime_rpc.Signature)
                return object;
            var message = new $root.pruntime_rpc.Signature();
            if (object.signedBy != null) {
                if (typeof object.signedBy !== "object")
                    throw TypeError(".pruntime_rpc.Signature.signedBy: object expected");
                message.signedBy = $root.pruntime_rpc.Certificate.fromObject(object.signedBy);
            }
            switch (object.signatureType) {
            case "Ed25519":
            case 0:
                message.signatureType = 0;
                break;
            case "Sr25519":
            case 1:
                message.signatureType = 1;
                break;
            case "Ecdsa":
            case 2:
                message.signatureType = 2;
                break;
            case "Ed25519WrapBytes":
            case 3:
                message.signatureType = 3;
                break;
            case "Sr25519WrapBytes":
            case 4:
                message.signatureType = 4;
                break;
            case "EcdsaWrapBytes":
            case 5:
                message.signatureType = 5;
                break;
            }
            if (object.signature != null)
                if (typeof object.signature === "string")
                    $util.base64.decode(object.signature, message.signature = $util.newBuffer($util.base64.length(object.signature)), 0);
                else if (object.signature.length)
                    message.signature = object.signature;
            return message;
        };

        /**
         * Creates a plain object from a Signature message. Also converts values to other types if specified.
         * @function toObject
         * @memberof pruntime_rpc.Signature
         * @static
         * @param {pruntime_rpc.Signature} message Signature
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        Signature.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            var object = {};
            if (options.defaults) {
                object.signedBy = null;
                object.signatureType = options.enums === String ? "Ed25519" : 0;
                if (options.bytes === String)
                    object.signature = "";
                else {
                    object.signature = [];
                    if (options.bytes !== Array)
                        object.signature = $util.newBuffer(object.signature);
                }
            }
            if (message.signedBy != null && message.hasOwnProperty("signedBy"))
                object.signedBy = $root.pruntime_rpc.Certificate.toObject(message.signedBy, options);
            if (message.signatureType != null && message.hasOwnProperty("signatureType"))
                object.signatureType = options.enums === String ? $root.pruntime_rpc.SignatureType[message.signatureType] : message.signatureType;
            if (message.signature != null && message.hasOwnProperty("signature"))
                object.signature = options.bytes === String ? $util.base64.encode(message.signature, 0, message.signature.length) : options.bytes === Array ? Array.prototype.slice.call(message.signature) : message.signature;
            return object;
        };

        /**
         * Converts this Signature to JSON.
         * @function toJSON
         * @memberof pruntime_rpc.Signature
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        Signature.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        return Signature;
    })();

    pruntime_rpc.Certificate = (function() {

        /**
         * Properties of a Certificate.
         * @memberof pruntime_rpc
         * @interface ICertificate
         * @property {Uint8Array|null} [encodedBody] Certificate encodedBody
         * @property {pruntime_rpc.ISignature|null} [signature] Certificate signature
         */

        /**
         * Constructs a new Certificate.
         * @memberof pruntime_rpc
         * @classdesc Represents a Certificate.
         * @implements ICertificate
         * @constructor
         * @param {pruntime_rpc.ICertificate=} [properties] Properties to set
         */
        function Certificate(properties) {
            if (properties)
                for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * Certificate encodedBody.
         * @member {Uint8Array} encodedBody
         * @memberof pruntime_rpc.Certificate
         * @instance
         */
        Certificate.prototype.encodedBody = $util.newBuffer([]);

        /**
         * Certificate signature.
         * @member {pruntime_rpc.ISignature|null|undefined} signature
         * @memberof pruntime_rpc.Certificate
         * @instance
         */
        Certificate.prototype.signature = null;

        /**
         * Creates a new Certificate instance using the specified properties.
         * @function create
         * @memberof pruntime_rpc.Certificate
         * @static
         * @param {pruntime_rpc.ICertificate=} [properties] Properties to set
         * @returns {pruntime_rpc.Certificate} Certificate instance
         */
        Certificate.create = function create(properties) {
            return new Certificate(properties);
        };

        /**
         * Encodes the specified Certificate message. Does not implicitly {@link pruntime_rpc.Certificate.verify|verify} messages.
         * @function encode
         * @memberof pruntime_rpc.Certificate
         * @static
         * @param {pruntime_rpc.ICertificate} message Certificate message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        Certificate.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.encodedBody != null && Object.hasOwnProperty.call(message, "encodedBody"))
                writer.uint32(/* id 1, wireType 2 =*/10).bytes(message.encodedBody);
            if (message.signature != null && Object.hasOwnProperty.call(message, "signature"))
                $root.pruntime_rpc.Signature.encode(message.signature, writer.uint32(/* id 2, wireType 2 =*/18).fork()).ldelim();
            return writer;
        };

        /**
         * Encodes the specified Certificate message, length delimited. Does not implicitly {@link pruntime_rpc.Certificate.verify|verify} messages.
         * @function encodeDelimited
         * @memberof pruntime_rpc.Certificate
         * @static
         * @param {pruntime_rpc.ICertificate} message Certificate message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        Certificate.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes a Certificate message from the specified reader or buffer.
         * @function decode
         * @memberof pruntime_rpc.Certificate
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {pruntime_rpc.Certificate} Certificate
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        Certificate.decode = function decode(reader, length) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            var end = length === undefined ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.Certificate();
            while (reader.pos < end) {
                var tag = reader.uint32();
                switch (tag >>> 3) {
                case 1:
                    message.encodedBody = reader.bytes();
                    break;
                case 2:
                    message.signature = $root.pruntime_rpc.Signature.decode(reader, reader.uint32());
                    break;
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a Certificate message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof pruntime_rpc.Certificate
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {pruntime_rpc.Certificate} Certificate
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        Certificate.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies a Certificate message.
         * @function verify
         * @memberof pruntime_rpc.Certificate
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        Certificate.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            if (message.encodedBody != null && message.hasOwnProperty("encodedBody"))
                if (!(message.encodedBody && typeof message.encodedBody.length === "number" || $util.isString(message.encodedBody)))
                    return "encodedBody: buffer expected";
            if (message.signature != null && message.hasOwnProperty("signature")) {
                var error = $root.pruntime_rpc.Signature.verify(message.signature);
                if (error)
                    return "signature." + error;
            }
            return null;
        };

        /**
         * Creates a Certificate message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof pruntime_rpc.Certificate
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {pruntime_rpc.Certificate} Certificate
         */
        Certificate.fromObject = function fromObject(object) {
            if (object instanceof $root.pruntime_rpc.Certificate)
                return object;
            var message = new $root.pruntime_rpc.Certificate();
            if (object.encodedBody != null)
                if (typeof object.encodedBody === "string")
                    $util.base64.decode(object.encodedBody, message.encodedBody = $util.newBuffer($util.base64.length(object.encodedBody)), 0);
                else if (object.encodedBody.length)
                    message.encodedBody = object.encodedBody;
            if (object.signature != null) {
                if (typeof object.signature !== "object")
                    throw TypeError(".pruntime_rpc.Certificate.signature: object expected");
                message.signature = $root.pruntime_rpc.Signature.fromObject(object.signature);
            }
            return message;
        };

        /**
         * Creates a plain object from a Certificate message. Also converts values to other types if specified.
         * @function toObject
         * @memberof pruntime_rpc.Certificate
         * @static
         * @param {pruntime_rpc.Certificate} message Certificate
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        Certificate.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            var object = {};
            if (options.defaults) {
                if (options.bytes === String)
                    object.encodedBody = "";
                else {
                    object.encodedBody = [];
                    if (options.bytes !== Array)
                        object.encodedBody = $util.newBuffer(object.encodedBody);
                }
                object.signature = null;
            }
            if (message.encodedBody != null && message.hasOwnProperty("encodedBody"))
                object.encodedBody = options.bytes === String ? $util.base64.encode(message.encodedBody, 0, message.encodedBody.length) : options.bytes === Array ? Array.prototype.slice.call(message.encodedBody) : message.encodedBody;
            if (message.signature != null && message.hasOwnProperty("signature"))
                object.signature = $root.pruntime_rpc.Signature.toObject(message.signature, options);
            return object;
        };

        /**
         * Converts this Certificate to JSON.
         * @function toJSON
         * @memberof pruntime_rpc.Certificate
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        Certificate.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        return Certificate;
    })();

    /**
     * SignatureType enum.
     * @name pruntime_rpc.SignatureType
     * @enum {number}
     * @property {number} Ed25519=0 Ed25519 value
     * @property {number} Sr25519=1 Sr25519 value
     * @property {number} Ecdsa=2 Ecdsa value
     * @property {number} Ed25519WrapBytes=3 Ed25519WrapBytes value
     * @property {number} Sr25519WrapBytes=4 Sr25519WrapBytes value
     * @property {number} EcdsaWrapBytes=5 EcdsaWrapBytes value
     */
    pruntime_rpc.SignatureType = (function() {
        var valuesById = {}, values = Object.create(valuesById);
        values[valuesById[0] = "Ed25519"] = 0;
        values[valuesById[1] = "Sr25519"] = 1;
        values[valuesById[2] = "Ecdsa"] = 2;
        values[valuesById[3] = "Ed25519WrapBytes"] = 3;
        values[valuesById[4] = "Sr25519WrapBytes"] = 4;
        values[valuesById[5] = "EcdsaWrapBytes"] = 5;
        return values;
    })();

    pruntime_rpc.ContractQueryResponse = (function() {

        /**
         * Properties of a ContractQueryResponse.
         * @memberof pruntime_rpc
         * @interface IContractQueryResponse
         * @property {Uint8Array|null} [encodedEncryptedData] ContractQueryResponse encodedEncryptedData
         */

        /**
         * Constructs a new ContractQueryResponse.
         * @memberof pruntime_rpc
         * @classdesc Represents a ContractQueryResponse.
         * @implements IContractQueryResponse
         * @constructor
         * @param {pruntime_rpc.IContractQueryResponse=} [properties] Properties to set
         */
        function ContractQueryResponse(properties) {
            if (properties)
                for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * ContractQueryResponse encodedEncryptedData.
         * @member {Uint8Array} encodedEncryptedData
         * @memberof pruntime_rpc.ContractQueryResponse
         * @instance
         */
        ContractQueryResponse.prototype.encodedEncryptedData = $util.newBuffer([]);

        /**
         * Creates a new ContractQueryResponse instance using the specified properties.
         * @function create
         * @memberof pruntime_rpc.ContractQueryResponse
         * @static
         * @param {pruntime_rpc.IContractQueryResponse=} [properties] Properties to set
         * @returns {pruntime_rpc.ContractQueryResponse} ContractQueryResponse instance
         */
        ContractQueryResponse.create = function create(properties) {
            return new ContractQueryResponse(properties);
        };

        /**
         * Encodes the specified ContractQueryResponse message. Does not implicitly {@link pruntime_rpc.ContractQueryResponse.verify|verify} messages.
         * @function encode
         * @memberof pruntime_rpc.ContractQueryResponse
         * @static
         * @param {pruntime_rpc.IContractQueryResponse} message ContractQueryResponse message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        ContractQueryResponse.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.encodedEncryptedData != null && Object.hasOwnProperty.call(message, "encodedEncryptedData"))
                writer.uint32(/* id 1, wireType 2 =*/10).bytes(message.encodedEncryptedData);
            return writer;
        };

        /**
         * Encodes the specified ContractQueryResponse message, length delimited. Does not implicitly {@link pruntime_rpc.ContractQueryResponse.verify|verify} messages.
         * @function encodeDelimited
         * @memberof pruntime_rpc.ContractQueryResponse
         * @static
         * @param {pruntime_rpc.IContractQueryResponse} message ContractQueryResponse message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        ContractQueryResponse.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes a ContractQueryResponse message from the specified reader or buffer.
         * @function decode
         * @memberof pruntime_rpc.ContractQueryResponse
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {pruntime_rpc.ContractQueryResponse} ContractQueryResponse
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        ContractQueryResponse.decode = function decode(reader, length) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            var end = length === undefined ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.ContractQueryResponse();
            while (reader.pos < end) {
                var tag = reader.uint32();
                switch (tag >>> 3) {
                case 1:
                    message.encodedEncryptedData = reader.bytes();
                    break;
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a ContractQueryResponse message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof pruntime_rpc.ContractQueryResponse
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {pruntime_rpc.ContractQueryResponse} ContractQueryResponse
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        ContractQueryResponse.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies a ContractQueryResponse message.
         * @function verify
         * @memberof pruntime_rpc.ContractQueryResponse
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        ContractQueryResponse.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            if (message.encodedEncryptedData != null && message.hasOwnProperty("encodedEncryptedData"))
                if (!(message.encodedEncryptedData && typeof message.encodedEncryptedData.length === "number" || $util.isString(message.encodedEncryptedData)))
                    return "encodedEncryptedData: buffer expected";
            return null;
        };

        /**
         * Creates a ContractQueryResponse message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof pruntime_rpc.ContractQueryResponse
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {pruntime_rpc.ContractQueryResponse} ContractQueryResponse
         */
        ContractQueryResponse.fromObject = function fromObject(object) {
            if (object instanceof $root.pruntime_rpc.ContractQueryResponse)
                return object;
            var message = new $root.pruntime_rpc.ContractQueryResponse();
            if (object.encodedEncryptedData != null)
                if (typeof object.encodedEncryptedData === "string")
                    $util.base64.decode(object.encodedEncryptedData, message.encodedEncryptedData = $util.newBuffer($util.base64.length(object.encodedEncryptedData)), 0);
                else if (object.encodedEncryptedData.length)
                    message.encodedEncryptedData = object.encodedEncryptedData;
            return message;
        };

        /**
         * Creates a plain object from a ContractQueryResponse message. Also converts values to other types if specified.
         * @function toObject
         * @memberof pruntime_rpc.ContractQueryResponse
         * @static
         * @param {pruntime_rpc.ContractQueryResponse} message ContractQueryResponse
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        ContractQueryResponse.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            var object = {};
            if (options.defaults)
                if (options.bytes === String)
                    object.encodedEncryptedData = "";
                else {
                    object.encodedEncryptedData = [];
                    if (options.bytes !== Array)
                        object.encodedEncryptedData = $util.newBuffer(object.encodedEncryptedData);
                }
            if (message.encodedEncryptedData != null && message.hasOwnProperty("encodedEncryptedData"))
                object.encodedEncryptedData = options.bytes === String ? $util.base64.encode(message.encodedEncryptedData, 0, message.encodedEncryptedData.length) : options.bytes === Array ? Array.prototype.slice.call(message.encodedEncryptedData) : message.encodedEncryptedData;
            return object;
        };

        /**
         * Converts this ContractQueryResponse to JSON.
         * @function toJSON
         * @memberof pruntime_rpc.ContractQueryResponse
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        ContractQueryResponse.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        return ContractQueryResponse;
    })();

    pruntime_rpc.GetWorkerStateRequest = (function() {

        /**
         * Properties of a GetWorkerStateRequest.
         * @memberof pruntime_rpc
         * @interface IGetWorkerStateRequest
         * @property {Uint8Array|null} [publicKey] GetWorkerStateRequest publicKey
         */

        /**
         * Constructs a new GetWorkerStateRequest.
         * @memberof pruntime_rpc
         * @classdesc Represents a GetWorkerStateRequest.
         * @implements IGetWorkerStateRequest
         * @constructor
         * @param {pruntime_rpc.IGetWorkerStateRequest=} [properties] Properties to set
         */
        function GetWorkerStateRequest(properties) {
            if (properties)
                for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * GetWorkerStateRequest publicKey.
         * @member {Uint8Array} publicKey
         * @memberof pruntime_rpc.GetWorkerStateRequest
         * @instance
         */
        GetWorkerStateRequest.prototype.publicKey = $util.newBuffer([]);

        /**
         * Creates a new GetWorkerStateRequest instance using the specified properties.
         * @function create
         * @memberof pruntime_rpc.GetWorkerStateRequest
         * @static
         * @param {pruntime_rpc.IGetWorkerStateRequest=} [properties] Properties to set
         * @returns {pruntime_rpc.GetWorkerStateRequest} GetWorkerStateRequest instance
         */
        GetWorkerStateRequest.create = function create(properties) {
            return new GetWorkerStateRequest(properties);
        };

        /**
         * Encodes the specified GetWorkerStateRequest message. Does not implicitly {@link pruntime_rpc.GetWorkerStateRequest.verify|verify} messages.
         * @function encode
         * @memberof pruntime_rpc.GetWorkerStateRequest
         * @static
         * @param {pruntime_rpc.IGetWorkerStateRequest} message GetWorkerStateRequest message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        GetWorkerStateRequest.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.publicKey != null && Object.hasOwnProperty.call(message, "publicKey"))
                writer.uint32(/* id 1, wireType 2 =*/10).bytes(message.publicKey);
            return writer;
        };

        /**
         * Encodes the specified GetWorkerStateRequest message, length delimited. Does not implicitly {@link pruntime_rpc.GetWorkerStateRequest.verify|verify} messages.
         * @function encodeDelimited
         * @memberof pruntime_rpc.GetWorkerStateRequest
         * @static
         * @param {pruntime_rpc.IGetWorkerStateRequest} message GetWorkerStateRequest message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        GetWorkerStateRequest.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes a GetWorkerStateRequest message from the specified reader or buffer.
         * @function decode
         * @memberof pruntime_rpc.GetWorkerStateRequest
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {pruntime_rpc.GetWorkerStateRequest} GetWorkerStateRequest
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        GetWorkerStateRequest.decode = function decode(reader, length) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            var end = length === undefined ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.GetWorkerStateRequest();
            while (reader.pos < end) {
                var tag = reader.uint32();
                switch (tag >>> 3) {
                case 1:
                    message.publicKey = reader.bytes();
                    break;
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a GetWorkerStateRequest message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof pruntime_rpc.GetWorkerStateRequest
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {pruntime_rpc.GetWorkerStateRequest} GetWorkerStateRequest
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        GetWorkerStateRequest.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies a GetWorkerStateRequest message.
         * @function verify
         * @memberof pruntime_rpc.GetWorkerStateRequest
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        GetWorkerStateRequest.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            if (message.publicKey != null && message.hasOwnProperty("publicKey"))
                if (!(message.publicKey && typeof message.publicKey.length === "number" || $util.isString(message.publicKey)))
                    return "publicKey: buffer expected";
            return null;
        };

        /**
         * Creates a GetWorkerStateRequest message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof pruntime_rpc.GetWorkerStateRequest
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {pruntime_rpc.GetWorkerStateRequest} GetWorkerStateRequest
         */
        GetWorkerStateRequest.fromObject = function fromObject(object) {
            if (object instanceof $root.pruntime_rpc.GetWorkerStateRequest)
                return object;
            var message = new $root.pruntime_rpc.GetWorkerStateRequest();
            if (object.publicKey != null)
                if (typeof object.publicKey === "string")
                    $util.base64.decode(object.publicKey, message.publicKey = $util.newBuffer($util.base64.length(object.publicKey)), 0);
                else if (object.publicKey.length)
                    message.publicKey = object.publicKey;
            return message;
        };

        /**
         * Creates a plain object from a GetWorkerStateRequest message. Also converts values to other types if specified.
         * @function toObject
         * @memberof pruntime_rpc.GetWorkerStateRequest
         * @static
         * @param {pruntime_rpc.GetWorkerStateRequest} message GetWorkerStateRequest
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        GetWorkerStateRequest.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            var object = {};
            if (options.defaults)
                if (options.bytes === String)
                    object.publicKey = "";
                else {
                    object.publicKey = [];
                    if (options.bytes !== Array)
                        object.publicKey = $util.newBuffer(object.publicKey);
                }
            if (message.publicKey != null && message.hasOwnProperty("publicKey"))
                object.publicKey = options.bytes === String ? $util.base64.encode(message.publicKey, 0, message.publicKey.length) : options.bytes === Array ? Array.prototype.slice.call(message.publicKey) : message.publicKey;
            return object;
        };

        /**
         * Converts this GetWorkerStateRequest to JSON.
         * @function toJSON
         * @memberof pruntime_rpc.GetWorkerStateRequest
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        GetWorkerStateRequest.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        return GetWorkerStateRequest;
    })();

    pruntime_rpc.WorkerStat = (function() {

        /**
         * Properties of a WorkerStat.
         * @memberof pruntime_rpc
         * @interface IWorkerStat
         * @property {number|null} [lastHeartbeatForBlock] WorkerStat lastHeartbeatForBlock
         * @property {number|null} [lastHeartbeatAtBlock] WorkerStat lastHeartbeatAtBlock
         * @property {pruntime_rpc.ResponsiveEvent|null} [lastGkResponsiveEvent] WorkerStat lastGkResponsiveEvent
         * @property {number|null} [lastGkResponsiveEventAtBlock] WorkerStat lastGkResponsiveEventAtBlock
         */

        /**
         * Constructs a new WorkerStat.
         * @memberof pruntime_rpc
         * @classdesc Represents a WorkerStat.
         * @implements IWorkerStat
         * @constructor
         * @param {pruntime_rpc.IWorkerStat=} [properties] Properties to set
         */
        function WorkerStat(properties) {
            if (properties)
                for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * WorkerStat lastHeartbeatForBlock.
         * @member {number} lastHeartbeatForBlock
         * @memberof pruntime_rpc.WorkerStat
         * @instance
         */
        WorkerStat.prototype.lastHeartbeatForBlock = 0;

        /**
         * WorkerStat lastHeartbeatAtBlock.
         * @member {number} lastHeartbeatAtBlock
         * @memberof pruntime_rpc.WorkerStat
         * @instance
         */
        WorkerStat.prototype.lastHeartbeatAtBlock = 0;

        /**
         * WorkerStat lastGkResponsiveEvent.
         * @member {pruntime_rpc.ResponsiveEvent} lastGkResponsiveEvent
         * @memberof pruntime_rpc.WorkerStat
         * @instance
         */
        WorkerStat.prototype.lastGkResponsiveEvent = 0;

        /**
         * WorkerStat lastGkResponsiveEventAtBlock.
         * @member {number} lastGkResponsiveEventAtBlock
         * @memberof pruntime_rpc.WorkerStat
         * @instance
         */
        WorkerStat.prototype.lastGkResponsiveEventAtBlock = 0;

        /**
         * Creates a new WorkerStat instance using the specified properties.
         * @function create
         * @memberof pruntime_rpc.WorkerStat
         * @static
         * @param {pruntime_rpc.IWorkerStat=} [properties] Properties to set
         * @returns {pruntime_rpc.WorkerStat} WorkerStat instance
         */
        WorkerStat.create = function create(properties) {
            return new WorkerStat(properties);
        };

        /**
         * Encodes the specified WorkerStat message. Does not implicitly {@link pruntime_rpc.WorkerStat.verify|verify} messages.
         * @function encode
         * @memberof pruntime_rpc.WorkerStat
         * @static
         * @param {pruntime_rpc.IWorkerStat} message WorkerStat message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        WorkerStat.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.lastHeartbeatForBlock != null && Object.hasOwnProperty.call(message, "lastHeartbeatForBlock"))
                writer.uint32(/* id 1, wireType 0 =*/8).uint32(message.lastHeartbeatForBlock);
            if (message.lastHeartbeatAtBlock != null && Object.hasOwnProperty.call(message, "lastHeartbeatAtBlock"))
                writer.uint32(/* id 2, wireType 0 =*/16).uint32(message.lastHeartbeatAtBlock);
            if (message.lastGkResponsiveEvent != null && Object.hasOwnProperty.call(message, "lastGkResponsiveEvent"))
                writer.uint32(/* id 3, wireType 0 =*/24).int32(message.lastGkResponsiveEvent);
            if (message.lastGkResponsiveEventAtBlock != null && Object.hasOwnProperty.call(message, "lastGkResponsiveEventAtBlock"))
                writer.uint32(/* id 4, wireType 0 =*/32).uint32(message.lastGkResponsiveEventAtBlock);
            return writer;
        };

        /**
         * Encodes the specified WorkerStat message, length delimited. Does not implicitly {@link pruntime_rpc.WorkerStat.verify|verify} messages.
         * @function encodeDelimited
         * @memberof pruntime_rpc.WorkerStat
         * @static
         * @param {pruntime_rpc.IWorkerStat} message WorkerStat message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        WorkerStat.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes a WorkerStat message from the specified reader or buffer.
         * @function decode
         * @memberof pruntime_rpc.WorkerStat
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {pruntime_rpc.WorkerStat} WorkerStat
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        WorkerStat.decode = function decode(reader, length) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            var end = length === undefined ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.WorkerStat();
            while (reader.pos < end) {
                var tag = reader.uint32();
                switch (tag >>> 3) {
                case 1:
                    message.lastHeartbeatForBlock = reader.uint32();
                    break;
                case 2:
                    message.lastHeartbeatAtBlock = reader.uint32();
                    break;
                case 3:
                    message.lastGkResponsiveEvent = reader.int32();
                    break;
                case 4:
                    message.lastGkResponsiveEventAtBlock = reader.uint32();
                    break;
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a WorkerStat message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof pruntime_rpc.WorkerStat
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {pruntime_rpc.WorkerStat} WorkerStat
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        WorkerStat.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies a WorkerStat message.
         * @function verify
         * @memberof pruntime_rpc.WorkerStat
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        WorkerStat.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            if (message.lastHeartbeatForBlock != null && message.hasOwnProperty("lastHeartbeatForBlock"))
                if (!$util.isInteger(message.lastHeartbeatForBlock))
                    return "lastHeartbeatForBlock: integer expected";
            if (message.lastHeartbeatAtBlock != null && message.hasOwnProperty("lastHeartbeatAtBlock"))
                if (!$util.isInteger(message.lastHeartbeatAtBlock))
                    return "lastHeartbeatAtBlock: integer expected";
            if (message.lastGkResponsiveEvent != null && message.hasOwnProperty("lastGkResponsiveEvent"))
                switch (message.lastGkResponsiveEvent) {
                default:
                    return "lastGkResponsiveEvent: enum value expected";
                case 0:
                case 1:
                case 2:
                    break;
                }
            if (message.lastGkResponsiveEventAtBlock != null && message.hasOwnProperty("lastGkResponsiveEventAtBlock"))
                if (!$util.isInteger(message.lastGkResponsiveEventAtBlock))
                    return "lastGkResponsiveEventAtBlock: integer expected";
            return null;
        };

        /**
         * Creates a WorkerStat message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof pruntime_rpc.WorkerStat
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {pruntime_rpc.WorkerStat} WorkerStat
         */
        WorkerStat.fromObject = function fromObject(object) {
            if (object instanceof $root.pruntime_rpc.WorkerStat)
                return object;
            var message = new $root.pruntime_rpc.WorkerStat();
            if (object.lastHeartbeatForBlock != null)
                message.lastHeartbeatForBlock = object.lastHeartbeatForBlock >>> 0;
            if (object.lastHeartbeatAtBlock != null)
                message.lastHeartbeatAtBlock = object.lastHeartbeatAtBlock >>> 0;
            switch (object.lastGkResponsiveEvent) {
            case "NoEvent":
            case 0:
                message.lastGkResponsiveEvent = 0;
                break;
            case "EnterUnresponsive":
            case 1:
                message.lastGkResponsiveEvent = 1;
                break;
            case "ExitUnresponsive":
            case 2:
                message.lastGkResponsiveEvent = 2;
                break;
            }
            if (object.lastGkResponsiveEventAtBlock != null)
                message.lastGkResponsiveEventAtBlock = object.lastGkResponsiveEventAtBlock >>> 0;
            return message;
        };

        /**
         * Creates a plain object from a WorkerStat message. Also converts values to other types if specified.
         * @function toObject
         * @memberof pruntime_rpc.WorkerStat
         * @static
         * @param {pruntime_rpc.WorkerStat} message WorkerStat
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        WorkerStat.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            var object = {};
            if (options.defaults) {
                object.lastHeartbeatForBlock = 0;
                object.lastHeartbeatAtBlock = 0;
                object.lastGkResponsiveEvent = options.enums === String ? "NoEvent" : 0;
                object.lastGkResponsiveEventAtBlock = 0;
            }
            if (message.lastHeartbeatForBlock != null && message.hasOwnProperty("lastHeartbeatForBlock"))
                object.lastHeartbeatForBlock = message.lastHeartbeatForBlock;
            if (message.lastHeartbeatAtBlock != null && message.hasOwnProperty("lastHeartbeatAtBlock"))
                object.lastHeartbeatAtBlock = message.lastHeartbeatAtBlock;
            if (message.lastGkResponsiveEvent != null && message.hasOwnProperty("lastGkResponsiveEvent"))
                object.lastGkResponsiveEvent = options.enums === String ? $root.pruntime_rpc.ResponsiveEvent[message.lastGkResponsiveEvent] : message.lastGkResponsiveEvent;
            if (message.lastGkResponsiveEventAtBlock != null && message.hasOwnProperty("lastGkResponsiveEventAtBlock"))
                object.lastGkResponsiveEventAtBlock = message.lastGkResponsiveEventAtBlock;
            return object;
        };

        /**
         * Converts this WorkerStat to JSON.
         * @function toJSON
         * @memberof pruntime_rpc.WorkerStat
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        WorkerStat.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        return WorkerStat;
    })();

    pruntime_rpc.WorkerState = (function() {

        /**
         * Properties of a WorkerState.
         * @memberof pruntime_rpc
         * @interface IWorkerState
         * @property {boolean|null} [registered] WorkerState registered
         * @property {boolean|null} [unresponsive] WorkerState unresponsive
         * @property {pruntime_rpc.IBenchState|null} [benchState] WorkerState benchState
         * @property {pruntime_rpc.IMiningState|null} [miningState] WorkerState miningState
         * @property {Array.<number>|null} [waitingHeartbeats] WorkerState waitingHeartbeats
         * @property {pruntime_rpc.ITokenomicInfo|null} [tokenomicInfo] WorkerState tokenomicInfo
         * @property {pruntime_rpc.IWorkerStat|null} [stat] WorkerState stat
         */

        /**
         * Constructs a new WorkerState.
         * @memberof pruntime_rpc
         * @classdesc Represents a WorkerState.
         * @implements IWorkerState
         * @constructor
         * @param {pruntime_rpc.IWorkerState=} [properties] Properties to set
         */
        function WorkerState(properties) {
            this.waitingHeartbeats = [];
            if (properties)
                for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * WorkerState registered.
         * @member {boolean} registered
         * @memberof pruntime_rpc.WorkerState
         * @instance
         */
        WorkerState.prototype.registered = false;

        /**
         * WorkerState unresponsive.
         * @member {boolean} unresponsive
         * @memberof pruntime_rpc.WorkerState
         * @instance
         */
        WorkerState.prototype.unresponsive = false;

        /**
         * WorkerState benchState.
         * @member {pruntime_rpc.IBenchState|null|undefined} benchState
         * @memberof pruntime_rpc.WorkerState
         * @instance
         */
        WorkerState.prototype.benchState = null;

        /**
         * WorkerState miningState.
         * @member {pruntime_rpc.IMiningState|null|undefined} miningState
         * @memberof pruntime_rpc.WorkerState
         * @instance
         */
        WorkerState.prototype.miningState = null;

        /**
         * WorkerState waitingHeartbeats.
         * @member {Array.<number>} waitingHeartbeats
         * @memberof pruntime_rpc.WorkerState
         * @instance
         */
        WorkerState.prototype.waitingHeartbeats = $util.emptyArray;

        /**
         * WorkerState tokenomicInfo.
         * @member {pruntime_rpc.ITokenomicInfo|null|undefined} tokenomicInfo
         * @memberof pruntime_rpc.WorkerState
         * @instance
         */
        WorkerState.prototype.tokenomicInfo = null;

        /**
         * WorkerState stat.
         * @member {pruntime_rpc.IWorkerStat|null|undefined} stat
         * @memberof pruntime_rpc.WorkerState
         * @instance
         */
        WorkerState.prototype.stat = null;

        /**
         * Creates a new WorkerState instance using the specified properties.
         * @function create
         * @memberof pruntime_rpc.WorkerState
         * @static
         * @param {pruntime_rpc.IWorkerState=} [properties] Properties to set
         * @returns {pruntime_rpc.WorkerState} WorkerState instance
         */
        WorkerState.create = function create(properties) {
            return new WorkerState(properties);
        };

        /**
         * Encodes the specified WorkerState message. Does not implicitly {@link pruntime_rpc.WorkerState.verify|verify} messages.
         * @function encode
         * @memberof pruntime_rpc.WorkerState
         * @static
         * @param {pruntime_rpc.IWorkerState} message WorkerState message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        WorkerState.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.registered != null && Object.hasOwnProperty.call(message, "registered"))
                writer.uint32(/* id 1, wireType 0 =*/8).bool(message.registered);
            if (message.unresponsive != null && Object.hasOwnProperty.call(message, "unresponsive"))
                writer.uint32(/* id 2, wireType 0 =*/16).bool(message.unresponsive);
            if (message.benchState != null && Object.hasOwnProperty.call(message, "benchState"))
                $root.pruntime_rpc.BenchState.encode(message.benchState, writer.uint32(/* id 3, wireType 2 =*/26).fork()).ldelim();
            if (message.miningState != null && Object.hasOwnProperty.call(message, "miningState"))
                $root.pruntime_rpc.MiningState.encode(message.miningState, writer.uint32(/* id 4, wireType 2 =*/34).fork()).ldelim();
            if (message.waitingHeartbeats != null && message.waitingHeartbeats.length) {
                writer.uint32(/* id 5, wireType 2 =*/42).fork();
                for (var i = 0; i < message.waitingHeartbeats.length; ++i)
                    writer.uint32(message.waitingHeartbeats[i]);
                writer.ldelim();
            }
            if (message.tokenomicInfo != null && Object.hasOwnProperty.call(message, "tokenomicInfo"))
                $root.pruntime_rpc.TokenomicInfo.encode(message.tokenomicInfo, writer.uint32(/* id 10, wireType 2 =*/82).fork()).ldelim();
            if (message.stat != null && Object.hasOwnProperty.call(message, "stat"))
                $root.pruntime_rpc.WorkerStat.encode(message.stat, writer.uint32(/* id 11, wireType 2 =*/90).fork()).ldelim();
            return writer;
        };

        /**
         * Encodes the specified WorkerState message, length delimited. Does not implicitly {@link pruntime_rpc.WorkerState.verify|verify} messages.
         * @function encodeDelimited
         * @memberof pruntime_rpc.WorkerState
         * @static
         * @param {pruntime_rpc.IWorkerState} message WorkerState message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        WorkerState.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes a WorkerState message from the specified reader or buffer.
         * @function decode
         * @memberof pruntime_rpc.WorkerState
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {pruntime_rpc.WorkerState} WorkerState
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        WorkerState.decode = function decode(reader, length) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            var end = length === undefined ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.WorkerState();
            while (reader.pos < end) {
                var tag = reader.uint32();
                switch (tag >>> 3) {
                case 1:
                    message.registered = reader.bool();
                    break;
                case 2:
                    message.unresponsive = reader.bool();
                    break;
                case 3:
                    message.benchState = $root.pruntime_rpc.BenchState.decode(reader, reader.uint32());
                    break;
                case 4:
                    message.miningState = $root.pruntime_rpc.MiningState.decode(reader, reader.uint32());
                    break;
                case 5:
                    if (!(message.waitingHeartbeats && message.waitingHeartbeats.length))
                        message.waitingHeartbeats = [];
                    if ((tag & 7) === 2) {
                        var end2 = reader.uint32() + reader.pos;
                        while (reader.pos < end2)
                            message.waitingHeartbeats.push(reader.uint32());
                    } else
                        message.waitingHeartbeats.push(reader.uint32());
                    break;
                case 10:
                    message.tokenomicInfo = $root.pruntime_rpc.TokenomicInfo.decode(reader, reader.uint32());
                    break;
                case 11:
                    message.stat = $root.pruntime_rpc.WorkerStat.decode(reader, reader.uint32());
                    break;
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a WorkerState message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof pruntime_rpc.WorkerState
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {pruntime_rpc.WorkerState} WorkerState
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        WorkerState.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies a WorkerState message.
         * @function verify
         * @memberof pruntime_rpc.WorkerState
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        WorkerState.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            if (message.registered != null && message.hasOwnProperty("registered"))
                if (typeof message.registered !== "boolean")
                    return "registered: boolean expected";
            if (message.unresponsive != null && message.hasOwnProperty("unresponsive"))
                if (typeof message.unresponsive !== "boolean")
                    return "unresponsive: boolean expected";
            if (message.benchState != null && message.hasOwnProperty("benchState")) {
                var error = $root.pruntime_rpc.BenchState.verify(message.benchState);
                if (error)
                    return "benchState." + error;
            }
            if (message.miningState != null && message.hasOwnProperty("miningState")) {
                var error = $root.pruntime_rpc.MiningState.verify(message.miningState);
                if (error)
                    return "miningState." + error;
            }
            if (message.waitingHeartbeats != null && message.hasOwnProperty("waitingHeartbeats")) {
                if (!Array.isArray(message.waitingHeartbeats))
                    return "waitingHeartbeats: array expected";
                for (var i = 0; i < message.waitingHeartbeats.length; ++i)
                    if (!$util.isInteger(message.waitingHeartbeats[i]))
                        return "waitingHeartbeats: integer[] expected";
            }
            if (message.tokenomicInfo != null && message.hasOwnProperty("tokenomicInfo")) {
                var error = $root.pruntime_rpc.TokenomicInfo.verify(message.tokenomicInfo);
                if (error)
                    return "tokenomicInfo." + error;
            }
            if (message.stat != null && message.hasOwnProperty("stat")) {
                var error = $root.pruntime_rpc.WorkerStat.verify(message.stat);
                if (error)
                    return "stat." + error;
            }
            return null;
        };

        /**
         * Creates a WorkerState message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof pruntime_rpc.WorkerState
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {pruntime_rpc.WorkerState} WorkerState
         */
        WorkerState.fromObject = function fromObject(object) {
            if (object instanceof $root.pruntime_rpc.WorkerState)
                return object;
            var message = new $root.pruntime_rpc.WorkerState();
            if (object.registered != null)
                message.registered = Boolean(object.registered);
            if (object.unresponsive != null)
                message.unresponsive = Boolean(object.unresponsive);
            if (object.benchState != null) {
                if (typeof object.benchState !== "object")
                    throw TypeError(".pruntime_rpc.WorkerState.benchState: object expected");
                message.benchState = $root.pruntime_rpc.BenchState.fromObject(object.benchState);
            }
            if (object.miningState != null) {
                if (typeof object.miningState !== "object")
                    throw TypeError(".pruntime_rpc.WorkerState.miningState: object expected");
                message.miningState = $root.pruntime_rpc.MiningState.fromObject(object.miningState);
            }
            if (object.waitingHeartbeats) {
                if (!Array.isArray(object.waitingHeartbeats))
                    throw TypeError(".pruntime_rpc.WorkerState.waitingHeartbeats: array expected");
                message.waitingHeartbeats = [];
                for (var i = 0; i < object.waitingHeartbeats.length; ++i)
                    message.waitingHeartbeats[i] = object.waitingHeartbeats[i] >>> 0;
            }
            if (object.tokenomicInfo != null) {
                if (typeof object.tokenomicInfo !== "object")
                    throw TypeError(".pruntime_rpc.WorkerState.tokenomicInfo: object expected");
                message.tokenomicInfo = $root.pruntime_rpc.TokenomicInfo.fromObject(object.tokenomicInfo);
            }
            if (object.stat != null) {
                if (typeof object.stat !== "object")
                    throw TypeError(".pruntime_rpc.WorkerState.stat: object expected");
                message.stat = $root.pruntime_rpc.WorkerStat.fromObject(object.stat);
            }
            return message;
        };

        /**
         * Creates a plain object from a WorkerState message. Also converts values to other types if specified.
         * @function toObject
         * @memberof pruntime_rpc.WorkerState
         * @static
         * @param {pruntime_rpc.WorkerState} message WorkerState
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        WorkerState.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            var object = {};
            if (options.arrays || options.defaults)
                object.waitingHeartbeats = [];
            if (options.defaults) {
                object.registered = false;
                object.unresponsive = false;
                object.benchState = null;
                object.miningState = null;
                object.tokenomicInfo = null;
                object.stat = null;
            }
            if (message.registered != null && message.hasOwnProperty("registered"))
                object.registered = message.registered;
            if (message.unresponsive != null && message.hasOwnProperty("unresponsive"))
                object.unresponsive = message.unresponsive;
            if (message.benchState != null && message.hasOwnProperty("benchState"))
                object.benchState = $root.pruntime_rpc.BenchState.toObject(message.benchState, options);
            if (message.miningState != null && message.hasOwnProperty("miningState"))
                object.miningState = $root.pruntime_rpc.MiningState.toObject(message.miningState, options);
            if (message.waitingHeartbeats && message.waitingHeartbeats.length) {
                object.waitingHeartbeats = [];
                for (var j = 0; j < message.waitingHeartbeats.length; ++j)
                    object.waitingHeartbeats[j] = message.waitingHeartbeats[j];
            }
            if (message.tokenomicInfo != null && message.hasOwnProperty("tokenomicInfo"))
                object.tokenomicInfo = $root.pruntime_rpc.TokenomicInfo.toObject(message.tokenomicInfo, options);
            if (message.stat != null && message.hasOwnProperty("stat"))
                object.stat = $root.pruntime_rpc.WorkerStat.toObject(message.stat, options);
            return object;
        };

        /**
         * Converts this WorkerState to JSON.
         * @function toJSON
         * @memberof pruntime_rpc.WorkerState
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        WorkerState.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        return WorkerState;
    })();

    pruntime_rpc.HandoverChallenge = (function() {

        /**
         * Properties of a HandoverChallenge.
         * @memberof pruntime_rpc
         * @interface IHandoverChallenge
         * @property {Uint8Array|null} [encodedChallenge] HandoverChallenge encodedChallenge
         */

        /**
         * Constructs a new HandoverChallenge.
         * @memberof pruntime_rpc
         * @classdesc Represents a HandoverChallenge.
         * @implements IHandoverChallenge
         * @constructor
         * @param {pruntime_rpc.IHandoverChallenge=} [properties] Properties to set
         */
        function HandoverChallenge(properties) {
            if (properties)
                for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * HandoverChallenge encodedChallenge.
         * @member {Uint8Array} encodedChallenge
         * @memberof pruntime_rpc.HandoverChallenge
         * @instance
         */
        HandoverChallenge.prototype.encodedChallenge = $util.newBuffer([]);

        /**
         * Creates a new HandoverChallenge instance using the specified properties.
         * @function create
         * @memberof pruntime_rpc.HandoverChallenge
         * @static
         * @param {pruntime_rpc.IHandoverChallenge=} [properties] Properties to set
         * @returns {pruntime_rpc.HandoverChallenge} HandoverChallenge instance
         */
        HandoverChallenge.create = function create(properties) {
            return new HandoverChallenge(properties);
        };

        /**
         * Encodes the specified HandoverChallenge message. Does not implicitly {@link pruntime_rpc.HandoverChallenge.verify|verify} messages.
         * @function encode
         * @memberof pruntime_rpc.HandoverChallenge
         * @static
         * @param {pruntime_rpc.IHandoverChallenge} message HandoverChallenge message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        HandoverChallenge.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.encodedChallenge != null && Object.hasOwnProperty.call(message, "encodedChallenge"))
                writer.uint32(/* id 1, wireType 2 =*/10).bytes(message.encodedChallenge);
            return writer;
        };

        /**
         * Encodes the specified HandoverChallenge message, length delimited. Does not implicitly {@link pruntime_rpc.HandoverChallenge.verify|verify} messages.
         * @function encodeDelimited
         * @memberof pruntime_rpc.HandoverChallenge
         * @static
         * @param {pruntime_rpc.IHandoverChallenge} message HandoverChallenge message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        HandoverChallenge.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes a HandoverChallenge message from the specified reader or buffer.
         * @function decode
         * @memberof pruntime_rpc.HandoverChallenge
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {pruntime_rpc.HandoverChallenge} HandoverChallenge
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        HandoverChallenge.decode = function decode(reader, length) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            var end = length === undefined ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.HandoverChallenge();
            while (reader.pos < end) {
                var tag = reader.uint32();
                switch (tag >>> 3) {
                case 1:
                    message.encodedChallenge = reader.bytes();
                    break;
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a HandoverChallenge message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof pruntime_rpc.HandoverChallenge
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {pruntime_rpc.HandoverChallenge} HandoverChallenge
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        HandoverChallenge.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies a HandoverChallenge message.
         * @function verify
         * @memberof pruntime_rpc.HandoverChallenge
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        HandoverChallenge.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            if (message.encodedChallenge != null && message.hasOwnProperty("encodedChallenge"))
                if (!(message.encodedChallenge && typeof message.encodedChallenge.length === "number" || $util.isString(message.encodedChallenge)))
                    return "encodedChallenge: buffer expected";
            return null;
        };

        /**
         * Creates a HandoverChallenge message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof pruntime_rpc.HandoverChallenge
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {pruntime_rpc.HandoverChallenge} HandoverChallenge
         */
        HandoverChallenge.fromObject = function fromObject(object) {
            if (object instanceof $root.pruntime_rpc.HandoverChallenge)
                return object;
            var message = new $root.pruntime_rpc.HandoverChallenge();
            if (object.encodedChallenge != null)
                if (typeof object.encodedChallenge === "string")
                    $util.base64.decode(object.encodedChallenge, message.encodedChallenge = $util.newBuffer($util.base64.length(object.encodedChallenge)), 0);
                else if (object.encodedChallenge.length)
                    message.encodedChallenge = object.encodedChallenge;
            return message;
        };

        /**
         * Creates a plain object from a HandoverChallenge message. Also converts values to other types if specified.
         * @function toObject
         * @memberof pruntime_rpc.HandoverChallenge
         * @static
         * @param {pruntime_rpc.HandoverChallenge} message HandoverChallenge
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        HandoverChallenge.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            var object = {};
            if (options.defaults)
                if (options.bytes === String)
                    object.encodedChallenge = "";
                else {
                    object.encodedChallenge = [];
                    if (options.bytes !== Array)
                        object.encodedChallenge = $util.newBuffer(object.encodedChallenge);
                }
            if (message.encodedChallenge != null && message.hasOwnProperty("encodedChallenge"))
                object.encodedChallenge = options.bytes === String ? $util.base64.encode(message.encodedChallenge, 0, message.encodedChallenge.length) : options.bytes === Array ? Array.prototype.slice.call(message.encodedChallenge) : message.encodedChallenge;
            return object;
        };

        /**
         * Converts this HandoverChallenge to JSON.
         * @function toJSON
         * @memberof pruntime_rpc.HandoverChallenge
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        HandoverChallenge.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        return HandoverChallenge;
    })();

    pruntime_rpc.HandoverChallengeResponse = (function() {

        /**
         * Properties of a HandoverChallengeResponse.
         * @memberof pruntime_rpc
         * @interface IHandoverChallengeResponse
         * @property {Uint8Array|null} [encodedChallengeHandler] HandoverChallengeResponse encodedChallengeHandler
         * @property {pruntime_rpc.IAttestation|null} [attestation] HandoverChallengeResponse attestation
         */

        /**
         * Constructs a new HandoverChallengeResponse.
         * @memberof pruntime_rpc
         * @classdesc Represents a HandoverChallengeResponse.
         * @implements IHandoverChallengeResponse
         * @constructor
         * @param {pruntime_rpc.IHandoverChallengeResponse=} [properties] Properties to set
         */
        function HandoverChallengeResponse(properties) {
            if (properties)
                for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * HandoverChallengeResponse encodedChallengeHandler.
         * @member {Uint8Array} encodedChallengeHandler
         * @memberof pruntime_rpc.HandoverChallengeResponse
         * @instance
         */
        HandoverChallengeResponse.prototype.encodedChallengeHandler = $util.newBuffer([]);

        /**
         * HandoverChallengeResponse attestation.
         * @member {pruntime_rpc.IAttestation|null|undefined} attestation
         * @memberof pruntime_rpc.HandoverChallengeResponse
         * @instance
         */
        HandoverChallengeResponse.prototype.attestation = null;

        /**
         * Creates a new HandoverChallengeResponse instance using the specified properties.
         * @function create
         * @memberof pruntime_rpc.HandoverChallengeResponse
         * @static
         * @param {pruntime_rpc.IHandoverChallengeResponse=} [properties] Properties to set
         * @returns {pruntime_rpc.HandoverChallengeResponse} HandoverChallengeResponse instance
         */
        HandoverChallengeResponse.create = function create(properties) {
            return new HandoverChallengeResponse(properties);
        };

        /**
         * Encodes the specified HandoverChallengeResponse message. Does not implicitly {@link pruntime_rpc.HandoverChallengeResponse.verify|verify} messages.
         * @function encode
         * @memberof pruntime_rpc.HandoverChallengeResponse
         * @static
         * @param {pruntime_rpc.IHandoverChallengeResponse} message HandoverChallengeResponse message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        HandoverChallengeResponse.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.encodedChallengeHandler != null && Object.hasOwnProperty.call(message, "encodedChallengeHandler"))
                writer.uint32(/* id 1, wireType 2 =*/10).bytes(message.encodedChallengeHandler);
            if (message.attestation != null && Object.hasOwnProperty.call(message, "attestation"))
                $root.pruntime_rpc.Attestation.encode(message.attestation, writer.uint32(/* id 2, wireType 2 =*/18).fork()).ldelim();
            return writer;
        };

        /**
         * Encodes the specified HandoverChallengeResponse message, length delimited. Does not implicitly {@link pruntime_rpc.HandoverChallengeResponse.verify|verify} messages.
         * @function encodeDelimited
         * @memberof pruntime_rpc.HandoverChallengeResponse
         * @static
         * @param {pruntime_rpc.IHandoverChallengeResponse} message HandoverChallengeResponse message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        HandoverChallengeResponse.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes a HandoverChallengeResponse message from the specified reader or buffer.
         * @function decode
         * @memberof pruntime_rpc.HandoverChallengeResponse
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {pruntime_rpc.HandoverChallengeResponse} HandoverChallengeResponse
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        HandoverChallengeResponse.decode = function decode(reader, length) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            var end = length === undefined ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.HandoverChallengeResponse();
            while (reader.pos < end) {
                var tag = reader.uint32();
                switch (tag >>> 3) {
                case 1:
                    message.encodedChallengeHandler = reader.bytes();
                    break;
                case 2:
                    message.attestation = $root.pruntime_rpc.Attestation.decode(reader, reader.uint32());
                    break;
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a HandoverChallengeResponse message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof pruntime_rpc.HandoverChallengeResponse
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {pruntime_rpc.HandoverChallengeResponse} HandoverChallengeResponse
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        HandoverChallengeResponse.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies a HandoverChallengeResponse message.
         * @function verify
         * @memberof pruntime_rpc.HandoverChallengeResponse
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        HandoverChallengeResponse.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            if (message.encodedChallengeHandler != null && message.hasOwnProperty("encodedChallengeHandler"))
                if (!(message.encodedChallengeHandler && typeof message.encodedChallengeHandler.length === "number" || $util.isString(message.encodedChallengeHandler)))
                    return "encodedChallengeHandler: buffer expected";
            if (message.attestation != null && message.hasOwnProperty("attestation")) {
                var error = $root.pruntime_rpc.Attestation.verify(message.attestation);
                if (error)
                    return "attestation." + error;
            }
            return null;
        };

        /**
         * Creates a HandoverChallengeResponse message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof pruntime_rpc.HandoverChallengeResponse
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {pruntime_rpc.HandoverChallengeResponse} HandoverChallengeResponse
         */
        HandoverChallengeResponse.fromObject = function fromObject(object) {
            if (object instanceof $root.pruntime_rpc.HandoverChallengeResponse)
                return object;
            var message = new $root.pruntime_rpc.HandoverChallengeResponse();
            if (object.encodedChallengeHandler != null)
                if (typeof object.encodedChallengeHandler === "string")
                    $util.base64.decode(object.encodedChallengeHandler, message.encodedChallengeHandler = $util.newBuffer($util.base64.length(object.encodedChallengeHandler)), 0);
                else if (object.encodedChallengeHandler.length)
                    message.encodedChallengeHandler = object.encodedChallengeHandler;
            if (object.attestation != null) {
                if (typeof object.attestation !== "object")
                    throw TypeError(".pruntime_rpc.HandoverChallengeResponse.attestation: object expected");
                message.attestation = $root.pruntime_rpc.Attestation.fromObject(object.attestation);
            }
            return message;
        };

        /**
         * Creates a plain object from a HandoverChallengeResponse message. Also converts values to other types if specified.
         * @function toObject
         * @memberof pruntime_rpc.HandoverChallengeResponse
         * @static
         * @param {pruntime_rpc.HandoverChallengeResponse} message HandoverChallengeResponse
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        HandoverChallengeResponse.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            var object = {};
            if (options.defaults) {
                if (options.bytes === String)
                    object.encodedChallengeHandler = "";
                else {
                    object.encodedChallengeHandler = [];
                    if (options.bytes !== Array)
                        object.encodedChallengeHandler = $util.newBuffer(object.encodedChallengeHandler);
                }
                object.attestation = null;
            }
            if (message.encodedChallengeHandler != null && message.hasOwnProperty("encodedChallengeHandler"))
                object.encodedChallengeHandler = options.bytes === String ? $util.base64.encode(message.encodedChallengeHandler, 0, message.encodedChallengeHandler.length) : options.bytes === Array ? Array.prototype.slice.call(message.encodedChallengeHandler) : message.encodedChallengeHandler;
            if (message.attestation != null && message.hasOwnProperty("attestation"))
                object.attestation = $root.pruntime_rpc.Attestation.toObject(message.attestation, options);
            return object;
        };

        /**
         * Converts this HandoverChallengeResponse to JSON.
         * @function toJSON
         * @memberof pruntime_rpc.HandoverChallengeResponse
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        HandoverChallengeResponse.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        return HandoverChallengeResponse;
    })();

    pruntime_rpc.HandoverWorkerKey = (function() {

        /**
         * Properties of a HandoverWorkerKey.
         * @memberof pruntime_rpc
         * @interface IHandoverWorkerKey
         * @property {Uint8Array|null} [encodedWorkerKey] HandoverWorkerKey encodedWorkerKey
         * @property {pruntime_rpc.IAttestation|null} [attestation] HandoverWorkerKey attestation
         */

        /**
         * Constructs a new HandoverWorkerKey.
         * @memberof pruntime_rpc
         * @classdesc Represents a HandoverWorkerKey.
         * @implements IHandoverWorkerKey
         * @constructor
         * @param {pruntime_rpc.IHandoverWorkerKey=} [properties] Properties to set
         */
        function HandoverWorkerKey(properties) {
            if (properties)
                for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * HandoverWorkerKey encodedWorkerKey.
         * @member {Uint8Array} encodedWorkerKey
         * @memberof pruntime_rpc.HandoverWorkerKey
         * @instance
         */
        HandoverWorkerKey.prototype.encodedWorkerKey = $util.newBuffer([]);

        /**
         * HandoverWorkerKey attestation.
         * @member {pruntime_rpc.IAttestation|null|undefined} attestation
         * @memberof pruntime_rpc.HandoverWorkerKey
         * @instance
         */
        HandoverWorkerKey.prototype.attestation = null;

        /**
         * Creates a new HandoverWorkerKey instance using the specified properties.
         * @function create
         * @memberof pruntime_rpc.HandoverWorkerKey
         * @static
         * @param {pruntime_rpc.IHandoverWorkerKey=} [properties] Properties to set
         * @returns {pruntime_rpc.HandoverWorkerKey} HandoverWorkerKey instance
         */
        HandoverWorkerKey.create = function create(properties) {
            return new HandoverWorkerKey(properties);
        };

        /**
         * Encodes the specified HandoverWorkerKey message. Does not implicitly {@link pruntime_rpc.HandoverWorkerKey.verify|verify} messages.
         * @function encode
         * @memberof pruntime_rpc.HandoverWorkerKey
         * @static
         * @param {pruntime_rpc.IHandoverWorkerKey} message HandoverWorkerKey message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        HandoverWorkerKey.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.encodedWorkerKey != null && Object.hasOwnProperty.call(message, "encodedWorkerKey"))
                writer.uint32(/* id 1, wireType 2 =*/10).bytes(message.encodedWorkerKey);
            if (message.attestation != null && Object.hasOwnProperty.call(message, "attestation"))
                $root.pruntime_rpc.Attestation.encode(message.attestation, writer.uint32(/* id 2, wireType 2 =*/18).fork()).ldelim();
            return writer;
        };

        /**
         * Encodes the specified HandoverWorkerKey message, length delimited. Does not implicitly {@link pruntime_rpc.HandoverWorkerKey.verify|verify} messages.
         * @function encodeDelimited
         * @memberof pruntime_rpc.HandoverWorkerKey
         * @static
         * @param {pruntime_rpc.IHandoverWorkerKey} message HandoverWorkerKey message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        HandoverWorkerKey.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes a HandoverWorkerKey message from the specified reader or buffer.
         * @function decode
         * @memberof pruntime_rpc.HandoverWorkerKey
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {pruntime_rpc.HandoverWorkerKey} HandoverWorkerKey
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        HandoverWorkerKey.decode = function decode(reader, length) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            var end = length === undefined ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.HandoverWorkerKey();
            while (reader.pos < end) {
                var tag = reader.uint32();
                switch (tag >>> 3) {
                case 1:
                    message.encodedWorkerKey = reader.bytes();
                    break;
                case 2:
                    message.attestation = $root.pruntime_rpc.Attestation.decode(reader, reader.uint32());
                    break;
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a HandoverWorkerKey message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof pruntime_rpc.HandoverWorkerKey
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {pruntime_rpc.HandoverWorkerKey} HandoverWorkerKey
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        HandoverWorkerKey.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies a HandoverWorkerKey message.
         * @function verify
         * @memberof pruntime_rpc.HandoverWorkerKey
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        HandoverWorkerKey.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            if (message.encodedWorkerKey != null && message.hasOwnProperty("encodedWorkerKey"))
                if (!(message.encodedWorkerKey && typeof message.encodedWorkerKey.length === "number" || $util.isString(message.encodedWorkerKey)))
                    return "encodedWorkerKey: buffer expected";
            if (message.attestation != null && message.hasOwnProperty("attestation")) {
                var error = $root.pruntime_rpc.Attestation.verify(message.attestation);
                if (error)
                    return "attestation." + error;
            }
            return null;
        };

        /**
         * Creates a HandoverWorkerKey message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof pruntime_rpc.HandoverWorkerKey
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {pruntime_rpc.HandoverWorkerKey} HandoverWorkerKey
         */
        HandoverWorkerKey.fromObject = function fromObject(object) {
            if (object instanceof $root.pruntime_rpc.HandoverWorkerKey)
                return object;
            var message = new $root.pruntime_rpc.HandoverWorkerKey();
            if (object.encodedWorkerKey != null)
                if (typeof object.encodedWorkerKey === "string")
                    $util.base64.decode(object.encodedWorkerKey, message.encodedWorkerKey = $util.newBuffer($util.base64.length(object.encodedWorkerKey)), 0);
                else if (object.encodedWorkerKey.length)
                    message.encodedWorkerKey = object.encodedWorkerKey;
            if (object.attestation != null) {
                if (typeof object.attestation !== "object")
                    throw TypeError(".pruntime_rpc.HandoverWorkerKey.attestation: object expected");
                message.attestation = $root.pruntime_rpc.Attestation.fromObject(object.attestation);
            }
            return message;
        };

        /**
         * Creates a plain object from a HandoverWorkerKey message. Also converts values to other types if specified.
         * @function toObject
         * @memberof pruntime_rpc.HandoverWorkerKey
         * @static
         * @param {pruntime_rpc.HandoverWorkerKey} message HandoverWorkerKey
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        HandoverWorkerKey.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            var object = {};
            if (options.defaults) {
                if (options.bytes === String)
                    object.encodedWorkerKey = "";
                else {
                    object.encodedWorkerKey = [];
                    if (options.bytes !== Array)
                        object.encodedWorkerKey = $util.newBuffer(object.encodedWorkerKey);
                }
                object.attestation = null;
            }
            if (message.encodedWorkerKey != null && message.hasOwnProperty("encodedWorkerKey"))
                object.encodedWorkerKey = options.bytes === String ? $util.base64.encode(message.encodedWorkerKey, 0, message.encodedWorkerKey.length) : options.bytes === Array ? Array.prototype.slice.call(message.encodedWorkerKey) : message.encodedWorkerKey;
            if (message.attestation != null && message.hasOwnProperty("attestation"))
                object.attestation = $root.pruntime_rpc.Attestation.toObject(message.attestation, options);
            return object;
        };

        /**
         * Converts this HandoverWorkerKey to JSON.
         * @function toJSON
         * @memberof pruntime_rpc.HandoverWorkerKey
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        HandoverWorkerKey.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        return HandoverWorkerKey;
    })();

    pruntime_rpc.BenchState = (function() {

        /**
         * Properties of a BenchState.
         * @memberof pruntime_rpc
         * @interface IBenchState
         * @property {number|null} [startBlock] BenchState startBlock
         * @property {number|Long|null} [startTime] BenchState startTime
         * @property {number|null} [duration] BenchState duration
         */

        /**
         * Constructs a new BenchState.
         * @memberof pruntime_rpc
         * @classdesc Represents a BenchState.
         * @implements IBenchState
         * @constructor
         * @param {pruntime_rpc.IBenchState=} [properties] Properties to set
         */
        function BenchState(properties) {
            if (properties)
                for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * BenchState startBlock.
         * @member {number} startBlock
         * @memberof pruntime_rpc.BenchState
         * @instance
         */
        BenchState.prototype.startBlock = 0;

        /**
         * BenchState startTime.
         * @member {number|Long} startTime
         * @memberof pruntime_rpc.BenchState
         * @instance
         */
        BenchState.prototype.startTime = $util.Long ? $util.Long.fromBits(0,0,true) : 0;

        /**
         * BenchState duration.
         * @member {number} duration
         * @memberof pruntime_rpc.BenchState
         * @instance
         */
        BenchState.prototype.duration = 0;

        /**
         * Creates a new BenchState instance using the specified properties.
         * @function create
         * @memberof pruntime_rpc.BenchState
         * @static
         * @param {pruntime_rpc.IBenchState=} [properties] Properties to set
         * @returns {pruntime_rpc.BenchState} BenchState instance
         */
        BenchState.create = function create(properties) {
            return new BenchState(properties);
        };

        /**
         * Encodes the specified BenchState message. Does not implicitly {@link pruntime_rpc.BenchState.verify|verify} messages.
         * @function encode
         * @memberof pruntime_rpc.BenchState
         * @static
         * @param {pruntime_rpc.IBenchState} message BenchState message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        BenchState.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.startBlock != null && Object.hasOwnProperty.call(message, "startBlock"))
                writer.uint32(/* id 1, wireType 0 =*/8).uint32(message.startBlock);
            if (message.startTime != null && Object.hasOwnProperty.call(message, "startTime"))
                writer.uint32(/* id 2, wireType 0 =*/16).uint64(message.startTime);
            if (message.duration != null && Object.hasOwnProperty.call(message, "duration"))
                writer.uint32(/* id 4, wireType 0 =*/32).uint32(message.duration);
            return writer;
        };

        /**
         * Encodes the specified BenchState message, length delimited. Does not implicitly {@link pruntime_rpc.BenchState.verify|verify} messages.
         * @function encodeDelimited
         * @memberof pruntime_rpc.BenchState
         * @static
         * @param {pruntime_rpc.IBenchState} message BenchState message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        BenchState.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes a BenchState message from the specified reader or buffer.
         * @function decode
         * @memberof pruntime_rpc.BenchState
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {pruntime_rpc.BenchState} BenchState
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        BenchState.decode = function decode(reader, length) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            var end = length === undefined ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.BenchState();
            while (reader.pos < end) {
                var tag = reader.uint32();
                switch (tag >>> 3) {
                case 1:
                    message.startBlock = reader.uint32();
                    break;
                case 2:
                    message.startTime = reader.uint64();
                    break;
                case 4:
                    message.duration = reader.uint32();
                    break;
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a BenchState message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof pruntime_rpc.BenchState
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {pruntime_rpc.BenchState} BenchState
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        BenchState.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies a BenchState message.
         * @function verify
         * @memberof pruntime_rpc.BenchState
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        BenchState.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            if (message.startBlock != null && message.hasOwnProperty("startBlock"))
                if (!$util.isInteger(message.startBlock))
                    return "startBlock: integer expected";
            if (message.startTime != null && message.hasOwnProperty("startTime"))
                if (!$util.isInteger(message.startTime) && !(message.startTime && $util.isInteger(message.startTime.low) && $util.isInteger(message.startTime.high)))
                    return "startTime: integer|Long expected";
            if (message.duration != null && message.hasOwnProperty("duration"))
                if (!$util.isInteger(message.duration))
                    return "duration: integer expected";
            return null;
        };

        /**
         * Creates a BenchState message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof pruntime_rpc.BenchState
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {pruntime_rpc.BenchState} BenchState
         */
        BenchState.fromObject = function fromObject(object) {
            if (object instanceof $root.pruntime_rpc.BenchState)
                return object;
            var message = new $root.pruntime_rpc.BenchState();
            if (object.startBlock != null)
                message.startBlock = object.startBlock >>> 0;
            if (object.startTime != null)
                if ($util.Long)
                    (message.startTime = $util.Long.fromValue(object.startTime)).unsigned = true;
                else if (typeof object.startTime === "string")
                    message.startTime = parseInt(object.startTime, 10);
                else if (typeof object.startTime === "number")
                    message.startTime = object.startTime;
                else if (typeof object.startTime === "object")
                    message.startTime = new $util.LongBits(object.startTime.low >>> 0, object.startTime.high >>> 0).toNumber(true);
            if (object.duration != null)
                message.duration = object.duration >>> 0;
            return message;
        };

        /**
         * Creates a plain object from a BenchState message. Also converts values to other types if specified.
         * @function toObject
         * @memberof pruntime_rpc.BenchState
         * @static
         * @param {pruntime_rpc.BenchState} message BenchState
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        BenchState.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            var object = {};
            if (options.defaults) {
                object.startBlock = 0;
                if ($util.Long) {
                    var long = new $util.Long(0, 0, true);
                    object.startTime = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : long;
                } else
                    object.startTime = options.longs === String ? "0" : 0;
                object.duration = 0;
            }
            if (message.startBlock != null && message.hasOwnProperty("startBlock"))
                object.startBlock = message.startBlock;
            if (message.startTime != null && message.hasOwnProperty("startTime"))
                if (typeof message.startTime === "number")
                    object.startTime = options.longs === String ? String(message.startTime) : message.startTime;
                else
                    object.startTime = options.longs === String ? $util.Long.prototype.toString.call(message.startTime) : options.longs === Number ? new $util.LongBits(message.startTime.low >>> 0, message.startTime.high >>> 0).toNumber(true) : message.startTime;
            if (message.duration != null && message.hasOwnProperty("duration"))
                object.duration = message.duration;
            return object;
        };

        /**
         * Converts this BenchState to JSON.
         * @function toJSON
         * @memberof pruntime_rpc.BenchState
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        BenchState.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        return BenchState;
    })();

    pruntime_rpc.MiningState = (function() {

        /**
         * Properties of a MiningState.
         * @memberof pruntime_rpc
         * @interface IMiningState
         * @property {number|null} [sessionId] MiningState sessionId
         * @property {boolean|null} [paused] MiningState paused
         * @property {number|Long|null} [startTime] MiningState startTime
         */

        /**
         * Constructs a new MiningState.
         * @memberof pruntime_rpc
         * @classdesc Represents a MiningState.
         * @implements IMiningState
         * @constructor
         * @param {pruntime_rpc.IMiningState=} [properties] Properties to set
         */
        function MiningState(properties) {
            if (properties)
                for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * MiningState sessionId.
         * @member {number} sessionId
         * @memberof pruntime_rpc.MiningState
         * @instance
         */
        MiningState.prototype.sessionId = 0;

        /**
         * MiningState paused.
         * @member {boolean} paused
         * @memberof pruntime_rpc.MiningState
         * @instance
         */
        MiningState.prototype.paused = false;

        /**
         * MiningState startTime.
         * @member {number|Long} startTime
         * @memberof pruntime_rpc.MiningState
         * @instance
         */
        MiningState.prototype.startTime = $util.Long ? $util.Long.fromBits(0,0,true) : 0;

        /**
         * Creates a new MiningState instance using the specified properties.
         * @function create
         * @memberof pruntime_rpc.MiningState
         * @static
         * @param {pruntime_rpc.IMiningState=} [properties] Properties to set
         * @returns {pruntime_rpc.MiningState} MiningState instance
         */
        MiningState.create = function create(properties) {
            return new MiningState(properties);
        };

        /**
         * Encodes the specified MiningState message. Does not implicitly {@link pruntime_rpc.MiningState.verify|verify} messages.
         * @function encode
         * @memberof pruntime_rpc.MiningState
         * @static
         * @param {pruntime_rpc.IMiningState} message MiningState message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        MiningState.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.sessionId != null && Object.hasOwnProperty.call(message, "sessionId"))
                writer.uint32(/* id 1, wireType 0 =*/8).uint32(message.sessionId);
            if (message.paused != null && Object.hasOwnProperty.call(message, "paused"))
                writer.uint32(/* id 2, wireType 0 =*/16).bool(message.paused);
            if (message.startTime != null && Object.hasOwnProperty.call(message, "startTime"))
                writer.uint32(/* id 3, wireType 0 =*/24).uint64(message.startTime);
            return writer;
        };

        /**
         * Encodes the specified MiningState message, length delimited. Does not implicitly {@link pruntime_rpc.MiningState.verify|verify} messages.
         * @function encodeDelimited
         * @memberof pruntime_rpc.MiningState
         * @static
         * @param {pruntime_rpc.IMiningState} message MiningState message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        MiningState.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes a MiningState message from the specified reader or buffer.
         * @function decode
         * @memberof pruntime_rpc.MiningState
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {pruntime_rpc.MiningState} MiningState
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        MiningState.decode = function decode(reader, length) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            var end = length === undefined ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.MiningState();
            while (reader.pos < end) {
                var tag = reader.uint32();
                switch (tag >>> 3) {
                case 1:
                    message.sessionId = reader.uint32();
                    break;
                case 2:
                    message.paused = reader.bool();
                    break;
                case 3:
                    message.startTime = reader.uint64();
                    break;
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a MiningState message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof pruntime_rpc.MiningState
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {pruntime_rpc.MiningState} MiningState
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        MiningState.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies a MiningState message.
         * @function verify
         * @memberof pruntime_rpc.MiningState
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        MiningState.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            if (message.sessionId != null && message.hasOwnProperty("sessionId"))
                if (!$util.isInteger(message.sessionId))
                    return "sessionId: integer expected";
            if (message.paused != null && message.hasOwnProperty("paused"))
                if (typeof message.paused !== "boolean")
                    return "paused: boolean expected";
            if (message.startTime != null && message.hasOwnProperty("startTime"))
                if (!$util.isInteger(message.startTime) && !(message.startTime && $util.isInteger(message.startTime.low) && $util.isInteger(message.startTime.high)))
                    return "startTime: integer|Long expected";
            return null;
        };

        /**
         * Creates a MiningState message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof pruntime_rpc.MiningState
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {pruntime_rpc.MiningState} MiningState
         */
        MiningState.fromObject = function fromObject(object) {
            if (object instanceof $root.pruntime_rpc.MiningState)
                return object;
            var message = new $root.pruntime_rpc.MiningState();
            if (object.sessionId != null)
                message.sessionId = object.sessionId >>> 0;
            if (object.paused != null)
                message.paused = Boolean(object.paused);
            if (object.startTime != null)
                if ($util.Long)
                    (message.startTime = $util.Long.fromValue(object.startTime)).unsigned = true;
                else if (typeof object.startTime === "string")
                    message.startTime = parseInt(object.startTime, 10);
                else if (typeof object.startTime === "number")
                    message.startTime = object.startTime;
                else if (typeof object.startTime === "object")
                    message.startTime = new $util.LongBits(object.startTime.low >>> 0, object.startTime.high >>> 0).toNumber(true);
            return message;
        };

        /**
         * Creates a plain object from a MiningState message. Also converts values to other types if specified.
         * @function toObject
         * @memberof pruntime_rpc.MiningState
         * @static
         * @param {pruntime_rpc.MiningState} message MiningState
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        MiningState.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            var object = {};
            if (options.defaults) {
                object.sessionId = 0;
                object.paused = false;
                if ($util.Long) {
                    var long = new $util.Long(0, 0, true);
                    object.startTime = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : long;
                } else
                    object.startTime = options.longs === String ? "0" : 0;
            }
            if (message.sessionId != null && message.hasOwnProperty("sessionId"))
                object.sessionId = message.sessionId;
            if (message.paused != null && message.hasOwnProperty("paused"))
                object.paused = message.paused;
            if (message.startTime != null && message.hasOwnProperty("startTime"))
                if (typeof message.startTime === "number")
                    object.startTime = options.longs === String ? String(message.startTime) : message.startTime;
                else
                    object.startTime = options.longs === String ? $util.Long.prototype.toString.call(message.startTime) : options.longs === Number ? new $util.LongBits(message.startTime.low >>> 0, message.startTime.high >>> 0).toNumber(true) : message.startTime;
            return object;
        };

        /**
         * Converts this MiningState to JSON.
         * @function toJSON
         * @memberof pruntime_rpc.MiningState
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        MiningState.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        return MiningState;
    })();

    pruntime_rpc.EchoMessage = (function() {

        /**
         * Properties of an EchoMessage.
         * @memberof pruntime_rpc
         * @interface IEchoMessage
         * @property {Uint8Array|null} [echoMsg] EchoMessage echoMsg
         */

        /**
         * Constructs a new EchoMessage.
         * @memberof pruntime_rpc
         * @classdesc Represents an EchoMessage.
         * @implements IEchoMessage
         * @constructor
         * @param {pruntime_rpc.IEchoMessage=} [properties] Properties to set
         */
        function EchoMessage(properties) {
            if (properties)
                for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * EchoMessage echoMsg.
         * @member {Uint8Array} echoMsg
         * @memberof pruntime_rpc.EchoMessage
         * @instance
         */
        EchoMessage.prototype.echoMsg = $util.newBuffer([]);

        /**
         * Creates a new EchoMessage instance using the specified properties.
         * @function create
         * @memberof pruntime_rpc.EchoMessage
         * @static
         * @param {pruntime_rpc.IEchoMessage=} [properties] Properties to set
         * @returns {pruntime_rpc.EchoMessage} EchoMessage instance
         */
        EchoMessage.create = function create(properties) {
            return new EchoMessage(properties);
        };

        /**
         * Encodes the specified EchoMessage message. Does not implicitly {@link pruntime_rpc.EchoMessage.verify|verify} messages.
         * @function encode
         * @memberof pruntime_rpc.EchoMessage
         * @static
         * @param {pruntime_rpc.IEchoMessage} message EchoMessage message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        EchoMessage.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.echoMsg != null && Object.hasOwnProperty.call(message, "echoMsg"))
                writer.uint32(/* id 1, wireType 2 =*/10).bytes(message.echoMsg);
            return writer;
        };

        /**
         * Encodes the specified EchoMessage message, length delimited. Does not implicitly {@link pruntime_rpc.EchoMessage.verify|verify} messages.
         * @function encodeDelimited
         * @memberof pruntime_rpc.EchoMessage
         * @static
         * @param {pruntime_rpc.IEchoMessage} message EchoMessage message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        EchoMessage.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes an EchoMessage message from the specified reader or buffer.
         * @function decode
         * @memberof pruntime_rpc.EchoMessage
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {pruntime_rpc.EchoMessage} EchoMessage
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        EchoMessage.decode = function decode(reader, length) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            var end = length === undefined ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.EchoMessage();
            while (reader.pos < end) {
                var tag = reader.uint32();
                switch (tag >>> 3) {
                case 1:
                    message.echoMsg = reader.bytes();
                    break;
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes an EchoMessage message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof pruntime_rpc.EchoMessage
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {pruntime_rpc.EchoMessage} EchoMessage
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        EchoMessage.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies an EchoMessage message.
         * @function verify
         * @memberof pruntime_rpc.EchoMessage
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        EchoMessage.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            if (message.echoMsg != null && message.hasOwnProperty("echoMsg"))
                if (!(message.echoMsg && typeof message.echoMsg.length === "number" || $util.isString(message.echoMsg)))
                    return "echoMsg: buffer expected";
            return null;
        };

        /**
         * Creates an EchoMessage message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof pruntime_rpc.EchoMessage
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {pruntime_rpc.EchoMessage} EchoMessage
         */
        EchoMessage.fromObject = function fromObject(object) {
            if (object instanceof $root.pruntime_rpc.EchoMessage)
                return object;
            var message = new $root.pruntime_rpc.EchoMessage();
            if (object.echoMsg != null)
                if (typeof object.echoMsg === "string")
                    $util.base64.decode(object.echoMsg, message.echoMsg = $util.newBuffer($util.base64.length(object.echoMsg)), 0);
                else if (object.echoMsg.length)
                    message.echoMsg = object.echoMsg;
            return message;
        };

        /**
         * Creates a plain object from an EchoMessage message. Also converts values to other types if specified.
         * @function toObject
         * @memberof pruntime_rpc.EchoMessage
         * @static
         * @param {pruntime_rpc.EchoMessage} message EchoMessage
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        EchoMessage.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            var object = {};
            if (options.defaults)
                if (options.bytes === String)
                    object.echoMsg = "";
                else {
                    object.echoMsg = [];
                    if (options.bytes !== Array)
                        object.echoMsg = $util.newBuffer(object.echoMsg);
                }
            if (message.echoMsg != null && message.hasOwnProperty("echoMsg"))
                object.echoMsg = options.bytes === String ? $util.base64.encode(message.echoMsg, 0, message.echoMsg.length) : options.bytes === Array ? Array.prototype.slice.call(message.echoMsg) : message.echoMsg;
            return object;
        };

        /**
         * Converts this EchoMessage to JSON.
         * @function toJSON
         * @memberof pruntime_rpc.EchoMessage
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        EchoMessage.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        return EchoMessage;
    })();

    /**
     * ResponsiveEvent enum.
     * @name pruntime_rpc.ResponsiveEvent
     * @enum {number}
     * @property {number} NoEvent=0 NoEvent value
     * @property {number} EnterUnresponsive=1 EnterUnresponsive value
     * @property {number} ExitUnresponsive=2 ExitUnresponsive value
     */
    pruntime_rpc.ResponsiveEvent = (function() {
        var valuesById = {}, values = Object.create(valuesById);
        values[valuesById[0] = "NoEvent"] = 0;
        values[valuesById[1] = "EnterUnresponsive"] = 1;
        values[valuesById[2] = "ExitUnresponsive"] = 2;
        return values;
    })();

    pruntime_rpc.AddEndpointRequest = (function() {

        /**
         * Properties of an AddEndpointRequest.
         * @memberof pruntime_rpc
         * @interface IAddEndpointRequest
         * @property {Uint8Array|null} [encodedEndpointType] AddEndpointRequest encodedEndpointType
         * @property {string|null} [endpoint] AddEndpointRequest endpoint
         */

        /**
         * Constructs a new AddEndpointRequest.
         * @memberof pruntime_rpc
         * @classdesc Represents an AddEndpointRequest.
         * @implements IAddEndpointRequest
         * @constructor
         * @param {pruntime_rpc.IAddEndpointRequest=} [properties] Properties to set
         */
        function AddEndpointRequest(properties) {
            if (properties)
                for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * AddEndpointRequest encodedEndpointType.
         * @member {Uint8Array} encodedEndpointType
         * @memberof pruntime_rpc.AddEndpointRequest
         * @instance
         */
        AddEndpointRequest.prototype.encodedEndpointType = $util.newBuffer([]);

        /**
         * AddEndpointRequest endpoint.
         * @member {string} endpoint
         * @memberof pruntime_rpc.AddEndpointRequest
         * @instance
         */
        AddEndpointRequest.prototype.endpoint = "";

        /**
         * Creates a new AddEndpointRequest instance using the specified properties.
         * @function create
         * @memberof pruntime_rpc.AddEndpointRequest
         * @static
         * @param {pruntime_rpc.IAddEndpointRequest=} [properties] Properties to set
         * @returns {pruntime_rpc.AddEndpointRequest} AddEndpointRequest instance
         */
        AddEndpointRequest.create = function create(properties) {
            return new AddEndpointRequest(properties);
        };

        /**
         * Encodes the specified AddEndpointRequest message. Does not implicitly {@link pruntime_rpc.AddEndpointRequest.verify|verify} messages.
         * @function encode
         * @memberof pruntime_rpc.AddEndpointRequest
         * @static
         * @param {pruntime_rpc.IAddEndpointRequest} message AddEndpointRequest message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        AddEndpointRequest.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.encodedEndpointType != null && Object.hasOwnProperty.call(message, "encodedEndpointType"))
                writer.uint32(/* id 1, wireType 2 =*/10).bytes(message.encodedEndpointType);
            if (message.endpoint != null && Object.hasOwnProperty.call(message, "endpoint"))
                writer.uint32(/* id 2, wireType 2 =*/18).string(message.endpoint);
            return writer;
        };

        /**
         * Encodes the specified AddEndpointRequest message, length delimited. Does not implicitly {@link pruntime_rpc.AddEndpointRequest.verify|verify} messages.
         * @function encodeDelimited
         * @memberof pruntime_rpc.AddEndpointRequest
         * @static
         * @param {pruntime_rpc.IAddEndpointRequest} message AddEndpointRequest message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        AddEndpointRequest.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes an AddEndpointRequest message from the specified reader or buffer.
         * @function decode
         * @memberof pruntime_rpc.AddEndpointRequest
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {pruntime_rpc.AddEndpointRequest} AddEndpointRequest
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        AddEndpointRequest.decode = function decode(reader, length) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            var end = length === undefined ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.AddEndpointRequest();
            while (reader.pos < end) {
                var tag = reader.uint32();
                switch (tag >>> 3) {
                case 1:
                    message.encodedEndpointType = reader.bytes();
                    break;
                case 2:
                    message.endpoint = reader.string();
                    break;
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes an AddEndpointRequest message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof pruntime_rpc.AddEndpointRequest
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {pruntime_rpc.AddEndpointRequest} AddEndpointRequest
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        AddEndpointRequest.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies an AddEndpointRequest message.
         * @function verify
         * @memberof pruntime_rpc.AddEndpointRequest
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        AddEndpointRequest.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            if (message.encodedEndpointType != null && message.hasOwnProperty("encodedEndpointType"))
                if (!(message.encodedEndpointType && typeof message.encodedEndpointType.length === "number" || $util.isString(message.encodedEndpointType)))
                    return "encodedEndpointType: buffer expected";
            if (message.endpoint != null && message.hasOwnProperty("endpoint"))
                if (!$util.isString(message.endpoint))
                    return "endpoint: string expected";
            return null;
        };

        /**
         * Creates an AddEndpointRequest message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof pruntime_rpc.AddEndpointRequest
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {pruntime_rpc.AddEndpointRequest} AddEndpointRequest
         */
        AddEndpointRequest.fromObject = function fromObject(object) {
            if (object instanceof $root.pruntime_rpc.AddEndpointRequest)
                return object;
            var message = new $root.pruntime_rpc.AddEndpointRequest();
            if (object.encodedEndpointType != null)
                if (typeof object.encodedEndpointType === "string")
                    $util.base64.decode(object.encodedEndpointType, message.encodedEndpointType = $util.newBuffer($util.base64.length(object.encodedEndpointType)), 0);
                else if (object.encodedEndpointType.length)
                    message.encodedEndpointType = object.encodedEndpointType;
            if (object.endpoint != null)
                message.endpoint = String(object.endpoint);
            return message;
        };

        /**
         * Creates a plain object from an AddEndpointRequest message. Also converts values to other types if specified.
         * @function toObject
         * @memberof pruntime_rpc.AddEndpointRequest
         * @static
         * @param {pruntime_rpc.AddEndpointRequest} message AddEndpointRequest
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        AddEndpointRequest.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            var object = {};
            if (options.defaults) {
                if (options.bytes === String)
                    object.encodedEndpointType = "";
                else {
                    object.encodedEndpointType = [];
                    if (options.bytes !== Array)
                        object.encodedEndpointType = $util.newBuffer(object.encodedEndpointType);
                }
                object.endpoint = "";
            }
            if (message.encodedEndpointType != null && message.hasOwnProperty("encodedEndpointType"))
                object.encodedEndpointType = options.bytes === String ? $util.base64.encode(message.encodedEndpointType, 0, message.encodedEndpointType.length) : options.bytes === Array ? Array.prototype.slice.call(message.encodedEndpointType) : message.encodedEndpointType;
            if (message.endpoint != null && message.hasOwnProperty("endpoint"))
                object.endpoint = message.endpoint;
            return object;
        };

        /**
         * Converts this AddEndpointRequest to JSON.
         * @function toJSON
         * @memberof pruntime_rpc.AddEndpointRequest
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        AddEndpointRequest.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        return AddEndpointRequest;
    })();

    pruntime_rpc.GetEndpointResponse = (function() {

        /**
         * Properties of a GetEndpointResponse.
         * @memberof pruntime_rpc
         * @interface IGetEndpointResponse
         * @property {Uint8Array|null} [encodedEndpointPayload] GetEndpointResponse encodedEndpointPayload
         * @property {Uint8Array|null} [signature] GetEndpointResponse signature
         */

        /**
         * Constructs a new GetEndpointResponse.
         * @memberof pruntime_rpc
         * @classdesc Represents a GetEndpointResponse.
         * @implements IGetEndpointResponse
         * @constructor
         * @param {pruntime_rpc.IGetEndpointResponse=} [properties] Properties to set
         */
        function GetEndpointResponse(properties) {
            if (properties)
                for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * GetEndpointResponse encodedEndpointPayload.
         * @member {Uint8Array|null|undefined} encodedEndpointPayload
         * @memberof pruntime_rpc.GetEndpointResponse
         * @instance
         */
        GetEndpointResponse.prototype.encodedEndpointPayload = null;

        /**
         * GetEndpointResponse signature.
         * @member {Uint8Array|null|undefined} signature
         * @memberof pruntime_rpc.GetEndpointResponse
         * @instance
         */
        GetEndpointResponse.prototype.signature = null;

        // OneOf field names bound to virtual getters and setters
        var $oneOfFields;

        /**
         * GetEndpointResponse _encodedEndpointPayload.
         * @member {"encodedEndpointPayload"|undefined} _encodedEndpointPayload
         * @memberof pruntime_rpc.GetEndpointResponse
         * @instance
         */
        Object.defineProperty(GetEndpointResponse.prototype, "_encodedEndpointPayload", {
            get: $util.oneOfGetter($oneOfFields = ["encodedEndpointPayload"]),
            set: $util.oneOfSetter($oneOfFields)
        });

        /**
         * GetEndpointResponse _signature.
         * @member {"signature"|undefined} _signature
         * @memberof pruntime_rpc.GetEndpointResponse
         * @instance
         */
        Object.defineProperty(GetEndpointResponse.prototype, "_signature", {
            get: $util.oneOfGetter($oneOfFields = ["signature"]),
            set: $util.oneOfSetter($oneOfFields)
        });

        /**
         * Creates a new GetEndpointResponse instance using the specified properties.
         * @function create
         * @memberof pruntime_rpc.GetEndpointResponse
         * @static
         * @param {pruntime_rpc.IGetEndpointResponse=} [properties] Properties to set
         * @returns {pruntime_rpc.GetEndpointResponse} GetEndpointResponse instance
         */
        GetEndpointResponse.create = function create(properties) {
            return new GetEndpointResponse(properties);
        };

        /**
         * Encodes the specified GetEndpointResponse message. Does not implicitly {@link pruntime_rpc.GetEndpointResponse.verify|verify} messages.
         * @function encode
         * @memberof pruntime_rpc.GetEndpointResponse
         * @static
         * @param {pruntime_rpc.IGetEndpointResponse} message GetEndpointResponse message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        GetEndpointResponse.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.encodedEndpointPayload != null && Object.hasOwnProperty.call(message, "encodedEndpointPayload"))
                writer.uint32(/* id 1, wireType 2 =*/10).bytes(message.encodedEndpointPayload);
            if (message.signature != null && Object.hasOwnProperty.call(message, "signature"))
                writer.uint32(/* id 2, wireType 2 =*/18).bytes(message.signature);
            return writer;
        };

        /**
         * Encodes the specified GetEndpointResponse message, length delimited. Does not implicitly {@link pruntime_rpc.GetEndpointResponse.verify|verify} messages.
         * @function encodeDelimited
         * @memberof pruntime_rpc.GetEndpointResponse
         * @static
         * @param {pruntime_rpc.IGetEndpointResponse} message GetEndpointResponse message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        GetEndpointResponse.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes a GetEndpointResponse message from the specified reader or buffer.
         * @function decode
         * @memberof pruntime_rpc.GetEndpointResponse
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {pruntime_rpc.GetEndpointResponse} GetEndpointResponse
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        GetEndpointResponse.decode = function decode(reader, length) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            var end = length === undefined ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.GetEndpointResponse();
            while (reader.pos < end) {
                var tag = reader.uint32();
                switch (tag >>> 3) {
                case 1:
                    message.encodedEndpointPayload = reader.bytes();
                    break;
                case 2:
                    message.signature = reader.bytes();
                    break;
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a GetEndpointResponse message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof pruntime_rpc.GetEndpointResponse
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {pruntime_rpc.GetEndpointResponse} GetEndpointResponse
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        GetEndpointResponse.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies a GetEndpointResponse message.
         * @function verify
         * @memberof pruntime_rpc.GetEndpointResponse
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        GetEndpointResponse.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            var properties = {};
            if (message.encodedEndpointPayload != null && message.hasOwnProperty("encodedEndpointPayload")) {
                properties._encodedEndpointPayload = 1;
                if (!(message.encodedEndpointPayload && typeof message.encodedEndpointPayload.length === "number" || $util.isString(message.encodedEndpointPayload)))
                    return "encodedEndpointPayload: buffer expected";
            }
            if (message.signature != null && message.hasOwnProperty("signature")) {
                properties._signature = 1;
                if (!(message.signature && typeof message.signature.length === "number" || $util.isString(message.signature)))
                    return "signature: buffer expected";
            }
            return null;
        };

        /**
         * Creates a GetEndpointResponse message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof pruntime_rpc.GetEndpointResponse
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {pruntime_rpc.GetEndpointResponse} GetEndpointResponse
         */
        GetEndpointResponse.fromObject = function fromObject(object) {
            if (object instanceof $root.pruntime_rpc.GetEndpointResponse)
                return object;
            var message = new $root.pruntime_rpc.GetEndpointResponse();
            if (object.encodedEndpointPayload != null)
                if (typeof object.encodedEndpointPayload === "string")
                    $util.base64.decode(object.encodedEndpointPayload, message.encodedEndpointPayload = $util.newBuffer($util.base64.length(object.encodedEndpointPayload)), 0);
                else if (object.encodedEndpointPayload.length)
                    message.encodedEndpointPayload = object.encodedEndpointPayload;
            if (object.signature != null)
                if (typeof object.signature === "string")
                    $util.base64.decode(object.signature, message.signature = $util.newBuffer($util.base64.length(object.signature)), 0);
                else if (object.signature.length)
                    message.signature = object.signature;
            return message;
        };

        /**
         * Creates a plain object from a GetEndpointResponse message. Also converts values to other types if specified.
         * @function toObject
         * @memberof pruntime_rpc.GetEndpointResponse
         * @static
         * @param {pruntime_rpc.GetEndpointResponse} message GetEndpointResponse
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        GetEndpointResponse.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            var object = {};
            if (message.encodedEndpointPayload != null && message.hasOwnProperty("encodedEndpointPayload")) {
                object.encodedEndpointPayload = options.bytes === String ? $util.base64.encode(message.encodedEndpointPayload, 0, message.encodedEndpointPayload.length) : options.bytes === Array ? Array.prototype.slice.call(message.encodedEndpointPayload) : message.encodedEndpointPayload;
                if (options.oneofs)
                    object._encodedEndpointPayload = "encodedEndpointPayload";
            }
            if (message.signature != null && message.hasOwnProperty("signature")) {
                object.signature = options.bytes === String ? $util.base64.encode(message.signature, 0, message.signature.length) : options.bytes === Array ? Array.prototype.slice.call(message.signature) : message.signature;
                if (options.oneofs)
                    object._signature = "signature";
            }
            return object;
        };

        /**
         * Converts this GetEndpointResponse to JSON.
         * @function toJSON
         * @memberof pruntime_rpc.GetEndpointResponse
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        GetEndpointResponse.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        return GetEndpointResponse;
    })();

    pruntime_rpc.SignEndpointsRequest = (function() {

        /**
         * Properties of a SignEndpointsRequest.
         * @memberof pruntime_rpc
         * @interface ISignEndpointsRequest
         * @property {Uint8Array|null} [encodedEndpoints] SignEndpointsRequest encodedEndpoints
         */

        /**
         * Constructs a new SignEndpointsRequest.
         * @memberof pruntime_rpc
         * @classdesc Represents a SignEndpointsRequest.
         * @implements ISignEndpointsRequest
         * @constructor
         * @param {pruntime_rpc.ISignEndpointsRequest=} [properties] Properties to set
         */
        function SignEndpointsRequest(properties) {
            if (properties)
                for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * SignEndpointsRequest encodedEndpoints.
         * @member {Uint8Array} encodedEndpoints
         * @memberof pruntime_rpc.SignEndpointsRequest
         * @instance
         */
        SignEndpointsRequest.prototype.encodedEndpoints = $util.newBuffer([]);

        /**
         * Creates a new SignEndpointsRequest instance using the specified properties.
         * @function create
         * @memberof pruntime_rpc.SignEndpointsRequest
         * @static
         * @param {pruntime_rpc.ISignEndpointsRequest=} [properties] Properties to set
         * @returns {pruntime_rpc.SignEndpointsRequest} SignEndpointsRequest instance
         */
        SignEndpointsRequest.create = function create(properties) {
            return new SignEndpointsRequest(properties);
        };

        /**
         * Encodes the specified SignEndpointsRequest message. Does not implicitly {@link pruntime_rpc.SignEndpointsRequest.verify|verify} messages.
         * @function encode
         * @memberof pruntime_rpc.SignEndpointsRequest
         * @static
         * @param {pruntime_rpc.ISignEndpointsRequest} message SignEndpointsRequest message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        SignEndpointsRequest.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.encodedEndpoints != null && Object.hasOwnProperty.call(message, "encodedEndpoints"))
                writer.uint32(/* id 1, wireType 2 =*/10).bytes(message.encodedEndpoints);
            return writer;
        };

        /**
         * Encodes the specified SignEndpointsRequest message, length delimited. Does not implicitly {@link pruntime_rpc.SignEndpointsRequest.verify|verify} messages.
         * @function encodeDelimited
         * @memberof pruntime_rpc.SignEndpointsRequest
         * @static
         * @param {pruntime_rpc.ISignEndpointsRequest} message SignEndpointsRequest message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        SignEndpointsRequest.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes a SignEndpointsRequest message from the specified reader or buffer.
         * @function decode
         * @memberof pruntime_rpc.SignEndpointsRequest
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {pruntime_rpc.SignEndpointsRequest} SignEndpointsRequest
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        SignEndpointsRequest.decode = function decode(reader, length) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            var end = length === undefined ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.SignEndpointsRequest();
            while (reader.pos < end) {
                var tag = reader.uint32();
                switch (tag >>> 3) {
                case 1:
                    message.encodedEndpoints = reader.bytes();
                    break;
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a SignEndpointsRequest message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof pruntime_rpc.SignEndpointsRequest
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {pruntime_rpc.SignEndpointsRequest} SignEndpointsRequest
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        SignEndpointsRequest.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies a SignEndpointsRequest message.
         * @function verify
         * @memberof pruntime_rpc.SignEndpointsRequest
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        SignEndpointsRequest.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            if (message.encodedEndpoints != null && message.hasOwnProperty("encodedEndpoints"))
                if (!(message.encodedEndpoints && typeof message.encodedEndpoints.length === "number" || $util.isString(message.encodedEndpoints)))
                    return "encodedEndpoints: buffer expected";
            return null;
        };

        /**
         * Creates a SignEndpointsRequest message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof pruntime_rpc.SignEndpointsRequest
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {pruntime_rpc.SignEndpointsRequest} SignEndpointsRequest
         */
        SignEndpointsRequest.fromObject = function fromObject(object) {
            if (object instanceof $root.pruntime_rpc.SignEndpointsRequest)
                return object;
            var message = new $root.pruntime_rpc.SignEndpointsRequest();
            if (object.encodedEndpoints != null)
                if (typeof object.encodedEndpoints === "string")
                    $util.base64.decode(object.encodedEndpoints, message.encodedEndpoints = $util.newBuffer($util.base64.length(object.encodedEndpoints)), 0);
                else if (object.encodedEndpoints.length)
                    message.encodedEndpoints = object.encodedEndpoints;
            return message;
        };

        /**
         * Creates a plain object from a SignEndpointsRequest message. Also converts values to other types if specified.
         * @function toObject
         * @memberof pruntime_rpc.SignEndpointsRequest
         * @static
         * @param {pruntime_rpc.SignEndpointsRequest} message SignEndpointsRequest
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        SignEndpointsRequest.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            var object = {};
            if (options.defaults)
                if (options.bytes === String)
                    object.encodedEndpoints = "";
                else {
                    object.encodedEndpoints = [];
                    if (options.bytes !== Array)
                        object.encodedEndpoints = $util.newBuffer(object.encodedEndpoints);
                }
            if (message.encodedEndpoints != null && message.hasOwnProperty("encodedEndpoints"))
                object.encodedEndpoints = options.bytes === String ? $util.base64.encode(message.encodedEndpoints, 0, message.encodedEndpoints.length) : options.bytes === Array ? Array.prototype.slice.call(message.encodedEndpoints) : message.encodedEndpoints;
            return object;
        };

        /**
         * Converts this SignEndpointsRequest to JSON.
         * @function toJSON
         * @memberof pruntime_rpc.SignEndpointsRequest
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        SignEndpointsRequest.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        return SignEndpointsRequest;
    })();

    pruntime_rpc.DerivePhalaI2pKeyResponse = (function() {

        /**
         * Properties of a DerivePhalaI2pKeyResponse.
         * @memberof pruntime_rpc
         * @interface IDerivePhalaI2pKeyResponse
         * @property {Uint8Array|null} [phalaI2pKey] DerivePhalaI2pKeyResponse phalaI2pKey
         */

        /**
         * Constructs a new DerivePhalaI2pKeyResponse.
         * @memberof pruntime_rpc
         * @classdesc Represents a DerivePhalaI2pKeyResponse.
         * @implements IDerivePhalaI2pKeyResponse
         * @constructor
         * @param {pruntime_rpc.IDerivePhalaI2pKeyResponse=} [properties] Properties to set
         */
        function DerivePhalaI2pKeyResponse(properties) {
            if (properties)
                for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * DerivePhalaI2pKeyResponse phalaI2pKey.
         * @member {Uint8Array} phalaI2pKey
         * @memberof pruntime_rpc.DerivePhalaI2pKeyResponse
         * @instance
         */
        DerivePhalaI2pKeyResponse.prototype.phalaI2pKey = $util.newBuffer([]);

        /**
         * Creates a new DerivePhalaI2pKeyResponse instance using the specified properties.
         * @function create
         * @memberof pruntime_rpc.DerivePhalaI2pKeyResponse
         * @static
         * @param {pruntime_rpc.IDerivePhalaI2pKeyResponse=} [properties] Properties to set
         * @returns {pruntime_rpc.DerivePhalaI2pKeyResponse} DerivePhalaI2pKeyResponse instance
         */
        DerivePhalaI2pKeyResponse.create = function create(properties) {
            return new DerivePhalaI2pKeyResponse(properties);
        };

        /**
         * Encodes the specified DerivePhalaI2pKeyResponse message. Does not implicitly {@link pruntime_rpc.DerivePhalaI2pKeyResponse.verify|verify} messages.
         * @function encode
         * @memberof pruntime_rpc.DerivePhalaI2pKeyResponse
         * @static
         * @param {pruntime_rpc.IDerivePhalaI2pKeyResponse} message DerivePhalaI2pKeyResponse message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        DerivePhalaI2pKeyResponse.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.phalaI2pKey != null && Object.hasOwnProperty.call(message, "phalaI2pKey"))
                writer.uint32(/* id 1, wireType 2 =*/10).bytes(message.phalaI2pKey);
            return writer;
        };

        /**
         * Encodes the specified DerivePhalaI2pKeyResponse message, length delimited. Does not implicitly {@link pruntime_rpc.DerivePhalaI2pKeyResponse.verify|verify} messages.
         * @function encodeDelimited
         * @memberof pruntime_rpc.DerivePhalaI2pKeyResponse
         * @static
         * @param {pruntime_rpc.IDerivePhalaI2pKeyResponse} message DerivePhalaI2pKeyResponse message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        DerivePhalaI2pKeyResponse.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes a DerivePhalaI2pKeyResponse message from the specified reader or buffer.
         * @function decode
         * @memberof pruntime_rpc.DerivePhalaI2pKeyResponse
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {pruntime_rpc.DerivePhalaI2pKeyResponse} DerivePhalaI2pKeyResponse
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        DerivePhalaI2pKeyResponse.decode = function decode(reader, length) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            var end = length === undefined ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.DerivePhalaI2pKeyResponse();
            while (reader.pos < end) {
                var tag = reader.uint32();
                switch (tag >>> 3) {
                case 1:
                    message.phalaI2pKey = reader.bytes();
                    break;
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a DerivePhalaI2pKeyResponse message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof pruntime_rpc.DerivePhalaI2pKeyResponse
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {pruntime_rpc.DerivePhalaI2pKeyResponse} DerivePhalaI2pKeyResponse
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        DerivePhalaI2pKeyResponse.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies a DerivePhalaI2pKeyResponse message.
         * @function verify
         * @memberof pruntime_rpc.DerivePhalaI2pKeyResponse
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        DerivePhalaI2pKeyResponse.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            if (message.phalaI2pKey != null && message.hasOwnProperty("phalaI2pKey"))
                if (!(message.phalaI2pKey && typeof message.phalaI2pKey.length === "number" || $util.isString(message.phalaI2pKey)))
                    return "phalaI2pKey: buffer expected";
            return null;
        };

        /**
         * Creates a DerivePhalaI2pKeyResponse message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof pruntime_rpc.DerivePhalaI2pKeyResponse
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {pruntime_rpc.DerivePhalaI2pKeyResponse} DerivePhalaI2pKeyResponse
         */
        DerivePhalaI2pKeyResponse.fromObject = function fromObject(object) {
            if (object instanceof $root.pruntime_rpc.DerivePhalaI2pKeyResponse)
                return object;
            var message = new $root.pruntime_rpc.DerivePhalaI2pKeyResponse();
            if (object.phalaI2pKey != null)
                if (typeof object.phalaI2pKey === "string")
                    $util.base64.decode(object.phalaI2pKey, message.phalaI2pKey = $util.newBuffer($util.base64.length(object.phalaI2pKey)), 0);
                else if (object.phalaI2pKey.length)
                    message.phalaI2pKey = object.phalaI2pKey;
            return message;
        };

        /**
         * Creates a plain object from a DerivePhalaI2pKeyResponse message. Also converts values to other types if specified.
         * @function toObject
         * @memberof pruntime_rpc.DerivePhalaI2pKeyResponse
         * @static
         * @param {pruntime_rpc.DerivePhalaI2pKeyResponse} message DerivePhalaI2pKeyResponse
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        DerivePhalaI2pKeyResponse.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            var object = {};
            if (options.defaults)
                if (options.bytes === String)
                    object.phalaI2pKey = "";
                else {
                    object.phalaI2pKey = [];
                    if (options.bytes !== Array)
                        object.phalaI2pKey = $util.newBuffer(object.phalaI2pKey);
                }
            if (message.phalaI2pKey != null && message.hasOwnProperty("phalaI2pKey"))
                object.phalaI2pKey = options.bytes === String ? $util.base64.encode(message.phalaI2pKey, 0, message.phalaI2pKey.length) : options.bytes === Array ? Array.prototype.slice.call(message.phalaI2pKey) : message.phalaI2pKey;
            return object;
        };

        /**
         * Converts this DerivePhalaI2pKeyResponse to JSON.
         * @function toJSON
         * @memberof pruntime_rpc.DerivePhalaI2pKeyResponse
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        DerivePhalaI2pKeyResponse.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        return DerivePhalaI2pKeyResponse;
    })();

    pruntime_rpc.TokenomicStat = (function() {

        /**
         * Properties of a TokenomicStat.
         * @memberof pruntime_rpc
         * @interface ITokenomicStat
         * @property {string|null} [lastPayout] TokenomicStat lastPayout
         * @property {number|null} [lastPayoutAtBlock] TokenomicStat lastPayoutAtBlock
         * @property {string|null} [totalPayout] TokenomicStat totalPayout
         * @property {number|null} [totalPayoutCount] TokenomicStat totalPayoutCount
         * @property {string|null} [lastSlash] TokenomicStat lastSlash
         * @property {number|null} [lastSlashAtBlock] TokenomicStat lastSlashAtBlock
         * @property {string|null} [totalSlash] TokenomicStat totalSlash
         * @property {number|null} [totalSlashCount] TokenomicStat totalSlashCount
         */

        /**
         * Constructs a new TokenomicStat.
         * @memberof pruntime_rpc
         * @classdesc Represents a TokenomicStat.
         * @implements ITokenomicStat
         * @constructor
         * @param {pruntime_rpc.ITokenomicStat=} [properties] Properties to set
         */
        function TokenomicStat(properties) {
            if (properties)
                for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * TokenomicStat lastPayout.
         * @member {string} lastPayout
         * @memberof pruntime_rpc.TokenomicStat
         * @instance
         */
        TokenomicStat.prototype.lastPayout = "";

        /**
         * TokenomicStat lastPayoutAtBlock.
         * @member {number} lastPayoutAtBlock
         * @memberof pruntime_rpc.TokenomicStat
         * @instance
         */
        TokenomicStat.prototype.lastPayoutAtBlock = 0;

        /**
         * TokenomicStat totalPayout.
         * @member {string} totalPayout
         * @memberof pruntime_rpc.TokenomicStat
         * @instance
         */
        TokenomicStat.prototype.totalPayout = "";

        /**
         * TokenomicStat totalPayoutCount.
         * @member {number} totalPayoutCount
         * @memberof pruntime_rpc.TokenomicStat
         * @instance
         */
        TokenomicStat.prototype.totalPayoutCount = 0;

        /**
         * TokenomicStat lastSlash.
         * @member {string} lastSlash
         * @memberof pruntime_rpc.TokenomicStat
         * @instance
         */
        TokenomicStat.prototype.lastSlash = "";

        /**
         * TokenomicStat lastSlashAtBlock.
         * @member {number} lastSlashAtBlock
         * @memberof pruntime_rpc.TokenomicStat
         * @instance
         */
        TokenomicStat.prototype.lastSlashAtBlock = 0;

        /**
         * TokenomicStat totalSlash.
         * @member {string} totalSlash
         * @memberof pruntime_rpc.TokenomicStat
         * @instance
         */
        TokenomicStat.prototype.totalSlash = "";

        /**
         * TokenomicStat totalSlashCount.
         * @member {number} totalSlashCount
         * @memberof pruntime_rpc.TokenomicStat
         * @instance
         */
        TokenomicStat.prototype.totalSlashCount = 0;

        /**
         * Creates a new TokenomicStat instance using the specified properties.
         * @function create
         * @memberof pruntime_rpc.TokenomicStat
         * @static
         * @param {pruntime_rpc.ITokenomicStat=} [properties] Properties to set
         * @returns {pruntime_rpc.TokenomicStat} TokenomicStat instance
         */
        TokenomicStat.create = function create(properties) {
            return new TokenomicStat(properties);
        };

        /**
         * Encodes the specified TokenomicStat message. Does not implicitly {@link pruntime_rpc.TokenomicStat.verify|verify} messages.
         * @function encode
         * @memberof pruntime_rpc.TokenomicStat
         * @static
         * @param {pruntime_rpc.ITokenomicStat} message TokenomicStat message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        TokenomicStat.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.lastPayout != null && Object.hasOwnProperty.call(message, "lastPayout"))
                writer.uint32(/* id 1, wireType 2 =*/10).string(message.lastPayout);
            if (message.lastPayoutAtBlock != null && Object.hasOwnProperty.call(message, "lastPayoutAtBlock"))
                writer.uint32(/* id 2, wireType 0 =*/16).uint32(message.lastPayoutAtBlock);
            if (message.totalPayout != null && Object.hasOwnProperty.call(message, "totalPayout"))
                writer.uint32(/* id 3, wireType 2 =*/26).string(message.totalPayout);
            if (message.totalPayoutCount != null && Object.hasOwnProperty.call(message, "totalPayoutCount"))
                writer.uint32(/* id 4, wireType 0 =*/32).uint32(message.totalPayoutCount);
            if (message.lastSlash != null && Object.hasOwnProperty.call(message, "lastSlash"))
                writer.uint32(/* id 5, wireType 2 =*/42).string(message.lastSlash);
            if (message.lastSlashAtBlock != null && Object.hasOwnProperty.call(message, "lastSlashAtBlock"))
                writer.uint32(/* id 6, wireType 0 =*/48).uint32(message.lastSlashAtBlock);
            if (message.totalSlash != null && Object.hasOwnProperty.call(message, "totalSlash"))
                writer.uint32(/* id 7, wireType 2 =*/58).string(message.totalSlash);
            if (message.totalSlashCount != null && Object.hasOwnProperty.call(message, "totalSlashCount"))
                writer.uint32(/* id 8, wireType 0 =*/64).uint32(message.totalSlashCount);
            return writer;
        };

        /**
         * Encodes the specified TokenomicStat message, length delimited. Does not implicitly {@link pruntime_rpc.TokenomicStat.verify|verify} messages.
         * @function encodeDelimited
         * @memberof pruntime_rpc.TokenomicStat
         * @static
         * @param {pruntime_rpc.ITokenomicStat} message TokenomicStat message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        TokenomicStat.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes a TokenomicStat message from the specified reader or buffer.
         * @function decode
         * @memberof pruntime_rpc.TokenomicStat
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {pruntime_rpc.TokenomicStat} TokenomicStat
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        TokenomicStat.decode = function decode(reader, length) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            var end = length === undefined ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.TokenomicStat();
            while (reader.pos < end) {
                var tag = reader.uint32();
                switch (tag >>> 3) {
                case 1:
                    message.lastPayout = reader.string();
                    break;
                case 2:
                    message.lastPayoutAtBlock = reader.uint32();
                    break;
                case 3:
                    message.totalPayout = reader.string();
                    break;
                case 4:
                    message.totalPayoutCount = reader.uint32();
                    break;
                case 5:
                    message.lastSlash = reader.string();
                    break;
                case 6:
                    message.lastSlashAtBlock = reader.uint32();
                    break;
                case 7:
                    message.totalSlash = reader.string();
                    break;
                case 8:
                    message.totalSlashCount = reader.uint32();
                    break;
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a TokenomicStat message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof pruntime_rpc.TokenomicStat
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {pruntime_rpc.TokenomicStat} TokenomicStat
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        TokenomicStat.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies a TokenomicStat message.
         * @function verify
         * @memberof pruntime_rpc.TokenomicStat
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        TokenomicStat.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            if (message.lastPayout != null && message.hasOwnProperty("lastPayout"))
                if (!$util.isString(message.lastPayout))
                    return "lastPayout: string expected";
            if (message.lastPayoutAtBlock != null && message.hasOwnProperty("lastPayoutAtBlock"))
                if (!$util.isInteger(message.lastPayoutAtBlock))
                    return "lastPayoutAtBlock: integer expected";
            if (message.totalPayout != null && message.hasOwnProperty("totalPayout"))
                if (!$util.isString(message.totalPayout))
                    return "totalPayout: string expected";
            if (message.totalPayoutCount != null && message.hasOwnProperty("totalPayoutCount"))
                if (!$util.isInteger(message.totalPayoutCount))
                    return "totalPayoutCount: integer expected";
            if (message.lastSlash != null && message.hasOwnProperty("lastSlash"))
                if (!$util.isString(message.lastSlash))
                    return "lastSlash: string expected";
            if (message.lastSlashAtBlock != null && message.hasOwnProperty("lastSlashAtBlock"))
                if (!$util.isInteger(message.lastSlashAtBlock))
                    return "lastSlashAtBlock: integer expected";
            if (message.totalSlash != null && message.hasOwnProperty("totalSlash"))
                if (!$util.isString(message.totalSlash))
                    return "totalSlash: string expected";
            if (message.totalSlashCount != null && message.hasOwnProperty("totalSlashCount"))
                if (!$util.isInteger(message.totalSlashCount))
                    return "totalSlashCount: integer expected";
            return null;
        };

        /**
         * Creates a TokenomicStat message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof pruntime_rpc.TokenomicStat
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {pruntime_rpc.TokenomicStat} TokenomicStat
         */
        TokenomicStat.fromObject = function fromObject(object) {
            if (object instanceof $root.pruntime_rpc.TokenomicStat)
                return object;
            var message = new $root.pruntime_rpc.TokenomicStat();
            if (object.lastPayout != null)
                message.lastPayout = String(object.lastPayout);
            if (object.lastPayoutAtBlock != null)
                message.lastPayoutAtBlock = object.lastPayoutAtBlock >>> 0;
            if (object.totalPayout != null)
                message.totalPayout = String(object.totalPayout);
            if (object.totalPayoutCount != null)
                message.totalPayoutCount = object.totalPayoutCount >>> 0;
            if (object.lastSlash != null)
                message.lastSlash = String(object.lastSlash);
            if (object.lastSlashAtBlock != null)
                message.lastSlashAtBlock = object.lastSlashAtBlock >>> 0;
            if (object.totalSlash != null)
                message.totalSlash = String(object.totalSlash);
            if (object.totalSlashCount != null)
                message.totalSlashCount = object.totalSlashCount >>> 0;
            return message;
        };

        /**
         * Creates a plain object from a TokenomicStat message. Also converts values to other types if specified.
         * @function toObject
         * @memberof pruntime_rpc.TokenomicStat
         * @static
         * @param {pruntime_rpc.TokenomicStat} message TokenomicStat
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        TokenomicStat.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            var object = {};
            if (options.defaults) {
                object.lastPayout = "";
                object.lastPayoutAtBlock = 0;
                object.totalPayout = "";
                object.totalPayoutCount = 0;
                object.lastSlash = "";
                object.lastSlashAtBlock = 0;
                object.totalSlash = "";
                object.totalSlashCount = 0;
            }
            if (message.lastPayout != null && message.hasOwnProperty("lastPayout"))
                object.lastPayout = message.lastPayout;
            if (message.lastPayoutAtBlock != null && message.hasOwnProperty("lastPayoutAtBlock"))
                object.lastPayoutAtBlock = message.lastPayoutAtBlock;
            if (message.totalPayout != null && message.hasOwnProperty("totalPayout"))
                object.totalPayout = message.totalPayout;
            if (message.totalPayoutCount != null && message.hasOwnProperty("totalPayoutCount"))
                object.totalPayoutCount = message.totalPayoutCount;
            if (message.lastSlash != null && message.hasOwnProperty("lastSlash"))
                object.lastSlash = message.lastSlash;
            if (message.lastSlashAtBlock != null && message.hasOwnProperty("lastSlashAtBlock"))
                object.lastSlashAtBlock = message.lastSlashAtBlock;
            if (message.totalSlash != null && message.hasOwnProperty("totalSlash"))
                object.totalSlash = message.totalSlash;
            if (message.totalSlashCount != null && message.hasOwnProperty("totalSlashCount"))
                object.totalSlashCount = message.totalSlashCount;
            return object;
        };

        /**
         * Converts this TokenomicStat to JSON.
         * @function toJSON
         * @memberof pruntime_rpc.TokenomicStat
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        TokenomicStat.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        return TokenomicStat;
    })();

    pruntime_rpc.TokenomicInfo = (function() {

        /**
         * Properties of a TokenomicInfo.
         * @memberof pruntime_rpc
         * @interface ITokenomicInfo
         * @property {string|null} [v] TokenomicInfo v
         * @property {string|null} [vInit] TokenomicInfo vInit
         * @property {string|null} [vDeductible] TokenomicInfo vDeductible
         * @property {string|null} [share] TokenomicInfo share
         * @property {number|Long|null} [vUpdateAt] TokenomicInfo vUpdateAt
         * @property {number|null} [vUpdateBlock] TokenomicInfo vUpdateBlock
         * @property {number|Long|null} [iterationLast] TokenomicInfo iterationLast
         * @property {number|Long|null} [challengeTimeLast] TokenomicInfo challengeTimeLast
         * @property {string|null} [pBench] TokenomicInfo pBench
         * @property {string|null} [pInstant] TokenomicInfo pInstant
         * @property {number|null} [confidenceLevel] TokenomicInfo confidenceLevel
         * @property {pruntime_rpc.ITokenomicStat|null} [stat] TokenomicInfo stat
         */

        /**
         * Constructs a new TokenomicInfo.
         * @memberof pruntime_rpc
         * @classdesc Represents a TokenomicInfo.
         * @implements ITokenomicInfo
         * @constructor
         * @param {pruntime_rpc.ITokenomicInfo=} [properties] Properties to set
         */
        function TokenomicInfo(properties) {
            if (properties)
                for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * TokenomicInfo v.
         * @member {string} v
         * @memberof pruntime_rpc.TokenomicInfo
         * @instance
         */
        TokenomicInfo.prototype.v = "";

        /**
         * TokenomicInfo vInit.
         * @member {string} vInit
         * @memberof pruntime_rpc.TokenomicInfo
         * @instance
         */
        TokenomicInfo.prototype.vInit = "";

        /**
         * TokenomicInfo vDeductible.
         * @member {string} vDeductible
         * @memberof pruntime_rpc.TokenomicInfo
         * @instance
         */
        TokenomicInfo.prototype.vDeductible = "";

        /**
         * TokenomicInfo share.
         * @member {string} share
         * @memberof pruntime_rpc.TokenomicInfo
         * @instance
         */
        TokenomicInfo.prototype.share = "";

        /**
         * TokenomicInfo vUpdateAt.
         * @member {number|Long} vUpdateAt
         * @memberof pruntime_rpc.TokenomicInfo
         * @instance
         */
        TokenomicInfo.prototype.vUpdateAt = $util.Long ? $util.Long.fromBits(0,0,true) : 0;

        /**
         * TokenomicInfo vUpdateBlock.
         * @member {number} vUpdateBlock
         * @memberof pruntime_rpc.TokenomicInfo
         * @instance
         */
        TokenomicInfo.prototype.vUpdateBlock = 0;

        /**
         * TokenomicInfo iterationLast.
         * @member {number|Long} iterationLast
         * @memberof pruntime_rpc.TokenomicInfo
         * @instance
         */
        TokenomicInfo.prototype.iterationLast = $util.Long ? $util.Long.fromBits(0,0,true) : 0;

        /**
         * TokenomicInfo challengeTimeLast.
         * @member {number|Long} challengeTimeLast
         * @memberof pruntime_rpc.TokenomicInfo
         * @instance
         */
        TokenomicInfo.prototype.challengeTimeLast = $util.Long ? $util.Long.fromBits(0,0,true) : 0;

        /**
         * TokenomicInfo pBench.
         * @member {string} pBench
         * @memberof pruntime_rpc.TokenomicInfo
         * @instance
         */
        TokenomicInfo.prototype.pBench = "";

        /**
         * TokenomicInfo pInstant.
         * @member {string} pInstant
         * @memberof pruntime_rpc.TokenomicInfo
         * @instance
         */
        TokenomicInfo.prototype.pInstant = "";

        /**
         * TokenomicInfo confidenceLevel.
         * @member {number} confidenceLevel
         * @memberof pruntime_rpc.TokenomicInfo
         * @instance
         */
        TokenomicInfo.prototype.confidenceLevel = 0;

        /**
         * TokenomicInfo stat.
         * @member {pruntime_rpc.ITokenomicStat|null|undefined} stat
         * @memberof pruntime_rpc.TokenomicInfo
         * @instance
         */
        TokenomicInfo.prototype.stat = null;

        /**
         * Creates a new TokenomicInfo instance using the specified properties.
         * @function create
         * @memberof pruntime_rpc.TokenomicInfo
         * @static
         * @param {pruntime_rpc.ITokenomicInfo=} [properties] Properties to set
         * @returns {pruntime_rpc.TokenomicInfo} TokenomicInfo instance
         */
        TokenomicInfo.create = function create(properties) {
            return new TokenomicInfo(properties);
        };

        /**
         * Encodes the specified TokenomicInfo message. Does not implicitly {@link pruntime_rpc.TokenomicInfo.verify|verify} messages.
         * @function encode
         * @memberof pruntime_rpc.TokenomicInfo
         * @static
         * @param {pruntime_rpc.ITokenomicInfo} message TokenomicInfo message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        TokenomicInfo.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.v != null && Object.hasOwnProperty.call(message, "v"))
                writer.uint32(/* id 1, wireType 2 =*/10).string(message.v);
            if (message.vInit != null && Object.hasOwnProperty.call(message, "vInit"))
                writer.uint32(/* id 2, wireType 2 =*/18).string(message.vInit);
            if (message.vUpdateAt != null && Object.hasOwnProperty.call(message, "vUpdateAt"))
                writer.uint32(/* id 4, wireType 0 =*/32).uint64(message.vUpdateAt);
            if (message.vUpdateBlock != null && Object.hasOwnProperty.call(message, "vUpdateBlock"))
                writer.uint32(/* id 5, wireType 0 =*/40).uint32(message.vUpdateBlock);
            if (message.iterationLast != null && Object.hasOwnProperty.call(message, "iterationLast"))
                writer.uint32(/* id 6, wireType 0 =*/48).uint64(message.iterationLast);
            if (message.challengeTimeLast != null && Object.hasOwnProperty.call(message, "challengeTimeLast"))
                writer.uint32(/* id 7, wireType 0 =*/56).uint64(message.challengeTimeLast);
            if (message.pBench != null && Object.hasOwnProperty.call(message, "pBench"))
                writer.uint32(/* id 8, wireType 2 =*/66).string(message.pBench);
            if (message.pInstant != null && Object.hasOwnProperty.call(message, "pInstant"))
                writer.uint32(/* id 9, wireType 2 =*/74).string(message.pInstant);
            if (message.confidenceLevel != null && Object.hasOwnProperty.call(message, "confidenceLevel"))
                writer.uint32(/* id 10, wireType 0 =*/80).uint32(message.confidenceLevel);
            if (message.vDeductible != null && Object.hasOwnProperty.call(message, "vDeductible"))
                writer.uint32(/* id 19, wireType 2 =*/154).string(message.vDeductible);
            if (message.share != null && Object.hasOwnProperty.call(message, "share"))
                writer.uint32(/* id 20, wireType 2 =*/162).string(message.share);
            if (message.stat != null && Object.hasOwnProperty.call(message, "stat"))
                $root.pruntime_rpc.TokenomicStat.encode(message.stat, writer.uint32(/* id 21, wireType 2 =*/170).fork()).ldelim();
            return writer;
        };

        /**
         * Encodes the specified TokenomicInfo message, length delimited. Does not implicitly {@link pruntime_rpc.TokenomicInfo.verify|verify} messages.
         * @function encodeDelimited
         * @memberof pruntime_rpc.TokenomicInfo
         * @static
         * @param {pruntime_rpc.ITokenomicInfo} message TokenomicInfo message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        TokenomicInfo.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes a TokenomicInfo message from the specified reader or buffer.
         * @function decode
         * @memberof pruntime_rpc.TokenomicInfo
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {pruntime_rpc.TokenomicInfo} TokenomicInfo
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        TokenomicInfo.decode = function decode(reader, length) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            var end = length === undefined ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.TokenomicInfo();
            while (reader.pos < end) {
                var tag = reader.uint32();
                switch (tag >>> 3) {
                case 1:
                    message.v = reader.string();
                    break;
                case 2:
                    message.vInit = reader.string();
                    break;
                case 19:
                    message.vDeductible = reader.string();
                    break;
                case 20:
                    message.share = reader.string();
                    break;
                case 4:
                    message.vUpdateAt = reader.uint64();
                    break;
                case 5:
                    message.vUpdateBlock = reader.uint32();
                    break;
                case 6:
                    message.iterationLast = reader.uint64();
                    break;
                case 7:
                    message.challengeTimeLast = reader.uint64();
                    break;
                case 8:
                    message.pBench = reader.string();
                    break;
                case 9:
                    message.pInstant = reader.string();
                    break;
                case 10:
                    message.confidenceLevel = reader.uint32();
                    break;
                case 21:
                    message.stat = $root.pruntime_rpc.TokenomicStat.decode(reader, reader.uint32());
                    break;
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a TokenomicInfo message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof pruntime_rpc.TokenomicInfo
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {pruntime_rpc.TokenomicInfo} TokenomicInfo
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        TokenomicInfo.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies a TokenomicInfo message.
         * @function verify
         * @memberof pruntime_rpc.TokenomicInfo
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        TokenomicInfo.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            if (message.v != null && message.hasOwnProperty("v"))
                if (!$util.isString(message.v))
                    return "v: string expected";
            if (message.vInit != null && message.hasOwnProperty("vInit"))
                if (!$util.isString(message.vInit))
                    return "vInit: string expected";
            if (message.vDeductible != null && message.hasOwnProperty("vDeductible"))
                if (!$util.isString(message.vDeductible))
                    return "vDeductible: string expected";
            if (message.share != null && message.hasOwnProperty("share"))
                if (!$util.isString(message.share))
                    return "share: string expected";
            if (message.vUpdateAt != null && message.hasOwnProperty("vUpdateAt"))
                if (!$util.isInteger(message.vUpdateAt) && !(message.vUpdateAt && $util.isInteger(message.vUpdateAt.low) && $util.isInteger(message.vUpdateAt.high)))
                    return "vUpdateAt: integer|Long expected";
            if (message.vUpdateBlock != null && message.hasOwnProperty("vUpdateBlock"))
                if (!$util.isInteger(message.vUpdateBlock))
                    return "vUpdateBlock: integer expected";
            if (message.iterationLast != null && message.hasOwnProperty("iterationLast"))
                if (!$util.isInteger(message.iterationLast) && !(message.iterationLast && $util.isInteger(message.iterationLast.low) && $util.isInteger(message.iterationLast.high)))
                    return "iterationLast: integer|Long expected";
            if (message.challengeTimeLast != null && message.hasOwnProperty("challengeTimeLast"))
                if (!$util.isInteger(message.challengeTimeLast) && !(message.challengeTimeLast && $util.isInteger(message.challengeTimeLast.low) && $util.isInteger(message.challengeTimeLast.high)))
                    return "challengeTimeLast: integer|Long expected";
            if (message.pBench != null && message.hasOwnProperty("pBench"))
                if (!$util.isString(message.pBench))
                    return "pBench: string expected";
            if (message.pInstant != null && message.hasOwnProperty("pInstant"))
                if (!$util.isString(message.pInstant))
                    return "pInstant: string expected";
            if (message.confidenceLevel != null && message.hasOwnProperty("confidenceLevel"))
                if (!$util.isInteger(message.confidenceLevel))
                    return "confidenceLevel: integer expected";
            if (message.stat != null && message.hasOwnProperty("stat")) {
                var error = $root.pruntime_rpc.TokenomicStat.verify(message.stat);
                if (error)
                    return "stat." + error;
            }
            return null;
        };

        /**
         * Creates a TokenomicInfo message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof pruntime_rpc.TokenomicInfo
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {pruntime_rpc.TokenomicInfo} TokenomicInfo
         */
        TokenomicInfo.fromObject = function fromObject(object) {
            if (object instanceof $root.pruntime_rpc.TokenomicInfo)
                return object;
            var message = new $root.pruntime_rpc.TokenomicInfo();
            if (object.v != null)
                message.v = String(object.v);
            if (object.vInit != null)
                message.vInit = String(object.vInit);
            if (object.vDeductible != null)
                message.vDeductible = String(object.vDeductible);
            if (object.share != null)
                message.share = String(object.share);
            if (object.vUpdateAt != null)
                if ($util.Long)
                    (message.vUpdateAt = $util.Long.fromValue(object.vUpdateAt)).unsigned = true;
                else if (typeof object.vUpdateAt === "string")
                    message.vUpdateAt = parseInt(object.vUpdateAt, 10);
                else if (typeof object.vUpdateAt === "number")
                    message.vUpdateAt = object.vUpdateAt;
                else if (typeof object.vUpdateAt === "object")
                    message.vUpdateAt = new $util.LongBits(object.vUpdateAt.low >>> 0, object.vUpdateAt.high >>> 0).toNumber(true);
            if (object.vUpdateBlock != null)
                message.vUpdateBlock = object.vUpdateBlock >>> 0;
            if (object.iterationLast != null)
                if ($util.Long)
                    (message.iterationLast = $util.Long.fromValue(object.iterationLast)).unsigned = true;
                else if (typeof object.iterationLast === "string")
                    message.iterationLast = parseInt(object.iterationLast, 10);
                else if (typeof object.iterationLast === "number")
                    message.iterationLast = object.iterationLast;
                else if (typeof object.iterationLast === "object")
                    message.iterationLast = new $util.LongBits(object.iterationLast.low >>> 0, object.iterationLast.high >>> 0).toNumber(true);
            if (object.challengeTimeLast != null)
                if ($util.Long)
                    (message.challengeTimeLast = $util.Long.fromValue(object.challengeTimeLast)).unsigned = true;
                else if (typeof object.challengeTimeLast === "string")
                    message.challengeTimeLast = parseInt(object.challengeTimeLast, 10);
                else if (typeof object.challengeTimeLast === "number")
                    message.challengeTimeLast = object.challengeTimeLast;
                else if (typeof object.challengeTimeLast === "object")
                    message.challengeTimeLast = new $util.LongBits(object.challengeTimeLast.low >>> 0, object.challengeTimeLast.high >>> 0).toNumber(true);
            if (object.pBench != null)
                message.pBench = String(object.pBench);
            if (object.pInstant != null)
                message.pInstant = String(object.pInstant);
            if (object.confidenceLevel != null)
                message.confidenceLevel = object.confidenceLevel >>> 0;
            if (object.stat != null) {
                if (typeof object.stat !== "object")
                    throw TypeError(".pruntime_rpc.TokenomicInfo.stat: object expected");
                message.stat = $root.pruntime_rpc.TokenomicStat.fromObject(object.stat);
            }
            return message;
        };

        /**
         * Creates a plain object from a TokenomicInfo message. Also converts values to other types if specified.
         * @function toObject
         * @memberof pruntime_rpc.TokenomicInfo
         * @static
         * @param {pruntime_rpc.TokenomicInfo} message TokenomicInfo
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        TokenomicInfo.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            var object = {};
            if (options.defaults) {
                object.v = "";
                object.vInit = "";
                if ($util.Long) {
                    var long = new $util.Long(0, 0, true);
                    object.vUpdateAt = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : long;
                } else
                    object.vUpdateAt = options.longs === String ? "0" : 0;
                object.vUpdateBlock = 0;
                if ($util.Long) {
                    var long = new $util.Long(0, 0, true);
                    object.iterationLast = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : long;
                } else
                    object.iterationLast = options.longs === String ? "0" : 0;
                if ($util.Long) {
                    var long = new $util.Long(0, 0, true);
                    object.challengeTimeLast = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : long;
                } else
                    object.challengeTimeLast = options.longs === String ? "0" : 0;
                object.pBench = "";
                object.pInstant = "";
                object.confidenceLevel = 0;
                object.vDeductible = "";
                object.share = "";
                object.stat = null;
            }
            if (message.v != null && message.hasOwnProperty("v"))
                object.v = message.v;
            if (message.vInit != null && message.hasOwnProperty("vInit"))
                object.vInit = message.vInit;
            if (message.vUpdateAt != null && message.hasOwnProperty("vUpdateAt"))
                if (typeof message.vUpdateAt === "number")
                    object.vUpdateAt = options.longs === String ? String(message.vUpdateAt) : message.vUpdateAt;
                else
                    object.vUpdateAt = options.longs === String ? $util.Long.prototype.toString.call(message.vUpdateAt) : options.longs === Number ? new $util.LongBits(message.vUpdateAt.low >>> 0, message.vUpdateAt.high >>> 0).toNumber(true) : message.vUpdateAt;
            if (message.vUpdateBlock != null && message.hasOwnProperty("vUpdateBlock"))
                object.vUpdateBlock = message.vUpdateBlock;
            if (message.iterationLast != null && message.hasOwnProperty("iterationLast"))
                if (typeof message.iterationLast === "number")
                    object.iterationLast = options.longs === String ? String(message.iterationLast) : message.iterationLast;
                else
                    object.iterationLast = options.longs === String ? $util.Long.prototype.toString.call(message.iterationLast) : options.longs === Number ? new $util.LongBits(message.iterationLast.low >>> 0, message.iterationLast.high >>> 0).toNumber(true) : message.iterationLast;
            if (message.challengeTimeLast != null && message.hasOwnProperty("challengeTimeLast"))
                if (typeof message.challengeTimeLast === "number")
                    object.challengeTimeLast = options.longs === String ? String(message.challengeTimeLast) : message.challengeTimeLast;
                else
                    object.challengeTimeLast = options.longs === String ? $util.Long.prototype.toString.call(message.challengeTimeLast) : options.longs === Number ? new $util.LongBits(message.challengeTimeLast.low >>> 0, message.challengeTimeLast.high >>> 0).toNumber(true) : message.challengeTimeLast;
            if (message.pBench != null && message.hasOwnProperty("pBench"))
                object.pBench = message.pBench;
            if (message.pInstant != null && message.hasOwnProperty("pInstant"))
                object.pInstant = message.pInstant;
            if (message.confidenceLevel != null && message.hasOwnProperty("confidenceLevel"))
                object.confidenceLevel = message.confidenceLevel;
            if (message.vDeductible != null && message.hasOwnProperty("vDeductible"))
                object.vDeductible = message.vDeductible;
            if (message.share != null && message.hasOwnProperty("share"))
                object.share = message.share;
            if (message.stat != null && message.hasOwnProperty("stat"))
                object.stat = $root.pruntime_rpc.TokenomicStat.toObject(message.stat, options);
            return object;
        };

        /**
         * Converts this TokenomicInfo to JSON.
         * @function toJSON
         * @memberof pruntime_rpc.TokenomicInfo
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        TokenomicInfo.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        return TokenomicInfo;
    })();

    pruntime_rpc.NetworkStatus = (function() {

        /**
         * Properties of a NetworkStatus.
         * @memberof pruntime_rpc
         * @interface INetworkStatus
         * @property {number|null} [publicRpcPort] NetworkStatus publicRpcPort
         * @property {pruntime_rpc.INetworkConfig|null} [config] NetworkStatus config
         */

        /**
         * Constructs a new NetworkStatus.
         * @memberof pruntime_rpc
         * @classdesc Represents a NetworkStatus.
         * @implements INetworkStatus
         * @constructor
         * @param {pruntime_rpc.INetworkStatus=} [properties] Properties to set
         */
        function NetworkStatus(properties) {
            if (properties)
                for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * NetworkStatus publicRpcPort.
         * @member {number|null|undefined} publicRpcPort
         * @memberof pruntime_rpc.NetworkStatus
         * @instance
         */
        NetworkStatus.prototype.publicRpcPort = null;

        /**
         * NetworkStatus config.
         * @member {pruntime_rpc.INetworkConfig|null|undefined} config
         * @memberof pruntime_rpc.NetworkStatus
         * @instance
         */
        NetworkStatus.prototype.config = null;

        // OneOf field names bound to virtual getters and setters
        var $oneOfFields;

        /**
         * NetworkStatus _publicRpcPort.
         * @member {"publicRpcPort"|undefined} _publicRpcPort
         * @memberof pruntime_rpc.NetworkStatus
         * @instance
         */
        Object.defineProperty(NetworkStatus.prototype, "_publicRpcPort", {
            get: $util.oneOfGetter($oneOfFields = ["publicRpcPort"]),
            set: $util.oneOfSetter($oneOfFields)
        });

        /**
         * NetworkStatus _config.
         * @member {"config"|undefined} _config
         * @memberof pruntime_rpc.NetworkStatus
         * @instance
         */
        Object.defineProperty(NetworkStatus.prototype, "_config", {
            get: $util.oneOfGetter($oneOfFields = ["config"]),
            set: $util.oneOfSetter($oneOfFields)
        });

        /**
         * Creates a new NetworkStatus instance using the specified properties.
         * @function create
         * @memberof pruntime_rpc.NetworkStatus
         * @static
         * @param {pruntime_rpc.INetworkStatus=} [properties] Properties to set
         * @returns {pruntime_rpc.NetworkStatus} NetworkStatus instance
         */
        NetworkStatus.create = function create(properties) {
            return new NetworkStatus(properties);
        };

        /**
         * Encodes the specified NetworkStatus message. Does not implicitly {@link pruntime_rpc.NetworkStatus.verify|verify} messages.
         * @function encode
         * @memberof pruntime_rpc.NetworkStatus
         * @static
         * @param {pruntime_rpc.INetworkStatus} message NetworkStatus message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        NetworkStatus.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.publicRpcPort != null && Object.hasOwnProperty.call(message, "publicRpcPort"))
                writer.uint32(/* id 1, wireType 0 =*/8).uint32(message.publicRpcPort);
            if (message.config != null && Object.hasOwnProperty.call(message, "config"))
                $root.pruntime_rpc.NetworkConfig.encode(message.config, writer.uint32(/* id 2, wireType 2 =*/18).fork()).ldelim();
            return writer;
        };

        /**
         * Encodes the specified NetworkStatus message, length delimited. Does not implicitly {@link pruntime_rpc.NetworkStatus.verify|verify} messages.
         * @function encodeDelimited
         * @memberof pruntime_rpc.NetworkStatus
         * @static
         * @param {pruntime_rpc.INetworkStatus} message NetworkStatus message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        NetworkStatus.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes a NetworkStatus message from the specified reader or buffer.
         * @function decode
         * @memberof pruntime_rpc.NetworkStatus
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {pruntime_rpc.NetworkStatus} NetworkStatus
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        NetworkStatus.decode = function decode(reader, length) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            var end = length === undefined ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.NetworkStatus();
            while (reader.pos < end) {
                var tag = reader.uint32();
                switch (tag >>> 3) {
                case 1:
                    message.publicRpcPort = reader.uint32();
                    break;
                case 2:
                    message.config = $root.pruntime_rpc.NetworkConfig.decode(reader, reader.uint32());
                    break;
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a NetworkStatus message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof pruntime_rpc.NetworkStatus
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {pruntime_rpc.NetworkStatus} NetworkStatus
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        NetworkStatus.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies a NetworkStatus message.
         * @function verify
         * @memberof pruntime_rpc.NetworkStatus
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        NetworkStatus.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            var properties = {};
            if (message.publicRpcPort != null && message.hasOwnProperty("publicRpcPort")) {
                properties._publicRpcPort = 1;
                if (!$util.isInteger(message.publicRpcPort))
                    return "publicRpcPort: integer expected";
            }
            if (message.config != null && message.hasOwnProperty("config")) {
                properties._config = 1;
                {
                    var error = $root.pruntime_rpc.NetworkConfig.verify(message.config);
                    if (error)
                        return "config." + error;
                }
            }
            return null;
        };

        /**
         * Creates a NetworkStatus message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof pruntime_rpc.NetworkStatus
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {pruntime_rpc.NetworkStatus} NetworkStatus
         */
        NetworkStatus.fromObject = function fromObject(object) {
            if (object instanceof $root.pruntime_rpc.NetworkStatus)
                return object;
            var message = new $root.pruntime_rpc.NetworkStatus();
            if (object.publicRpcPort != null)
                message.publicRpcPort = object.publicRpcPort >>> 0;
            if (object.config != null) {
                if (typeof object.config !== "object")
                    throw TypeError(".pruntime_rpc.NetworkStatus.config: object expected");
                message.config = $root.pruntime_rpc.NetworkConfig.fromObject(object.config);
            }
            return message;
        };

        /**
         * Creates a plain object from a NetworkStatus message. Also converts values to other types if specified.
         * @function toObject
         * @memberof pruntime_rpc.NetworkStatus
         * @static
         * @param {pruntime_rpc.NetworkStatus} message NetworkStatus
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        NetworkStatus.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            var object = {};
            if (message.publicRpcPort != null && message.hasOwnProperty("publicRpcPort")) {
                object.publicRpcPort = message.publicRpcPort;
                if (options.oneofs)
                    object._publicRpcPort = "publicRpcPort";
            }
            if (message.config != null && message.hasOwnProperty("config")) {
                object.config = $root.pruntime_rpc.NetworkConfig.toObject(message.config, options);
                if (options.oneofs)
                    object._config = "config";
            }
            return object;
        };

        /**
         * Converts this NetworkStatus to JSON.
         * @function toJSON
         * @memberof pruntime_rpc.NetworkStatus
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        NetworkStatus.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        return NetworkStatus;
    })();

    pruntime_rpc.NetworkConfig = (function() {

        /**
         * Properties of a NetworkConfig.
         * @memberof pruntime_rpc
         * @interface INetworkConfig
         * @property {string|null} [allProxy] NetworkConfig allProxy
         * @property {string|null} [i2pProxy] NetworkConfig i2pProxy
         */

        /**
         * Constructs a new NetworkConfig.
         * @memberof pruntime_rpc
         * @classdesc Represents a NetworkConfig.
         * @implements INetworkConfig
         * @constructor
         * @param {pruntime_rpc.INetworkConfig=} [properties] Properties to set
         */
        function NetworkConfig(properties) {
            if (properties)
                for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * NetworkConfig allProxy.
         * @member {string} allProxy
         * @memberof pruntime_rpc.NetworkConfig
         * @instance
         */
        NetworkConfig.prototype.allProxy = "";

        /**
         * NetworkConfig i2pProxy.
         * @member {string} i2pProxy
         * @memberof pruntime_rpc.NetworkConfig
         * @instance
         */
        NetworkConfig.prototype.i2pProxy = "";

        /**
         * Creates a new NetworkConfig instance using the specified properties.
         * @function create
         * @memberof pruntime_rpc.NetworkConfig
         * @static
         * @param {pruntime_rpc.INetworkConfig=} [properties] Properties to set
         * @returns {pruntime_rpc.NetworkConfig} NetworkConfig instance
         */
        NetworkConfig.create = function create(properties) {
            return new NetworkConfig(properties);
        };

        /**
         * Encodes the specified NetworkConfig message. Does not implicitly {@link pruntime_rpc.NetworkConfig.verify|verify} messages.
         * @function encode
         * @memberof pruntime_rpc.NetworkConfig
         * @static
         * @param {pruntime_rpc.INetworkConfig} message NetworkConfig message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        NetworkConfig.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.allProxy != null && Object.hasOwnProperty.call(message, "allProxy"))
                writer.uint32(/* id 2, wireType 2 =*/18).string(message.allProxy);
            if (message.i2pProxy != null && Object.hasOwnProperty.call(message, "i2pProxy"))
                writer.uint32(/* id 3, wireType 2 =*/26).string(message.i2pProxy);
            return writer;
        };

        /**
         * Encodes the specified NetworkConfig message, length delimited. Does not implicitly {@link pruntime_rpc.NetworkConfig.verify|verify} messages.
         * @function encodeDelimited
         * @memberof pruntime_rpc.NetworkConfig
         * @static
         * @param {pruntime_rpc.INetworkConfig} message NetworkConfig message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        NetworkConfig.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes a NetworkConfig message from the specified reader or buffer.
         * @function decode
         * @memberof pruntime_rpc.NetworkConfig
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {pruntime_rpc.NetworkConfig} NetworkConfig
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        NetworkConfig.decode = function decode(reader, length) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            var end = length === undefined ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.NetworkConfig();
            while (reader.pos < end) {
                var tag = reader.uint32();
                switch (tag >>> 3) {
                case 2:
                    message.allProxy = reader.string();
                    break;
                case 3:
                    message.i2pProxy = reader.string();
                    break;
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a NetworkConfig message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof pruntime_rpc.NetworkConfig
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {pruntime_rpc.NetworkConfig} NetworkConfig
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        NetworkConfig.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies a NetworkConfig message.
         * @function verify
         * @memberof pruntime_rpc.NetworkConfig
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        NetworkConfig.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            if (message.allProxy != null && message.hasOwnProperty("allProxy"))
                if (!$util.isString(message.allProxy))
                    return "allProxy: string expected";
            if (message.i2pProxy != null && message.hasOwnProperty("i2pProxy"))
                if (!$util.isString(message.i2pProxy))
                    return "i2pProxy: string expected";
            return null;
        };

        /**
         * Creates a NetworkConfig message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof pruntime_rpc.NetworkConfig
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {pruntime_rpc.NetworkConfig} NetworkConfig
         */
        NetworkConfig.fromObject = function fromObject(object) {
            if (object instanceof $root.pruntime_rpc.NetworkConfig)
                return object;
            var message = new $root.pruntime_rpc.NetworkConfig();
            if (object.allProxy != null)
                message.allProxy = String(object.allProxy);
            if (object.i2pProxy != null)
                message.i2pProxy = String(object.i2pProxy);
            return message;
        };

        /**
         * Creates a plain object from a NetworkConfig message. Also converts values to other types if specified.
         * @function toObject
         * @memberof pruntime_rpc.NetworkConfig
         * @static
         * @param {pruntime_rpc.NetworkConfig} message NetworkConfig
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        NetworkConfig.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            var object = {};
            if (options.defaults) {
                object.allProxy = "";
                object.i2pProxy = "";
            }
            if (message.allProxy != null && message.hasOwnProperty("allProxy"))
                object.allProxy = message.allProxy;
            if (message.i2pProxy != null && message.hasOwnProperty("i2pProxy"))
                object.i2pProxy = message.i2pProxy;
            return object;
        };

        /**
         * Converts this NetworkConfig to JSON.
         * @function toJSON
         * @memberof pruntime_rpc.NetworkConfig
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        NetworkConfig.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        return NetworkConfig;
    })();

    pruntime_rpc.HttpHeader = (function() {

        /**
         * Properties of a HttpHeader.
         * @memberof pruntime_rpc
         * @interface IHttpHeader
         * @property {string|null} [name] HttpHeader name
         * @property {string|null} [value] HttpHeader value
         */

        /**
         * Constructs a new HttpHeader.
         * @memberof pruntime_rpc
         * @classdesc Represents a HttpHeader.
         * @implements IHttpHeader
         * @constructor
         * @param {pruntime_rpc.IHttpHeader=} [properties] Properties to set
         */
        function HttpHeader(properties) {
            if (properties)
                for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * HttpHeader name.
         * @member {string} name
         * @memberof pruntime_rpc.HttpHeader
         * @instance
         */
        HttpHeader.prototype.name = "";

        /**
         * HttpHeader value.
         * @member {string} value
         * @memberof pruntime_rpc.HttpHeader
         * @instance
         */
        HttpHeader.prototype.value = "";

        /**
         * Creates a new HttpHeader instance using the specified properties.
         * @function create
         * @memberof pruntime_rpc.HttpHeader
         * @static
         * @param {pruntime_rpc.IHttpHeader=} [properties] Properties to set
         * @returns {pruntime_rpc.HttpHeader} HttpHeader instance
         */
        HttpHeader.create = function create(properties) {
            return new HttpHeader(properties);
        };

        /**
         * Encodes the specified HttpHeader message. Does not implicitly {@link pruntime_rpc.HttpHeader.verify|verify} messages.
         * @function encode
         * @memberof pruntime_rpc.HttpHeader
         * @static
         * @param {pruntime_rpc.IHttpHeader} message HttpHeader message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        HttpHeader.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.name != null && Object.hasOwnProperty.call(message, "name"))
                writer.uint32(/* id 1, wireType 2 =*/10).string(message.name);
            if (message.value != null && Object.hasOwnProperty.call(message, "value"))
                writer.uint32(/* id 2, wireType 2 =*/18).string(message.value);
            return writer;
        };

        /**
         * Encodes the specified HttpHeader message, length delimited. Does not implicitly {@link pruntime_rpc.HttpHeader.verify|verify} messages.
         * @function encodeDelimited
         * @memberof pruntime_rpc.HttpHeader
         * @static
         * @param {pruntime_rpc.IHttpHeader} message HttpHeader message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        HttpHeader.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes a HttpHeader message from the specified reader or buffer.
         * @function decode
         * @memberof pruntime_rpc.HttpHeader
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {pruntime_rpc.HttpHeader} HttpHeader
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        HttpHeader.decode = function decode(reader, length) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            var end = length === undefined ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.HttpHeader();
            while (reader.pos < end) {
                var tag = reader.uint32();
                switch (tag >>> 3) {
                case 1:
                    message.name = reader.string();
                    break;
                case 2:
                    message.value = reader.string();
                    break;
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a HttpHeader message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof pruntime_rpc.HttpHeader
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {pruntime_rpc.HttpHeader} HttpHeader
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        HttpHeader.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies a HttpHeader message.
         * @function verify
         * @memberof pruntime_rpc.HttpHeader
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        HttpHeader.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            if (message.name != null && message.hasOwnProperty("name"))
                if (!$util.isString(message.name))
                    return "name: string expected";
            if (message.value != null && message.hasOwnProperty("value"))
                if (!$util.isString(message.value))
                    return "value: string expected";
            return null;
        };

        /**
         * Creates a HttpHeader message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof pruntime_rpc.HttpHeader
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {pruntime_rpc.HttpHeader} HttpHeader
         */
        HttpHeader.fromObject = function fromObject(object) {
            if (object instanceof $root.pruntime_rpc.HttpHeader)
                return object;
            var message = new $root.pruntime_rpc.HttpHeader();
            if (object.name != null)
                message.name = String(object.name);
            if (object.value != null)
                message.value = String(object.value);
            return message;
        };

        /**
         * Creates a plain object from a HttpHeader message. Also converts values to other types if specified.
         * @function toObject
         * @memberof pruntime_rpc.HttpHeader
         * @static
         * @param {pruntime_rpc.HttpHeader} message HttpHeader
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        HttpHeader.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            var object = {};
            if (options.defaults) {
                object.name = "";
                object.value = "";
            }
            if (message.name != null && message.hasOwnProperty("name"))
                object.name = message.name;
            if (message.value != null && message.hasOwnProperty("value"))
                object.value = message.value;
            return object;
        };

        /**
         * Converts this HttpHeader to JSON.
         * @function toJSON
         * @memberof pruntime_rpc.HttpHeader
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        HttpHeader.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        return HttpHeader;
    })();

    pruntime_rpc.HttpRequest = (function() {

        /**
         * Properties of a HttpRequest.
         * @memberof pruntime_rpc
         * @interface IHttpRequest
         * @property {string|null} [url] HttpRequest url
         * @property {string|null} [method] HttpRequest method
         * @property {Array.<pruntime_rpc.IHttpHeader>|null} [headers] HttpRequest headers
         * @property {Uint8Array|null} [body] HttpRequest body
         */

        /**
         * Constructs a new HttpRequest.
         * @memberof pruntime_rpc
         * @classdesc Represents a HttpRequest.
         * @implements IHttpRequest
         * @constructor
         * @param {pruntime_rpc.IHttpRequest=} [properties] Properties to set
         */
        function HttpRequest(properties) {
            this.headers = [];
            if (properties)
                for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * HttpRequest url.
         * @member {string} url
         * @memberof pruntime_rpc.HttpRequest
         * @instance
         */
        HttpRequest.prototype.url = "";

        /**
         * HttpRequest method.
         * @member {string} method
         * @memberof pruntime_rpc.HttpRequest
         * @instance
         */
        HttpRequest.prototype.method = "";

        /**
         * HttpRequest headers.
         * @member {Array.<pruntime_rpc.IHttpHeader>} headers
         * @memberof pruntime_rpc.HttpRequest
         * @instance
         */
        HttpRequest.prototype.headers = $util.emptyArray;

        /**
         * HttpRequest body.
         * @member {Uint8Array} body
         * @memberof pruntime_rpc.HttpRequest
         * @instance
         */
        HttpRequest.prototype.body = $util.newBuffer([]);

        /**
         * Creates a new HttpRequest instance using the specified properties.
         * @function create
         * @memberof pruntime_rpc.HttpRequest
         * @static
         * @param {pruntime_rpc.IHttpRequest=} [properties] Properties to set
         * @returns {pruntime_rpc.HttpRequest} HttpRequest instance
         */
        HttpRequest.create = function create(properties) {
            return new HttpRequest(properties);
        };

        /**
         * Encodes the specified HttpRequest message. Does not implicitly {@link pruntime_rpc.HttpRequest.verify|verify} messages.
         * @function encode
         * @memberof pruntime_rpc.HttpRequest
         * @static
         * @param {pruntime_rpc.IHttpRequest} message HttpRequest message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        HttpRequest.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.url != null && Object.hasOwnProperty.call(message, "url"))
                writer.uint32(/* id 1, wireType 2 =*/10).string(message.url);
            if (message.method != null && Object.hasOwnProperty.call(message, "method"))
                writer.uint32(/* id 2, wireType 2 =*/18).string(message.method);
            if (message.headers != null && message.headers.length)
                for (var i = 0; i < message.headers.length; ++i)
                    $root.pruntime_rpc.HttpHeader.encode(message.headers[i], writer.uint32(/* id 3, wireType 2 =*/26).fork()).ldelim();
            if (message.body != null && Object.hasOwnProperty.call(message, "body"))
                writer.uint32(/* id 4, wireType 2 =*/34).bytes(message.body);
            return writer;
        };

        /**
         * Encodes the specified HttpRequest message, length delimited. Does not implicitly {@link pruntime_rpc.HttpRequest.verify|verify} messages.
         * @function encodeDelimited
         * @memberof pruntime_rpc.HttpRequest
         * @static
         * @param {pruntime_rpc.IHttpRequest} message HttpRequest message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        HttpRequest.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes a HttpRequest message from the specified reader or buffer.
         * @function decode
         * @memberof pruntime_rpc.HttpRequest
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {pruntime_rpc.HttpRequest} HttpRequest
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        HttpRequest.decode = function decode(reader, length) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            var end = length === undefined ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.HttpRequest();
            while (reader.pos < end) {
                var tag = reader.uint32();
                switch (tag >>> 3) {
                case 1:
                    message.url = reader.string();
                    break;
                case 2:
                    message.method = reader.string();
                    break;
                case 3:
                    if (!(message.headers && message.headers.length))
                        message.headers = [];
                    message.headers.push($root.pruntime_rpc.HttpHeader.decode(reader, reader.uint32()));
                    break;
                case 4:
                    message.body = reader.bytes();
                    break;
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a HttpRequest message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof pruntime_rpc.HttpRequest
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {pruntime_rpc.HttpRequest} HttpRequest
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        HttpRequest.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies a HttpRequest message.
         * @function verify
         * @memberof pruntime_rpc.HttpRequest
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        HttpRequest.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            if (message.url != null && message.hasOwnProperty("url"))
                if (!$util.isString(message.url))
                    return "url: string expected";
            if (message.method != null && message.hasOwnProperty("method"))
                if (!$util.isString(message.method))
                    return "method: string expected";
            if (message.headers != null && message.hasOwnProperty("headers")) {
                if (!Array.isArray(message.headers))
                    return "headers: array expected";
                for (var i = 0; i < message.headers.length; ++i) {
                    var error = $root.pruntime_rpc.HttpHeader.verify(message.headers[i]);
                    if (error)
                        return "headers." + error;
                }
            }
            if (message.body != null && message.hasOwnProperty("body"))
                if (!(message.body && typeof message.body.length === "number" || $util.isString(message.body)))
                    return "body: buffer expected";
            return null;
        };

        /**
         * Creates a HttpRequest message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof pruntime_rpc.HttpRequest
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {pruntime_rpc.HttpRequest} HttpRequest
         */
        HttpRequest.fromObject = function fromObject(object) {
            if (object instanceof $root.pruntime_rpc.HttpRequest)
                return object;
            var message = new $root.pruntime_rpc.HttpRequest();
            if (object.url != null)
                message.url = String(object.url);
            if (object.method != null)
                message.method = String(object.method);
            if (object.headers) {
                if (!Array.isArray(object.headers))
                    throw TypeError(".pruntime_rpc.HttpRequest.headers: array expected");
                message.headers = [];
                for (var i = 0; i < object.headers.length; ++i) {
                    if (typeof object.headers[i] !== "object")
                        throw TypeError(".pruntime_rpc.HttpRequest.headers: object expected");
                    message.headers[i] = $root.pruntime_rpc.HttpHeader.fromObject(object.headers[i]);
                }
            }
            if (object.body != null)
                if (typeof object.body === "string")
                    $util.base64.decode(object.body, message.body = $util.newBuffer($util.base64.length(object.body)), 0);
                else if (object.body.length)
                    message.body = object.body;
            return message;
        };

        /**
         * Creates a plain object from a HttpRequest message. Also converts values to other types if specified.
         * @function toObject
         * @memberof pruntime_rpc.HttpRequest
         * @static
         * @param {pruntime_rpc.HttpRequest} message HttpRequest
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        HttpRequest.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            var object = {};
            if (options.arrays || options.defaults)
                object.headers = [];
            if (options.defaults) {
                object.url = "";
                object.method = "";
                if (options.bytes === String)
                    object.body = "";
                else {
                    object.body = [];
                    if (options.bytes !== Array)
                        object.body = $util.newBuffer(object.body);
                }
            }
            if (message.url != null && message.hasOwnProperty("url"))
                object.url = message.url;
            if (message.method != null && message.hasOwnProperty("method"))
                object.method = message.method;
            if (message.headers && message.headers.length) {
                object.headers = [];
                for (var j = 0; j < message.headers.length; ++j)
                    object.headers[j] = $root.pruntime_rpc.HttpHeader.toObject(message.headers[j], options);
            }
            if (message.body != null && message.hasOwnProperty("body"))
                object.body = options.bytes === String ? $util.base64.encode(message.body, 0, message.body.length) : options.bytes === Array ? Array.prototype.slice.call(message.body) : message.body;
            return object;
        };

        /**
         * Converts this HttpRequest to JSON.
         * @function toJSON
         * @memberof pruntime_rpc.HttpRequest
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        HttpRequest.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        return HttpRequest;
    })();

    pruntime_rpc.HttpResponse = (function() {

        /**
         * Properties of a HttpResponse.
         * @memberof pruntime_rpc
         * @interface IHttpResponse
         * @property {number|null} [statusCode] HttpResponse statusCode
         * @property {Array.<pruntime_rpc.IHttpHeader>|null} [headers] HttpResponse headers
         * @property {Uint8Array|null} [body] HttpResponse body
         */

        /**
         * Constructs a new HttpResponse.
         * @memberof pruntime_rpc
         * @classdesc Represents a HttpResponse.
         * @implements IHttpResponse
         * @constructor
         * @param {pruntime_rpc.IHttpResponse=} [properties] Properties to set
         */
        function HttpResponse(properties) {
            this.headers = [];
            if (properties)
                for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * HttpResponse statusCode.
         * @member {number} statusCode
         * @memberof pruntime_rpc.HttpResponse
         * @instance
         */
        HttpResponse.prototype.statusCode = 0;

        /**
         * HttpResponse headers.
         * @member {Array.<pruntime_rpc.IHttpHeader>} headers
         * @memberof pruntime_rpc.HttpResponse
         * @instance
         */
        HttpResponse.prototype.headers = $util.emptyArray;

        /**
         * HttpResponse body.
         * @member {Uint8Array} body
         * @memberof pruntime_rpc.HttpResponse
         * @instance
         */
        HttpResponse.prototype.body = $util.newBuffer([]);

        /**
         * Creates a new HttpResponse instance using the specified properties.
         * @function create
         * @memberof pruntime_rpc.HttpResponse
         * @static
         * @param {pruntime_rpc.IHttpResponse=} [properties] Properties to set
         * @returns {pruntime_rpc.HttpResponse} HttpResponse instance
         */
        HttpResponse.create = function create(properties) {
            return new HttpResponse(properties);
        };

        /**
         * Encodes the specified HttpResponse message. Does not implicitly {@link pruntime_rpc.HttpResponse.verify|verify} messages.
         * @function encode
         * @memberof pruntime_rpc.HttpResponse
         * @static
         * @param {pruntime_rpc.IHttpResponse} message HttpResponse message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        HttpResponse.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.statusCode != null && Object.hasOwnProperty.call(message, "statusCode"))
                writer.uint32(/* id 1, wireType 0 =*/8).uint32(message.statusCode);
            if (message.headers != null && message.headers.length)
                for (var i = 0; i < message.headers.length; ++i)
                    $root.pruntime_rpc.HttpHeader.encode(message.headers[i], writer.uint32(/* id 2, wireType 2 =*/18).fork()).ldelim();
            if (message.body != null && Object.hasOwnProperty.call(message, "body"))
                writer.uint32(/* id 3, wireType 2 =*/26).bytes(message.body);
            return writer;
        };

        /**
         * Encodes the specified HttpResponse message, length delimited. Does not implicitly {@link pruntime_rpc.HttpResponse.verify|verify} messages.
         * @function encodeDelimited
         * @memberof pruntime_rpc.HttpResponse
         * @static
         * @param {pruntime_rpc.IHttpResponse} message HttpResponse message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        HttpResponse.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes a HttpResponse message from the specified reader or buffer.
         * @function decode
         * @memberof pruntime_rpc.HttpResponse
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {pruntime_rpc.HttpResponse} HttpResponse
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        HttpResponse.decode = function decode(reader, length) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            var end = length === undefined ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.HttpResponse();
            while (reader.pos < end) {
                var tag = reader.uint32();
                switch (tag >>> 3) {
                case 1:
                    message.statusCode = reader.uint32();
                    break;
                case 2:
                    if (!(message.headers && message.headers.length))
                        message.headers = [];
                    message.headers.push($root.pruntime_rpc.HttpHeader.decode(reader, reader.uint32()));
                    break;
                case 3:
                    message.body = reader.bytes();
                    break;
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a HttpResponse message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof pruntime_rpc.HttpResponse
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {pruntime_rpc.HttpResponse} HttpResponse
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        HttpResponse.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies a HttpResponse message.
         * @function verify
         * @memberof pruntime_rpc.HttpResponse
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        HttpResponse.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            if (message.statusCode != null && message.hasOwnProperty("statusCode"))
                if (!$util.isInteger(message.statusCode))
                    return "statusCode: integer expected";
            if (message.headers != null && message.hasOwnProperty("headers")) {
                if (!Array.isArray(message.headers))
                    return "headers: array expected";
                for (var i = 0; i < message.headers.length; ++i) {
                    var error = $root.pruntime_rpc.HttpHeader.verify(message.headers[i]);
                    if (error)
                        return "headers." + error;
                }
            }
            if (message.body != null && message.hasOwnProperty("body"))
                if (!(message.body && typeof message.body.length === "number" || $util.isString(message.body)))
                    return "body: buffer expected";
            return null;
        };

        /**
         * Creates a HttpResponse message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof pruntime_rpc.HttpResponse
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {pruntime_rpc.HttpResponse} HttpResponse
         */
        HttpResponse.fromObject = function fromObject(object) {
            if (object instanceof $root.pruntime_rpc.HttpResponse)
                return object;
            var message = new $root.pruntime_rpc.HttpResponse();
            if (object.statusCode != null)
                message.statusCode = object.statusCode >>> 0;
            if (object.headers) {
                if (!Array.isArray(object.headers))
                    throw TypeError(".pruntime_rpc.HttpResponse.headers: array expected");
                message.headers = [];
                for (var i = 0; i < object.headers.length; ++i) {
                    if (typeof object.headers[i] !== "object")
                        throw TypeError(".pruntime_rpc.HttpResponse.headers: object expected");
                    message.headers[i] = $root.pruntime_rpc.HttpHeader.fromObject(object.headers[i]);
                }
            }
            if (object.body != null)
                if (typeof object.body === "string")
                    $util.base64.decode(object.body, message.body = $util.newBuffer($util.base64.length(object.body)), 0);
                else if (object.body.length)
                    message.body = object.body;
            return message;
        };

        /**
         * Creates a plain object from a HttpResponse message. Also converts values to other types if specified.
         * @function toObject
         * @memberof pruntime_rpc.HttpResponse
         * @static
         * @param {pruntime_rpc.HttpResponse} message HttpResponse
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        HttpResponse.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            var object = {};
            if (options.arrays || options.defaults)
                object.headers = [];
            if (options.defaults) {
                object.statusCode = 0;
                if (options.bytes === String)
                    object.body = "";
                else {
                    object.body = [];
                    if (options.bytes !== Array)
                        object.body = $util.newBuffer(object.body);
                }
            }
            if (message.statusCode != null && message.hasOwnProperty("statusCode"))
                object.statusCode = message.statusCode;
            if (message.headers && message.headers.length) {
                object.headers = [];
                for (var j = 0; j < message.headers.length; ++j)
                    object.headers[j] = $root.pruntime_rpc.HttpHeader.toObject(message.headers[j], options);
            }
            if (message.body != null && message.hasOwnProperty("body"))
                object.body = options.bytes === String ? $util.base64.encode(message.body, 0, message.body.length) : options.bytes === Array ? Array.prototype.slice.call(message.body) : message.body;
            return object;
        };

        /**
         * Converts this HttpResponse to JSON.
         * @function toJSON
         * @memberof pruntime_rpc.HttpResponse
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        HttpResponse.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        return HttpResponse;
    })();

    pruntime_rpc.GetContractInfoRequest = (function() {

        /**
         * Properties of a GetContractInfoRequest.
         * @memberof pruntime_rpc
         * @interface IGetContractInfoRequest
         * @property {Array.<string>|null} [contractIds] GetContractInfoRequest contractIds
         */

        /**
         * Constructs a new GetContractInfoRequest.
         * @memberof pruntime_rpc
         * @classdesc Represents a GetContractInfoRequest.
         * @implements IGetContractInfoRequest
         * @constructor
         * @param {pruntime_rpc.IGetContractInfoRequest=} [properties] Properties to set
         */
        function GetContractInfoRequest(properties) {
            this.contractIds = [];
            if (properties)
                for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * GetContractInfoRequest contractIds.
         * @member {Array.<string>} contractIds
         * @memberof pruntime_rpc.GetContractInfoRequest
         * @instance
         */
        GetContractInfoRequest.prototype.contractIds = $util.emptyArray;

        /**
         * Creates a new GetContractInfoRequest instance using the specified properties.
         * @function create
         * @memberof pruntime_rpc.GetContractInfoRequest
         * @static
         * @param {pruntime_rpc.IGetContractInfoRequest=} [properties] Properties to set
         * @returns {pruntime_rpc.GetContractInfoRequest} GetContractInfoRequest instance
         */
        GetContractInfoRequest.create = function create(properties) {
            return new GetContractInfoRequest(properties);
        };

        /**
         * Encodes the specified GetContractInfoRequest message. Does not implicitly {@link pruntime_rpc.GetContractInfoRequest.verify|verify} messages.
         * @function encode
         * @memberof pruntime_rpc.GetContractInfoRequest
         * @static
         * @param {pruntime_rpc.IGetContractInfoRequest} message GetContractInfoRequest message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        GetContractInfoRequest.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.contractIds != null && message.contractIds.length)
                for (var i = 0; i < message.contractIds.length; ++i)
                    writer.uint32(/* id 1, wireType 2 =*/10).string(message.contractIds[i]);
            return writer;
        };

        /**
         * Encodes the specified GetContractInfoRequest message, length delimited. Does not implicitly {@link pruntime_rpc.GetContractInfoRequest.verify|verify} messages.
         * @function encodeDelimited
         * @memberof pruntime_rpc.GetContractInfoRequest
         * @static
         * @param {pruntime_rpc.IGetContractInfoRequest} message GetContractInfoRequest message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        GetContractInfoRequest.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes a GetContractInfoRequest message from the specified reader or buffer.
         * @function decode
         * @memberof pruntime_rpc.GetContractInfoRequest
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {pruntime_rpc.GetContractInfoRequest} GetContractInfoRequest
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        GetContractInfoRequest.decode = function decode(reader, length) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            var end = length === undefined ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.GetContractInfoRequest();
            while (reader.pos < end) {
                var tag = reader.uint32();
                switch (tag >>> 3) {
                case 1:
                    if (!(message.contractIds && message.contractIds.length))
                        message.contractIds = [];
                    message.contractIds.push(reader.string());
                    break;
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a GetContractInfoRequest message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof pruntime_rpc.GetContractInfoRequest
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {pruntime_rpc.GetContractInfoRequest} GetContractInfoRequest
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        GetContractInfoRequest.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies a GetContractInfoRequest message.
         * @function verify
         * @memberof pruntime_rpc.GetContractInfoRequest
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        GetContractInfoRequest.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            if (message.contractIds != null && message.hasOwnProperty("contractIds")) {
                if (!Array.isArray(message.contractIds))
                    return "contractIds: array expected";
                for (var i = 0; i < message.contractIds.length; ++i)
                    if (!$util.isString(message.contractIds[i]))
                        return "contractIds: string[] expected";
            }
            return null;
        };

        /**
         * Creates a GetContractInfoRequest message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof pruntime_rpc.GetContractInfoRequest
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {pruntime_rpc.GetContractInfoRequest} GetContractInfoRequest
         */
        GetContractInfoRequest.fromObject = function fromObject(object) {
            if (object instanceof $root.pruntime_rpc.GetContractInfoRequest)
                return object;
            var message = new $root.pruntime_rpc.GetContractInfoRequest();
            if (object.contractIds) {
                if (!Array.isArray(object.contractIds))
                    throw TypeError(".pruntime_rpc.GetContractInfoRequest.contractIds: array expected");
                message.contractIds = [];
                for (var i = 0; i < object.contractIds.length; ++i)
                    message.contractIds[i] = String(object.contractIds[i]);
            }
            return message;
        };

        /**
         * Creates a plain object from a GetContractInfoRequest message. Also converts values to other types if specified.
         * @function toObject
         * @memberof pruntime_rpc.GetContractInfoRequest
         * @static
         * @param {pruntime_rpc.GetContractInfoRequest} message GetContractInfoRequest
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        GetContractInfoRequest.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            var object = {};
            if (options.arrays || options.defaults)
                object.contractIds = [];
            if (message.contractIds && message.contractIds.length) {
                object.contractIds = [];
                for (var j = 0; j < message.contractIds.length; ++j)
                    object.contractIds[j] = message.contractIds[j];
            }
            return object;
        };

        /**
         * Converts this GetContractInfoRequest to JSON.
         * @function toJSON
         * @memberof pruntime_rpc.GetContractInfoRequest
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        GetContractInfoRequest.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        return GetContractInfoRequest;
    })();

    pruntime_rpc.GetContractInfoResponse = (function() {

        /**
         * Properties of a GetContractInfoResponse.
         * @memberof pruntime_rpc
         * @interface IGetContractInfoResponse
         * @property {Array.<pruntime_rpc.IContractInfo>|null} [contracts] GetContractInfoResponse contracts
         */

        /**
         * Constructs a new GetContractInfoResponse.
         * @memberof pruntime_rpc
         * @classdesc Represents a GetContractInfoResponse.
         * @implements IGetContractInfoResponse
         * @constructor
         * @param {pruntime_rpc.IGetContractInfoResponse=} [properties] Properties to set
         */
        function GetContractInfoResponse(properties) {
            this.contracts = [];
            if (properties)
                for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * GetContractInfoResponse contracts.
         * @member {Array.<pruntime_rpc.IContractInfo>} contracts
         * @memberof pruntime_rpc.GetContractInfoResponse
         * @instance
         */
        GetContractInfoResponse.prototype.contracts = $util.emptyArray;

        /**
         * Creates a new GetContractInfoResponse instance using the specified properties.
         * @function create
         * @memberof pruntime_rpc.GetContractInfoResponse
         * @static
         * @param {pruntime_rpc.IGetContractInfoResponse=} [properties] Properties to set
         * @returns {pruntime_rpc.GetContractInfoResponse} GetContractInfoResponse instance
         */
        GetContractInfoResponse.create = function create(properties) {
            return new GetContractInfoResponse(properties);
        };

        /**
         * Encodes the specified GetContractInfoResponse message. Does not implicitly {@link pruntime_rpc.GetContractInfoResponse.verify|verify} messages.
         * @function encode
         * @memberof pruntime_rpc.GetContractInfoResponse
         * @static
         * @param {pruntime_rpc.IGetContractInfoResponse} message GetContractInfoResponse message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        GetContractInfoResponse.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.contracts != null && message.contracts.length)
                for (var i = 0; i < message.contracts.length; ++i)
                    $root.pruntime_rpc.ContractInfo.encode(message.contracts[i], writer.uint32(/* id 1, wireType 2 =*/10).fork()).ldelim();
            return writer;
        };

        /**
         * Encodes the specified GetContractInfoResponse message, length delimited. Does not implicitly {@link pruntime_rpc.GetContractInfoResponse.verify|verify} messages.
         * @function encodeDelimited
         * @memberof pruntime_rpc.GetContractInfoResponse
         * @static
         * @param {pruntime_rpc.IGetContractInfoResponse} message GetContractInfoResponse message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        GetContractInfoResponse.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes a GetContractInfoResponse message from the specified reader or buffer.
         * @function decode
         * @memberof pruntime_rpc.GetContractInfoResponse
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {pruntime_rpc.GetContractInfoResponse} GetContractInfoResponse
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        GetContractInfoResponse.decode = function decode(reader, length) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            var end = length === undefined ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.GetContractInfoResponse();
            while (reader.pos < end) {
                var tag = reader.uint32();
                switch (tag >>> 3) {
                case 1:
                    if (!(message.contracts && message.contracts.length))
                        message.contracts = [];
                    message.contracts.push($root.pruntime_rpc.ContractInfo.decode(reader, reader.uint32()));
                    break;
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a GetContractInfoResponse message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof pruntime_rpc.GetContractInfoResponse
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {pruntime_rpc.GetContractInfoResponse} GetContractInfoResponse
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        GetContractInfoResponse.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies a GetContractInfoResponse message.
         * @function verify
         * @memberof pruntime_rpc.GetContractInfoResponse
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        GetContractInfoResponse.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            if (message.contracts != null && message.hasOwnProperty("contracts")) {
                if (!Array.isArray(message.contracts))
                    return "contracts: array expected";
                for (var i = 0; i < message.contracts.length; ++i) {
                    var error = $root.pruntime_rpc.ContractInfo.verify(message.contracts[i]);
                    if (error)
                        return "contracts." + error;
                }
            }
            return null;
        };

        /**
         * Creates a GetContractInfoResponse message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof pruntime_rpc.GetContractInfoResponse
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {pruntime_rpc.GetContractInfoResponse} GetContractInfoResponse
         */
        GetContractInfoResponse.fromObject = function fromObject(object) {
            if (object instanceof $root.pruntime_rpc.GetContractInfoResponse)
                return object;
            var message = new $root.pruntime_rpc.GetContractInfoResponse();
            if (object.contracts) {
                if (!Array.isArray(object.contracts))
                    throw TypeError(".pruntime_rpc.GetContractInfoResponse.contracts: array expected");
                message.contracts = [];
                for (var i = 0; i < object.contracts.length; ++i) {
                    if (typeof object.contracts[i] !== "object")
                        throw TypeError(".pruntime_rpc.GetContractInfoResponse.contracts: object expected");
                    message.contracts[i] = $root.pruntime_rpc.ContractInfo.fromObject(object.contracts[i]);
                }
            }
            return message;
        };

        /**
         * Creates a plain object from a GetContractInfoResponse message. Also converts values to other types if specified.
         * @function toObject
         * @memberof pruntime_rpc.GetContractInfoResponse
         * @static
         * @param {pruntime_rpc.GetContractInfoResponse} message GetContractInfoResponse
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        GetContractInfoResponse.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            var object = {};
            if (options.arrays || options.defaults)
                object.contracts = [];
            if (message.contracts && message.contracts.length) {
                object.contracts = [];
                for (var j = 0; j < message.contracts.length; ++j)
                    object.contracts[j] = $root.pruntime_rpc.ContractInfo.toObject(message.contracts[j], options);
            }
            return object;
        };

        /**
         * Converts this GetContractInfoResponse to JSON.
         * @function toJSON
         * @memberof pruntime_rpc.GetContractInfoResponse
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        GetContractInfoResponse.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        return GetContractInfoResponse;
    })();

    pruntime_rpc.ContractInfo = (function() {

        /**
         * Properties of a ContractInfo.
         * @memberof pruntime_rpc
         * @interface IContractInfo
         * @property {string|null} [id] ContractInfo id
         * @property {string|null} [codeHash] ContractInfo codeHash
         * @property {number|null} [weight] ContractInfo weight
         * @property {pruntime_rpc.ISidevmInfo|null} [sidevm] ContractInfo sidevm
         */

        /**
         * Constructs a new ContractInfo.
         * @memberof pruntime_rpc
         * @classdesc Represents a ContractInfo.
         * @implements IContractInfo
         * @constructor
         * @param {pruntime_rpc.IContractInfo=} [properties] Properties to set
         */
        function ContractInfo(properties) {
            if (properties)
                for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * ContractInfo id.
         * @member {string} id
         * @memberof pruntime_rpc.ContractInfo
         * @instance
         */
        ContractInfo.prototype.id = "";

        /**
         * ContractInfo codeHash.
         * @member {string} codeHash
         * @memberof pruntime_rpc.ContractInfo
         * @instance
         */
        ContractInfo.prototype.codeHash = "";

        /**
         * ContractInfo weight.
         * @member {number} weight
         * @memberof pruntime_rpc.ContractInfo
         * @instance
         */
        ContractInfo.prototype.weight = 0;

        /**
         * ContractInfo sidevm.
         * @member {pruntime_rpc.ISidevmInfo|null|undefined} sidevm
         * @memberof pruntime_rpc.ContractInfo
         * @instance
         */
        ContractInfo.prototype.sidevm = null;

        /**
         * Creates a new ContractInfo instance using the specified properties.
         * @function create
         * @memberof pruntime_rpc.ContractInfo
         * @static
         * @param {pruntime_rpc.IContractInfo=} [properties] Properties to set
         * @returns {pruntime_rpc.ContractInfo} ContractInfo instance
         */
        ContractInfo.create = function create(properties) {
            return new ContractInfo(properties);
        };

        /**
         * Encodes the specified ContractInfo message. Does not implicitly {@link pruntime_rpc.ContractInfo.verify|verify} messages.
         * @function encode
         * @memberof pruntime_rpc.ContractInfo
         * @static
         * @param {pruntime_rpc.IContractInfo} message ContractInfo message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        ContractInfo.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.id != null && Object.hasOwnProperty.call(message, "id"))
                writer.uint32(/* id 1, wireType 2 =*/10).string(message.id);
            if (message.codeHash != null && Object.hasOwnProperty.call(message, "codeHash"))
                writer.uint32(/* id 2, wireType 2 =*/18).string(message.codeHash);
            if (message.weight != null && Object.hasOwnProperty.call(message, "weight"))
                writer.uint32(/* id 3, wireType 0 =*/24).uint32(message.weight);
            if (message.sidevm != null && Object.hasOwnProperty.call(message, "sidevm"))
                $root.pruntime_rpc.SidevmInfo.encode(message.sidevm, writer.uint32(/* id 4, wireType 2 =*/34).fork()).ldelim();
            return writer;
        };

        /**
         * Encodes the specified ContractInfo message, length delimited. Does not implicitly {@link pruntime_rpc.ContractInfo.verify|verify} messages.
         * @function encodeDelimited
         * @memberof pruntime_rpc.ContractInfo
         * @static
         * @param {pruntime_rpc.IContractInfo} message ContractInfo message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        ContractInfo.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes a ContractInfo message from the specified reader or buffer.
         * @function decode
         * @memberof pruntime_rpc.ContractInfo
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {pruntime_rpc.ContractInfo} ContractInfo
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        ContractInfo.decode = function decode(reader, length) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            var end = length === undefined ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.ContractInfo();
            while (reader.pos < end) {
                var tag = reader.uint32();
                switch (tag >>> 3) {
                case 1:
                    message.id = reader.string();
                    break;
                case 2:
                    message.codeHash = reader.string();
                    break;
                case 3:
                    message.weight = reader.uint32();
                    break;
                case 4:
                    message.sidevm = $root.pruntime_rpc.SidevmInfo.decode(reader, reader.uint32());
                    break;
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a ContractInfo message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof pruntime_rpc.ContractInfo
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {pruntime_rpc.ContractInfo} ContractInfo
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        ContractInfo.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies a ContractInfo message.
         * @function verify
         * @memberof pruntime_rpc.ContractInfo
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        ContractInfo.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            if (message.id != null && message.hasOwnProperty("id"))
                if (!$util.isString(message.id))
                    return "id: string expected";
            if (message.codeHash != null && message.hasOwnProperty("codeHash"))
                if (!$util.isString(message.codeHash))
                    return "codeHash: string expected";
            if (message.weight != null && message.hasOwnProperty("weight"))
                if (!$util.isInteger(message.weight))
                    return "weight: integer expected";
            if (message.sidevm != null && message.hasOwnProperty("sidevm")) {
                var error = $root.pruntime_rpc.SidevmInfo.verify(message.sidevm);
                if (error)
                    return "sidevm." + error;
            }
            return null;
        };

        /**
         * Creates a ContractInfo message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof pruntime_rpc.ContractInfo
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {pruntime_rpc.ContractInfo} ContractInfo
         */
        ContractInfo.fromObject = function fromObject(object) {
            if (object instanceof $root.pruntime_rpc.ContractInfo)
                return object;
            var message = new $root.pruntime_rpc.ContractInfo();
            if (object.id != null)
                message.id = String(object.id);
            if (object.codeHash != null)
                message.codeHash = String(object.codeHash);
            if (object.weight != null)
                message.weight = object.weight >>> 0;
            if (object.sidevm != null) {
                if (typeof object.sidevm !== "object")
                    throw TypeError(".pruntime_rpc.ContractInfo.sidevm: object expected");
                message.sidevm = $root.pruntime_rpc.SidevmInfo.fromObject(object.sidevm);
            }
            return message;
        };

        /**
         * Creates a plain object from a ContractInfo message. Also converts values to other types if specified.
         * @function toObject
         * @memberof pruntime_rpc.ContractInfo
         * @static
         * @param {pruntime_rpc.ContractInfo} message ContractInfo
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        ContractInfo.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            var object = {};
            if (options.defaults) {
                object.id = "";
                object.codeHash = "";
                object.weight = 0;
                object.sidevm = null;
            }
            if (message.id != null && message.hasOwnProperty("id"))
                object.id = message.id;
            if (message.codeHash != null && message.hasOwnProperty("codeHash"))
                object.codeHash = message.codeHash;
            if (message.weight != null && message.hasOwnProperty("weight"))
                object.weight = message.weight;
            if (message.sidevm != null && message.hasOwnProperty("sidevm"))
                object.sidevm = $root.pruntime_rpc.SidevmInfo.toObject(message.sidevm, options);
            return object;
        };

        /**
         * Converts this ContractInfo to JSON.
         * @function toJSON
         * @memberof pruntime_rpc.ContractInfo
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        ContractInfo.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        return ContractInfo;
    })();

    pruntime_rpc.SidevmInfo = (function() {

        /**
         * Properties of a SidevmInfo.
         * @memberof pruntime_rpc
         * @interface ISidevmInfo
         * @property {string|null} [state] SidevmInfo state
         * @property {string|null} [codeHash] SidevmInfo codeHash
         * @property {string|null} [startTime] SidevmInfo startTime
         * @property {string|null} [stopReason] SidevmInfo stopReason
         */

        /**
         * Constructs a new SidevmInfo.
         * @memberof pruntime_rpc
         * @classdesc Represents a SidevmInfo.
         * @implements ISidevmInfo
         * @constructor
         * @param {pruntime_rpc.ISidevmInfo=} [properties] Properties to set
         */
        function SidevmInfo(properties) {
            if (properties)
                for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * SidevmInfo state.
         * @member {string} state
         * @memberof pruntime_rpc.SidevmInfo
         * @instance
         */
        SidevmInfo.prototype.state = "";

        /**
         * SidevmInfo codeHash.
         * @member {string} codeHash
         * @memberof pruntime_rpc.SidevmInfo
         * @instance
         */
        SidevmInfo.prototype.codeHash = "";

        /**
         * SidevmInfo startTime.
         * @member {string} startTime
         * @memberof pruntime_rpc.SidevmInfo
         * @instance
         */
        SidevmInfo.prototype.startTime = "";

        /**
         * SidevmInfo stopReason.
         * @member {string} stopReason
         * @memberof pruntime_rpc.SidevmInfo
         * @instance
         */
        SidevmInfo.prototype.stopReason = "";

        /**
         * Creates a new SidevmInfo instance using the specified properties.
         * @function create
         * @memberof pruntime_rpc.SidevmInfo
         * @static
         * @param {pruntime_rpc.ISidevmInfo=} [properties] Properties to set
         * @returns {pruntime_rpc.SidevmInfo} SidevmInfo instance
         */
        SidevmInfo.create = function create(properties) {
            return new SidevmInfo(properties);
        };

        /**
         * Encodes the specified SidevmInfo message. Does not implicitly {@link pruntime_rpc.SidevmInfo.verify|verify} messages.
         * @function encode
         * @memberof pruntime_rpc.SidevmInfo
         * @static
         * @param {pruntime_rpc.ISidevmInfo} message SidevmInfo message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        SidevmInfo.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.state != null && Object.hasOwnProperty.call(message, "state"))
                writer.uint32(/* id 1, wireType 2 =*/10).string(message.state);
            if (message.codeHash != null && Object.hasOwnProperty.call(message, "codeHash"))
                writer.uint32(/* id 2, wireType 2 =*/18).string(message.codeHash);
            if (message.startTime != null && Object.hasOwnProperty.call(message, "startTime"))
                writer.uint32(/* id 3, wireType 2 =*/26).string(message.startTime);
            if (message.stopReason != null && Object.hasOwnProperty.call(message, "stopReason"))
                writer.uint32(/* id 4, wireType 2 =*/34).string(message.stopReason);
            return writer;
        };

        /**
         * Encodes the specified SidevmInfo message, length delimited. Does not implicitly {@link pruntime_rpc.SidevmInfo.verify|verify} messages.
         * @function encodeDelimited
         * @memberof pruntime_rpc.SidevmInfo
         * @static
         * @param {pruntime_rpc.ISidevmInfo} message SidevmInfo message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        SidevmInfo.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes a SidevmInfo message from the specified reader or buffer.
         * @function decode
         * @memberof pruntime_rpc.SidevmInfo
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {pruntime_rpc.SidevmInfo} SidevmInfo
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        SidevmInfo.decode = function decode(reader, length) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            var end = length === undefined ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.SidevmInfo();
            while (reader.pos < end) {
                var tag = reader.uint32();
                switch (tag >>> 3) {
                case 1:
                    message.state = reader.string();
                    break;
                case 2:
                    message.codeHash = reader.string();
                    break;
                case 3:
                    message.startTime = reader.string();
                    break;
                case 4:
                    message.stopReason = reader.string();
                    break;
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a SidevmInfo message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof pruntime_rpc.SidevmInfo
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {pruntime_rpc.SidevmInfo} SidevmInfo
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        SidevmInfo.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies a SidevmInfo message.
         * @function verify
         * @memberof pruntime_rpc.SidevmInfo
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        SidevmInfo.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            if (message.state != null && message.hasOwnProperty("state"))
                if (!$util.isString(message.state))
                    return "state: string expected";
            if (message.codeHash != null && message.hasOwnProperty("codeHash"))
                if (!$util.isString(message.codeHash))
                    return "codeHash: string expected";
            if (message.startTime != null && message.hasOwnProperty("startTime"))
                if (!$util.isString(message.startTime))
                    return "startTime: string expected";
            if (message.stopReason != null && message.hasOwnProperty("stopReason"))
                if (!$util.isString(message.stopReason))
                    return "stopReason: string expected";
            return null;
        };

        /**
         * Creates a SidevmInfo message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof pruntime_rpc.SidevmInfo
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {pruntime_rpc.SidevmInfo} SidevmInfo
         */
        SidevmInfo.fromObject = function fromObject(object) {
            if (object instanceof $root.pruntime_rpc.SidevmInfo)
                return object;
            var message = new $root.pruntime_rpc.SidevmInfo();
            if (object.state != null)
                message.state = String(object.state);
            if (object.codeHash != null)
                message.codeHash = String(object.codeHash);
            if (object.startTime != null)
                message.startTime = String(object.startTime);
            if (object.stopReason != null)
                message.stopReason = String(object.stopReason);
            return message;
        };

        /**
         * Creates a plain object from a SidevmInfo message. Also converts values to other types if specified.
         * @function toObject
         * @memberof pruntime_rpc.SidevmInfo
         * @static
         * @param {pruntime_rpc.SidevmInfo} message SidevmInfo
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        SidevmInfo.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            var object = {};
            if (options.defaults) {
                object.state = "";
                object.codeHash = "";
                object.startTime = "";
                object.stopReason = "";
            }
            if (message.state != null && message.hasOwnProperty("state"))
                object.state = message.state;
            if (message.codeHash != null && message.hasOwnProperty("codeHash"))
                object.codeHash = message.codeHash;
            if (message.startTime != null && message.hasOwnProperty("startTime"))
                object.startTime = message.startTime;
            if (message.stopReason != null && message.hasOwnProperty("stopReason"))
                object.stopReason = message.stopReason;
            return object;
        };

        /**
         * Converts this SidevmInfo to JSON.
         * @function toJSON
         * @memberof pruntime_rpc.SidevmInfo
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        SidevmInfo.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        return SidevmInfo;
    })();

    pruntime_rpc.GetClusterInfoResponse = (function() {

        /**
         * Properties of a GetClusterInfoResponse.
         * @memberof pruntime_rpc
         * @interface IGetClusterInfoResponse
         * @property {Array.<pruntime_rpc.IClusterInfo>|null} [clusters] GetClusterInfoResponse clusters
         */

        /**
         * Constructs a new GetClusterInfoResponse.
         * @memberof pruntime_rpc
         * @classdesc Represents a GetClusterInfoResponse.
         * @implements IGetClusterInfoResponse
         * @constructor
         * @param {pruntime_rpc.IGetClusterInfoResponse=} [properties] Properties to set
         */
        function GetClusterInfoResponse(properties) {
            this.clusters = [];
            if (properties)
                for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * GetClusterInfoResponse clusters.
         * @member {Array.<pruntime_rpc.IClusterInfo>} clusters
         * @memberof pruntime_rpc.GetClusterInfoResponse
         * @instance
         */
        GetClusterInfoResponse.prototype.clusters = $util.emptyArray;

        /**
         * Creates a new GetClusterInfoResponse instance using the specified properties.
         * @function create
         * @memberof pruntime_rpc.GetClusterInfoResponse
         * @static
         * @param {pruntime_rpc.IGetClusterInfoResponse=} [properties] Properties to set
         * @returns {pruntime_rpc.GetClusterInfoResponse} GetClusterInfoResponse instance
         */
        GetClusterInfoResponse.create = function create(properties) {
            return new GetClusterInfoResponse(properties);
        };

        /**
         * Encodes the specified GetClusterInfoResponse message. Does not implicitly {@link pruntime_rpc.GetClusterInfoResponse.verify|verify} messages.
         * @function encode
         * @memberof pruntime_rpc.GetClusterInfoResponse
         * @static
         * @param {pruntime_rpc.IGetClusterInfoResponse} message GetClusterInfoResponse message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        GetClusterInfoResponse.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.clusters != null && message.clusters.length)
                for (var i = 0; i < message.clusters.length; ++i)
                    $root.pruntime_rpc.ClusterInfo.encode(message.clusters[i], writer.uint32(/* id 1, wireType 2 =*/10).fork()).ldelim();
            return writer;
        };

        /**
         * Encodes the specified GetClusterInfoResponse message, length delimited. Does not implicitly {@link pruntime_rpc.GetClusterInfoResponse.verify|verify} messages.
         * @function encodeDelimited
         * @memberof pruntime_rpc.GetClusterInfoResponse
         * @static
         * @param {pruntime_rpc.IGetClusterInfoResponse} message GetClusterInfoResponse message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        GetClusterInfoResponse.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes a GetClusterInfoResponse message from the specified reader or buffer.
         * @function decode
         * @memberof pruntime_rpc.GetClusterInfoResponse
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {pruntime_rpc.GetClusterInfoResponse} GetClusterInfoResponse
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        GetClusterInfoResponse.decode = function decode(reader, length) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            var end = length === undefined ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.GetClusterInfoResponse();
            while (reader.pos < end) {
                var tag = reader.uint32();
                switch (tag >>> 3) {
                case 1:
                    if (!(message.clusters && message.clusters.length))
                        message.clusters = [];
                    message.clusters.push($root.pruntime_rpc.ClusterInfo.decode(reader, reader.uint32()));
                    break;
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a GetClusterInfoResponse message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof pruntime_rpc.GetClusterInfoResponse
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {pruntime_rpc.GetClusterInfoResponse} GetClusterInfoResponse
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        GetClusterInfoResponse.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies a GetClusterInfoResponse message.
         * @function verify
         * @memberof pruntime_rpc.GetClusterInfoResponse
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        GetClusterInfoResponse.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            if (message.clusters != null && message.hasOwnProperty("clusters")) {
                if (!Array.isArray(message.clusters))
                    return "clusters: array expected";
                for (var i = 0; i < message.clusters.length; ++i) {
                    var error = $root.pruntime_rpc.ClusterInfo.verify(message.clusters[i]);
                    if (error)
                        return "clusters." + error;
                }
            }
            return null;
        };

        /**
         * Creates a GetClusterInfoResponse message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof pruntime_rpc.GetClusterInfoResponse
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {pruntime_rpc.GetClusterInfoResponse} GetClusterInfoResponse
         */
        GetClusterInfoResponse.fromObject = function fromObject(object) {
            if (object instanceof $root.pruntime_rpc.GetClusterInfoResponse)
                return object;
            var message = new $root.pruntime_rpc.GetClusterInfoResponse();
            if (object.clusters) {
                if (!Array.isArray(object.clusters))
                    throw TypeError(".pruntime_rpc.GetClusterInfoResponse.clusters: array expected");
                message.clusters = [];
                for (var i = 0; i < object.clusters.length; ++i) {
                    if (typeof object.clusters[i] !== "object")
                        throw TypeError(".pruntime_rpc.GetClusterInfoResponse.clusters: object expected");
                    message.clusters[i] = $root.pruntime_rpc.ClusterInfo.fromObject(object.clusters[i]);
                }
            }
            return message;
        };

        /**
         * Creates a plain object from a GetClusterInfoResponse message. Also converts values to other types if specified.
         * @function toObject
         * @memberof pruntime_rpc.GetClusterInfoResponse
         * @static
         * @param {pruntime_rpc.GetClusterInfoResponse} message GetClusterInfoResponse
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        GetClusterInfoResponse.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            var object = {};
            if (options.arrays || options.defaults)
                object.clusters = [];
            if (message.clusters && message.clusters.length) {
                object.clusters = [];
                for (var j = 0; j < message.clusters.length; ++j)
                    object.clusters[j] = $root.pruntime_rpc.ClusterInfo.toObject(message.clusters[j], options);
            }
            return object;
        };

        /**
         * Converts this GetClusterInfoResponse to JSON.
         * @function toJSON
         * @memberof pruntime_rpc.GetClusterInfoResponse
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        GetClusterInfoResponse.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        return GetClusterInfoResponse;
    })();

    pruntime_rpc.ClusterInfo = (function() {

        /**
         * Properties of a ClusterInfo.
         * @memberof pruntime_rpc
         * @interface IClusterInfo
         * @property {string|null} [id] ClusterInfo id
         * @property {string|null} [version] ClusterInfo version
         * @property {string|null} [stateRoot] ClusterInfo stateRoot
         * @property {Array.<string>|null} [contracts] ClusterInfo contracts
         */

        /**
         * Constructs a new ClusterInfo.
         * @memberof pruntime_rpc
         * @classdesc Represents a ClusterInfo.
         * @implements IClusterInfo
         * @constructor
         * @param {pruntime_rpc.IClusterInfo=} [properties] Properties to set
         */
        function ClusterInfo(properties) {
            this.contracts = [];
            if (properties)
                for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * ClusterInfo id.
         * @member {string} id
         * @memberof pruntime_rpc.ClusterInfo
         * @instance
         */
        ClusterInfo.prototype.id = "";

        /**
         * ClusterInfo version.
         * @member {string} version
         * @memberof pruntime_rpc.ClusterInfo
         * @instance
         */
        ClusterInfo.prototype.version = "";

        /**
         * ClusterInfo stateRoot.
         * @member {string} stateRoot
         * @memberof pruntime_rpc.ClusterInfo
         * @instance
         */
        ClusterInfo.prototype.stateRoot = "";

        /**
         * ClusterInfo contracts.
         * @member {Array.<string>} contracts
         * @memberof pruntime_rpc.ClusterInfo
         * @instance
         */
        ClusterInfo.prototype.contracts = $util.emptyArray;

        /**
         * Creates a new ClusterInfo instance using the specified properties.
         * @function create
         * @memberof pruntime_rpc.ClusterInfo
         * @static
         * @param {pruntime_rpc.IClusterInfo=} [properties] Properties to set
         * @returns {pruntime_rpc.ClusterInfo} ClusterInfo instance
         */
        ClusterInfo.create = function create(properties) {
            return new ClusterInfo(properties);
        };

        /**
         * Encodes the specified ClusterInfo message. Does not implicitly {@link pruntime_rpc.ClusterInfo.verify|verify} messages.
         * @function encode
         * @memberof pruntime_rpc.ClusterInfo
         * @static
         * @param {pruntime_rpc.IClusterInfo} message ClusterInfo message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        ClusterInfo.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.id != null && Object.hasOwnProperty.call(message, "id"))
                writer.uint32(/* id 1, wireType 2 =*/10).string(message.id);
            if (message.version != null && Object.hasOwnProperty.call(message, "version"))
                writer.uint32(/* id 2, wireType 2 =*/18).string(message.version);
            if (message.stateRoot != null && Object.hasOwnProperty.call(message, "stateRoot"))
                writer.uint32(/* id 3, wireType 2 =*/26).string(message.stateRoot);
            if (message.contracts != null && message.contracts.length)
                for (var i = 0; i < message.contracts.length; ++i)
                    writer.uint32(/* id 4, wireType 2 =*/34).string(message.contracts[i]);
            return writer;
        };

        /**
         * Encodes the specified ClusterInfo message, length delimited. Does not implicitly {@link pruntime_rpc.ClusterInfo.verify|verify} messages.
         * @function encodeDelimited
         * @memberof pruntime_rpc.ClusterInfo
         * @static
         * @param {pruntime_rpc.IClusterInfo} message ClusterInfo message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        ClusterInfo.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes a ClusterInfo message from the specified reader or buffer.
         * @function decode
         * @memberof pruntime_rpc.ClusterInfo
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {pruntime_rpc.ClusterInfo} ClusterInfo
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        ClusterInfo.decode = function decode(reader, length) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            var end = length === undefined ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.ClusterInfo();
            while (reader.pos < end) {
                var tag = reader.uint32();
                switch (tag >>> 3) {
                case 1:
                    message.id = reader.string();
                    break;
                case 2:
                    message.version = reader.string();
                    break;
                case 3:
                    message.stateRoot = reader.string();
                    break;
                case 4:
                    if (!(message.contracts && message.contracts.length))
                        message.contracts = [];
                    message.contracts.push(reader.string());
                    break;
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a ClusterInfo message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof pruntime_rpc.ClusterInfo
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {pruntime_rpc.ClusterInfo} ClusterInfo
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        ClusterInfo.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies a ClusterInfo message.
         * @function verify
         * @memberof pruntime_rpc.ClusterInfo
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        ClusterInfo.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            if (message.id != null && message.hasOwnProperty("id"))
                if (!$util.isString(message.id))
                    return "id: string expected";
            if (message.version != null && message.hasOwnProperty("version"))
                if (!$util.isString(message.version))
                    return "version: string expected";
            if (message.stateRoot != null && message.hasOwnProperty("stateRoot"))
                if (!$util.isString(message.stateRoot))
                    return "stateRoot: string expected";
            if (message.contracts != null && message.hasOwnProperty("contracts")) {
                if (!Array.isArray(message.contracts))
                    return "contracts: array expected";
                for (var i = 0; i < message.contracts.length; ++i)
                    if (!$util.isString(message.contracts[i]))
                        return "contracts: string[] expected";
            }
            return null;
        };

        /**
         * Creates a ClusterInfo message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof pruntime_rpc.ClusterInfo
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {pruntime_rpc.ClusterInfo} ClusterInfo
         */
        ClusterInfo.fromObject = function fromObject(object) {
            if (object instanceof $root.pruntime_rpc.ClusterInfo)
                return object;
            var message = new $root.pruntime_rpc.ClusterInfo();
            if (object.id != null)
                message.id = String(object.id);
            if (object.version != null)
                message.version = String(object.version);
            if (object.stateRoot != null)
                message.stateRoot = String(object.stateRoot);
            if (object.contracts) {
                if (!Array.isArray(object.contracts))
                    throw TypeError(".pruntime_rpc.ClusterInfo.contracts: array expected");
                message.contracts = [];
                for (var i = 0; i < object.contracts.length; ++i)
                    message.contracts[i] = String(object.contracts[i]);
            }
            return message;
        };

        /**
         * Creates a plain object from a ClusterInfo message. Also converts values to other types if specified.
         * @function toObject
         * @memberof pruntime_rpc.ClusterInfo
         * @static
         * @param {pruntime_rpc.ClusterInfo} message ClusterInfo
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        ClusterInfo.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            var object = {};
            if (options.arrays || options.defaults)
                object.contracts = [];
            if (options.defaults) {
                object.id = "";
                object.version = "";
                object.stateRoot = "";
            }
            if (message.id != null && message.hasOwnProperty("id"))
                object.id = message.id;
            if (message.version != null && message.hasOwnProperty("version"))
                object.version = message.version;
            if (message.stateRoot != null && message.hasOwnProperty("stateRoot"))
                object.stateRoot = message.stateRoot;
            if (message.contracts && message.contracts.length) {
                object.contracts = [];
                for (var j = 0; j < message.contracts.length; ++j)
                    object.contracts[j] = message.contracts[j];
            }
            return object;
        };

        /**
         * Converts this ClusterInfo to JSON.
         * @function toJSON
         * @memberof pruntime_rpc.ClusterInfo
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        ClusterInfo.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        return ClusterInfo;
    })();

    pruntime_rpc.SidevmCode = (function() {

        /**
         * Properties of a SidevmCode.
         * @memberof pruntime_rpc
         * @interface ISidevmCode
         * @property {Uint8Array|null} [contract] SidevmCode contract
         * @property {Uint8Array|null} [code] SidevmCode code
         */

        /**
         * Constructs a new SidevmCode.
         * @memberof pruntime_rpc
         * @classdesc Represents a SidevmCode.
         * @implements ISidevmCode
         * @constructor
         * @param {pruntime_rpc.ISidevmCode=} [properties] Properties to set
         */
        function SidevmCode(properties) {
            if (properties)
                for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * SidevmCode contract.
         * @member {Uint8Array} contract
         * @memberof pruntime_rpc.SidevmCode
         * @instance
         */
        SidevmCode.prototype.contract = $util.newBuffer([]);

        /**
         * SidevmCode code.
         * @member {Uint8Array} code
         * @memberof pruntime_rpc.SidevmCode
         * @instance
         */
        SidevmCode.prototype.code = $util.newBuffer([]);

        /**
         * Creates a new SidevmCode instance using the specified properties.
         * @function create
         * @memberof pruntime_rpc.SidevmCode
         * @static
         * @param {pruntime_rpc.ISidevmCode=} [properties] Properties to set
         * @returns {pruntime_rpc.SidevmCode} SidevmCode instance
         */
        SidevmCode.create = function create(properties) {
            return new SidevmCode(properties);
        };

        /**
         * Encodes the specified SidevmCode message. Does not implicitly {@link pruntime_rpc.SidevmCode.verify|verify} messages.
         * @function encode
         * @memberof pruntime_rpc.SidevmCode
         * @static
         * @param {pruntime_rpc.ISidevmCode} message SidevmCode message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        SidevmCode.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.contract != null && Object.hasOwnProperty.call(message, "contract"))
                writer.uint32(/* id 1, wireType 2 =*/10).bytes(message.contract);
            if (message.code != null && Object.hasOwnProperty.call(message, "code"))
                writer.uint32(/* id 2, wireType 2 =*/18).bytes(message.code);
            return writer;
        };

        /**
         * Encodes the specified SidevmCode message, length delimited. Does not implicitly {@link pruntime_rpc.SidevmCode.verify|verify} messages.
         * @function encodeDelimited
         * @memberof pruntime_rpc.SidevmCode
         * @static
         * @param {pruntime_rpc.ISidevmCode} message SidevmCode message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        SidevmCode.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes a SidevmCode message from the specified reader or buffer.
         * @function decode
         * @memberof pruntime_rpc.SidevmCode
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {pruntime_rpc.SidevmCode} SidevmCode
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        SidevmCode.decode = function decode(reader, length) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            var end = length === undefined ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.SidevmCode();
            while (reader.pos < end) {
                var tag = reader.uint32();
                switch (tag >>> 3) {
                case 1:
                    message.contract = reader.bytes();
                    break;
                case 2:
                    message.code = reader.bytes();
                    break;
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a SidevmCode message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof pruntime_rpc.SidevmCode
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {pruntime_rpc.SidevmCode} SidevmCode
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        SidevmCode.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies a SidevmCode message.
         * @function verify
         * @memberof pruntime_rpc.SidevmCode
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        SidevmCode.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            if (message.contract != null && message.hasOwnProperty("contract"))
                if (!(message.contract && typeof message.contract.length === "number" || $util.isString(message.contract)))
                    return "contract: buffer expected";
            if (message.code != null && message.hasOwnProperty("code"))
                if (!(message.code && typeof message.code.length === "number" || $util.isString(message.code)))
                    return "code: buffer expected";
            return null;
        };

        /**
         * Creates a SidevmCode message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof pruntime_rpc.SidevmCode
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {pruntime_rpc.SidevmCode} SidevmCode
         */
        SidevmCode.fromObject = function fromObject(object) {
            if (object instanceof $root.pruntime_rpc.SidevmCode)
                return object;
            var message = new $root.pruntime_rpc.SidevmCode();
            if (object.contract != null)
                if (typeof object.contract === "string")
                    $util.base64.decode(object.contract, message.contract = $util.newBuffer($util.base64.length(object.contract)), 0);
                else if (object.contract.length)
                    message.contract = object.contract;
            if (object.code != null)
                if (typeof object.code === "string")
                    $util.base64.decode(object.code, message.code = $util.newBuffer($util.base64.length(object.code)), 0);
                else if (object.code.length)
                    message.code = object.code;
            return message;
        };

        /**
         * Creates a plain object from a SidevmCode message. Also converts values to other types if specified.
         * @function toObject
         * @memberof pruntime_rpc.SidevmCode
         * @static
         * @param {pruntime_rpc.SidevmCode} message SidevmCode
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        SidevmCode.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            var object = {};
            if (options.defaults) {
                if (options.bytes === String)
                    object.contract = "";
                else {
                    object.contract = [];
                    if (options.bytes !== Array)
                        object.contract = $util.newBuffer(object.contract);
                }
                if (options.bytes === String)
                    object.code = "";
                else {
                    object.code = [];
                    if (options.bytes !== Array)
                        object.code = $util.newBuffer(object.code);
                }
            }
            if (message.contract != null && message.hasOwnProperty("contract"))
                object.contract = options.bytes === String ? $util.base64.encode(message.contract, 0, message.contract.length) : options.bytes === Array ? Array.prototype.slice.call(message.contract) : message.contract;
            if (message.code != null && message.hasOwnProperty("code"))
                object.code = options.bytes === String ? $util.base64.encode(message.code, 0, message.code.length) : options.bytes === Array ? Array.prototype.slice.call(message.code) : message.code;
            return object;
        };

        /**
         * Converts this SidevmCode to JSON.
         * @function toJSON
         * @memberof pruntime_rpc.SidevmCode
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        SidevmCode.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        return SidevmCode;
    })();

    pruntime_rpc.ContractParameters = (function() {

        /**
         * Properties of a ContractParameters.
         * @memberof pruntime_rpc
         * @interface IContractParameters
         * @property {string|null} [deployer] ContractParameters deployer
         * @property {string|null} [clusterId] ContractParameters clusterId
         * @property {string|null} [codeHash] ContractParameters codeHash
         * @property {string|null} [salt] ContractParameters salt
         */

        /**
         * Constructs a new ContractParameters.
         * @memberof pruntime_rpc
         * @classdesc Represents a ContractParameters.
         * @implements IContractParameters
         * @constructor
         * @param {pruntime_rpc.IContractParameters=} [properties] Properties to set
         */
        function ContractParameters(properties) {
            if (properties)
                for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * ContractParameters deployer.
         * @member {string} deployer
         * @memberof pruntime_rpc.ContractParameters
         * @instance
         */
        ContractParameters.prototype.deployer = "";

        /**
         * ContractParameters clusterId.
         * @member {string} clusterId
         * @memberof pruntime_rpc.ContractParameters
         * @instance
         */
        ContractParameters.prototype.clusterId = "";

        /**
         * ContractParameters codeHash.
         * @member {string} codeHash
         * @memberof pruntime_rpc.ContractParameters
         * @instance
         */
        ContractParameters.prototype.codeHash = "";

        /**
         * ContractParameters salt.
         * @member {string} salt
         * @memberof pruntime_rpc.ContractParameters
         * @instance
         */
        ContractParameters.prototype.salt = "";

        /**
         * Creates a new ContractParameters instance using the specified properties.
         * @function create
         * @memberof pruntime_rpc.ContractParameters
         * @static
         * @param {pruntime_rpc.IContractParameters=} [properties] Properties to set
         * @returns {pruntime_rpc.ContractParameters} ContractParameters instance
         */
        ContractParameters.create = function create(properties) {
            return new ContractParameters(properties);
        };

        /**
         * Encodes the specified ContractParameters message. Does not implicitly {@link pruntime_rpc.ContractParameters.verify|verify} messages.
         * @function encode
         * @memberof pruntime_rpc.ContractParameters
         * @static
         * @param {pruntime_rpc.IContractParameters} message ContractParameters message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        ContractParameters.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.deployer != null && Object.hasOwnProperty.call(message, "deployer"))
                writer.uint32(/* id 1, wireType 2 =*/10).string(message.deployer);
            if (message.clusterId != null && Object.hasOwnProperty.call(message, "clusterId"))
                writer.uint32(/* id 2, wireType 2 =*/18).string(message.clusterId);
            if (message.codeHash != null && Object.hasOwnProperty.call(message, "codeHash"))
                writer.uint32(/* id 3, wireType 2 =*/26).string(message.codeHash);
            if (message.salt != null && Object.hasOwnProperty.call(message, "salt"))
                writer.uint32(/* id 4, wireType 2 =*/34).string(message.salt);
            return writer;
        };

        /**
         * Encodes the specified ContractParameters message, length delimited. Does not implicitly {@link pruntime_rpc.ContractParameters.verify|verify} messages.
         * @function encodeDelimited
         * @memberof pruntime_rpc.ContractParameters
         * @static
         * @param {pruntime_rpc.IContractParameters} message ContractParameters message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        ContractParameters.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes a ContractParameters message from the specified reader or buffer.
         * @function decode
         * @memberof pruntime_rpc.ContractParameters
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {pruntime_rpc.ContractParameters} ContractParameters
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        ContractParameters.decode = function decode(reader, length) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            var end = length === undefined ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.ContractParameters();
            while (reader.pos < end) {
                var tag = reader.uint32();
                switch (tag >>> 3) {
                case 1:
                    message.deployer = reader.string();
                    break;
                case 2:
                    message.clusterId = reader.string();
                    break;
                case 3:
                    message.codeHash = reader.string();
                    break;
                case 4:
                    message.salt = reader.string();
                    break;
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a ContractParameters message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof pruntime_rpc.ContractParameters
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {pruntime_rpc.ContractParameters} ContractParameters
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        ContractParameters.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies a ContractParameters message.
         * @function verify
         * @memberof pruntime_rpc.ContractParameters
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        ContractParameters.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            if (message.deployer != null && message.hasOwnProperty("deployer"))
                if (!$util.isString(message.deployer))
                    return "deployer: string expected";
            if (message.clusterId != null && message.hasOwnProperty("clusterId"))
                if (!$util.isString(message.clusterId))
                    return "clusterId: string expected";
            if (message.codeHash != null && message.hasOwnProperty("codeHash"))
                if (!$util.isString(message.codeHash))
                    return "codeHash: string expected";
            if (message.salt != null && message.hasOwnProperty("salt"))
                if (!$util.isString(message.salt))
                    return "salt: string expected";
            return null;
        };

        /**
         * Creates a ContractParameters message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof pruntime_rpc.ContractParameters
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {pruntime_rpc.ContractParameters} ContractParameters
         */
        ContractParameters.fromObject = function fromObject(object) {
            if (object instanceof $root.pruntime_rpc.ContractParameters)
                return object;
            var message = new $root.pruntime_rpc.ContractParameters();
            if (object.deployer != null)
                message.deployer = String(object.deployer);
            if (object.clusterId != null)
                message.clusterId = String(object.clusterId);
            if (object.codeHash != null)
                message.codeHash = String(object.codeHash);
            if (object.salt != null)
                message.salt = String(object.salt);
            return message;
        };

        /**
         * Creates a plain object from a ContractParameters message. Also converts values to other types if specified.
         * @function toObject
         * @memberof pruntime_rpc.ContractParameters
         * @static
         * @param {pruntime_rpc.ContractParameters} message ContractParameters
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        ContractParameters.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            var object = {};
            if (options.defaults) {
                object.deployer = "";
                object.clusterId = "";
                object.codeHash = "";
                object.salt = "";
            }
            if (message.deployer != null && message.hasOwnProperty("deployer"))
                object.deployer = message.deployer;
            if (message.clusterId != null && message.hasOwnProperty("clusterId"))
                object.clusterId = message.clusterId;
            if (message.codeHash != null && message.hasOwnProperty("codeHash"))
                object.codeHash = message.codeHash;
            if (message.salt != null && message.hasOwnProperty("salt"))
                object.salt = message.salt;
            return object;
        };

        /**
         * Converts this ContractParameters to JSON.
         * @function toJSON
         * @memberof pruntime_rpc.ContractParameters
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        ContractParameters.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        return ContractParameters;
    })();

    pruntime_rpc.ContractId = (function() {

        /**
         * Properties of a ContractId.
         * @memberof pruntime_rpc
         * @interface IContractId
         * @property {string|null} [id] ContractId id
         */

        /**
         * Constructs a new ContractId.
         * @memberof pruntime_rpc
         * @classdesc Represents a ContractId.
         * @implements IContractId
         * @constructor
         * @param {pruntime_rpc.IContractId=} [properties] Properties to set
         */
        function ContractId(properties) {
            if (properties)
                for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * ContractId id.
         * @member {string} id
         * @memberof pruntime_rpc.ContractId
         * @instance
         */
        ContractId.prototype.id = "";

        /**
         * Creates a new ContractId instance using the specified properties.
         * @function create
         * @memberof pruntime_rpc.ContractId
         * @static
         * @param {pruntime_rpc.IContractId=} [properties] Properties to set
         * @returns {pruntime_rpc.ContractId} ContractId instance
         */
        ContractId.create = function create(properties) {
            return new ContractId(properties);
        };

        /**
         * Encodes the specified ContractId message. Does not implicitly {@link pruntime_rpc.ContractId.verify|verify} messages.
         * @function encode
         * @memberof pruntime_rpc.ContractId
         * @static
         * @param {pruntime_rpc.IContractId} message ContractId message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        ContractId.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.id != null && Object.hasOwnProperty.call(message, "id"))
                writer.uint32(/* id 1, wireType 2 =*/10).string(message.id);
            return writer;
        };

        /**
         * Encodes the specified ContractId message, length delimited. Does not implicitly {@link pruntime_rpc.ContractId.verify|verify} messages.
         * @function encodeDelimited
         * @memberof pruntime_rpc.ContractId
         * @static
         * @param {pruntime_rpc.IContractId} message ContractId message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        ContractId.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes a ContractId message from the specified reader or buffer.
         * @function decode
         * @memberof pruntime_rpc.ContractId
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {pruntime_rpc.ContractId} ContractId
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        ContractId.decode = function decode(reader, length) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            var end = length === undefined ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.ContractId();
            while (reader.pos < end) {
                var tag = reader.uint32();
                switch (tag >>> 3) {
                case 1:
                    message.id = reader.string();
                    break;
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a ContractId message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof pruntime_rpc.ContractId
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {pruntime_rpc.ContractId} ContractId
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        ContractId.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies a ContractId message.
         * @function verify
         * @memberof pruntime_rpc.ContractId
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        ContractId.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            if (message.id != null && message.hasOwnProperty("id"))
                if (!$util.isString(message.id))
                    return "id: string expected";
            return null;
        };

        /**
         * Creates a ContractId message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof pruntime_rpc.ContractId
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {pruntime_rpc.ContractId} ContractId
         */
        ContractId.fromObject = function fromObject(object) {
            if (object instanceof $root.pruntime_rpc.ContractId)
                return object;
            var message = new $root.pruntime_rpc.ContractId();
            if (object.id != null)
                message.id = String(object.id);
            return message;
        };

        /**
         * Creates a plain object from a ContractId message. Also converts values to other types if specified.
         * @function toObject
         * @memberof pruntime_rpc.ContractId
         * @static
         * @param {pruntime_rpc.ContractId} message ContractId
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        ContractId.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            var object = {};
            if (options.defaults)
                object.id = "";
            if (message.id != null && message.hasOwnProperty("id"))
                object.id = message.id;
            return object;
        };

        /**
         * Converts this ContractId to JSON.
         * @function toJSON
         * @memberof pruntime_rpc.ContractId
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        ContractId.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        return ContractId;
    })();

    return pruntime_rpc;
})();

$root.google = (function() {

    /**
     * Namespace google.
     * @exports google
     * @namespace
     */
    var google = {};

    google.protobuf = (function() {

        /**
         * Namespace protobuf.
         * @memberof google
         * @namespace
         */
        var protobuf = {};

        protobuf.Empty = (function() {

            /**
             * Properties of an Empty.
             * @memberof google.protobuf
             * @interface IEmpty
             */

            /**
             * Constructs a new Empty.
             * @memberof google.protobuf
             * @classdesc Represents an Empty.
             * @implements IEmpty
             * @constructor
             * @param {google.protobuf.IEmpty=} [properties] Properties to set
             */
            function Empty(properties) {
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * Creates a new Empty instance using the specified properties.
             * @function create
             * @memberof google.protobuf.Empty
             * @static
             * @param {google.protobuf.IEmpty=} [properties] Properties to set
             * @returns {google.protobuf.Empty} Empty instance
             */
            Empty.create = function create(properties) {
                return new Empty(properties);
            };

            /**
             * Encodes the specified Empty message. Does not implicitly {@link google.protobuf.Empty.verify|verify} messages.
             * @function encode
             * @memberof google.protobuf.Empty
             * @static
             * @param {google.protobuf.IEmpty} message Empty message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            Empty.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                return writer;
            };

            /**
             * Encodes the specified Empty message, length delimited. Does not implicitly {@link google.protobuf.Empty.verify|verify} messages.
             * @function encodeDelimited
             * @memberof google.protobuf.Empty
             * @static
             * @param {google.protobuf.IEmpty} message Empty message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            Empty.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes an Empty message from the specified reader or buffer.
             * @function decode
             * @memberof google.protobuf.Empty
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {google.protobuf.Empty} Empty
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            Empty.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                var end = length === undefined ? reader.len : reader.pos + length, message = new $root.google.protobuf.Empty();
                while (reader.pos < end) {
                    var tag = reader.uint32();
                    switch (tag >>> 3) {
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes an Empty message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof google.protobuf.Empty
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {google.protobuf.Empty} Empty
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            Empty.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies an Empty message.
             * @function verify
             * @memberof google.protobuf.Empty
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            Empty.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                return null;
            };

            /**
             * Creates an Empty message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof google.protobuf.Empty
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {google.protobuf.Empty} Empty
             */
            Empty.fromObject = function fromObject(object) {
                if (object instanceof $root.google.protobuf.Empty)
                    return object;
                return new $root.google.protobuf.Empty();
            };

            /**
             * Creates a plain object from an Empty message. Also converts values to other types if specified.
             * @function toObject
             * @memberof google.protobuf.Empty
             * @static
             * @param {google.protobuf.Empty} message Empty
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            Empty.toObject = function toObject() {
                return {};
            };

            /**
             * Converts this Empty to JSON.
             * @function toJSON
             * @memberof google.protobuf.Empty
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            Empty.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            return Empty;
        })();

        return protobuf;
    })();

    return google;
})();

module.exports = $root;

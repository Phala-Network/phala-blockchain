/*eslint-disable block-scoped-var, id-length, no-control-regex, no-magic-numbers, no-prototype-builtins, no-redeclare, no-shadow, no-var, sort-vars*/
"use strict";

var $protobuf = require("protobufjs/minimal");

// Common aliases
var $Reader = $protobuf.Reader, $Writer = $protobuf.Writer, $util = $protobuf.util;

// Exported root namespace
var $root = $protobuf.roots["default"] || ($protobuf.roots["default"] = {});

$root.prpc = (function() {

    /**
     * Namespace prpc.
     * @exports prpc
     * @namespace
     */
    var prpc = {};

    prpc.PrpcError = (function() {

        /**
         * Properties of a PrpcError.
         * @memberof prpc
         * @interface IPrpcError
         * @property {string|null} [message] The error description
         */

        /**
         * Constructs a new PrpcError.
         * @memberof prpc
         * @classdesc The final Error type of RPCs to be serialized to protobuf.
         * @implements IPrpcError
         * @constructor
         * @param {prpc.IPrpcError=} [properties] Properties to set
         */
        function PrpcError(properties) {
            if (properties)
                for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * The error description
         * @member {string} message
         * @memberof prpc.PrpcError
         * @instance
         */
        PrpcError.prototype.message = "";

        /**
         * Creates a new PrpcError instance using the specified properties.
         * @function create
         * @memberof prpc.PrpcError
         * @static
         * @param {prpc.IPrpcError=} [properties] Properties to set
         * @returns {prpc.PrpcError} PrpcError instance
         */
        PrpcError.create = function create(properties) {
            return new PrpcError(properties);
        };

        /**
         * Encodes the specified PrpcError message. Does not implicitly {@link prpc.PrpcError.verify|verify} messages.
         * @function encode
         * @memberof prpc.PrpcError
         * @static
         * @param {prpc.IPrpcError} message PrpcError message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        PrpcError.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.message != null && Object.hasOwnProperty.call(message, "message"))
                writer.uint32(/* id 1, wireType 2 =*/10).string(message.message);
            return writer;
        };

        /**
         * Encodes the specified PrpcError message, length delimited. Does not implicitly {@link prpc.PrpcError.verify|verify} messages.
         * @function encodeDelimited
         * @memberof prpc.PrpcError
         * @static
         * @param {prpc.IPrpcError} message PrpcError message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        PrpcError.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes a PrpcError message from the specified reader or buffer.
         * @function decode
         * @memberof prpc.PrpcError
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {prpc.PrpcError} PrpcError
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        PrpcError.decode = function decode(reader, length) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            var end = length === undefined ? reader.len : reader.pos + length, message = new $root.prpc.PrpcError();
            while (reader.pos < end) {
                var tag = reader.uint32();
                switch (tag >>> 3) {
                case 1: {
                        message.message = reader.string();
                        break;
                    }
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a PrpcError message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof prpc.PrpcError
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {prpc.PrpcError} PrpcError
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        PrpcError.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies a PrpcError message.
         * @function verify
         * @memberof prpc.PrpcError
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        PrpcError.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            if (message.message != null && message.hasOwnProperty("message"))
                if (!$util.isString(message.message))
                    return "message: string expected";
            return null;
        };

        /**
         * Creates a PrpcError message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof prpc.PrpcError
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {prpc.PrpcError} PrpcError
         */
        PrpcError.fromObject = function fromObject(object) {
            if (object instanceof $root.prpc.PrpcError)
                return object;
            var message = new $root.prpc.PrpcError();
            if (object.message != null)
                message.message = String(object.message);
            return message;
        };

        /**
         * Creates a plain object from a PrpcError message. Also converts values to other types if specified.
         * @function toObject
         * @memberof prpc.PrpcError
         * @static
         * @param {prpc.PrpcError} message PrpcError
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        PrpcError.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            var object = {};
            if (options.defaults)
                object.message = "";
            if (message.message != null && message.hasOwnProperty("message"))
                object.message = message.message;
            return object;
        };

        /**
         * Converts this PrpcError to JSON.
         * @function toJSON
         * @memberof prpc.PrpcError
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        PrpcError.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for PrpcError
         * @function getTypeUrl
         * @memberof prpc.PrpcError
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        PrpcError.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/prpc.PrpcError";
        };

        return PrpcError;
    })();

    return prpc;
})();

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

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#getNetworkConfig}.
         * @memberof pruntime_rpc.PhactoryAPI
         * @typedef GetNetworkConfigCallback
         * @type {function}
         * @param {Error|null} error Error, if any
         * @param {pruntime_rpc.NetworkConfigResponse} [response] NetworkConfigResponse
         */

        /**
         * Calls GetNetworkConfig.
         * @function getNetworkConfig
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {google.protobuf.IEmpty} request Empty message or plain object
         * @param {pruntime_rpc.PhactoryAPI.GetNetworkConfigCallback} callback Node-style callback called with the error, if any, and NetworkConfigResponse
         * @returns {undefined}
         * @variation 1
         */
        Object.defineProperty(PhactoryAPI.prototype.getNetworkConfig = function getNetworkConfig(request, callback) {
            return this.rpcCall(getNetworkConfig, $root.google.protobuf.Empty, $root.pruntime_rpc.NetworkConfigResponse, request, callback);
        }, "name", { value: "GetNetworkConfig" });

        /**
         * Calls GetNetworkConfig.
         * @function getNetworkConfig
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {google.protobuf.IEmpty} request Empty message or plain object
         * @returns {Promise<pruntime_rpc.NetworkConfigResponse>} Promise
         * @variation 2
         */

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#loadChainState}.
         * @memberof pruntime_rpc.PhactoryAPI
         * @typedef LoadChainStateCallback
         * @type {function}
         * @param {Error|null} error Error, if any
         * @param {google.protobuf.Empty} [response] Empty
         */

        /**
         * Calls LoadChainState.
         * @function loadChainState
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {pruntime_rpc.IChainState} request ChainState message or plain object
         * @param {pruntime_rpc.PhactoryAPI.LoadChainStateCallback} callback Node-style callback called with the error, if any, and Empty
         * @returns {undefined}
         * @variation 1
         */
        Object.defineProperty(PhactoryAPI.prototype.loadChainState = function loadChainState(request, callback) {
            return this.rpcCall(loadChainState, $root.pruntime_rpc.ChainState, $root.google.protobuf.Empty, request, callback);
        }, "name", { value: "LoadChainState" });

        /**
         * Calls LoadChainState.
         * @function loadChainState
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {pruntime_rpc.IChainState} request ChainState message or plain object
         * @returns {Promise<google.protobuf.Empty>} Promise
         * @variation 2
         */

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#stop}.
         * @memberof pruntime_rpc.PhactoryAPI
         * @typedef StopCallback
         * @type {function}
         * @param {Error|null} error Error, if any
         * @param {google.protobuf.Empty} [response] Empty
         */

        /**
         * Calls Stop.
         * @function stop
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {pruntime_rpc.IStopOptions} request StopOptions message or plain object
         * @param {pruntime_rpc.PhactoryAPI.StopCallback} callback Node-style callback called with the error, if any, and Empty
         * @returns {undefined}
         * @variation 1
         */
        Object.defineProperty(PhactoryAPI.prototype.stop = function stop(request, callback) {
            return this.rpcCall(stop, $root.pruntime_rpc.StopOptions, $root.google.protobuf.Empty, request, callback);
        }, "name", { value: "Stop" });

        /**
         * Calls Stop.
         * @function stop
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {pruntime_rpc.IStopOptions} request StopOptions message or plain object
         * @returns {Promise<google.protobuf.Empty>} Promise
         * @variation 2
         */

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#loadStorageProof}.
         * @memberof pruntime_rpc.PhactoryAPI
         * @typedef LoadStorageProofCallback
         * @type {function}
         * @param {Error|null} error Error, if any
         * @param {google.protobuf.Empty} [response] Empty
         */

        /**
         * Calls LoadStorageProof.
         * @function loadStorageProof
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {pruntime_rpc.IStorageProof} request StorageProof message or plain object
         * @param {pruntime_rpc.PhactoryAPI.LoadStorageProofCallback} callback Node-style callback called with the error, if any, and Empty
         * @returns {undefined}
         * @variation 1
         */
        Object.defineProperty(PhactoryAPI.prototype.loadStorageProof = function loadStorageProof(request, callback) {
            return this.rpcCall(loadStorageProof, $root.pruntime_rpc.StorageProof, $root.google.protobuf.Empty, request, callback);
        }, "name", { value: "LoadStorageProof" });

        /**
         * Calls LoadStorageProof.
         * @function loadStorageProof
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {pruntime_rpc.IStorageProof} request StorageProof message or plain object
         * @returns {Promise<google.protobuf.Empty>} Promise
         * @variation 2
         */

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#takeCheckpoint}.
         * @memberof pruntime_rpc.PhactoryAPI
         * @typedef TakeCheckpointCallback
         * @type {function}
         * @param {Error|null} error Error, if any
         * @param {pruntime_rpc.SyncedTo} [response] SyncedTo
         */

        /**
         * Calls TakeCheckpoint.
         * @function takeCheckpoint
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {google.protobuf.IEmpty} request Empty message or plain object
         * @param {pruntime_rpc.PhactoryAPI.TakeCheckpointCallback} callback Node-style callback called with the error, if any, and SyncedTo
         * @returns {undefined}
         * @variation 1
         */
        Object.defineProperty(PhactoryAPI.prototype.takeCheckpoint = function takeCheckpoint(request, callback) {
            return this.rpcCall(takeCheckpoint, $root.google.protobuf.Empty, $root.pruntime_rpc.SyncedTo, request, callback);
        }, "name", { value: "TakeCheckpoint" });

        /**
         * Calls TakeCheckpoint.
         * @function takeCheckpoint
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {google.protobuf.IEmpty} request Empty message or plain object
         * @returns {Promise<pruntime_rpc.SyncedTo>} Promise
         * @variation 2
         */

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#statistics}.
         * @memberof pruntime_rpc.PhactoryAPI
         * @typedef StatisticsCallback
         * @type {function}
         * @param {Error|null} error Error, if any
         * @param {pruntime_rpc.StatisticsResponse} [response] StatisticsResponse
         */

        /**
         * Calls Statistics.
         * @function statistics
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {pruntime_rpc.IStatisticsReqeust} request StatisticsReqeust message or plain object
         * @param {pruntime_rpc.PhactoryAPI.StatisticsCallback} callback Node-style callback called with the error, if any, and StatisticsResponse
         * @returns {undefined}
         * @variation 1
         */
        Object.defineProperty(PhactoryAPI.prototype.statistics = function statistics(request, callback) {
            return this.rpcCall(statistics, $root.pruntime_rpc.StatisticsReqeust, $root.pruntime_rpc.StatisticsResponse, request, callback);
        }, "name", { value: "Statistics" });

        /**
         * Calls Statistics.
         * @function statistics
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {pruntime_rpc.IStatisticsReqeust} request StatisticsReqeust message or plain object
         * @returns {Promise<pruntime_rpc.StatisticsResponse>} Promise
         * @variation 2
         */

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#generateClusterStateRequest}.
         * @memberof pruntime_rpc.PhactoryAPI
         * @typedef GenerateClusterStateRequestCallback
         * @type {function}
         * @param {Error|null} error Error, if any
         * @param {pruntime_rpc.SaveClusterStateArguments} [response] SaveClusterStateArguments
         */

        /**
         * Calls GenerateClusterStateRequest.
         * @function generateClusterStateRequest
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {google.protobuf.IEmpty} request Empty message or plain object
         * @param {pruntime_rpc.PhactoryAPI.GenerateClusterStateRequestCallback} callback Node-style callback called with the error, if any, and SaveClusterStateArguments
         * @returns {undefined}
         * @variation 1
         */
        Object.defineProperty(PhactoryAPI.prototype.generateClusterStateRequest = function generateClusterStateRequest(request, callback) {
            return this.rpcCall(generateClusterStateRequest, $root.google.protobuf.Empty, $root.pruntime_rpc.SaveClusterStateArguments, request, callback);
        }, "name", { value: "GenerateClusterStateRequest" });

        /**
         * Calls GenerateClusterStateRequest.
         * @function generateClusterStateRequest
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {google.protobuf.IEmpty} request Empty message or plain object
         * @returns {Promise<pruntime_rpc.SaveClusterStateArguments>} Promise
         * @variation 2
         */

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#saveClusterState}.
         * @memberof pruntime_rpc.PhactoryAPI
         * @typedef SaveClusterStateCallback
         * @type {function}
         * @param {Error|null} error Error, if any
         * @param {pruntime_rpc.SaveClusterStateResponse} [response] SaveClusterStateResponse
         */

        /**
         * Calls SaveClusterState.
         * @function saveClusterState
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {pruntime_rpc.ISaveClusterStateArguments} request SaveClusterStateArguments message or plain object
         * @param {pruntime_rpc.PhactoryAPI.SaveClusterStateCallback} callback Node-style callback called with the error, if any, and SaveClusterStateResponse
         * @returns {undefined}
         * @variation 1
         */
        Object.defineProperty(PhactoryAPI.prototype.saveClusterState = function saveClusterState(request, callback) {
            return this.rpcCall(saveClusterState, $root.pruntime_rpc.SaveClusterStateArguments, $root.pruntime_rpc.SaveClusterStateResponse, request, callback);
        }, "name", { value: "SaveClusterState" });

        /**
         * Calls SaveClusterState.
         * @function saveClusterState
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {pruntime_rpc.ISaveClusterStateArguments} request SaveClusterStateArguments message or plain object
         * @returns {Promise<pruntime_rpc.SaveClusterStateResponse>} Promise
         * @variation 2
         */

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#loadClusterState}.
         * @memberof pruntime_rpc.PhactoryAPI
         * @typedef LoadClusterStateCallback
         * @type {function}
         * @param {Error|null} error Error, if any
         * @param {google.protobuf.Empty} [response] Empty
         */

        /**
         * Calls LoadClusterState.
         * @function loadClusterState
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {pruntime_rpc.ISaveClusterStateResponse} request SaveClusterStateResponse message or plain object
         * @param {pruntime_rpc.PhactoryAPI.LoadClusterStateCallback} callback Node-style callback called with the error, if any, and Empty
         * @returns {undefined}
         * @variation 1
         */
        Object.defineProperty(PhactoryAPI.prototype.loadClusterState = function loadClusterState(request, callback) {
            return this.rpcCall(loadClusterState, $root.pruntime_rpc.SaveClusterStateResponse, $root.google.protobuf.Empty, request, callback);
        }, "name", { value: "LoadClusterState" });

        /**
         * Calls LoadClusterState.
         * @function loadClusterState
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {pruntime_rpc.ISaveClusterStateResponse} request SaveClusterStateResponse message or plain object
         * @returns {Promise<google.protobuf.Empty>} Promise
         * @variation 2
         */

        /**
         * Callback as used by {@link pruntime_rpc.PhactoryAPI#tryUpgradePinkRuntime}.
         * @memberof pruntime_rpc.PhactoryAPI
         * @typedef TryUpgradePinkRuntimeCallback
         * @type {function}
         * @param {Error|null} error Error, if any
         * @param {google.protobuf.Empty} [response] Empty
         */

        /**
         * Calls TryUpgradePinkRuntime.
         * @function tryUpgradePinkRuntime
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {pruntime_rpc.IPinkRuntimeVersion} request PinkRuntimeVersion message or plain object
         * @param {pruntime_rpc.PhactoryAPI.TryUpgradePinkRuntimeCallback} callback Node-style callback called with the error, if any, and Empty
         * @returns {undefined}
         * @variation 1
         */
        Object.defineProperty(PhactoryAPI.prototype.tryUpgradePinkRuntime = function tryUpgradePinkRuntime(request, callback) {
            return this.rpcCall(tryUpgradePinkRuntime, $root.pruntime_rpc.PinkRuntimeVersion, $root.google.protobuf.Empty, request, callback);
        }, "name", { value: "TryUpgradePinkRuntime" });

        /**
         * Calls TryUpgradePinkRuntime.
         * @function tryUpgradePinkRuntime
         * @memberof pruntime_rpc.PhactoryAPI
         * @instance
         * @param {pruntime_rpc.IPinkRuntimeVersion} request PinkRuntimeVersion message or plain object
         * @returns {Promise<google.protobuf.Empty>} Promise
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
         * @property {pruntime_rpc.ISystemInfo|null} [system] PhactoryInfo system
         * @property {boolean|null} [canLoadChainState] PhactoryInfo canLoadChainState
         * @property {number|null} [safeModeLevel] PhactoryInfo safeModeLevel
         * @property {number|Long|null} [currentBlockTime] PhactoryInfo currentBlockTime
         * @property {string|null} [maxSupportedPinkRuntimeVersion] PhactoryInfo maxSupportedPinkRuntimeVersion
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
         * PhactoryInfo system.
         * @member {pruntime_rpc.ISystemInfo|null|undefined} system
         * @memberof pruntime_rpc.PhactoryInfo
         * @instance
         */
        PhactoryInfo.prototype.system = null;

        /**
         * PhactoryInfo canLoadChainState.
         * @member {boolean} canLoadChainState
         * @memberof pruntime_rpc.PhactoryInfo
         * @instance
         */
        PhactoryInfo.prototype.canLoadChainState = false;

        /**
         * PhactoryInfo safeModeLevel.
         * @member {number} safeModeLevel
         * @memberof pruntime_rpc.PhactoryInfo
         * @instance
         */
        PhactoryInfo.prototype.safeModeLevel = 0;

        /**
         * PhactoryInfo currentBlockTime.
         * @member {number|Long} currentBlockTime
         * @memberof pruntime_rpc.PhactoryInfo
         * @instance
         */
        PhactoryInfo.prototype.currentBlockTime = $util.Long ? $util.Long.fromBits(0,0,true) : 0;

        /**
         * PhactoryInfo maxSupportedPinkRuntimeVersion.
         * @member {string} maxSupportedPinkRuntimeVersion
         * @memberof pruntime_rpc.PhactoryInfo
         * @instance
         */
        PhactoryInfo.prototype.maxSupportedPinkRuntimeVersion = "";

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
            if (message.system != null && Object.hasOwnProperty.call(message, "system"))
                $root.pruntime_rpc.SystemInfo.encode(message.system, writer.uint32(/* id 23, wireType 2 =*/186).fork()).ldelim();
            if (message.canLoadChainState != null && Object.hasOwnProperty.call(message, "canLoadChainState"))
                writer.uint32(/* id 24, wireType 0 =*/192).bool(message.canLoadChainState);
            if (message.safeModeLevel != null && Object.hasOwnProperty.call(message, "safeModeLevel"))
                writer.uint32(/* id 25, wireType 0 =*/200).uint32(message.safeModeLevel);
            if (message.currentBlockTime != null && Object.hasOwnProperty.call(message, "currentBlockTime"))
                writer.uint32(/* id 26, wireType 0 =*/208).uint64(message.currentBlockTime);
            if (message.maxSupportedPinkRuntimeVersion != null && Object.hasOwnProperty.call(message, "maxSupportedPinkRuntimeVersion"))
                writer.uint32(/* id 27, wireType 2 =*/218).string(message.maxSupportedPinkRuntimeVersion);
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
                case 1: {
                        message.initialized = reader.bool();
                        break;
                    }
                case 2: {
                        message.registered = reader.bool();
                        break;
                    }
                case 4: {
                        message.genesisBlockHash = reader.string();
                        break;
                    }
                case 5: {
                        message.publicKey = reader.string();
                        break;
                    }
                case 6: {
                        message.ecdhPublicKey = reader.string();
                        break;
                    }
                case 7: {
                        message.headernum = reader.uint32();
                        break;
                    }
                case 8: {
                        message.paraHeadernum = reader.uint32();
                        break;
                    }
                case 9: {
                        message.blocknum = reader.uint32();
                        break;
                    }
                case 10: {
                        message.stateRoot = reader.string();
                        break;
                    }
                case 11: {
                        message.devMode = reader.bool();
                        break;
                    }
                case 12: {
                        message.pendingMessages = reader.uint64();
                        break;
                    }
                case 13: {
                        message.score = reader.uint64();
                        break;
                    }
                case 14: {
                        message.gatekeeper = $root.pruntime_rpc.GatekeeperStatus.decode(reader, reader.uint32());
                        break;
                    }
                case 15: {
                        message.version = reader.string();
                        break;
                    }
                case 16: {
                        message.gitRevision = reader.string();
                        break;
                    }
                case 18: {
                        message.memoryUsage = $root.pruntime_rpc.MemoryUsage.decode(reader, reader.uint32());
                        break;
                    }
                case 21: {
                        message.waitingForParaheaders = reader.bool();
                        break;
                    }
                case 23: {
                        message.system = $root.pruntime_rpc.SystemInfo.decode(reader, reader.uint32());
                        break;
                    }
                case 24: {
                        message.canLoadChainState = reader.bool();
                        break;
                    }
                case 25: {
                        message.safeModeLevel = reader.uint32();
                        break;
                    }
                case 26: {
                        message.currentBlockTime = reader.uint64();
                        break;
                    }
                case 27: {
                        message.maxSupportedPinkRuntimeVersion = reader.string();
                        break;
                    }
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
            if (message.system != null && message.hasOwnProperty("system")) {
                var error = $root.pruntime_rpc.SystemInfo.verify(message.system);
                if (error)
                    return "system." + error;
            }
            if (message.canLoadChainState != null && message.hasOwnProperty("canLoadChainState"))
                if (typeof message.canLoadChainState !== "boolean")
                    return "canLoadChainState: boolean expected";
            if (message.safeModeLevel != null && message.hasOwnProperty("safeModeLevel"))
                if (!$util.isInteger(message.safeModeLevel))
                    return "safeModeLevel: integer expected";
            if (message.currentBlockTime != null && message.hasOwnProperty("currentBlockTime"))
                if (!$util.isInteger(message.currentBlockTime) && !(message.currentBlockTime && $util.isInteger(message.currentBlockTime.low) && $util.isInteger(message.currentBlockTime.high)))
                    return "currentBlockTime: integer|Long expected";
            if (message.maxSupportedPinkRuntimeVersion != null && message.hasOwnProperty("maxSupportedPinkRuntimeVersion"))
                if (!$util.isString(message.maxSupportedPinkRuntimeVersion))
                    return "maxSupportedPinkRuntimeVersion: string expected";
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
            if (object.system != null) {
                if (typeof object.system !== "object")
                    throw TypeError(".pruntime_rpc.PhactoryInfo.system: object expected");
                message.system = $root.pruntime_rpc.SystemInfo.fromObject(object.system);
            }
            if (object.canLoadChainState != null)
                message.canLoadChainState = Boolean(object.canLoadChainState);
            if (object.safeModeLevel != null)
                message.safeModeLevel = object.safeModeLevel >>> 0;
            if (object.currentBlockTime != null)
                if ($util.Long)
                    (message.currentBlockTime = $util.Long.fromValue(object.currentBlockTime)).unsigned = true;
                else if (typeof object.currentBlockTime === "string")
                    message.currentBlockTime = parseInt(object.currentBlockTime, 10);
                else if (typeof object.currentBlockTime === "number")
                    message.currentBlockTime = object.currentBlockTime;
                else if (typeof object.currentBlockTime === "object")
                    message.currentBlockTime = new $util.LongBits(object.currentBlockTime.low >>> 0, object.currentBlockTime.high >>> 0).toNumber(true);
            if (object.maxSupportedPinkRuntimeVersion != null)
                message.maxSupportedPinkRuntimeVersion = String(object.maxSupportedPinkRuntimeVersion);
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
                object.system = null;
                object.canLoadChainState = false;
                object.safeModeLevel = 0;
                if ($util.Long) {
                    var long = new $util.Long(0, 0, true);
                    object.currentBlockTime = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : long;
                } else
                    object.currentBlockTime = options.longs === String ? "0" : 0;
                object.maxSupportedPinkRuntimeVersion = "";
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
            if (message.system != null && message.hasOwnProperty("system"))
                object.system = $root.pruntime_rpc.SystemInfo.toObject(message.system, options);
            if (message.canLoadChainState != null && message.hasOwnProperty("canLoadChainState"))
                object.canLoadChainState = message.canLoadChainState;
            if (message.safeModeLevel != null && message.hasOwnProperty("safeModeLevel"))
                object.safeModeLevel = message.safeModeLevel;
            if (message.currentBlockTime != null && message.hasOwnProperty("currentBlockTime"))
                if (typeof message.currentBlockTime === "number")
                    object.currentBlockTime = options.longs === String ? String(message.currentBlockTime) : message.currentBlockTime;
                else
                    object.currentBlockTime = options.longs === String ? $util.Long.prototype.toString.call(message.currentBlockTime) : options.longs === Number ? new $util.LongBits(message.currentBlockTime.low >>> 0, message.currentBlockTime.high >>> 0).toNumber(true) : message.currentBlockTime;
            if (message.maxSupportedPinkRuntimeVersion != null && message.hasOwnProperty("maxSupportedPinkRuntimeVersion"))
                object.maxSupportedPinkRuntimeVersion = message.maxSupportedPinkRuntimeVersion;
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

        /**
         * Gets the default type url for PhactoryInfo
         * @function getTypeUrl
         * @memberof pruntime_rpc.PhactoryInfo
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        PhactoryInfo.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/pruntime_rpc.PhactoryInfo";
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
         * @property {number|null} [maxSupportedConsensusVersion] SystemInfo maxSupportedConsensusVersion
         * @property {number|null} [genesisBlock] SystemInfo genesisBlock
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
         * SystemInfo maxSupportedConsensusVersion.
         * @member {number} maxSupportedConsensusVersion
         * @memberof pruntime_rpc.SystemInfo
         * @instance
         */
        SystemInfo.prototype.maxSupportedConsensusVersion = 0;

        /**
         * SystemInfo genesisBlock.
         * @member {number} genesisBlock
         * @memberof pruntime_rpc.SystemInfo
         * @instance
         */
        SystemInfo.prototype.genesisBlock = 0;

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
            if (message.maxSupportedConsensusVersion != null && Object.hasOwnProperty.call(message, "maxSupportedConsensusVersion"))
                writer.uint32(/* id 7, wireType 0 =*/56).uint32(message.maxSupportedConsensusVersion);
            if (message.genesisBlock != null && Object.hasOwnProperty.call(message, "genesisBlock"))
                writer.uint32(/* id 8, wireType 0 =*/64).uint32(message.genesisBlock);
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
                case 1: {
                        message.registered = reader.bool();
                        break;
                    }
                case 2: {
                        message.publicKey = reader.string();
                        break;
                    }
                case 3: {
                        message.ecdhPublicKey = reader.string();
                        break;
                    }
                case 4: {
                        message.gatekeeper = $root.pruntime_rpc.GatekeeperStatus.decode(reader, reader.uint32());
                        break;
                    }
                case 5: {
                        message.numberOfClusters = reader.uint64();
                        break;
                    }
                case 6: {
                        message.numberOfContracts = reader.uint64();
                        break;
                    }
                case 7: {
                        message.maxSupportedConsensusVersion = reader.uint32();
                        break;
                    }
                case 8: {
                        message.genesisBlock = reader.uint32();
                        break;
                    }
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
            if (message.maxSupportedConsensusVersion != null && message.hasOwnProperty("maxSupportedConsensusVersion"))
                if (!$util.isInteger(message.maxSupportedConsensusVersion))
                    return "maxSupportedConsensusVersion: integer expected";
            if (message.genesisBlock != null && message.hasOwnProperty("genesisBlock"))
                if (!$util.isInteger(message.genesisBlock))
                    return "genesisBlock: integer expected";
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
            if (object.maxSupportedConsensusVersion != null)
                message.maxSupportedConsensusVersion = object.maxSupportedConsensusVersion >>> 0;
            if (object.genesisBlock != null)
                message.genesisBlock = object.genesisBlock >>> 0;
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
                object.maxSupportedConsensusVersion = 0;
                object.genesisBlock = 0;
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
            if (message.maxSupportedConsensusVersion != null && message.hasOwnProperty("maxSupportedConsensusVersion"))
                object.maxSupportedConsensusVersion = message.maxSupportedConsensusVersion;
            if (message.genesisBlock != null && message.hasOwnProperty("genesisBlock"))
                object.genesisBlock = message.genesisBlock;
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

        /**
         * Gets the default type url for SystemInfo
         * @function getTypeUrl
         * @memberof pruntime_rpc.SystemInfo
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        SystemInfo.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/pruntime_rpc.SystemInfo";
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
                case 1: {
                        message.role = reader.int32();
                        break;
                    }
                case 2: {
                        message.masterPublicKey = reader.string();
                        break;
                    }
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
            default:
                if (typeof object.role === "number") {
                    message.role = object.role;
                    break;
                }
                break;
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
                object.role = options.enums === String ? $root.pruntime_rpc.GatekeeperRole[message.role] === undefined ? message.role : $root.pruntime_rpc.GatekeeperRole[message.role] : message.role;
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

        /**
         * Gets the default type url for GatekeeperStatus
         * @function getTypeUrl
         * @memberof pruntime_rpc.GatekeeperStatus
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        GatekeeperStatus.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/pruntime_rpc.GatekeeperStatus";
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
         * @property {number|Long|null} [free] MemoryUsage free
         * @property {number|Long|null} [rustSpike] MemoryUsage rustSpike
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
         * MemoryUsage free.
         * @member {number|Long} free
         * @memberof pruntime_rpc.MemoryUsage
         * @instance
         */
        MemoryUsage.prototype.free = $util.Long ? $util.Long.fromBits(0,0,true) : 0;

        /**
         * MemoryUsage rustSpike.
         * @member {number|Long} rustSpike
         * @memberof pruntime_rpc.MemoryUsage
         * @instance
         */
        MemoryUsage.prototype.rustSpike = $util.Long ? $util.Long.fromBits(0,0,true) : 0;

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
            if (message.free != null && Object.hasOwnProperty.call(message, "free"))
                writer.uint32(/* id 4, wireType 0 =*/32).uint64(message.free);
            if (message.rustSpike != null && Object.hasOwnProperty.call(message, "rustSpike"))
                writer.uint32(/* id 5, wireType 0 =*/40).uint64(message.rustSpike);
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
                case 1: {
                        message.rustUsed = reader.uint64();
                        break;
                    }
                case 2: {
                        message.rustPeakUsed = reader.uint64();
                        break;
                    }
                case 3: {
                        message.totalPeakUsed = reader.uint64();
                        break;
                    }
                case 4: {
                        message.free = reader.uint64();
                        break;
                    }
                case 5: {
                        message.rustSpike = reader.uint64();
                        break;
                    }
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
            if (message.free != null && message.hasOwnProperty("free"))
                if (!$util.isInteger(message.free) && !(message.free && $util.isInteger(message.free.low) && $util.isInteger(message.free.high)))
                    return "free: integer|Long expected";
            if (message.rustSpike != null && message.hasOwnProperty("rustSpike"))
                if (!$util.isInteger(message.rustSpike) && !(message.rustSpike && $util.isInteger(message.rustSpike.low) && $util.isInteger(message.rustSpike.high)))
                    return "rustSpike: integer|Long expected";
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
            if (object.free != null)
                if ($util.Long)
                    (message.free = $util.Long.fromValue(object.free)).unsigned = true;
                else if (typeof object.free === "string")
                    message.free = parseInt(object.free, 10);
                else if (typeof object.free === "number")
                    message.free = object.free;
                else if (typeof object.free === "object")
                    message.free = new $util.LongBits(object.free.low >>> 0, object.free.high >>> 0).toNumber(true);
            if (object.rustSpike != null)
                if ($util.Long)
                    (message.rustSpike = $util.Long.fromValue(object.rustSpike)).unsigned = true;
                else if (typeof object.rustSpike === "string")
                    message.rustSpike = parseInt(object.rustSpike, 10);
                else if (typeof object.rustSpike === "number")
                    message.rustSpike = object.rustSpike;
                else if (typeof object.rustSpike === "object")
                    message.rustSpike = new $util.LongBits(object.rustSpike.low >>> 0, object.rustSpike.high >>> 0).toNumber(true);
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
                if ($util.Long) {
                    var long = new $util.Long(0, 0, true);
                    object.free = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : long;
                } else
                    object.free = options.longs === String ? "0" : 0;
                if ($util.Long) {
                    var long = new $util.Long(0, 0, true);
                    object.rustSpike = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : long;
                } else
                    object.rustSpike = options.longs === String ? "0" : 0;
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
            if (message.free != null && message.hasOwnProperty("free"))
                if (typeof message.free === "number")
                    object.free = options.longs === String ? String(message.free) : message.free;
                else
                    object.free = options.longs === String ? $util.Long.prototype.toString.call(message.free) : options.longs === Number ? new $util.LongBits(message.free.low >>> 0, message.free.high >>> 0).toNumber(true) : message.free;
            if (message.rustSpike != null && message.hasOwnProperty("rustSpike"))
                if (typeof message.rustSpike === "number")
                    object.rustSpike = options.longs === String ? String(message.rustSpike) : message.rustSpike;
                else
                    object.rustSpike = options.longs === String ? $util.Long.prototype.toString.call(message.rustSpike) : options.longs === Number ? new $util.LongBits(message.rustSpike.low >>> 0, message.rustSpike.high >>> 0).toNumber(true) : message.rustSpike;
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

        /**
         * Gets the default type url for MemoryUsage
         * @function getTypeUrl
         * @memberof pruntime_rpc.MemoryUsage
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        MemoryUsage.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/pruntime_rpc.MemoryUsage";
        };

        return MemoryUsage;
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
                case 1: {
                        message.forceRefreshRa = reader.bool();
                        break;
                    }
                case 2: {
                        message.encodedOperator = reader.bytes();
                        break;
                    }
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
                else if (object.encodedOperator.length >= 0)
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

        /**
         * Gets the default type url for GetRuntimeInfoRequest
         * @function getTypeUrl
         * @memberof pruntime_rpc.GetRuntimeInfoRequest
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        GetRuntimeInfoRequest.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/pruntime_rpc.GetRuntimeInfoRequest";
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
                case 1: {
                        message.encodedRuntimeInfo = reader.bytes();
                        break;
                    }
                case 2: {
                        message.encodedGenesisBlockHash = reader.bytes();
                        break;
                    }
                case 3: {
                        message.encodedPublicKey = reader.bytes();
                        break;
                    }
                case 4: {
                        message.encodedEcdhPublicKey = reader.bytes();
                        break;
                    }
                case 5: {
                        message.attestation = $root.pruntime_rpc.Attestation.decode(reader, reader.uint32());
                        break;
                    }
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
                else if (object.encodedRuntimeInfo.length >= 0)
                    message.encodedRuntimeInfo = object.encodedRuntimeInfo;
            if (object.encodedGenesisBlockHash != null)
                if (typeof object.encodedGenesisBlockHash === "string")
                    $util.base64.decode(object.encodedGenesisBlockHash, message.encodedGenesisBlockHash = $util.newBuffer($util.base64.length(object.encodedGenesisBlockHash)), 0);
                else if (object.encodedGenesisBlockHash.length >= 0)
                    message.encodedGenesisBlockHash = object.encodedGenesisBlockHash;
            if (object.encodedPublicKey != null)
                if (typeof object.encodedPublicKey === "string")
                    $util.base64.decode(object.encodedPublicKey, message.encodedPublicKey = $util.newBuffer($util.base64.length(object.encodedPublicKey)), 0);
                else if (object.encodedPublicKey.length >= 0)
                    message.encodedPublicKey = object.encodedPublicKey;
            if (object.encodedEcdhPublicKey != null)
                if (typeof object.encodedEcdhPublicKey === "string")
                    $util.base64.decode(object.encodedEcdhPublicKey, message.encodedEcdhPublicKey = $util.newBuffer($util.base64.length(object.encodedEcdhPublicKey)), 0);
                else if (object.encodedEcdhPublicKey.length >= 0)
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

        /**
         * Gets the default type url for InitRuntimeResponse
         * @function getTypeUrl
         * @memberof pruntime_rpc.InitRuntimeResponse
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        InitRuntimeResponse.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/pruntime_rpc.InitRuntimeResponse";
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
                case 1: {
                        message.version = reader.int32();
                        break;
                    }
                case 2: {
                        message.provider = reader.string();
                        break;
                    }
                case 3: {
                        message.payload = $root.pruntime_rpc.AttestationReport.decode(reader, reader.uint32());
                        break;
                    }
                case 5: {
                        message.encodedReport = reader.bytes();
                        break;
                    }
                case 4: {
                        message.timestamp = reader.uint64();
                        break;
                    }
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
                else if (object.encodedReport.length >= 0)
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

        /**
         * Gets the default type url for Attestation
         * @function getTypeUrl
         * @memberof pruntime_rpc.Attestation
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        Attestation.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/pruntime_rpc.Attestation";
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
                case 1: {
                        message.report = reader.string();
                        break;
                    }
                case 2: {
                        message.signature = reader.bytes();
                        break;
                    }
                case 3: {
                        message.signingCert = reader.bytes();
                        break;
                    }
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
                else if (object.signature.length >= 0)
                    message.signature = object.signature;
            if (object.signingCert != null)
                if (typeof object.signingCert === "string")
                    $util.base64.decode(object.signingCert, message.signingCert = $util.newBuffer($util.base64.length(object.signingCert)), 0);
                else if (object.signingCert.length >= 0)
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

        /**
         * Gets the default type url for AttestationReport
         * @function getTypeUrl
         * @memberof pruntime_rpc.AttestationReport
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        AttestationReport.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/pruntime_rpc.AttestationReport";
        };

        return AttestationReport;
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
                case 1: {
                        message.encodedEncryptedData = reader.bytes();
                        break;
                    }
                case 2: {
                        message.signature = $root.pruntime_rpc.Signature.decode(reader, reader.uint32());
                        break;
                    }
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
                else if (object.encodedEncryptedData.length >= 0)
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

        /**
         * Gets the default type url for ContractQueryRequest
         * @function getTypeUrl
         * @memberof pruntime_rpc.ContractQueryRequest
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        ContractQueryRequest.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/pruntime_rpc.ContractQueryRequest";
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
                case 1: {
                        message.signedBy = $root.pruntime_rpc.Certificate.decode(reader, reader.uint32());
                        break;
                    }
                case 2: {
                        message.signatureType = reader.int32();
                        break;
                    }
                case 3: {
                        message.signature = reader.bytes();
                        break;
                    }
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
                case 6:
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
            default:
                if (typeof object.signatureType === "number") {
                    message.signatureType = object.signatureType;
                    break;
                }
                break;
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
            case "Eip712":
            case 6:
                message.signatureType = 6;
                break;
            }
            if (object.signature != null)
                if (typeof object.signature === "string")
                    $util.base64.decode(object.signature, message.signature = $util.newBuffer($util.base64.length(object.signature)), 0);
                else if (object.signature.length >= 0)
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
                object.signatureType = options.enums === String ? $root.pruntime_rpc.SignatureType[message.signatureType] === undefined ? message.signatureType : $root.pruntime_rpc.SignatureType[message.signatureType] : message.signatureType;
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

        /**
         * Gets the default type url for Signature
         * @function getTypeUrl
         * @memberof pruntime_rpc.Signature
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        Signature.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/pruntime_rpc.Signature";
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
                case 1: {
                        message.encodedBody = reader.bytes();
                        break;
                    }
                case 2: {
                        message.signature = $root.pruntime_rpc.Signature.decode(reader, reader.uint32());
                        break;
                    }
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
                else if (object.encodedBody.length >= 0)
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

        /**
         * Gets the default type url for Certificate
         * @function getTypeUrl
         * @memberof pruntime_rpc.Certificate
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        Certificate.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/pruntime_rpc.Certificate";
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
     * @property {number} Eip712=6 Eip712 value
     */
    pruntime_rpc.SignatureType = (function() {
        var valuesById = {}, values = Object.create(valuesById);
        values[valuesById[0] = "Ed25519"] = 0;
        values[valuesById[1] = "Sr25519"] = 1;
        values[valuesById[2] = "Ecdsa"] = 2;
        values[valuesById[3] = "Ed25519WrapBytes"] = 3;
        values[valuesById[4] = "Sr25519WrapBytes"] = 4;
        values[valuesById[5] = "EcdsaWrapBytes"] = 5;
        values[valuesById[6] = "Eip712"] = 6;
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
                case 1: {
                        message.encodedEncryptedData = reader.bytes();
                        break;
                    }
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
                else if (object.encodedEncryptedData.length >= 0)
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

        /**
         * Gets the default type url for ContractQueryResponse
         * @function getTypeUrl
         * @memberof pruntime_rpc.ContractQueryResponse
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        ContractQueryResponse.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/pruntime_rpc.ContractQueryResponse";
        };

        return ContractQueryResponse;
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
                case 1: {
                        message.encodedEndpointType = reader.bytes();
                        break;
                    }
                case 2: {
                        message.endpoint = reader.string();
                        break;
                    }
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
                else if (object.encodedEndpointType.length >= 0)
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

        /**
         * Gets the default type url for AddEndpointRequest
         * @function getTypeUrl
         * @memberof pruntime_rpc.AddEndpointRequest
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        AddEndpointRequest.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/pruntime_rpc.AddEndpointRequest";
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
                case 1: {
                        message.encodedEndpointPayload = reader.bytes();
                        break;
                    }
                case 2: {
                        message.signature = reader.bytes();
                        break;
                    }
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
                else if (object.encodedEndpointPayload.length >= 0)
                    message.encodedEndpointPayload = object.encodedEndpointPayload;
            if (object.signature != null)
                if (typeof object.signature === "string")
                    $util.base64.decode(object.signature, message.signature = $util.newBuffer($util.base64.length(object.signature)), 0);
                else if (object.signature.length >= 0)
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

        /**
         * Gets the default type url for GetEndpointResponse
         * @function getTypeUrl
         * @memberof pruntime_rpc.GetEndpointResponse
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        GetEndpointResponse.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/pruntime_rpc.GetEndpointResponse";
        };

        return GetEndpointResponse;
    })();

    pruntime_rpc.GetContractInfoRequest = (function() {

        /**
         * Properties of a GetContractInfoRequest.
         * @memberof pruntime_rpc
         * @interface IGetContractInfoRequest
         * @property {Array.<string>|null} [contracts] GetContractInfoRequest contracts
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
            this.contracts = [];
            if (properties)
                for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * GetContractInfoRequest contracts.
         * @member {Array.<string>} contracts
         * @memberof pruntime_rpc.GetContractInfoRequest
         * @instance
         */
        GetContractInfoRequest.prototype.contracts = $util.emptyArray;

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
            if (message.contracts != null && message.contracts.length)
                for (var i = 0; i < message.contracts.length; ++i)
                    writer.uint32(/* id 1, wireType 2 =*/10).string(message.contracts[i]);
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
                case 1: {
                        if (!(message.contracts && message.contracts.length))
                            message.contracts = [];
                        message.contracts.push(reader.string());
                        break;
                    }
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
            if (object.contracts) {
                if (!Array.isArray(object.contracts))
                    throw TypeError(".pruntime_rpc.GetContractInfoRequest.contracts: array expected");
                message.contracts = [];
                for (var i = 0; i < object.contracts.length; ++i)
                    message.contracts[i] = String(object.contracts[i]);
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
                object.contracts = [];
            if (message.contracts && message.contracts.length) {
                object.contracts = [];
                for (var j = 0; j < message.contracts.length; ++j)
                    object.contracts[j] = message.contracts[j];
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

        /**
         * Gets the default type url for GetContractInfoRequest
         * @function getTypeUrl
         * @memberof pruntime_rpc.GetContractInfoRequest
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        GetContractInfoRequest.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/pruntime_rpc.GetContractInfoRequest";
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
                case 1: {
                        if (!(message.contracts && message.contracts.length))
                            message.contracts = [];
                        message.contracts.push($root.pruntime_rpc.ContractInfo.decode(reader, reader.uint32()));
                        break;
                    }
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

        /**
         * Gets the default type url for GetContractInfoResponse
         * @function getTypeUrl
         * @memberof pruntime_rpc.GetContractInfoResponse
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        GetContractInfoResponse.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/pruntime_rpc.GetContractInfoResponse";
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
                case 1: {
                        message.id = reader.string();
                        break;
                    }
                case 2: {
                        message.codeHash = reader.string();
                        break;
                    }
                case 3: {
                        message.weight = reader.uint32();
                        break;
                    }
                case 4: {
                        message.sidevm = $root.pruntime_rpc.SidevmInfo.decode(reader, reader.uint32());
                        break;
                    }
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

        /**
         * Gets the default type url for ContractInfo
         * @function getTypeUrl
         * @memberof pruntime_rpc.ContractInfo
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        ContractInfo.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/pruntime_rpc.ContractInfo";
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
                case 1: {
                        message.state = reader.string();
                        break;
                    }
                case 2: {
                        message.codeHash = reader.string();
                        break;
                    }
                case 3: {
                        message.startTime = reader.string();
                        break;
                    }
                case 4: {
                        message.stopReason = reader.string();
                        break;
                    }
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

        /**
         * Gets the default type url for SidevmInfo
         * @function getTypeUrl
         * @memberof pruntime_rpc.SidevmInfo
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        SidevmInfo.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/pruntime_rpc.SidevmInfo";
        };

        return SidevmInfo;
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
                case 1: {
                        message.contract = reader.bytes();
                        break;
                    }
                case 2: {
                        message.code = reader.bytes();
                        break;
                    }
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
                else if (object.contract.length >= 0)
                    message.contract = object.contract;
            if (object.code != null)
                if (typeof object.code === "string")
                    $util.base64.decode(object.code, message.code = $util.newBuffer($util.base64.length(object.code)), 0);
                else if (object.code.length >= 0)
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

        /**
         * Gets the default type url for SidevmCode
         * @function getTypeUrl
         * @memberof pruntime_rpc.SidevmCode
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        SidevmCode.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/pruntime_rpc.SidevmCode";
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
                case 1: {
                        message.deployer = reader.string();
                        break;
                    }
                case 2: {
                        message.clusterId = reader.string();
                        break;
                    }
                case 3: {
                        message.codeHash = reader.string();
                        break;
                    }
                case 4: {
                        message.salt = reader.string();
                        break;
                    }
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

        /**
         * Gets the default type url for ContractParameters
         * @function getTypeUrl
         * @memberof pruntime_rpc.ContractParameters
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        ContractParameters.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/pruntime_rpc.ContractParameters";
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
                case 1: {
                        message.id = reader.string();
                        break;
                    }
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

        /**
         * Gets the default type url for ContractId
         * @function getTypeUrl
         * @memberof pruntime_rpc.ContractId
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        ContractId.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/pruntime_rpc.ContractId";
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

            /**
             * Gets the default type url for Empty
             * @function getTypeUrl
             * @memberof google.protobuf.Empty
             * @static
             * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns {string} The default type url
             */
            Empty.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
                if (typeUrlPrefix === undefined) {
                    typeUrlPrefix = "type.googleapis.com";
                }
                return typeUrlPrefix + "/google.protobuf.Empty";
            };

            return Empty;
        })();

        return protobuf;
    })();

    return google;
})();

module.exports = $root;

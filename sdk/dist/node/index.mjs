// src/lib/types.ts
var types = {
  ContractId: "H256",
  EcdhPublicKey: "SpCoreSr25519Public",
  ContractQueryHead: {
    id: "ContractId",
    nonce: "[u8; 32]"
  },
  CertificateBody: {
    pubkey: "Vec<u8>",
    ttl: "u32",
    config_bits: "u32"
  },
  EncryptedData: {
    iv: "[u8; 12]",
    pubkey: "EcdhPublicKey",
    data: "Vec<u8>"
  },
  CommandPayload: {
    _enum: {
      Plain: null,
      Encrypted: "EncryptedData"
    }
  },
  InkQueryData: {
    _enum: {
      InkMessage: {
        payload: "Vec<u8>",
        deposit: "u128",
        transfer: "u128"
      },
      SidevmMessage: "Vec<u8>",
      InkInstantiate: {
        codeHash: "H256",
        salt: "Vec<u8>",
        instantiateData: "Vec<u8>",
        deposit: "u128",
        transfer: "u128"
      }
    }
  },
  InkQuery: {
    head: "ContractQueryHead",
    data: "InkQueryData"
  },
  InkQueryError: {
    _enum: {
      BadOrigin: null,
      RuntimeError: "String"
    }
  },
  InkQueryOk: {
    _enum: {
      InkMessageReturn: "Vec<u8>"
    }
  },
  InkResponse: {
    nonce: "[u8; 32]",
    result: "Result<InkQueryOk, InkQueryError>"
  },
  InkMessage: {
    nonce: "Vec<u8>",
    message: "Vec<u8>",
    transfer: "u128",
    gasLimit: "u64",
    storageDepositLimit: "Option<u128>"
  },
  InkCommand: { _enum: { InkMessage: "InkMessage" } }
};

// src/lib/hex.ts
import { randomBytes } from "crypto-browserify";
var randomHex = (size = 12) => randomBytes(size).toString("hex");

// src/create.ts
import {
  BN,
  hexAddPrefix as hexAddPrefix2,
  hexStripPrefix as hexStripPrefix2,
  hexToU8a as hexToU8a2,
  stringToHex,
  u8aToHex
} from "@polkadot/util";
import {
  sr25519Agree,
  sr25519KeypairFromSeed,
  sr25519Sign,
  waitReady
} from "@polkadot/wasm-crypto";
import axios, { AxiosError } from "axios";
import { from } from "rxjs";

// src/lib/aes-256-gcm.ts
import { hexAddPrefix, hexStripPrefix, hexToU8a } from "@polkadot/util";
import { createCipheriv, createDecipheriv } from "crypto-browserify";
import { Buffer } from "buffer";
var ALGO = "aes-256-gcm";
var AUTH_TAG_LENGTH = 32;
var toU8a = (param) => {
  if (typeof param === "string") {
    param = hexAddPrefix(param);
    return hexToU8a(param);
  }
  return param;
};
var encrypt = (data, key, iv) => {
  data = hexStripPrefix(data);
  const cipher = createCipheriv(
    ALGO,
    toU8a(key),
    Buffer.from(toU8a(iv))
  );
  const enc = cipher.update(data, "hex", "hex");
  cipher.final();
  return `${enc}${cipher.getAuthTag().toString("hex")}`;
};
var decrypt = (enc, key, iv) => {
  enc = hexStripPrefix(enc);
  const decipher = createDecipheriv(
    ALGO,
    toU8a(key),
    Buffer.from(toU8a(iv))
  );
  const authTag = hexToU8a(hexAddPrefix(enc.slice(-AUTH_TAG_LENGTH)));
  decipher.setAuthTag(authTag);
  const data = decipher.update(enc.slice(0, -AUTH_TAG_LENGTH), "hex", "hex");
  decipher.final();
  return data;
};

// src/proto/index.js
import * as $protobuf from "protobufjs/minimal";
var $Reader = $protobuf.Reader;
var $Writer = $protobuf.Writer;
var $util = $protobuf.util;
var $root = $protobuf.roots["default"] || ($protobuf.roots["default"] = {});
var prpc = $root.prpc = (() => {
  const prpc2 = {};
  prpc2.PrpcError = function() {
    function PrpcError(properties) {
      if (properties) {
        for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
          if (properties[keys[i]] != null)
            this[keys[i]] = properties[keys[i]];
      }
    }
    PrpcError.prototype.message = "";
    PrpcError.create = function create2(properties) {
      return new PrpcError(properties);
    };
    PrpcError.encode = function encode(message, writer) {
      if (!writer)
        writer = $Writer.create();
      if (message.message != null && Object.hasOwnProperty.call(message, "message"))
        writer.uint32(10).string(message.message);
      return writer;
    };
    PrpcError.encodeDelimited = function encodeDelimited(message, writer) {
      return this.encode(message, writer).ldelim();
    };
    PrpcError.decode = function decode(reader, length) {
      if (!(reader instanceof $Reader))
        reader = $Reader.create(reader);
      let end = length === void 0 ? reader.len : reader.pos + length, message = new $root.prpc.PrpcError();
      while (reader.pos < end) {
        let tag = reader.uint32();
        switch (tag >>> 3) {
          case 1:
            message.message = reader.string();
            break;
          default:
            reader.skipType(tag & 7);
            break;
        }
      }
      return message;
    };
    PrpcError.decodeDelimited = function decodeDelimited(reader) {
      if (!(reader instanceof $Reader))
        reader = new $Reader(reader);
      return this.decode(reader, reader.uint32());
    };
    PrpcError.verify = function verify(message) {
      if (typeof message !== "object" || message === null)
        return "object expected";
      if (message.message != null && message.hasOwnProperty("message")) {
        if (!$util.isString(message.message))
          return "message: string expected";
      }
      return null;
    };
    PrpcError.fromObject = function fromObject(object) {
      if (object instanceof $root.prpc.PrpcError)
        return object;
      let message = new $root.prpc.PrpcError();
      if (object.message != null)
        message.message = String(object.message);
      return message;
    };
    PrpcError.toObject = function toObject(message, options) {
      if (!options)
        options = {};
      let object = {};
      if (options.defaults)
        object.message = "";
      if (message.message != null && message.hasOwnProperty("message"))
        object.message = message.message;
      return object;
    };
    PrpcError.prototype.toJSON = function toJSON() {
      return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
    };
    return PrpcError;
  }();
  return prpc2;
})();
var pruntime_rpc = $root.pruntime_rpc = (() => {
  const pruntime_rpc2 = {};
  pruntime_rpc2.PhactoryAPI = function() {
    function PhactoryAPI(rpcImpl, requestDelimited, responseDelimited) {
      $protobuf.rpc.Service.call(
        this,
        rpcImpl,
        requestDelimited,
        responseDelimited
      );
    }
    (PhactoryAPI.prototype = Object.create(
      $protobuf.rpc.Service.prototype
    )).constructor = PhactoryAPI;
    PhactoryAPI.create = function create2(rpcImpl, requestDelimited, responseDelimited) {
      return new this(rpcImpl, requestDelimited, responseDelimited);
    };
    Object.defineProperty(
      PhactoryAPI.prototype.getInfo = function getInfo(request, callback) {
        return this.rpcCall(
          getInfo,
          $root.google.protobuf.Empty,
          $root.pruntime_rpc.PhactoryInfo,
          request,
          callback
        );
      },
      "name",
      { value: "GetInfo" }
    );
    Object.defineProperty(
      PhactoryAPI.prototype.syncHeader = function syncHeader(request, callback) {
        return this.rpcCall(
          syncHeader,
          $root.pruntime_rpc.HeadersToSync,
          $root.pruntime_rpc.SyncedTo,
          request,
          callback
        );
      },
      "name",
      { value: "SyncHeader" }
    );
    Object.defineProperty(
      PhactoryAPI.prototype.syncParaHeader = function syncParaHeader(request, callback) {
        return this.rpcCall(
          syncParaHeader,
          $root.pruntime_rpc.ParaHeadersToSync,
          $root.pruntime_rpc.SyncedTo,
          request,
          callback
        );
      },
      "name",
      { value: "SyncParaHeader" }
    );
    Object.defineProperty(
      PhactoryAPI.prototype.syncCombinedHeaders = function syncCombinedHeaders(request, callback) {
        return this.rpcCall(
          syncCombinedHeaders,
          $root.pruntime_rpc.CombinedHeadersToSync,
          $root.pruntime_rpc.HeadersSyncedTo,
          request,
          callback
        );
      },
      "name",
      { value: "SyncCombinedHeaders" }
    );
    Object.defineProperty(
      PhactoryAPI.prototype.dispatchBlocks = function dispatchBlocks(request, callback) {
        return this.rpcCall(
          dispatchBlocks,
          $root.pruntime_rpc.Blocks,
          $root.pruntime_rpc.SyncedTo,
          request,
          callback
        );
      },
      "name",
      { value: "DispatchBlocks" }
    );
    Object.defineProperty(
      PhactoryAPI.prototype.initRuntime = function initRuntime(request, callback) {
        return this.rpcCall(
          initRuntime,
          $root.pruntime_rpc.InitRuntimeRequest,
          $root.pruntime_rpc.InitRuntimeResponse,
          request,
          callback
        );
      },
      "name",
      { value: "InitRuntime" }
    );
    Object.defineProperty(
      PhactoryAPI.prototype.getRuntimeInfo = function getRuntimeInfo(request, callback) {
        return this.rpcCall(
          getRuntimeInfo,
          $root.pruntime_rpc.GetRuntimeInfoRequest,
          $root.pruntime_rpc.InitRuntimeResponse,
          request,
          callback
        );
      },
      "name",
      { value: "GetRuntimeInfo" }
    );
    Object.defineProperty(
      PhactoryAPI.prototype.getEgressMessages = function getEgressMessages(request, callback) {
        return this.rpcCall(
          getEgressMessages,
          $root.google.protobuf.Empty,
          $root.pruntime_rpc.GetEgressMessagesResponse,
          request,
          callback
        );
      },
      "name",
      { value: "GetEgressMessages" }
    );
    Object.defineProperty(
      PhactoryAPI.prototype.contractQuery = function contractQuery(request, callback) {
        return this.rpcCall(
          contractQuery,
          $root.pruntime_rpc.ContractQueryRequest,
          $root.pruntime_rpc.ContractQueryResponse,
          request,
          callback
        );
      },
      "name",
      { value: "ContractQuery" }
    );
    Object.defineProperty(
      PhactoryAPI.prototype.getWorkerState = function getWorkerState(request, callback) {
        return this.rpcCall(
          getWorkerState,
          $root.pruntime_rpc.GetWorkerStateRequest,
          $root.pruntime_rpc.WorkerState,
          request,
          callback
        );
      },
      "name",
      { value: "GetWorkerState" }
    );
    Object.defineProperty(
      PhactoryAPI.prototype.addEndpoint = function addEndpoint(request, callback) {
        return this.rpcCall(
          addEndpoint,
          $root.pruntime_rpc.AddEndpointRequest,
          $root.pruntime_rpc.GetEndpointResponse,
          request,
          callback
        );
      },
      "name",
      { value: "AddEndpoint" }
    );
    Object.defineProperty(
      PhactoryAPI.prototype.refreshEndpointSigningTime = function refreshEndpointSigningTime(request, callback) {
        return this.rpcCall(
          refreshEndpointSigningTime,
          $root.google.protobuf.Empty,
          $root.pruntime_rpc.GetEndpointResponse,
          request,
          callback
        );
      },
      "name",
      { value: "RefreshEndpointSigningTime" }
    );
    Object.defineProperty(
      PhactoryAPI.prototype.getEndpointInfo = function getEndpointInfo(request, callback) {
        return this.rpcCall(
          getEndpointInfo,
          $root.google.protobuf.Empty,
          $root.pruntime_rpc.GetEndpointResponse,
          request,
          callback
        );
      },
      "name",
      { value: "GetEndpointInfo" }
    );
    Object.defineProperty(
      PhactoryAPI.prototype.signEndpointInfo = function signEndpointInfo(request, callback) {
        return this.rpcCall(
          signEndpointInfo,
          $root.pruntime_rpc.SignEndpointsRequest,
          $root.pruntime_rpc.GetEndpointResponse,
          request,
          callback
        );
      },
      "name",
      { value: "SignEndpointInfo" }
    );
    Object.defineProperty(
      PhactoryAPI.prototype.derivePhalaI2pKey = function derivePhalaI2pKey(request, callback) {
        return this.rpcCall(
          derivePhalaI2pKey,
          $root.google.protobuf.Empty,
          $root.pruntime_rpc.DerivePhalaI2pKeyResponse,
          request,
          callback
        );
      },
      "name",
      { value: "DerivePhalaI2pKey" }
    );
    Object.defineProperty(
      PhactoryAPI.prototype.echo = function echo(request, callback) {
        return this.rpcCall(
          echo,
          $root.pruntime_rpc.EchoMessage,
          $root.pruntime_rpc.EchoMessage,
          request,
          callback
        );
      },
      "name",
      { value: "Echo" }
    );
    Object.defineProperty(
      PhactoryAPI.prototype.handoverCreateChallenge = function handoverCreateChallenge(request, callback) {
        return this.rpcCall(
          handoverCreateChallenge,
          $root.google.protobuf.Empty,
          $root.pruntime_rpc.HandoverChallenge,
          request,
          callback
        );
      },
      "name",
      { value: "HandoverCreateChallenge" }
    );
    Object.defineProperty(
      PhactoryAPI.prototype.handoverStart = function handoverStart(request, callback) {
        return this.rpcCall(
          handoverStart,
          $root.pruntime_rpc.HandoverChallengeResponse,
          $root.pruntime_rpc.HandoverWorkerKey,
          request,
          callback
        );
      },
      "name",
      { value: "HandoverStart" }
    );
    Object.defineProperty(
      PhactoryAPI.prototype.handoverAcceptChallenge = function handoverAcceptChallenge(request, callback) {
        return this.rpcCall(
          handoverAcceptChallenge,
          $root.pruntime_rpc.HandoverChallenge,
          $root.pruntime_rpc.HandoverChallengeResponse,
          request,
          callback
        );
      },
      "name",
      { value: "HandoverAcceptChallenge" }
    );
    Object.defineProperty(
      PhactoryAPI.prototype.handoverReceive = function handoverReceive(request, callback) {
        return this.rpcCall(
          handoverReceive,
          $root.pruntime_rpc.HandoverWorkerKey,
          $root.google.protobuf.Empty,
          request,
          callback
        );
      },
      "name",
      { value: "HandoverReceive" }
    );
    Object.defineProperty(
      PhactoryAPI.prototype.configNetwork = function configNetwork(request, callback) {
        return this.rpcCall(
          configNetwork,
          $root.pruntime_rpc.NetworkConfig,
          $root.google.protobuf.Empty,
          request,
          callback
        );
      },
      "name",
      { value: "ConfigNetwork" }
    );
    Object.defineProperty(
      PhactoryAPI.prototype.httpFetch = function httpFetch(request, callback) {
        return this.rpcCall(
          httpFetch,
          $root.pruntime_rpc.HttpRequest,
          $root.pruntime_rpc.HttpResponse,
          request,
          callback
        );
      },
      "name",
      { value: "HttpFetch" }
    );
    return PhactoryAPI;
  }();
  pruntime_rpc2.PhactoryInfo = function() {
    function PhactoryInfo(properties) {
      if (properties) {
        for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
          if (properties[keys[i]] != null)
            this[keys[i]] = properties[keys[i]];
      }
    }
    PhactoryInfo.prototype.initialized = false;
    PhactoryInfo.prototype.registered = false;
    PhactoryInfo.prototype.genesisBlockHash = null;
    PhactoryInfo.prototype.publicKey = null;
    PhactoryInfo.prototype.ecdhPublicKey = null;
    PhactoryInfo.prototype.headernum = 0;
    PhactoryInfo.prototype.paraHeadernum = 0;
    PhactoryInfo.prototype.blocknum = 0;
    PhactoryInfo.prototype.stateRoot = "";
    PhactoryInfo.prototype.devMode = false;
    PhactoryInfo.prototype.pendingMessages = $util.Long ? $util.Long.fromBits(0, 0, true) : 0;
    PhactoryInfo.prototype.score = $util.Long ? $util.Long.fromBits(0, 0, true) : 0;
    PhactoryInfo.prototype.gatekeeper = null;
    PhactoryInfo.prototype.version = "";
    PhactoryInfo.prototype.gitRevision = "";
    PhactoryInfo.prototype.runningSideTasks = $util.Long ? $util.Long.fromBits(0, 0, true) : 0;
    PhactoryInfo.prototype.memoryUsage = null;
    PhactoryInfo.prototype.waitingForParaheaders = false;
    PhactoryInfo.prototype.networkStatus = null;
    PhactoryInfo.prototype.system = null;
    let $oneOfFields;
    Object.defineProperty(PhactoryInfo.prototype, "_genesisBlockHash", {
      get: $util.oneOfGetter($oneOfFields = ["genesisBlockHash"]),
      set: $util.oneOfSetter($oneOfFields)
    });
    Object.defineProperty(PhactoryInfo.prototype, "_publicKey", {
      get: $util.oneOfGetter($oneOfFields = ["publicKey"]),
      set: $util.oneOfSetter($oneOfFields)
    });
    Object.defineProperty(PhactoryInfo.prototype, "_ecdhPublicKey", {
      get: $util.oneOfGetter($oneOfFields = ["ecdhPublicKey"]),
      set: $util.oneOfSetter($oneOfFields)
    });
    PhactoryInfo.create = function create2(properties) {
      return new PhactoryInfo(properties);
    };
    PhactoryInfo.encode = function encode(message, writer) {
      if (!writer)
        writer = $Writer.create();
      if (message.initialized != null && Object.hasOwnProperty.call(message, "initialized"))
        writer.uint32(8).bool(message.initialized);
      if (message.registered != null && Object.hasOwnProperty.call(message, "registered"))
        writer.uint32(16).bool(message.registered);
      if (message.genesisBlockHash != null && Object.hasOwnProperty.call(message, "genesisBlockHash"))
        writer.uint32(34).string(message.genesisBlockHash);
      if (message.publicKey != null && Object.hasOwnProperty.call(message, "publicKey"))
        writer.uint32(42).string(message.publicKey);
      if (message.ecdhPublicKey != null && Object.hasOwnProperty.call(message, "ecdhPublicKey"))
        writer.uint32(50).string(message.ecdhPublicKey);
      if (message.headernum != null && Object.hasOwnProperty.call(message, "headernum"))
        writer.uint32(56).uint32(message.headernum);
      if (message.paraHeadernum != null && Object.hasOwnProperty.call(message, "paraHeadernum"))
        writer.uint32(64).uint32(message.paraHeadernum);
      if (message.blocknum != null && Object.hasOwnProperty.call(message, "blocknum"))
        writer.uint32(72).uint32(message.blocknum);
      if (message.stateRoot != null && Object.hasOwnProperty.call(message, "stateRoot"))
        writer.uint32(82).string(message.stateRoot);
      if (message.devMode != null && Object.hasOwnProperty.call(message, "devMode"))
        writer.uint32(88).bool(message.devMode);
      if (message.pendingMessages != null && Object.hasOwnProperty.call(message, "pendingMessages"))
        writer.uint32(96).uint64(message.pendingMessages);
      if (message.score != null && Object.hasOwnProperty.call(message, "score"))
        writer.uint32(104).uint64(message.score);
      if (message.gatekeeper != null && Object.hasOwnProperty.call(message, "gatekeeper"))
        $root.pruntime_rpc.GatekeeperStatus.encode(
          message.gatekeeper,
          writer.uint32(114).fork()
        ).ldelim();
      if (message.version != null && Object.hasOwnProperty.call(message, "version"))
        writer.uint32(122).string(message.version);
      if (message.gitRevision != null && Object.hasOwnProperty.call(message, "gitRevision"))
        writer.uint32(130).string(message.gitRevision);
      if (message.runningSideTasks != null && Object.hasOwnProperty.call(message, "runningSideTasks"))
        writer.uint32(136).uint64(message.runningSideTasks);
      if (message.memoryUsage != null && Object.hasOwnProperty.call(message, "memoryUsage"))
        $root.pruntime_rpc.MemoryUsage.encode(
          message.memoryUsage,
          writer.uint32(146).fork()
        ).ldelim();
      if (message.waitingForParaheaders != null && Object.hasOwnProperty.call(message, "waitingForParaheaders"))
        writer.uint32(168).bool(message.waitingForParaheaders);
      if (message.networkStatus != null && Object.hasOwnProperty.call(message, "networkStatus"))
        $root.pruntime_rpc.NetworkStatus.encode(
          message.networkStatus,
          writer.uint32(178).fork()
        ).ldelim();
      if (message.system != null && Object.hasOwnProperty.call(message, "system"))
        $root.pruntime_rpc.SystemInfo.encode(
          message.system,
          writer.uint32(186).fork()
        ).ldelim();
      return writer;
    };
    PhactoryInfo.encodeDelimited = function encodeDelimited(message, writer) {
      return this.encode(message, writer).ldelim();
    };
    PhactoryInfo.decode = function decode(reader, length) {
      if (!(reader instanceof $Reader))
        reader = $Reader.create(reader);
      let end = length === void 0 ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.PhactoryInfo();
      while (reader.pos < end) {
        let tag = reader.uint32();
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
            message.gatekeeper = $root.pruntime_rpc.GatekeeperStatus.decode(
              reader,
              reader.uint32()
            );
            break;
          case 15:
            message.version = reader.string();
            break;
          case 16:
            message.gitRevision = reader.string();
            break;
          case 17:
            message.runningSideTasks = reader.uint64();
            break;
          case 18:
            message.memoryUsage = $root.pruntime_rpc.MemoryUsage.decode(
              reader,
              reader.uint32()
            );
            break;
          case 21:
            message.waitingForParaheaders = reader.bool();
            break;
          case 22:
            message.networkStatus = $root.pruntime_rpc.NetworkStatus.decode(
              reader,
              reader.uint32()
            );
            break;
          case 23:
            message.system = $root.pruntime_rpc.SystemInfo.decode(
              reader,
              reader.uint32()
            );
            break;
          default:
            reader.skipType(tag & 7);
            break;
        }
      }
      return message;
    };
    PhactoryInfo.decodeDelimited = function decodeDelimited(reader) {
      if (!(reader instanceof $Reader))
        reader = new $Reader(reader);
      return this.decode(reader, reader.uint32());
    };
    PhactoryInfo.verify = function verify(message) {
      if (typeof message !== "object" || message === null)
        return "object expected";
      let properties = {};
      if (message.initialized != null && message.hasOwnProperty("initialized")) {
        if (typeof message.initialized !== "boolean")
          return "initialized: boolean expected";
      }
      if (message.registered != null && message.hasOwnProperty("registered")) {
        if (typeof message.registered !== "boolean")
          return "registered: boolean expected";
      }
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
      if (message.headernum != null && message.hasOwnProperty("headernum")) {
        if (!$util.isInteger(message.headernum))
          return "headernum: integer expected";
      }
      if (message.paraHeadernum != null && message.hasOwnProperty("paraHeadernum")) {
        if (!$util.isInteger(message.paraHeadernum))
          return "paraHeadernum: integer expected";
      }
      if (message.blocknum != null && message.hasOwnProperty("blocknum")) {
        if (!$util.isInteger(message.blocknum))
          return "blocknum: integer expected";
      }
      if (message.stateRoot != null && message.hasOwnProperty("stateRoot")) {
        if (!$util.isString(message.stateRoot))
          return "stateRoot: string expected";
      }
      if (message.devMode != null && message.hasOwnProperty("devMode")) {
        if (typeof message.devMode !== "boolean")
          return "devMode: boolean expected";
      }
      if (message.pendingMessages != null && message.hasOwnProperty("pendingMessages")) {
        if (!$util.isInteger(message.pendingMessages) && !(message.pendingMessages && $util.isInteger(message.pendingMessages.low) && $util.isInteger(message.pendingMessages.high)))
          return "pendingMessages: integer|Long expected";
      }
      if (message.score != null && message.hasOwnProperty("score")) {
        if (!$util.isInteger(message.score) && !(message.score && $util.isInteger(message.score.low) && $util.isInteger(message.score.high)))
          return "score: integer|Long expected";
      }
      if (message.gatekeeper != null && message.hasOwnProperty("gatekeeper")) {
        let error = $root.pruntime_rpc.GatekeeperStatus.verify(
          message.gatekeeper
        );
        if (error)
          return "gatekeeper." + error;
      }
      if (message.version != null && message.hasOwnProperty("version")) {
        if (!$util.isString(message.version))
          return "version: string expected";
      }
      if (message.gitRevision != null && message.hasOwnProperty("gitRevision")) {
        if (!$util.isString(message.gitRevision))
          return "gitRevision: string expected";
      }
      if (message.runningSideTasks != null && message.hasOwnProperty("runningSideTasks")) {
        if (!$util.isInteger(message.runningSideTasks) && !(message.runningSideTasks && $util.isInteger(message.runningSideTasks.low) && $util.isInteger(message.runningSideTasks.high)))
          return "runningSideTasks: integer|Long expected";
      }
      if (message.memoryUsage != null && message.hasOwnProperty("memoryUsage")) {
        let error = $root.pruntime_rpc.MemoryUsage.verify(message.memoryUsage);
        if (error)
          return "memoryUsage." + error;
      }
      if (message.waitingForParaheaders != null && message.hasOwnProperty("waitingForParaheaders")) {
        if (typeof message.waitingForParaheaders !== "boolean")
          return "waitingForParaheaders: boolean expected";
      }
      if (message.networkStatus != null && message.hasOwnProperty("networkStatus")) {
        let error = $root.pruntime_rpc.NetworkStatus.verify(
          message.networkStatus
        );
        if (error)
          return "networkStatus." + error;
      }
      if (message.system != null && message.hasOwnProperty("system")) {
        let error = $root.pruntime_rpc.SystemInfo.verify(message.system);
        if (error)
          return "system." + error;
      }
      return null;
    };
    PhactoryInfo.fromObject = function fromObject(object) {
      if (object instanceof $root.pruntime_rpc.PhactoryInfo)
        return object;
      let message = new $root.pruntime_rpc.PhactoryInfo();
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
      if (object.pendingMessages != null) {
        if ($util.Long)
          (message.pendingMessages = $util.Long.fromValue(
            object.pendingMessages
          )).unsigned = true;
        else if (typeof object.pendingMessages === "string")
          message.pendingMessages = parseInt(object.pendingMessages, 10);
        else if (typeof object.pendingMessages === "number")
          message.pendingMessages = object.pendingMessages;
        else if (typeof object.pendingMessages === "object")
          message.pendingMessages = new $util.LongBits(
            object.pendingMessages.low >>> 0,
            object.pendingMessages.high >>> 0
          ).toNumber(true);
      }
      if (object.score != null) {
        if ($util.Long)
          (message.score = $util.Long.fromValue(object.score)).unsigned = true;
        else if (typeof object.score === "string")
          message.score = parseInt(object.score, 10);
        else if (typeof object.score === "number")
          message.score = object.score;
        else if (typeof object.score === "object")
          message.score = new $util.LongBits(
            object.score.low >>> 0,
            object.score.high >>> 0
          ).toNumber(true);
      }
      if (object.gatekeeper != null) {
        if (typeof object.gatekeeper !== "object")
          throw TypeError(
            ".pruntime_rpc.PhactoryInfo.gatekeeper: object expected"
          );
        message.gatekeeper = $root.pruntime_rpc.GatekeeperStatus.fromObject(
          object.gatekeeper
        );
      }
      if (object.version != null)
        message.version = String(object.version);
      if (object.gitRevision != null)
        message.gitRevision = String(object.gitRevision);
      if (object.runningSideTasks != null) {
        if ($util.Long)
          (message.runningSideTasks = $util.Long.fromValue(
            object.runningSideTasks
          )).unsigned = true;
        else if (typeof object.runningSideTasks === "string")
          message.runningSideTasks = parseInt(object.runningSideTasks, 10);
        else if (typeof object.runningSideTasks === "number")
          message.runningSideTasks = object.runningSideTasks;
        else if (typeof object.runningSideTasks === "object")
          message.runningSideTasks = new $util.LongBits(
            object.runningSideTasks.low >>> 0,
            object.runningSideTasks.high >>> 0
          ).toNumber(true);
      }
      if (object.memoryUsage != null) {
        if (typeof object.memoryUsage !== "object")
          throw TypeError(
            ".pruntime_rpc.PhactoryInfo.memoryUsage: object expected"
          );
        message.memoryUsage = $root.pruntime_rpc.MemoryUsage.fromObject(
          object.memoryUsage
        );
      }
      if (object.waitingForParaheaders != null)
        message.waitingForParaheaders = Boolean(object.waitingForParaheaders);
      if (object.networkStatus != null) {
        if (typeof object.networkStatus !== "object")
          throw TypeError(
            ".pruntime_rpc.PhactoryInfo.networkStatus: object expected"
          );
        message.networkStatus = $root.pruntime_rpc.NetworkStatus.fromObject(
          object.networkStatus
        );
      }
      if (object.system != null) {
        if (typeof object.system !== "object")
          throw TypeError(".pruntime_rpc.PhactoryInfo.system: object expected");
        message.system = $root.pruntime_rpc.SystemInfo.fromObject(
          object.system
        );
      }
      return message;
    };
    PhactoryInfo.toObject = function toObject(message, options) {
      if (!options)
        options = {};
      let object = {};
      if (options.defaults) {
        object.initialized = false;
        object.registered = false;
        object.headernum = 0;
        object.paraHeadernum = 0;
        object.blocknum = 0;
        object.stateRoot = "";
        object.devMode = false;
        if ($util.Long) {
          let long = new $util.Long(0, 0, true);
          object.pendingMessages = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : long;
        } else
          object.pendingMessages = options.longs === String ? "0" : 0;
        if ($util.Long) {
          let long = new $util.Long(0, 0, true);
          object.score = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : long;
        } else
          object.score = options.longs === String ? "0" : 0;
        object.gatekeeper = null;
        object.version = "";
        object.gitRevision = "";
        if ($util.Long) {
          let long = new $util.Long(0, 0, true);
          object.runningSideTasks = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : long;
        } else
          object.runningSideTasks = options.longs === String ? "0" : 0;
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
          object.pendingMessages = options.longs === String ? $util.Long.prototype.toString.call(message.pendingMessages) : options.longs === Number ? new $util.LongBits(
            message.pendingMessages.low >>> 0,
            message.pendingMessages.high >>> 0
          ).toNumber(true) : message.pendingMessages;
      if (message.score != null && message.hasOwnProperty("score"))
        if (typeof message.score === "number")
          object.score = options.longs === String ? String(message.score) : message.score;
        else
          object.score = options.longs === String ? $util.Long.prototype.toString.call(message.score) : options.longs === Number ? new $util.LongBits(
            message.score.low >>> 0,
            message.score.high >>> 0
          ).toNumber(true) : message.score;
      if (message.gatekeeper != null && message.hasOwnProperty("gatekeeper"))
        object.gatekeeper = $root.pruntime_rpc.GatekeeperStatus.toObject(
          message.gatekeeper,
          options
        );
      if (message.version != null && message.hasOwnProperty("version"))
        object.version = message.version;
      if (message.gitRevision != null && message.hasOwnProperty("gitRevision"))
        object.gitRevision = message.gitRevision;
      if (message.runningSideTasks != null && message.hasOwnProperty("runningSideTasks"))
        if (typeof message.runningSideTasks === "number")
          object.runningSideTasks = options.longs === String ? String(message.runningSideTasks) : message.runningSideTasks;
        else
          object.runningSideTasks = options.longs === String ? $util.Long.prototype.toString.call(message.runningSideTasks) : options.longs === Number ? new $util.LongBits(
            message.runningSideTasks.low >>> 0,
            message.runningSideTasks.high >>> 0
          ).toNumber(true) : message.runningSideTasks;
      if (message.memoryUsage != null && message.hasOwnProperty("memoryUsage"))
        object.memoryUsage = $root.pruntime_rpc.MemoryUsage.toObject(
          message.memoryUsage,
          options
        );
      if (message.waitingForParaheaders != null && message.hasOwnProperty("waitingForParaheaders"))
        object.waitingForParaheaders = message.waitingForParaheaders;
      if (message.networkStatus != null && message.hasOwnProperty("networkStatus"))
        object.networkStatus = $root.pruntime_rpc.NetworkStatus.toObject(
          message.networkStatus,
          options
        );
      if (message.system != null && message.hasOwnProperty("system"))
        object.system = $root.pruntime_rpc.SystemInfo.toObject(
          message.system,
          options
        );
      return object;
    };
    PhactoryInfo.prototype.toJSON = function toJSON() {
      return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
    };
    return PhactoryInfo;
  }();
  pruntime_rpc2.SystemInfo = function() {
    function SystemInfo(properties) {
      if (properties) {
        for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
          if (properties[keys[i]] != null)
            this[keys[i]] = properties[keys[i]];
      }
    }
    SystemInfo.prototype.registered = false;
    SystemInfo.prototype.publicKey = "";
    SystemInfo.prototype.ecdhPublicKey = "";
    SystemInfo.prototype.gatekeeper = null;
    SystemInfo.prototype.numberOfClusters = $util.Long ? $util.Long.fromBits(0, 0, true) : 0;
    SystemInfo.prototype.numberOfContracts = $util.Long ? $util.Long.fromBits(0, 0, true) : 0;
    SystemInfo.prototype.consensusVersion = 0;
    SystemInfo.prototype.maxSupportedConsensusVersion = 0;
    SystemInfo.create = function create2(properties) {
      return new SystemInfo(properties);
    };
    SystemInfo.encode = function encode(message, writer) {
      if (!writer)
        writer = $Writer.create();
      if (message.registered != null && Object.hasOwnProperty.call(message, "registered"))
        writer.uint32(8).bool(message.registered);
      if (message.publicKey != null && Object.hasOwnProperty.call(message, "publicKey"))
        writer.uint32(18).string(message.publicKey);
      if (message.ecdhPublicKey != null && Object.hasOwnProperty.call(message, "ecdhPublicKey"))
        writer.uint32(26).string(message.ecdhPublicKey);
      if (message.gatekeeper != null && Object.hasOwnProperty.call(message, "gatekeeper"))
        $root.pruntime_rpc.GatekeeperStatus.encode(
          message.gatekeeper,
          writer.uint32(34).fork()
        ).ldelim();
      if (message.numberOfClusters != null && Object.hasOwnProperty.call(message, "numberOfClusters"))
        writer.uint32(40).uint64(message.numberOfClusters);
      if (message.numberOfContracts != null && Object.hasOwnProperty.call(message, "numberOfContracts"))
        writer.uint32(48).uint64(message.numberOfContracts);
      if (message.consensusVersion != null && Object.hasOwnProperty.call(message, "consensusVersion"))
        writer.uint32(56).uint32(message.consensusVersion);
      if (message.maxSupportedConsensusVersion != null && Object.hasOwnProperty.call(message, "maxSupportedConsensusVersion"))
        writer.uint32(64).uint32(message.maxSupportedConsensusVersion);
      return writer;
    };
    SystemInfo.encodeDelimited = function encodeDelimited(message, writer) {
      return this.encode(message, writer).ldelim();
    };
    SystemInfo.decode = function decode(reader, length) {
      if (!(reader instanceof $Reader))
        reader = $Reader.create(reader);
      let end = length === void 0 ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.SystemInfo();
      while (reader.pos < end) {
        let tag = reader.uint32();
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
            message.gatekeeper = $root.pruntime_rpc.GatekeeperStatus.decode(
              reader,
              reader.uint32()
            );
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
    SystemInfo.decodeDelimited = function decodeDelimited(reader) {
      if (!(reader instanceof $Reader))
        reader = new $Reader(reader);
      return this.decode(reader, reader.uint32());
    };
    SystemInfo.verify = function verify(message) {
      if (typeof message !== "object" || message === null)
        return "object expected";
      if (message.registered != null && message.hasOwnProperty("registered")) {
        if (typeof message.registered !== "boolean")
          return "registered: boolean expected";
      }
      if (message.publicKey != null && message.hasOwnProperty("publicKey")) {
        if (!$util.isString(message.publicKey))
          return "publicKey: string expected";
      }
      if (message.ecdhPublicKey != null && message.hasOwnProperty("ecdhPublicKey")) {
        if (!$util.isString(message.ecdhPublicKey))
          return "ecdhPublicKey: string expected";
      }
      if (message.gatekeeper != null && message.hasOwnProperty("gatekeeper")) {
        let error = $root.pruntime_rpc.GatekeeperStatus.verify(
          message.gatekeeper
        );
        if (error)
          return "gatekeeper." + error;
      }
      if (message.numberOfClusters != null && message.hasOwnProperty("numberOfClusters")) {
        if (!$util.isInteger(message.numberOfClusters) && !(message.numberOfClusters && $util.isInteger(message.numberOfClusters.low) && $util.isInteger(message.numberOfClusters.high)))
          return "numberOfClusters: integer|Long expected";
      }
      if (message.numberOfContracts != null && message.hasOwnProperty("numberOfContracts")) {
        if (!$util.isInteger(message.numberOfContracts) && !(message.numberOfContracts && $util.isInteger(message.numberOfContracts.low) && $util.isInteger(message.numberOfContracts.high)))
          return "numberOfContracts: integer|Long expected";
      }
      if (message.consensusVersion != null && message.hasOwnProperty("consensusVersion")) {
        if (!$util.isInteger(message.consensusVersion))
          return "consensusVersion: integer expected";
      }
      if (message.maxSupportedConsensusVersion != null && message.hasOwnProperty("maxSupportedConsensusVersion")) {
        if (!$util.isInteger(message.maxSupportedConsensusVersion))
          return "maxSupportedConsensusVersion: integer expected";
      }
      return null;
    };
    SystemInfo.fromObject = function fromObject(object) {
      if (object instanceof $root.pruntime_rpc.SystemInfo)
        return object;
      let message = new $root.pruntime_rpc.SystemInfo();
      if (object.registered != null)
        message.registered = Boolean(object.registered);
      if (object.publicKey != null)
        message.publicKey = String(object.publicKey);
      if (object.ecdhPublicKey != null)
        message.ecdhPublicKey = String(object.ecdhPublicKey);
      if (object.gatekeeper != null) {
        if (typeof object.gatekeeper !== "object")
          throw TypeError(
            ".pruntime_rpc.SystemInfo.gatekeeper: object expected"
          );
        message.gatekeeper = $root.pruntime_rpc.GatekeeperStatus.fromObject(
          object.gatekeeper
        );
      }
      if (object.numberOfClusters != null) {
        if ($util.Long)
          (message.numberOfClusters = $util.Long.fromValue(
            object.numberOfClusters
          )).unsigned = true;
        else if (typeof object.numberOfClusters === "string")
          message.numberOfClusters = parseInt(object.numberOfClusters, 10);
        else if (typeof object.numberOfClusters === "number")
          message.numberOfClusters = object.numberOfClusters;
        else if (typeof object.numberOfClusters === "object")
          message.numberOfClusters = new $util.LongBits(
            object.numberOfClusters.low >>> 0,
            object.numberOfClusters.high >>> 0
          ).toNumber(true);
      }
      if (object.numberOfContracts != null) {
        if ($util.Long)
          (message.numberOfContracts = $util.Long.fromValue(
            object.numberOfContracts
          )).unsigned = true;
        else if (typeof object.numberOfContracts === "string")
          message.numberOfContracts = parseInt(object.numberOfContracts, 10);
        else if (typeof object.numberOfContracts === "number")
          message.numberOfContracts = object.numberOfContracts;
        else if (typeof object.numberOfContracts === "object")
          message.numberOfContracts = new $util.LongBits(
            object.numberOfContracts.low >>> 0,
            object.numberOfContracts.high >>> 0
          ).toNumber(true);
      }
      if (object.consensusVersion != null)
        message.consensusVersion = object.consensusVersion >>> 0;
      if (object.maxSupportedConsensusVersion != null)
        message.maxSupportedConsensusVersion = object.maxSupportedConsensusVersion >>> 0;
      return message;
    };
    SystemInfo.toObject = function toObject(message, options) {
      if (!options)
        options = {};
      let object = {};
      if (options.defaults) {
        object.registered = false;
        object.publicKey = "";
        object.ecdhPublicKey = "";
        object.gatekeeper = null;
        if ($util.Long) {
          let long = new $util.Long(0, 0, true);
          object.numberOfClusters = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : long;
        } else
          object.numberOfClusters = options.longs === String ? "0" : 0;
        if ($util.Long) {
          let long = new $util.Long(0, 0, true);
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
        object.gatekeeper = $root.pruntime_rpc.GatekeeperStatus.toObject(
          message.gatekeeper,
          options
        );
      if (message.numberOfClusters != null && message.hasOwnProperty("numberOfClusters"))
        if (typeof message.numberOfClusters === "number")
          object.numberOfClusters = options.longs === String ? String(message.numberOfClusters) : message.numberOfClusters;
        else
          object.numberOfClusters = options.longs === String ? $util.Long.prototype.toString.call(message.numberOfClusters) : options.longs === Number ? new $util.LongBits(
            message.numberOfClusters.low >>> 0,
            message.numberOfClusters.high >>> 0
          ).toNumber(true) : message.numberOfClusters;
      if (message.numberOfContracts != null && message.hasOwnProperty("numberOfContracts"))
        if (typeof message.numberOfContracts === "number")
          object.numberOfContracts = options.longs === String ? String(message.numberOfContracts) : message.numberOfContracts;
        else
          object.numberOfContracts = options.longs === String ? $util.Long.prototype.toString.call(message.numberOfContracts) : options.longs === Number ? new $util.LongBits(
            message.numberOfContracts.low >>> 0,
            message.numberOfContracts.high >>> 0
          ).toNumber(true) : message.numberOfContracts;
      if (message.consensusVersion != null && message.hasOwnProperty("consensusVersion"))
        object.consensusVersion = message.consensusVersion;
      if (message.maxSupportedConsensusVersion != null && message.hasOwnProperty("maxSupportedConsensusVersion"))
        object.maxSupportedConsensusVersion = message.maxSupportedConsensusVersion;
      return object;
    };
    SystemInfo.prototype.toJSON = function toJSON() {
      return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
    };
    return SystemInfo;
  }();
  pruntime_rpc2.GatekeeperRole = function() {
    const valuesById = {}, values = Object.create(valuesById);
    values[valuesById[0] = "None"] = 0;
    values[valuesById[1] = "Dummy"] = 1;
    values[valuesById[2] = "Active"] = 2;
    return values;
  }();
  pruntime_rpc2.GatekeeperStatus = function() {
    function GatekeeperStatus(properties) {
      if (properties) {
        for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
          if (properties[keys[i]] != null)
            this[keys[i]] = properties[keys[i]];
      }
    }
    GatekeeperStatus.prototype.role = 0;
    GatekeeperStatus.prototype.masterPublicKey = "";
    GatekeeperStatus.create = function create2(properties) {
      return new GatekeeperStatus(properties);
    };
    GatekeeperStatus.encode = function encode(message, writer) {
      if (!writer)
        writer = $Writer.create();
      if (message.role != null && Object.hasOwnProperty.call(message, "role"))
        writer.uint32(8).int32(message.role);
      if (message.masterPublicKey != null && Object.hasOwnProperty.call(message, "masterPublicKey"))
        writer.uint32(18).string(message.masterPublicKey);
      return writer;
    };
    GatekeeperStatus.encodeDelimited = function encodeDelimited(message, writer) {
      return this.encode(message, writer).ldelim();
    };
    GatekeeperStatus.decode = function decode(reader, length) {
      if (!(reader instanceof $Reader))
        reader = $Reader.create(reader);
      let end = length === void 0 ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.GatekeeperStatus();
      while (reader.pos < end) {
        let tag = reader.uint32();
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
    GatekeeperStatus.decodeDelimited = function decodeDelimited(reader) {
      if (!(reader instanceof $Reader))
        reader = new $Reader(reader);
      return this.decode(reader, reader.uint32());
    };
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
      if (message.masterPublicKey != null && message.hasOwnProperty("masterPublicKey")) {
        if (!$util.isString(message.masterPublicKey))
          return "masterPublicKey: string expected";
      }
      return null;
    };
    GatekeeperStatus.fromObject = function fromObject(object) {
      if (object instanceof $root.pruntime_rpc.GatekeeperStatus)
        return object;
      let message = new $root.pruntime_rpc.GatekeeperStatus();
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
    GatekeeperStatus.toObject = function toObject(message, options) {
      if (!options)
        options = {};
      let object = {};
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
    GatekeeperStatus.prototype.toJSON = function toJSON() {
      return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
    };
    return GatekeeperStatus;
  }();
  pruntime_rpc2.MemoryUsage = function() {
    function MemoryUsage(properties) {
      if (properties) {
        for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
          if (properties[keys[i]] != null)
            this[keys[i]] = properties[keys[i]];
      }
    }
    MemoryUsage.prototype.rustUsed = $util.Long ? $util.Long.fromBits(0, 0, true) : 0;
    MemoryUsage.prototype.rustPeakUsed = $util.Long ? $util.Long.fromBits(0, 0, true) : 0;
    MemoryUsage.prototype.totalPeakUsed = $util.Long ? $util.Long.fromBits(0, 0, true) : 0;
    MemoryUsage.create = function create2(properties) {
      return new MemoryUsage(properties);
    };
    MemoryUsage.encode = function encode(message, writer) {
      if (!writer)
        writer = $Writer.create();
      if (message.rustUsed != null && Object.hasOwnProperty.call(message, "rustUsed"))
        writer.uint32(8).uint64(message.rustUsed);
      if (message.rustPeakUsed != null && Object.hasOwnProperty.call(message, "rustPeakUsed"))
        writer.uint32(16).uint64(message.rustPeakUsed);
      if (message.totalPeakUsed != null && Object.hasOwnProperty.call(message, "totalPeakUsed"))
        writer.uint32(24).uint64(message.totalPeakUsed);
      return writer;
    };
    MemoryUsage.encodeDelimited = function encodeDelimited(message, writer) {
      return this.encode(message, writer).ldelim();
    };
    MemoryUsage.decode = function decode(reader, length) {
      if (!(reader instanceof $Reader))
        reader = $Reader.create(reader);
      let end = length === void 0 ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.MemoryUsage();
      while (reader.pos < end) {
        let tag = reader.uint32();
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
    MemoryUsage.decodeDelimited = function decodeDelimited(reader) {
      if (!(reader instanceof $Reader))
        reader = new $Reader(reader);
      return this.decode(reader, reader.uint32());
    };
    MemoryUsage.verify = function verify(message) {
      if (typeof message !== "object" || message === null)
        return "object expected";
      if (message.rustUsed != null && message.hasOwnProperty("rustUsed")) {
        if (!$util.isInteger(message.rustUsed) && !(message.rustUsed && $util.isInteger(message.rustUsed.low) && $util.isInteger(message.rustUsed.high)))
          return "rustUsed: integer|Long expected";
      }
      if (message.rustPeakUsed != null && message.hasOwnProperty("rustPeakUsed")) {
        if (!$util.isInteger(message.rustPeakUsed) && !(message.rustPeakUsed && $util.isInteger(message.rustPeakUsed.low) && $util.isInteger(message.rustPeakUsed.high)))
          return "rustPeakUsed: integer|Long expected";
      }
      if (message.totalPeakUsed != null && message.hasOwnProperty("totalPeakUsed")) {
        if (!$util.isInteger(message.totalPeakUsed) && !(message.totalPeakUsed && $util.isInteger(message.totalPeakUsed.low) && $util.isInteger(message.totalPeakUsed.high)))
          return "totalPeakUsed: integer|Long expected";
      }
      return null;
    };
    MemoryUsage.fromObject = function fromObject(object) {
      if (object instanceof $root.pruntime_rpc.MemoryUsage)
        return object;
      let message = new $root.pruntime_rpc.MemoryUsage();
      if (object.rustUsed != null) {
        if ($util.Long)
          (message.rustUsed = $util.Long.fromValue(
            object.rustUsed
          )).unsigned = true;
        else if (typeof object.rustUsed === "string")
          message.rustUsed = parseInt(object.rustUsed, 10);
        else if (typeof object.rustUsed === "number")
          message.rustUsed = object.rustUsed;
        else if (typeof object.rustUsed === "object")
          message.rustUsed = new $util.LongBits(
            object.rustUsed.low >>> 0,
            object.rustUsed.high >>> 0
          ).toNumber(true);
      }
      if (object.rustPeakUsed != null) {
        if ($util.Long)
          (message.rustPeakUsed = $util.Long.fromValue(
            object.rustPeakUsed
          )).unsigned = true;
        else if (typeof object.rustPeakUsed === "string")
          message.rustPeakUsed = parseInt(object.rustPeakUsed, 10);
        else if (typeof object.rustPeakUsed === "number")
          message.rustPeakUsed = object.rustPeakUsed;
        else if (typeof object.rustPeakUsed === "object")
          message.rustPeakUsed = new $util.LongBits(
            object.rustPeakUsed.low >>> 0,
            object.rustPeakUsed.high >>> 0
          ).toNumber(true);
      }
      if (object.totalPeakUsed != null) {
        if ($util.Long)
          (message.totalPeakUsed = $util.Long.fromValue(
            object.totalPeakUsed
          )).unsigned = true;
        else if (typeof object.totalPeakUsed === "string")
          message.totalPeakUsed = parseInt(object.totalPeakUsed, 10);
        else if (typeof object.totalPeakUsed === "number")
          message.totalPeakUsed = object.totalPeakUsed;
        else if (typeof object.totalPeakUsed === "object")
          message.totalPeakUsed = new $util.LongBits(
            object.totalPeakUsed.low >>> 0,
            object.totalPeakUsed.high >>> 0
          ).toNumber(true);
      }
      return message;
    };
    MemoryUsage.toObject = function toObject(message, options) {
      if (!options)
        options = {};
      let object = {};
      if (options.defaults) {
        if ($util.Long) {
          let long = new $util.Long(0, 0, true);
          object.rustUsed = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : long;
        } else
          object.rustUsed = options.longs === String ? "0" : 0;
        if ($util.Long) {
          let long = new $util.Long(0, 0, true);
          object.rustPeakUsed = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : long;
        } else
          object.rustPeakUsed = options.longs === String ? "0" : 0;
        if ($util.Long) {
          let long = new $util.Long(0, 0, true);
          object.totalPeakUsed = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : long;
        } else
          object.totalPeakUsed = options.longs === String ? "0" : 0;
      }
      if (message.rustUsed != null && message.hasOwnProperty("rustUsed"))
        if (typeof message.rustUsed === "number")
          object.rustUsed = options.longs === String ? String(message.rustUsed) : message.rustUsed;
        else
          object.rustUsed = options.longs === String ? $util.Long.prototype.toString.call(message.rustUsed) : options.longs === Number ? new $util.LongBits(
            message.rustUsed.low >>> 0,
            message.rustUsed.high >>> 0
          ).toNumber(true) : message.rustUsed;
      if (message.rustPeakUsed != null && message.hasOwnProperty("rustPeakUsed"))
        if (typeof message.rustPeakUsed === "number")
          object.rustPeakUsed = options.longs === String ? String(message.rustPeakUsed) : message.rustPeakUsed;
        else
          object.rustPeakUsed = options.longs === String ? $util.Long.prototype.toString.call(message.rustPeakUsed) : options.longs === Number ? new $util.LongBits(
            message.rustPeakUsed.low >>> 0,
            message.rustPeakUsed.high >>> 0
          ).toNumber(true) : message.rustPeakUsed;
      if (message.totalPeakUsed != null && message.hasOwnProperty("totalPeakUsed"))
        if (typeof message.totalPeakUsed === "number")
          object.totalPeakUsed = options.longs === String ? String(message.totalPeakUsed) : message.totalPeakUsed;
        else
          object.totalPeakUsed = options.longs === String ? $util.Long.prototype.toString.call(message.totalPeakUsed) : options.longs === Number ? new $util.LongBits(
            message.totalPeakUsed.low >>> 0,
            message.totalPeakUsed.high >>> 0
          ).toNumber(true) : message.totalPeakUsed;
      return object;
    };
    MemoryUsage.prototype.toJSON = function toJSON() {
      return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
    };
    return MemoryUsage;
  }();
  pruntime_rpc2.SyncedTo = function() {
    function SyncedTo(properties) {
      if (properties) {
        for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
          if (properties[keys[i]] != null)
            this[keys[i]] = properties[keys[i]];
      }
    }
    SyncedTo.prototype.syncedTo = 0;
    SyncedTo.create = function create2(properties) {
      return new SyncedTo(properties);
    };
    SyncedTo.encode = function encode(message, writer) {
      if (!writer)
        writer = $Writer.create();
      if (message.syncedTo != null && Object.hasOwnProperty.call(message, "syncedTo"))
        writer.uint32(8).uint32(message.syncedTo);
      return writer;
    };
    SyncedTo.encodeDelimited = function encodeDelimited(message, writer) {
      return this.encode(message, writer).ldelim();
    };
    SyncedTo.decode = function decode(reader, length) {
      if (!(reader instanceof $Reader))
        reader = $Reader.create(reader);
      let end = length === void 0 ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.SyncedTo();
      while (reader.pos < end) {
        let tag = reader.uint32();
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
    SyncedTo.decodeDelimited = function decodeDelimited(reader) {
      if (!(reader instanceof $Reader))
        reader = new $Reader(reader);
      return this.decode(reader, reader.uint32());
    };
    SyncedTo.verify = function verify(message) {
      if (typeof message !== "object" || message === null)
        return "object expected";
      if (message.syncedTo != null && message.hasOwnProperty("syncedTo")) {
        if (!$util.isInteger(message.syncedTo))
          return "syncedTo: integer expected";
      }
      return null;
    };
    SyncedTo.fromObject = function fromObject(object) {
      if (object instanceof $root.pruntime_rpc.SyncedTo)
        return object;
      let message = new $root.pruntime_rpc.SyncedTo();
      if (object.syncedTo != null)
        message.syncedTo = object.syncedTo >>> 0;
      return message;
    };
    SyncedTo.toObject = function toObject(message, options) {
      if (!options)
        options = {};
      let object = {};
      if (options.defaults)
        object.syncedTo = 0;
      if (message.syncedTo != null && message.hasOwnProperty("syncedTo"))
        object.syncedTo = message.syncedTo;
      return object;
    };
    SyncedTo.prototype.toJSON = function toJSON() {
      return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
    };
    return SyncedTo;
  }();
  pruntime_rpc2.HeadersToSync = function() {
    function HeadersToSync(properties) {
      if (properties) {
        for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
          if (properties[keys[i]] != null)
            this[keys[i]] = properties[keys[i]];
      }
    }
    HeadersToSync.prototype.encodedHeaders = $util.newBuffer([]);
    HeadersToSync.prototype.encodedAuthoritySetChange = null;
    let $oneOfFields;
    Object.defineProperty(
      HeadersToSync.prototype,
      "_encodedAuthoritySetChange",
      {
        get: $util.oneOfGetter($oneOfFields = ["encodedAuthoritySetChange"]),
        set: $util.oneOfSetter($oneOfFields)
      }
    );
    HeadersToSync.create = function create2(properties) {
      return new HeadersToSync(properties);
    };
    HeadersToSync.encode = function encode(message, writer) {
      if (!writer)
        writer = $Writer.create();
      if (message.encodedHeaders != null && Object.hasOwnProperty.call(message, "encodedHeaders"))
        writer.uint32(10).bytes(message.encodedHeaders);
      if (message.encodedAuthoritySetChange != null && Object.hasOwnProperty.call(message, "encodedAuthoritySetChange"))
        writer.uint32(18).bytes(message.encodedAuthoritySetChange);
      return writer;
    };
    HeadersToSync.encodeDelimited = function encodeDelimited(message, writer) {
      return this.encode(message, writer).ldelim();
    };
    HeadersToSync.decode = function decode(reader, length) {
      if (!(reader instanceof $Reader))
        reader = $Reader.create(reader);
      let end = length === void 0 ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.HeadersToSync();
      while (reader.pos < end) {
        let tag = reader.uint32();
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
    HeadersToSync.decodeDelimited = function decodeDelimited(reader) {
      if (!(reader instanceof $Reader))
        reader = new $Reader(reader);
      return this.decode(reader, reader.uint32());
    };
    HeadersToSync.verify = function verify(message) {
      if (typeof message !== "object" || message === null)
        return "object expected";
      let properties = {};
      if (message.encodedHeaders != null && message.hasOwnProperty("encodedHeaders")) {
        if (!(message.encodedHeaders && typeof message.encodedHeaders.length === "number" || $util.isString(message.encodedHeaders)))
          return "encodedHeaders: buffer expected";
      }
      if (message.encodedAuthoritySetChange != null && message.hasOwnProperty("encodedAuthoritySetChange")) {
        properties._encodedAuthoritySetChange = 1;
        if (!(message.encodedAuthoritySetChange && typeof message.encodedAuthoritySetChange.length === "number" || $util.isString(message.encodedAuthoritySetChange)))
          return "encodedAuthoritySetChange: buffer expected";
      }
      return null;
    };
    HeadersToSync.fromObject = function fromObject(object) {
      if (object instanceof $root.pruntime_rpc.HeadersToSync)
        return object;
      let message = new $root.pruntime_rpc.HeadersToSync();
      if (object.encodedHeaders != null) {
        if (typeof object.encodedHeaders === "string")
          $util.base64.decode(
            object.encodedHeaders,
            message.encodedHeaders = $util.newBuffer(
              $util.base64.length(object.encodedHeaders)
            ),
            0
          );
        else if (object.encodedHeaders.length)
          message.encodedHeaders = object.encodedHeaders;
      }
      if (object.encodedAuthoritySetChange != null) {
        if (typeof object.encodedAuthoritySetChange === "string")
          $util.base64.decode(
            object.encodedAuthoritySetChange,
            message.encodedAuthoritySetChange = $util.newBuffer(
              $util.base64.length(object.encodedAuthoritySetChange)
            ),
            0
          );
        else if (object.encodedAuthoritySetChange.length)
          message.encodedAuthoritySetChange = object.encodedAuthoritySetChange;
      }
      return message;
    };
    HeadersToSync.toObject = function toObject(message, options) {
      if (!options)
        options = {};
      let object = {};
      if (options.defaults)
        if (options.bytes === String)
          object.encodedHeaders = "";
        else {
          object.encodedHeaders = [];
          if (options.bytes !== Array)
            object.encodedHeaders = $util.newBuffer(object.encodedHeaders);
        }
      if (message.encodedHeaders != null && message.hasOwnProperty("encodedHeaders"))
        object.encodedHeaders = options.bytes === String ? $util.base64.encode(
          message.encodedHeaders,
          0,
          message.encodedHeaders.length
        ) : options.bytes === Array ? Array.prototype.slice.call(message.encodedHeaders) : message.encodedHeaders;
      if (message.encodedAuthoritySetChange != null && message.hasOwnProperty("encodedAuthoritySetChange")) {
        object.encodedAuthoritySetChange = options.bytes === String ? $util.base64.encode(
          message.encodedAuthoritySetChange,
          0,
          message.encodedAuthoritySetChange.length
        ) : options.bytes === Array ? Array.prototype.slice.call(message.encodedAuthoritySetChange) : message.encodedAuthoritySetChange;
        if (options.oneofs)
          object._encodedAuthoritySetChange = "encodedAuthoritySetChange";
      }
      return object;
    };
    HeadersToSync.prototype.toJSON = function toJSON() {
      return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
    };
    return HeadersToSync;
  }();
  pruntime_rpc2.ParaHeadersToSync = function() {
    function ParaHeadersToSync(properties) {
      this.proof = [];
      if (properties) {
        for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
          if (properties[keys[i]] != null)
            this[keys[i]] = properties[keys[i]];
      }
    }
    ParaHeadersToSync.prototype.encodedHeaders = $util.newBuffer([]);
    ParaHeadersToSync.prototype.proof = $util.emptyArray;
    ParaHeadersToSync.create = function create2(properties) {
      return new ParaHeadersToSync(properties);
    };
    ParaHeadersToSync.encode = function encode(message, writer) {
      if (!writer)
        writer = $Writer.create();
      if (message.encodedHeaders != null && Object.hasOwnProperty.call(message, "encodedHeaders"))
        writer.uint32(10).bytes(message.encodedHeaders);
      if (message.proof != null && message.proof.length)
        for (let i = 0; i < message.proof.length; ++i)
          writer.uint32(18).bytes(message.proof[i]);
      return writer;
    };
    ParaHeadersToSync.encodeDelimited = function encodeDelimited(message, writer) {
      return this.encode(message, writer).ldelim();
    };
    ParaHeadersToSync.decode = function decode(reader, length) {
      if (!(reader instanceof $Reader))
        reader = $Reader.create(reader);
      let end = length === void 0 ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.ParaHeadersToSync();
      while (reader.pos < end) {
        let tag = reader.uint32();
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
    ParaHeadersToSync.decodeDelimited = function decodeDelimited(reader) {
      if (!(reader instanceof $Reader))
        reader = new $Reader(reader);
      return this.decode(reader, reader.uint32());
    };
    ParaHeadersToSync.verify = function verify(message) {
      if (typeof message !== "object" || message === null)
        return "object expected";
      if (message.encodedHeaders != null && message.hasOwnProperty("encodedHeaders")) {
        if (!(message.encodedHeaders && typeof message.encodedHeaders.length === "number" || $util.isString(message.encodedHeaders)))
          return "encodedHeaders: buffer expected";
      }
      if (message.proof != null && message.hasOwnProperty("proof")) {
        if (!Array.isArray(message.proof))
          return "proof: array expected";
        for (let i = 0; i < message.proof.length; ++i)
          if (!(message.proof[i] && typeof message.proof[i].length === "number" || $util.isString(message.proof[i])))
            return "proof: buffer[] expected";
      }
      return null;
    };
    ParaHeadersToSync.fromObject = function fromObject(object) {
      if (object instanceof $root.pruntime_rpc.ParaHeadersToSync)
        return object;
      let message = new $root.pruntime_rpc.ParaHeadersToSync();
      if (object.encodedHeaders != null) {
        if (typeof object.encodedHeaders === "string")
          $util.base64.decode(
            object.encodedHeaders,
            message.encodedHeaders = $util.newBuffer(
              $util.base64.length(object.encodedHeaders)
            ),
            0
          );
        else if (object.encodedHeaders.length)
          message.encodedHeaders = object.encodedHeaders;
      }
      if (object.proof) {
        if (!Array.isArray(object.proof))
          throw TypeError(
            ".pruntime_rpc.ParaHeadersToSync.proof: array expected"
          );
        message.proof = [];
        for (let i = 0; i < object.proof.length; ++i)
          if (typeof object.proof[i] === "string")
            $util.base64.decode(
              object.proof[i],
              message.proof[i] = $util.newBuffer(
                $util.base64.length(object.proof[i])
              ),
              0
            );
          else if (object.proof[i].length)
            message.proof[i] = object.proof[i];
      }
      return message;
    };
    ParaHeadersToSync.toObject = function toObject(message, options) {
      if (!options)
        options = {};
      let object = {};
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
        object.encodedHeaders = options.bytes === String ? $util.base64.encode(
          message.encodedHeaders,
          0,
          message.encodedHeaders.length
        ) : options.bytes === Array ? Array.prototype.slice.call(message.encodedHeaders) : message.encodedHeaders;
      if (message.proof && message.proof.length) {
        object.proof = [];
        for (let j = 0; j < message.proof.length; ++j)
          object.proof[j] = options.bytes === String ? $util.base64.encode(
            message.proof[j],
            0,
            message.proof[j].length
          ) : options.bytes === Array ? Array.prototype.slice.call(message.proof[j]) : message.proof[j];
      }
      return object;
    };
    ParaHeadersToSync.prototype.toJSON = function toJSON() {
      return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
    };
    return ParaHeadersToSync;
  }();
  pruntime_rpc2.CombinedHeadersToSync = function() {
    function CombinedHeadersToSync(properties) {
      this.proof = [];
      if (properties) {
        for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
          if (properties[keys[i]] != null)
            this[keys[i]] = properties[keys[i]];
      }
    }
    CombinedHeadersToSync.prototype.encodedRelaychainHeaders = $util.newBuffer(
      []
    );
    CombinedHeadersToSync.prototype.authoritySetChange = null;
    CombinedHeadersToSync.prototype.encodedParachainHeaders = $util.newBuffer(
      []
    );
    CombinedHeadersToSync.prototype.proof = $util.emptyArray;
    let $oneOfFields;
    Object.defineProperty(
      CombinedHeadersToSync.prototype,
      "_authoritySetChange",
      {
        get: $util.oneOfGetter($oneOfFields = ["authoritySetChange"]),
        set: $util.oneOfSetter($oneOfFields)
      }
    );
    CombinedHeadersToSync.create = function create2(properties) {
      return new CombinedHeadersToSync(properties);
    };
    CombinedHeadersToSync.encode = function encode(message, writer) {
      if (!writer)
        writer = $Writer.create();
      if (message.encodedRelaychainHeaders != null && Object.hasOwnProperty.call(message, "encodedRelaychainHeaders"))
        writer.uint32(10).bytes(message.encodedRelaychainHeaders);
      if (message.authoritySetChange != null && Object.hasOwnProperty.call(message, "authoritySetChange"))
        writer.uint32(18).bytes(message.authoritySetChange);
      if (message.encodedParachainHeaders != null && Object.hasOwnProperty.call(message, "encodedParachainHeaders"))
        writer.uint32(26).bytes(message.encodedParachainHeaders);
      if (message.proof != null && message.proof.length)
        for (let i = 0; i < message.proof.length; ++i)
          writer.uint32(34).bytes(message.proof[i]);
      return writer;
    };
    CombinedHeadersToSync.encodeDelimited = function encodeDelimited(message, writer) {
      return this.encode(message, writer).ldelim();
    };
    CombinedHeadersToSync.decode = function decode(reader, length) {
      if (!(reader instanceof $Reader))
        reader = $Reader.create(reader);
      let end = length === void 0 ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.CombinedHeadersToSync();
      while (reader.pos < end) {
        let tag = reader.uint32();
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
    CombinedHeadersToSync.decodeDelimited = function decodeDelimited(reader) {
      if (!(reader instanceof $Reader))
        reader = new $Reader(reader);
      return this.decode(reader, reader.uint32());
    };
    CombinedHeadersToSync.verify = function verify(message) {
      if (typeof message !== "object" || message === null)
        return "object expected";
      let properties = {};
      if (message.encodedRelaychainHeaders != null && message.hasOwnProperty("encodedRelaychainHeaders")) {
        if (!(message.encodedRelaychainHeaders && typeof message.encodedRelaychainHeaders.length === "number" || $util.isString(message.encodedRelaychainHeaders)))
          return "encodedRelaychainHeaders: buffer expected";
      }
      if (message.authoritySetChange != null && message.hasOwnProperty("authoritySetChange")) {
        properties._authoritySetChange = 1;
        if (!(message.authoritySetChange && typeof message.authoritySetChange.length === "number" || $util.isString(message.authoritySetChange)))
          return "authoritySetChange: buffer expected";
      }
      if (message.encodedParachainHeaders != null && message.hasOwnProperty("encodedParachainHeaders")) {
        if (!(message.encodedParachainHeaders && typeof message.encodedParachainHeaders.length === "number" || $util.isString(message.encodedParachainHeaders)))
          return "encodedParachainHeaders: buffer expected";
      }
      if (message.proof != null && message.hasOwnProperty("proof")) {
        if (!Array.isArray(message.proof))
          return "proof: array expected";
        for (let i = 0; i < message.proof.length; ++i)
          if (!(message.proof[i] && typeof message.proof[i].length === "number" || $util.isString(message.proof[i])))
            return "proof: buffer[] expected";
      }
      return null;
    };
    CombinedHeadersToSync.fromObject = function fromObject(object) {
      if (object instanceof $root.pruntime_rpc.CombinedHeadersToSync)
        return object;
      let message = new $root.pruntime_rpc.CombinedHeadersToSync();
      if (object.encodedRelaychainHeaders != null) {
        if (typeof object.encodedRelaychainHeaders === "string")
          $util.base64.decode(
            object.encodedRelaychainHeaders,
            message.encodedRelaychainHeaders = $util.newBuffer(
              $util.base64.length(object.encodedRelaychainHeaders)
            ),
            0
          );
        else if (object.encodedRelaychainHeaders.length)
          message.encodedRelaychainHeaders = object.encodedRelaychainHeaders;
      }
      if (object.authoritySetChange != null) {
        if (typeof object.authoritySetChange === "string")
          $util.base64.decode(
            object.authoritySetChange,
            message.authoritySetChange = $util.newBuffer(
              $util.base64.length(object.authoritySetChange)
            ),
            0
          );
        else if (object.authoritySetChange.length)
          message.authoritySetChange = object.authoritySetChange;
      }
      if (object.encodedParachainHeaders != null) {
        if (typeof object.encodedParachainHeaders === "string")
          $util.base64.decode(
            object.encodedParachainHeaders,
            message.encodedParachainHeaders = $util.newBuffer(
              $util.base64.length(object.encodedParachainHeaders)
            ),
            0
          );
        else if (object.encodedParachainHeaders.length)
          message.encodedParachainHeaders = object.encodedParachainHeaders;
      }
      if (object.proof) {
        if (!Array.isArray(object.proof))
          throw TypeError(
            ".pruntime_rpc.CombinedHeadersToSync.proof: array expected"
          );
        message.proof = [];
        for (let i = 0; i < object.proof.length; ++i)
          if (typeof object.proof[i] === "string")
            $util.base64.decode(
              object.proof[i],
              message.proof[i] = $util.newBuffer(
                $util.base64.length(object.proof[i])
              ),
              0
            );
          else if (object.proof[i].length)
            message.proof[i] = object.proof[i];
      }
      return message;
    };
    CombinedHeadersToSync.toObject = function toObject(message, options) {
      if (!options)
        options = {};
      let object = {};
      if (options.arrays || options.defaults)
        object.proof = [];
      if (options.defaults) {
        if (options.bytes === String)
          object.encodedRelaychainHeaders = "";
        else {
          object.encodedRelaychainHeaders = [];
          if (options.bytes !== Array)
            object.encodedRelaychainHeaders = $util.newBuffer(
              object.encodedRelaychainHeaders
            );
        }
        if (options.bytes === String)
          object.encodedParachainHeaders = "";
        else {
          object.encodedParachainHeaders = [];
          if (options.bytes !== Array)
            object.encodedParachainHeaders = $util.newBuffer(
              object.encodedParachainHeaders
            );
        }
      }
      if (message.encodedRelaychainHeaders != null && message.hasOwnProperty("encodedRelaychainHeaders"))
        object.encodedRelaychainHeaders = options.bytes === String ? $util.base64.encode(
          message.encodedRelaychainHeaders,
          0,
          message.encodedRelaychainHeaders.length
        ) : options.bytes === Array ? Array.prototype.slice.call(message.encodedRelaychainHeaders) : message.encodedRelaychainHeaders;
      if (message.authoritySetChange != null && message.hasOwnProperty("authoritySetChange")) {
        object.authoritySetChange = options.bytes === String ? $util.base64.encode(
          message.authoritySetChange,
          0,
          message.authoritySetChange.length
        ) : options.bytes === Array ? Array.prototype.slice.call(message.authoritySetChange) : message.authoritySetChange;
        if (options.oneofs)
          object._authoritySetChange = "authoritySetChange";
      }
      if (message.encodedParachainHeaders != null && message.hasOwnProperty("encodedParachainHeaders"))
        object.encodedParachainHeaders = options.bytes === String ? $util.base64.encode(
          message.encodedParachainHeaders,
          0,
          message.encodedParachainHeaders.length
        ) : options.bytes === Array ? Array.prototype.slice.call(message.encodedParachainHeaders) : message.encodedParachainHeaders;
      if (message.proof && message.proof.length) {
        object.proof = [];
        for (let j = 0; j < message.proof.length; ++j)
          object.proof[j] = options.bytes === String ? $util.base64.encode(
            message.proof[j],
            0,
            message.proof[j].length
          ) : options.bytes === Array ? Array.prototype.slice.call(message.proof[j]) : message.proof[j];
      }
      return object;
    };
    CombinedHeadersToSync.prototype.toJSON = function toJSON() {
      return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
    };
    return CombinedHeadersToSync;
  }();
  pruntime_rpc2.HeadersSyncedTo = function() {
    function HeadersSyncedTo(properties) {
      if (properties) {
        for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
          if (properties[keys[i]] != null)
            this[keys[i]] = properties[keys[i]];
      }
    }
    HeadersSyncedTo.prototype.relaychainSyncedTo = 0;
    HeadersSyncedTo.prototype.parachainSyncedTo = 0;
    HeadersSyncedTo.create = function create2(properties) {
      return new HeadersSyncedTo(properties);
    };
    HeadersSyncedTo.encode = function encode(message, writer) {
      if (!writer)
        writer = $Writer.create();
      if (message.relaychainSyncedTo != null && Object.hasOwnProperty.call(message, "relaychainSyncedTo"))
        writer.uint32(8).uint32(message.relaychainSyncedTo);
      if (message.parachainSyncedTo != null && Object.hasOwnProperty.call(message, "parachainSyncedTo"))
        writer.uint32(16).uint32(message.parachainSyncedTo);
      return writer;
    };
    HeadersSyncedTo.encodeDelimited = function encodeDelimited(message, writer) {
      return this.encode(message, writer).ldelim();
    };
    HeadersSyncedTo.decode = function decode(reader, length) {
      if (!(reader instanceof $Reader))
        reader = $Reader.create(reader);
      let end = length === void 0 ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.HeadersSyncedTo();
      while (reader.pos < end) {
        let tag = reader.uint32();
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
    HeadersSyncedTo.decodeDelimited = function decodeDelimited(reader) {
      if (!(reader instanceof $Reader))
        reader = new $Reader(reader);
      return this.decode(reader, reader.uint32());
    };
    HeadersSyncedTo.verify = function verify(message) {
      if (typeof message !== "object" || message === null)
        return "object expected";
      if (message.relaychainSyncedTo != null && message.hasOwnProperty("relaychainSyncedTo")) {
        if (!$util.isInteger(message.relaychainSyncedTo))
          return "relaychainSyncedTo: integer expected";
      }
      if (message.parachainSyncedTo != null && message.hasOwnProperty("parachainSyncedTo")) {
        if (!$util.isInteger(message.parachainSyncedTo))
          return "parachainSyncedTo: integer expected";
      }
      return null;
    };
    HeadersSyncedTo.fromObject = function fromObject(object) {
      if (object instanceof $root.pruntime_rpc.HeadersSyncedTo)
        return object;
      let message = new $root.pruntime_rpc.HeadersSyncedTo();
      if (object.relaychainSyncedTo != null)
        message.relaychainSyncedTo = object.relaychainSyncedTo >>> 0;
      if (object.parachainSyncedTo != null)
        message.parachainSyncedTo = object.parachainSyncedTo >>> 0;
      return message;
    };
    HeadersSyncedTo.toObject = function toObject(message, options) {
      if (!options)
        options = {};
      let object = {};
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
    HeadersSyncedTo.prototype.toJSON = function toJSON() {
      return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
    };
    return HeadersSyncedTo;
  }();
  pruntime_rpc2.Blocks = function() {
    function Blocks(properties) {
      if (properties) {
        for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
          if (properties[keys[i]] != null)
            this[keys[i]] = properties[keys[i]];
      }
    }
    Blocks.prototype.encodedBlocks = $util.newBuffer([]);
    Blocks.create = function create2(properties) {
      return new Blocks(properties);
    };
    Blocks.encode = function encode(message, writer) {
      if (!writer)
        writer = $Writer.create();
      if (message.encodedBlocks != null && Object.hasOwnProperty.call(message, "encodedBlocks"))
        writer.uint32(10).bytes(message.encodedBlocks);
      return writer;
    };
    Blocks.encodeDelimited = function encodeDelimited(message, writer) {
      return this.encode(message, writer).ldelim();
    };
    Blocks.decode = function decode(reader, length) {
      if (!(reader instanceof $Reader))
        reader = $Reader.create(reader);
      let end = length === void 0 ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.Blocks();
      while (reader.pos < end) {
        let tag = reader.uint32();
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
    Blocks.decodeDelimited = function decodeDelimited(reader) {
      if (!(reader instanceof $Reader))
        reader = new $Reader(reader);
      return this.decode(reader, reader.uint32());
    };
    Blocks.verify = function verify(message) {
      if (typeof message !== "object" || message === null)
        return "object expected";
      if (message.encodedBlocks != null && message.hasOwnProperty("encodedBlocks")) {
        if (!(message.encodedBlocks && typeof message.encodedBlocks.length === "number" || $util.isString(message.encodedBlocks)))
          return "encodedBlocks: buffer expected";
      }
      return null;
    };
    Blocks.fromObject = function fromObject(object) {
      if (object instanceof $root.pruntime_rpc.Blocks)
        return object;
      let message = new $root.pruntime_rpc.Blocks();
      if (object.encodedBlocks != null) {
        if (typeof object.encodedBlocks === "string")
          $util.base64.decode(
            object.encodedBlocks,
            message.encodedBlocks = $util.newBuffer(
              $util.base64.length(object.encodedBlocks)
            ),
            0
          );
        else if (object.encodedBlocks.length)
          message.encodedBlocks = object.encodedBlocks;
      }
      return message;
    };
    Blocks.toObject = function toObject(message, options) {
      if (!options)
        options = {};
      let object = {};
      if (options.defaults)
        if (options.bytes === String)
          object.encodedBlocks = "";
        else {
          object.encodedBlocks = [];
          if (options.bytes !== Array)
            object.encodedBlocks = $util.newBuffer(object.encodedBlocks);
        }
      if (message.encodedBlocks != null && message.hasOwnProperty("encodedBlocks"))
        object.encodedBlocks = options.bytes === String ? $util.base64.encode(
          message.encodedBlocks,
          0,
          message.encodedBlocks.length
        ) : options.bytes === Array ? Array.prototype.slice.call(message.encodedBlocks) : message.encodedBlocks;
      return object;
    };
    Blocks.prototype.toJSON = function toJSON() {
      return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
    };
    return Blocks;
  }();
  pruntime_rpc2.InitRuntimeRequest = function() {
    function InitRuntimeRequest(properties) {
      if (properties) {
        for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
          if (properties[keys[i]] != null)
            this[keys[i]] = properties[keys[i]];
      }
    }
    InitRuntimeRequest.prototype.skipRa = false;
    InitRuntimeRequest.prototype.encodedGenesisInfo = $util.newBuffer([]);
    InitRuntimeRequest.prototype.debugSetKey = null;
    InitRuntimeRequest.prototype.encodedGenesisState = $util.newBuffer([]);
    InitRuntimeRequest.prototype.encodedOperator = null;
    InitRuntimeRequest.prototype.isParachain = false;
    let $oneOfFields;
    Object.defineProperty(InitRuntimeRequest.prototype, "_debugSetKey", {
      get: $util.oneOfGetter($oneOfFields = ["debugSetKey"]),
      set: $util.oneOfSetter($oneOfFields)
    });
    Object.defineProperty(InitRuntimeRequest.prototype, "_encodedOperator", {
      get: $util.oneOfGetter($oneOfFields = ["encodedOperator"]),
      set: $util.oneOfSetter($oneOfFields)
    });
    InitRuntimeRequest.create = function create2(properties) {
      return new InitRuntimeRequest(properties);
    };
    InitRuntimeRequest.encode = function encode(message, writer) {
      if (!writer)
        writer = $Writer.create();
      if (message.skipRa != null && Object.hasOwnProperty.call(message, "skipRa"))
        writer.uint32(8).bool(message.skipRa);
      if (message.encodedGenesisInfo != null && Object.hasOwnProperty.call(message, "encodedGenesisInfo"))
        writer.uint32(18).bytes(message.encodedGenesisInfo);
      if (message.debugSetKey != null && Object.hasOwnProperty.call(message, "debugSetKey"))
        writer.uint32(26).bytes(message.debugSetKey);
      if (message.encodedGenesisState != null && Object.hasOwnProperty.call(message, "encodedGenesisState"))
        writer.uint32(34).bytes(message.encodedGenesisState);
      if (message.encodedOperator != null && Object.hasOwnProperty.call(message, "encodedOperator"))
        writer.uint32(42).bytes(message.encodedOperator);
      if (message.isParachain != null && Object.hasOwnProperty.call(message, "isParachain"))
        writer.uint32(48).bool(message.isParachain);
      return writer;
    };
    InitRuntimeRequest.encodeDelimited = function encodeDelimited(message, writer) {
      return this.encode(message, writer).ldelim();
    };
    InitRuntimeRequest.decode = function decode(reader, length) {
      if (!(reader instanceof $Reader))
        reader = $Reader.create(reader);
      let end = length === void 0 ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.InitRuntimeRequest();
      while (reader.pos < end) {
        let tag = reader.uint32();
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
          default:
            reader.skipType(tag & 7);
            break;
        }
      }
      return message;
    };
    InitRuntimeRequest.decodeDelimited = function decodeDelimited(reader) {
      if (!(reader instanceof $Reader))
        reader = new $Reader(reader);
      return this.decode(reader, reader.uint32());
    };
    InitRuntimeRequest.verify = function verify(message) {
      if (typeof message !== "object" || message === null)
        return "object expected";
      let properties = {};
      if (message.skipRa != null && message.hasOwnProperty("skipRa")) {
        if (typeof message.skipRa !== "boolean")
          return "skipRa: boolean expected";
      }
      if (message.encodedGenesisInfo != null && message.hasOwnProperty("encodedGenesisInfo")) {
        if (!(message.encodedGenesisInfo && typeof message.encodedGenesisInfo.length === "number" || $util.isString(message.encodedGenesisInfo)))
          return "encodedGenesisInfo: buffer expected";
      }
      if (message.debugSetKey != null && message.hasOwnProperty("debugSetKey")) {
        properties._debugSetKey = 1;
        if (!(message.debugSetKey && typeof message.debugSetKey.length === "number" || $util.isString(message.debugSetKey)))
          return "debugSetKey: buffer expected";
      }
      if (message.encodedGenesisState != null && message.hasOwnProperty("encodedGenesisState")) {
        if (!(message.encodedGenesisState && typeof message.encodedGenesisState.length === "number" || $util.isString(message.encodedGenesisState)))
          return "encodedGenesisState: buffer expected";
      }
      if (message.encodedOperator != null && message.hasOwnProperty("encodedOperator")) {
        properties._encodedOperator = 1;
        if (!(message.encodedOperator && typeof message.encodedOperator.length === "number" || $util.isString(message.encodedOperator)))
          return "encodedOperator: buffer expected";
      }
      if (message.isParachain != null && message.hasOwnProperty("isParachain")) {
        if (typeof message.isParachain !== "boolean")
          return "isParachain: boolean expected";
      }
      return null;
    };
    InitRuntimeRequest.fromObject = function fromObject(object) {
      if (object instanceof $root.pruntime_rpc.InitRuntimeRequest)
        return object;
      let message = new $root.pruntime_rpc.InitRuntimeRequest();
      if (object.skipRa != null)
        message.skipRa = Boolean(object.skipRa);
      if (object.encodedGenesisInfo != null) {
        if (typeof object.encodedGenesisInfo === "string")
          $util.base64.decode(
            object.encodedGenesisInfo,
            message.encodedGenesisInfo = $util.newBuffer(
              $util.base64.length(object.encodedGenesisInfo)
            ),
            0
          );
        else if (object.encodedGenesisInfo.length)
          message.encodedGenesisInfo = object.encodedGenesisInfo;
      }
      if (object.debugSetKey != null) {
        if (typeof object.debugSetKey === "string")
          $util.base64.decode(
            object.debugSetKey,
            message.debugSetKey = $util.newBuffer(
              $util.base64.length(object.debugSetKey)
            ),
            0
          );
        else if (object.debugSetKey.length)
          message.debugSetKey = object.debugSetKey;
      }
      if (object.encodedGenesisState != null) {
        if (typeof object.encodedGenesisState === "string")
          $util.base64.decode(
            object.encodedGenesisState,
            message.encodedGenesisState = $util.newBuffer(
              $util.base64.length(object.encodedGenesisState)
            ),
            0
          );
        else if (object.encodedGenesisState.length)
          message.encodedGenesisState = object.encodedGenesisState;
      }
      if (object.encodedOperator != null) {
        if (typeof object.encodedOperator === "string")
          $util.base64.decode(
            object.encodedOperator,
            message.encodedOperator = $util.newBuffer(
              $util.base64.length(object.encodedOperator)
            ),
            0
          );
        else if (object.encodedOperator.length)
          message.encodedOperator = object.encodedOperator;
      }
      if (object.isParachain != null)
        message.isParachain = Boolean(object.isParachain);
      return message;
    };
    InitRuntimeRequest.toObject = function toObject(message, options) {
      if (!options)
        options = {};
      let object = {};
      if (options.defaults) {
        object.skipRa = false;
        if (options.bytes === String)
          object.encodedGenesisInfo = "";
        else {
          object.encodedGenesisInfo = [];
          if (options.bytes !== Array)
            object.encodedGenesisInfo = $util.newBuffer(
              object.encodedGenesisInfo
            );
        }
        if (options.bytes === String)
          object.encodedGenesisState = "";
        else {
          object.encodedGenesisState = [];
          if (options.bytes !== Array)
            object.encodedGenesisState = $util.newBuffer(
              object.encodedGenesisState
            );
        }
        object.isParachain = false;
      }
      if (message.skipRa != null && message.hasOwnProperty("skipRa"))
        object.skipRa = message.skipRa;
      if (message.encodedGenesisInfo != null && message.hasOwnProperty("encodedGenesisInfo"))
        object.encodedGenesisInfo = options.bytes === String ? $util.base64.encode(
          message.encodedGenesisInfo,
          0,
          message.encodedGenesisInfo.length
        ) : options.bytes === Array ? Array.prototype.slice.call(message.encodedGenesisInfo) : message.encodedGenesisInfo;
      if (message.debugSetKey != null && message.hasOwnProperty("debugSetKey")) {
        object.debugSetKey = options.bytes === String ? $util.base64.encode(
          message.debugSetKey,
          0,
          message.debugSetKey.length
        ) : options.bytes === Array ? Array.prototype.slice.call(message.debugSetKey) : message.debugSetKey;
        if (options.oneofs)
          object._debugSetKey = "debugSetKey";
      }
      if (message.encodedGenesisState != null && message.hasOwnProperty("encodedGenesisState"))
        object.encodedGenesisState = options.bytes === String ? $util.base64.encode(
          message.encodedGenesisState,
          0,
          message.encodedGenesisState.length
        ) : options.bytes === Array ? Array.prototype.slice.call(message.encodedGenesisState) : message.encodedGenesisState;
      if (message.encodedOperator != null && message.hasOwnProperty("encodedOperator")) {
        object.encodedOperator = options.bytes === String ? $util.base64.encode(
          message.encodedOperator,
          0,
          message.encodedOperator.length
        ) : options.bytes === Array ? Array.prototype.slice.call(message.encodedOperator) : message.encodedOperator;
        if (options.oneofs)
          object._encodedOperator = "encodedOperator";
      }
      if (message.isParachain != null && message.hasOwnProperty("isParachain"))
        object.isParachain = message.isParachain;
      return object;
    };
    InitRuntimeRequest.prototype.toJSON = function toJSON() {
      return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
    };
    return InitRuntimeRequest;
  }();
  pruntime_rpc2.GetRuntimeInfoRequest = function() {
    function GetRuntimeInfoRequest(properties) {
      if (properties) {
        for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
          if (properties[keys[i]] != null)
            this[keys[i]] = properties[keys[i]];
      }
    }
    GetRuntimeInfoRequest.prototype.forceRefreshRa = false;
    GetRuntimeInfoRequest.prototype.encodedOperator = null;
    let $oneOfFields;
    Object.defineProperty(GetRuntimeInfoRequest.prototype, "_encodedOperator", {
      get: $util.oneOfGetter($oneOfFields = ["encodedOperator"]),
      set: $util.oneOfSetter($oneOfFields)
    });
    GetRuntimeInfoRequest.create = function create2(properties) {
      return new GetRuntimeInfoRequest(properties);
    };
    GetRuntimeInfoRequest.encode = function encode(message, writer) {
      if (!writer)
        writer = $Writer.create();
      if (message.forceRefreshRa != null && Object.hasOwnProperty.call(message, "forceRefreshRa"))
        writer.uint32(8).bool(message.forceRefreshRa);
      if (message.encodedOperator != null && Object.hasOwnProperty.call(message, "encodedOperator"))
        writer.uint32(18).bytes(message.encodedOperator);
      return writer;
    };
    GetRuntimeInfoRequest.encodeDelimited = function encodeDelimited(message, writer) {
      return this.encode(message, writer).ldelim();
    };
    GetRuntimeInfoRequest.decode = function decode(reader, length) {
      if (!(reader instanceof $Reader))
        reader = $Reader.create(reader);
      let end = length === void 0 ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.GetRuntimeInfoRequest();
      while (reader.pos < end) {
        let tag = reader.uint32();
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
    GetRuntimeInfoRequest.decodeDelimited = function decodeDelimited(reader) {
      if (!(reader instanceof $Reader))
        reader = new $Reader(reader);
      return this.decode(reader, reader.uint32());
    };
    GetRuntimeInfoRequest.verify = function verify(message) {
      if (typeof message !== "object" || message === null)
        return "object expected";
      let properties = {};
      if (message.forceRefreshRa != null && message.hasOwnProperty("forceRefreshRa")) {
        if (typeof message.forceRefreshRa !== "boolean")
          return "forceRefreshRa: boolean expected";
      }
      if (message.encodedOperator != null && message.hasOwnProperty("encodedOperator")) {
        properties._encodedOperator = 1;
        if (!(message.encodedOperator && typeof message.encodedOperator.length === "number" || $util.isString(message.encodedOperator)))
          return "encodedOperator: buffer expected";
      }
      return null;
    };
    GetRuntimeInfoRequest.fromObject = function fromObject(object) {
      if (object instanceof $root.pruntime_rpc.GetRuntimeInfoRequest)
        return object;
      let message = new $root.pruntime_rpc.GetRuntimeInfoRequest();
      if (object.forceRefreshRa != null)
        message.forceRefreshRa = Boolean(object.forceRefreshRa);
      if (object.encodedOperator != null) {
        if (typeof object.encodedOperator === "string")
          $util.base64.decode(
            object.encodedOperator,
            message.encodedOperator = $util.newBuffer(
              $util.base64.length(object.encodedOperator)
            ),
            0
          );
        else if (object.encodedOperator.length)
          message.encodedOperator = object.encodedOperator;
      }
      return message;
    };
    GetRuntimeInfoRequest.toObject = function toObject(message, options) {
      if (!options)
        options = {};
      let object = {};
      if (options.defaults)
        object.forceRefreshRa = false;
      if (message.forceRefreshRa != null && message.hasOwnProperty("forceRefreshRa"))
        object.forceRefreshRa = message.forceRefreshRa;
      if (message.encodedOperator != null && message.hasOwnProperty("encodedOperator")) {
        object.encodedOperator = options.bytes === String ? $util.base64.encode(
          message.encodedOperator,
          0,
          message.encodedOperator.length
        ) : options.bytes === Array ? Array.prototype.slice.call(message.encodedOperator) : message.encodedOperator;
        if (options.oneofs)
          object._encodedOperator = "encodedOperator";
      }
      return object;
    };
    GetRuntimeInfoRequest.prototype.toJSON = function toJSON() {
      return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
    };
    return GetRuntimeInfoRequest;
  }();
  pruntime_rpc2.InitRuntimeResponse = function() {
    function InitRuntimeResponse(properties) {
      if (properties) {
        for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
          if (properties[keys[i]] != null)
            this[keys[i]] = properties[keys[i]];
      }
    }
    InitRuntimeResponse.prototype.encodedRuntimeInfo = $util.newBuffer([]);
    InitRuntimeResponse.prototype.encodedGenesisBlockHash = $util.newBuffer([]);
    InitRuntimeResponse.prototype.encodedPublicKey = $util.newBuffer([]);
    InitRuntimeResponse.prototype.encodedEcdhPublicKey = $util.newBuffer([]);
    InitRuntimeResponse.prototype.attestation = null;
    let $oneOfFields;
    Object.defineProperty(InitRuntimeResponse.prototype, "_attestation", {
      get: $util.oneOfGetter($oneOfFields = ["attestation"]),
      set: $util.oneOfSetter($oneOfFields)
    });
    InitRuntimeResponse.create = function create2(properties) {
      return new InitRuntimeResponse(properties);
    };
    InitRuntimeResponse.encode = function encode(message, writer) {
      if (!writer)
        writer = $Writer.create();
      if (message.encodedRuntimeInfo != null && Object.hasOwnProperty.call(message, "encodedRuntimeInfo"))
        writer.uint32(10).bytes(message.encodedRuntimeInfo);
      if (message.encodedGenesisBlockHash != null && Object.hasOwnProperty.call(message, "encodedGenesisBlockHash"))
        writer.uint32(18).bytes(message.encodedGenesisBlockHash);
      if (message.encodedPublicKey != null && Object.hasOwnProperty.call(message, "encodedPublicKey"))
        writer.uint32(26).bytes(message.encodedPublicKey);
      if (message.encodedEcdhPublicKey != null && Object.hasOwnProperty.call(message, "encodedEcdhPublicKey"))
        writer.uint32(34).bytes(message.encodedEcdhPublicKey);
      if (message.attestation != null && Object.hasOwnProperty.call(message, "attestation"))
        $root.pruntime_rpc.Attestation.encode(
          message.attestation,
          writer.uint32(42).fork()
        ).ldelim();
      return writer;
    };
    InitRuntimeResponse.encodeDelimited = function encodeDelimited(message, writer) {
      return this.encode(message, writer).ldelim();
    };
    InitRuntimeResponse.decode = function decode(reader, length) {
      if (!(reader instanceof $Reader))
        reader = $Reader.create(reader);
      let end = length === void 0 ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.InitRuntimeResponse();
      while (reader.pos < end) {
        let tag = reader.uint32();
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
            message.attestation = $root.pruntime_rpc.Attestation.decode(
              reader,
              reader.uint32()
            );
            break;
          default:
            reader.skipType(tag & 7);
            break;
        }
      }
      return message;
    };
    InitRuntimeResponse.decodeDelimited = function decodeDelimited(reader) {
      if (!(reader instanceof $Reader))
        reader = new $Reader(reader);
      return this.decode(reader, reader.uint32());
    };
    InitRuntimeResponse.verify = function verify(message) {
      if (typeof message !== "object" || message === null)
        return "object expected";
      let properties = {};
      if (message.encodedRuntimeInfo != null && message.hasOwnProperty("encodedRuntimeInfo")) {
        if (!(message.encodedRuntimeInfo && typeof message.encodedRuntimeInfo.length === "number" || $util.isString(message.encodedRuntimeInfo)))
          return "encodedRuntimeInfo: buffer expected";
      }
      if (message.encodedGenesisBlockHash != null && message.hasOwnProperty("encodedGenesisBlockHash")) {
        if (!(message.encodedGenesisBlockHash && typeof message.encodedGenesisBlockHash.length === "number" || $util.isString(message.encodedGenesisBlockHash)))
          return "encodedGenesisBlockHash: buffer expected";
      }
      if (message.encodedPublicKey != null && message.hasOwnProperty("encodedPublicKey")) {
        if (!(message.encodedPublicKey && typeof message.encodedPublicKey.length === "number" || $util.isString(message.encodedPublicKey)))
          return "encodedPublicKey: buffer expected";
      }
      if (message.encodedEcdhPublicKey != null && message.hasOwnProperty("encodedEcdhPublicKey")) {
        if (!(message.encodedEcdhPublicKey && typeof message.encodedEcdhPublicKey.length === "number" || $util.isString(message.encodedEcdhPublicKey)))
          return "encodedEcdhPublicKey: buffer expected";
      }
      if (message.attestation != null && message.hasOwnProperty("attestation")) {
        properties._attestation = 1;
        {
          let error = $root.pruntime_rpc.Attestation.verify(
            message.attestation
          );
          if (error)
            return "attestation." + error;
        }
      }
      return null;
    };
    InitRuntimeResponse.fromObject = function fromObject(object) {
      if (object instanceof $root.pruntime_rpc.InitRuntimeResponse)
        return object;
      let message = new $root.pruntime_rpc.InitRuntimeResponse();
      if (object.encodedRuntimeInfo != null) {
        if (typeof object.encodedRuntimeInfo === "string")
          $util.base64.decode(
            object.encodedRuntimeInfo,
            message.encodedRuntimeInfo = $util.newBuffer(
              $util.base64.length(object.encodedRuntimeInfo)
            ),
            0
          );
        else if (object.encodedRuntimeInfo.length)
          message.encodedRuntimeInfo = object.encodedRuntimeInfo;
      }
      if (object.encodedGenesisBlockHash != null) {
        if (typeof object.encodedGenesisBlockHash === "string")
          $util.base64.decode(
            object.encodedGenesisBlockHash,
            message.encodedGenesisBlockHash = $util.newBuffer(
              $util.base64.length(object.encodedGenesisBlockHash)
            ),
            0
          );
        else if (object.encodedGenesisBlockHash.length)
          message.encodedGenesisBlockHash = object.encodedGenesisBlockHash;
      }
      if (object.encodedPublicKey != null) {
        if (typeof object.encodedPublicKey === "string")
          $util.base64.decode(
            object.encodedPublicKey,
            message.encodedPublicKey = $util.newBuffer(
              $util.base64.length(object.encodedPublicKey)
            ),
            0
          );
        else if (object.encodedPublicKey.length)
          message.encodedPublicKey = object.encodedPublicKey;
      }
      if (object.encodedEcdhPublicKey != null) {
        if (typeof object.encodedEcdhPublicKey === "string")
          $util.base64.decode(
            object.encodedEcdhPublicKey,
            message.encodedEcdhPublicKey = $util.newBuffer(
              $util.base64.length(object.encodedEcdhPublicKey)
            ),
            0
          );
        else if (object.encodedEcdhPublicKey.length)
          message.encodedEcdhPublicKey = object.encodedEcdhPublicKey;
      }
      if (object.attestation != null) {
        if (typeof object.attestation !== "object")
          throw TypeError(
            ".pruntime_rpc.InitRuntimeResponse.attestation: object expected"
          );
        message.attestation = $root.pruntime_rpc.Attestation.fromObject(
          object.attestation
        );
      }
      return message;
    };
    InitRuntimeResponse.toObject = function toObject(message, options) {
      if (!options)
        options = {};
      let object = {};
      if (options.defaults) {
        if (options.bytes === String)
          object.encodedRuntimeInfo = "";
        else {
          object.encodedRuntimeInfo = [];
          if (options.bytes !== Array)
            object.encodedRuntimeInfo = $util.newBuffer(
              object.encodedRuntimeInfo
            );
        }
        if (options.bytes === String)
          object.encodedGenesisBlockHash = "";
        else {
          object.encodedGenesisBlockHash = [];
          if (options.bytes !== Array)
            object.encodedGenesisBlockHash = $util.newBuffer(
              object.encodedGenesisBlockHash
            );
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
            object.encodedEcdhPublicKey = $util.newBuffer(
              object.encodedEcdhPublicKey
            );
        }
      }
      if (message.encodedRuntimeInfo != null && message.hasOwnProperty("encodedRuntimeInfo"))
        object.encodedRuntimeInfo = options.bytes === String ? $util.base64.encode(
          message.encodedRuntimeInfo,
          0,
          message.encodedRuntimeInfo.length
        ) : options.bytes === Array ? Array.prototype.slice.call(message.encodedRuntimeInfo) : message.encodedRuntimeInfo;
      if (message.encodedGenesisBlockHash != null && message.hasOwnProperty("encodedGenesisBlockHash"))
        object.encodedGenesisBlockHash = options.bytes === String ? $util.base64.encode(
          message.encodedGenesisBlockHash,
          0,
          message.encodedGenesisBlockHash.length
        ) : options.bytes === Array ? Array.prototype.slice.call(message.encodedGenesisBlockHash) : message.encodedGenesisBlockHash;
      if (message.encodedPublicKey != null && message.hasOwnProperty("encodedPublicKey"))
        object.encodedPublicKey = options.bytes === String ? $util.base64.encode(
          message.encodedPublicKey,
          0,
          message.encodedPublicKey.length
        ) : options.bytes === Array ? Array.prototype.slice.call(message.encodedPublicKey) : message.encodedPublicKey;
      if (message.encodedEcdhPublicKey != null && message.hasOwnProperty("encodedEcdhPublicKey"))
        object.encodedEcdhPublicKey = options.bytes === String ? $util.base64.encode(
          message.encodedEcdhPublicKey,
          0,
          message.encodedEcdhPublicKey.length
        ) : options.bytes === Array ? Array.prototype.slice.call(message.encodedEcdhPublicKey) : message.encodedEcdhPublicKey;
      if (message.attestation != null && message.hasOwnProperty("attestation")) {
        object.attestation = $root.pruntime_rpc.Attestation.toObject(
          message.attestation,
          options
        );
        if (options.oneofs)
          object._attestation = "attestation";
      }
      return object;
    };
    InitRuntimeResponse.prototype.toJSON = function toJSON() {
      return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
    };
    return InitRuntimeResponse;
  }();
  pruntime_rpc2.Attestation = function() {
    function Attestation(properties) {
      if (properties) {
        for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
          if (properties[keys[i]] != null)
            this[keys[i]] = properties[keys[i]];
      }
    }
    Attestation.prototype.version = 0;
    Attestation.prototype.provider = "";
    Attestation.prototype.payload = null;
    Attestation.prototype.timestamp = $util.Long ? $util.Long.fromBits(0, 0, true) : 0;
    Attestation.create = function create2(properties) {
      return new Attestation(properties);
    };
    Attestation.encode = function encode(message, writer) {
      if (!writer)
        writer = $Writer.create();
      if (message.version != null && Object.hasOwnProperty.call(message, "version"))
        writer.uint32(8).int32(message.version);
      if (message.provider != null && Object.hasOwnProperty.call(message, "provider"))
        writer.uint32(18).string(message.provider);
      if (message.payload != null && Object.hasOwnProperty.call(message, "payload"))
        $root.pruntime_rpc.AttestationReport.encode(
          message.payload,
          writer.uint32(26).fork()
        ).ldelim();
      if (message.timestamp != null && Object.hasOwnProperty.call(message, "timestamp"))
        writer.uint32(32).uint64(message.timestamp);
      return writer;
    };
    Attestation.encodeDelimited = function encodeDelimited(message, writer) {
      return this.encode(message, writer).ldelim();
    };
    Attestation.decode = function decode(reader, length) {
      if (!(reader instanceof $Reader))
        reader = $Reader.create(reader);
      let end = length === void 0 ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.Attestation();
      while (reader.pos < end) {
        let tag = reader.uint32();
        switch (tag >>> 3) {
          case 1:
            message.version = reader.int32();
            break;
          case 2:
            message.provider = reader.string();
            break;
          case 3:
            message.payload = $root.pruntime_rpc.AttestationReport.decode(
              reader,
              reader.uint32()
            );
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
    Attestation.decodeDelimited = function decodeDelimited(reader) {
      if (!(reader instanceof $Reader))
        reader = new $Reader(reader);
      return this.decode(reader, reader.uint32());
    };
    Attestation.verify = function verify(message) {
      if (typeof message !== "object" || message === null)
        return "object expected";
      if (message.version != null && message.hasOwnProperty("version")) {
        if (!$util.isInteger(message.version))
          return "version: integer expected";
      }
      if (message.provider != null && message.hasOwnProperty("provider")) {
        if (!$util.isString(message.provider))
          return "provider: string expected";
      }
      if (message.payload != null && message.hasOwnProperty("payload")) {
        let error = $root.pruntime_rpc.AttestationReport.verify(
          message.payload
        );
        if (error)
          return "payload." + error;
      }
      if (message.timestamp != null && message.hasOwnProperty("timestamp")) {
        if (!$util.isInteger(message.timestamp) && !(message.timestamp && $util.isInteger(message.timestamp.low) && $util.isInteger(message.timestamp.high)))
          return "timestamp: integer|Long expected";
      }
      return null;
    };
    Attestation.fromObject = function fromObject(object) {
      if (object instanceof $root.pruntime_rpc.Attestation)
        return object;
      let message = new $root.pruntime_rpc.Attestation();
      if (object.version != null)
        message.version = object.version | 0;
      if (object.provider != null)
        message.provider = String(object.provider);
      if (object.payload != null) {
        if (typeof object.payload !== "object")
          throw TypeError(".pruntime_rpc.Attestation.payload: object expected");
        message.payload = $root.pruntime_rpc.AttestationReport.fromObject(
          object.payload
        );
      }
      if (object.timestamp != null) {
        if ($util.Long)
          (message.timestamp = $util.Long.fromValue(
            object.timestamp
          )).unsigned = true;
        else if (typeof object.timestamp === "string")
          message.timestamp = parseInt(object.timestamp, 10);
        else if (typeof object.timestamp === "number")
          message.timestamp = object.timestamp;
        else if (typeof object.timestamp === "object")
          message.timestamp = new $util.LongBits(
            object.timestamp.low >>> 0,
            object.timestamp.high >>> 0
          ).toNumber(true);
      }
      return message;
    };
    Attestation.toObject = function toObject(message, options) {
      if (!options)
        options = {};
      let object = {};
      if (options.defaults) {
        object.version = 0;
        object.provider = "";
        object.payload = null;
        if ($util.Long) {
          let long = new $util.Long(0, 0, true);
          object.timestamp = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : long;
        } else
          object.timestamp = options.longs === String ? "0" : 0;
      }
      if (message.version != null && message.hasOwnProperty("version"))
        object.version = message.version;
      if (message.provider != null && message.hasOwnProperty("provider"))
        object.provider = message.provider;
      if (message.payload != null && message.hasOwnProperty("payload"))
        object.payload = $root.pruntime_rpc.AttestationReport.toObject(
          message.payload,
          options
        );
      if (message.timestamp != null && message.hasOwnProperty("timestamp"))
        if (typeof message.timestamp === "number")
          object.timestamp = options.longs === String ? String(message.timestamp) : message.timestamp;
        else
          object.timestamp = options.longs === String ? $util.Long.prototype.toString.call(message.timestamp) : options.longs === Number ? new $util.LongBits(
            message.timestamp.low >>> 0,
            message.timestamp.high >>> 0
          ).toNumber(true) : message.timestamp;
      return object;
    };
    Attestation.prototype.toJSON = function toJSON() {
      return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
    };
    return Attestation;
  }();
  pruntime_rpc2.AttestationReport = function() {
    function AttestationReport(properties) {
      if (properties) {
        for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
          if (properties[keys[i]] != null)
            this[keys[i]] = properties[keys[i]];
      }
    }
    AttestationReport.prototype.report = "";
    AttestationReport.prototype.signature = $util.newBuffer([]);
    AttestationReport.prototype.signingCert = $util.newBuffer([]);
    AttestationReport.create = function create2(properties) {
      return new AttestationReport(properties);
    };
    AttestationReport.encode = function encode(message, writer) {
      if (!writer)
        writer = $Writer.create();
      if (message.report != null && Object.hasOwnProperty.call(message, "report"))
        writer.uint32(10).string(message.report);
      if (message.signature != null && Object.hasOwnProperty.call(message, "signature"))
        writer.uint32(18).bytes(message.signature);
      if (message.signingCert != null && Object.hasOwnProperty.call(message, "signingCert"))
        writer.uint32(26).bytes(message.signingCert);
      return writer;
    };
    AttestationReport.encodeDelimited = function encodeDelimited(message, writer) {
      return this.encode(message, writer).ldelim();
    };
    AttestationReport.decode = function decode(reader, length) {
      if (!(reader instanceof $Reader))
        reader = $Reader.create(reader);
      let end = length === void 0 ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.AttestationReport();
      while (reader.pos < end) {
        let tag = reader.uint32();
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
    AttestationReport.decodeDelimited = function decodeDelimited(reader) {
      if (!(reader instanceof $Reader))
        reader = new $Reader(reader);
      return this.decode(reader, reader.uint32());
    };
    AttestationReport.verify = function verify(message) {
      if (typeof message !== "object" || message === null)
        return "object expected";
      if (message.report != null && message.hasOwnProperty("report")) {
        if (!$util.isString(message.report))
          return "report: string expected";
      }
      if (message.signature != null && message.hasOwnProperty("signature")) {
        if (!(message.signature && typeof message.signature.length === "number" || $util.isString(message.signature)))
          return "signature: buffer expected";
      }
      if (message.signingCert != null && message.hasOwnProperty("signingCert")) {
        if (!(message.signingCert && typeof message.signingCert.length === "number" || $util.isString(message.signingCert)))
          return "signingCert: buffer expected";
      }
      return null;
    };
    AttestationReport.fromObject = function fromObject(object) {
      if (object instanceof $root.pruntime_rpc.AttestationReport)
        return object;
      let message = new $root.pruntime_rpc.AttestationReport();
      if (object.report != null)
        message.report = String(object.report);
      if (object.signature != null) {
        if (typeof object.signature === "string")
          $util.base64.decode(
            object.signature,
            message.signature = $util.newBuffer(
              $util.base64.length(object.signature)
            ),
            0
          );
        else if (object.signature.length)
          message.signature = object.signature;
      }
      if (object.signingCert != null) {
        if (typeof object.signingCert === "string")
          $util.base64.decode(
            object.signingCert,
            message.signingCert = $util.newBuffer(
              $util.base64.length(object.signingCert)
            ),
            0
          );
        else if (object.signingCert.length)
          message.signingCert = object.signingCert;
      }
      return message;
    };
    AttestationReport.toObject = function toObject(message, options) {
      if (!options)
        options = {};
      let object = {};
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
        object.signature = options.bytes === String ? $util.base64.encode(
          message.signature,
          0,
          message.signature.length
        ) : options.bytes === Array ? Array.prototype.slice.call(message.signature) : message.signature;
      if (message.signingCert != null && message.hasOwnProperty("signingCert"))
        object.signingCert = options.bytes === String ? $util.base64.encode(
          message.signingCert,
          0,
          message.signingCert.length
        ) : options.bytes === Array ? Array.prototype.slice.call(message.signingCert) : message.signingCert;
      return object;
    };
    AttestationReport.prototype.toJSON = function toJSON() {
      return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
    };
    return AttestationReport;
  }();
  pruntime_rpc2.GetEgressMessagesResponse = function() {
    function GetEgressMessagesResponse(properties) {
      if (properties) {
        for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
          if (properties[keys[i]] != null)
            this[keys[i]] = properties[keys[i]];
      }
    }
    GetEgressMessagesResponse.prototype.encodedMessages = $util.newBuffer([]);
    GetEgressMessagesResponse.create = function create2(properties) {
      return new GetEgressMessagesResponse(properties);
    };
    GetEgressMessagesResponse.encode = function encode(message, writer) {
      if (!writer)
        writer = $Writer.create();
      if (message.encodedMessages != null && Object.hasOwnProperty.call(message, "encodedMessages"))
        writer.uint32(10).bytes(message.encodedMessages);
      return writer;
    };
    GetEgressMessagesResponse.encodeDelimited = function encodeDelimited(message, writer) {
      return this.encode(message, writer).ldelim();
    };
    GetEgressMessagesResponse.decode = function decode(reader, length) {
      if (!(reader instanceof $Reader))
        reader = $Reader.create(reader);
      let end = length === void 0 ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.GetEgressMessagesResponse();
      while (reader.pos < end) {
        let tag = reader.uint32();
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
    GetEgressMessagesResponse.decodeDelimited = function decodeDelimited(reader) {
      if (!(reader instanceof $Reader))
        reader = new $Reader(reader);
      return this.decode(reader, reader.uint32());
    };
    GetEgressMessagesResponse.verify = function verify(message) {
      if (typeof message !== "object" || message === null)
        return "object expected";
      if (message.encodedMessages != null && message.hasOwnProperty("encodedMessages")) {
        if (!(message.encodedMessages && typeof message.encodedMessages.length === "number" || $util.isString(message.encodedMessages)))
          return "encodedMessages: buffer expected";
      }
      return null;
    };
    GetEgressMessagesResponse.fromObject = function fromObject(object) {
      if (object instanceof $root.pruntime_rpc.GetEgressMessagesResponse)
        return object;
      let message = new $root.pruntime_rpc.GetEgressMessagesResponse();
      if (object.encodedMessages != null) {
        if (typeof object.encodedMessages === "string")
          $util.base64.decode(
            object.encodedMessages,
            message.encodedMessages = $util.newBuffer(
              $util.base64.length(object.encodedMessages)
            ),
            0
          );
        else if (object.encodedMessages.length)
          message.encodedMessages = object.encodedMessages;
      }
      return message;
    };
    GetEgressMessagesResponse.toObject = function toObject(message, options) {
      if (!options)
        options = {};
      let object = {};
      if (options.defaults)
        if (options.bytes === String)
          object.encodedMessages = "";
        else {
          object.encodedMessages = [];
          if (options.bytes !== Array)
            object.encodedMessages = $util.newBuffer(object.encodedMessages);
        }
      if (message.encodedMessages != null && message.hasOwnProperty("encodedMessages"))
        object.encodedMessages = options.bytes === String ? $util.base64.encode(
          message.encodedMessages,
          0,
          message.encodedMessages.length
        ) : options.bytes === Array ? Array.prototype.slice.call(message.encodedMessages) : message.encodedMessages;
      return object;
    };
    GetEgressMessagesResponse.prototype.toJSON = function toJSON() {
      return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
    };
    return GetEgressMessagesResponse;
  }();
  pruntime_rpc2.ContractQueryRequest = function() {
    function ContractQueryRequest(properties) {
      if (properties) {
        for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
          if (properties[keys[i]] != null)
            this[keys[i]] = properties[keys[i]];
      }
    }
    ContractQueryRequest.prototype.encodedEncryptedData = $util.newBuffer([]);
    ContractQueryRequest.prototype.signature = null;
    ContractQueryRequest.create = function create2(properties) {
      return new ContractQueryRequest(properties);
    };
    ContractQueryRequest.encode = function encode(message, writer) {
      if (!writer)
        writer = $Writer.create();
      if (message.encodedEncryptedData != null && Object.hasOwnProperty.call(message, "encodedEncryptedData"))
        writer.uint32(10).bytes(message.encodedEncryptedData);
      if (message.signature != null && Object.hasOwnProperty.call(message, "signature"))
        $root.pruntime_rpc.Signature.encode(
          message.signature,
          writer.uint32(18).fork()
        ).ldelim();
      return writer;
    };
    ContractQueryRequest.encodeDelimited = function encodeDelimited(message, writer) {
      return this.encode(message, writer).ldelim();
    };
    ContractQueryRequest.decode = function decode(reader, length) {
      if (!(reader instanceof $Reader))
        reader = $Reader.create(reader);
      let end = length === void 0 ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.ContractQueryRequest();
      while (reader.pos < end) {
        let tag = reader.uint32();
        switch (tag >>> 3) {
          case 1:
            message.encodedEncryptedData = reader.bytes();
            break;
          case 2:
            message.signature = $root.pruntime_rpc.Signature.decode(
              reader,
              reader.uint32()
            );
            break;
          default:
            reader.skipType(tag & 7);
            break;
        }
      }
      return message;
    };
    ContractQueryRequest.decodeDelimited = function decodeDelimited(reader) {
      if (!(reader instanceof $Reader))
        reader = new $Reader(reader);
      return this.decode(reader, reader.uint32());
    };
    ContractQueryRequest.verify = function verify(message) {
      if (typeof message !== "object" || message === null)
        return "object expected";
      if (message.encodedEncryptedData != null && message.hasOwnProperty("encodedEncryptedData")) {
        if (!(message.encodedEncryptedData && typeof message.encodedEncryptedData.length === "number" || $util.isString(message.encodedEncryptedData)))
          return "encodedEncryptedData: buffer expected";
      }
      if (message.signature != null && message.hasOwnProperty("signature")) {
        let error = $root.pruntime_rpc.Signature.verify(message.signature);
        if (error)
          return "signature." + error;
      }
      return null;
    };
    ContractQueryRequest.fromObject = function fromObject(object) {
      if (object instanceof $root.pruntime_rpc.ContractQueryRequest)
        return object;
      let message = new $root.pruntime_rpc.ContractQueryRequest();
      if (object.encodedEncryptedData != null) {
        if (typeof object.encodedEncryptedData === "string")
          $util.base64.decode(
            object.encodedEncryptedData,
            message.encodedEncryptedData = $util.newBuffer(
              $util.base64.length(object.encodedEncryptedData)
            ),
            0
          );
        else if (object.encodedEncryptedData.length)
          message.encodedEncryptedData = object.encodedEncryptedData;
      }
      if (object.signature != null) {
        if (typeof object.signature !== "object")
          throw TypeError(
            ".pruntime_rpc.ContractQueryRequest.signature: object expected"
          );
        message.signature = $root.pruntime_rpc.Signature.fromObject(
          object.signature
        );
      }
      return message;
    };
    ContractQueryRequest.toObject = function toObject(message, options) {
      if (!options)
        options = {};
      let object = {};
      if (options.defaults) {
        if (options.bytes === String)
          object.encodedEncryptedData = "";
        else {
          object.encodedEncryptedData = [];
          if (options.bytes !== Array)
            object.encodedEncryptedData = $util.newBuffer(
              object.encodedEncryptedData
            );
        }
        object.signature = null;
      }
      if (message.encodedEncryptedData != null && message.hasOwnProperty("encodedEncryptedData"))
        object.encodedEncryptedData = options.bytes === String ? $util.base64.encode(
          message.encodedEncryptedData,
          0,
          message.encodedEncryptedData.length
        ) : options.bytes === Array ? Array.prototype.slice.call(message.encodedEncryptedData) : message.encodedEncryptedData;
      if (message.signature != null && message.hasOwnProperty("signature"))
        object.signature = $root.pruntime_rpc.Signature.toObject(
          message.signature,
          options
        );
      return object;
    };
    ContractQueryRequest.prototype.toJSON = function toJSON() {
      return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
    };
    return ContractQueryRequest;
  }();
  pruntime_rpc2.Signature = function() {
    function Signature(properties) {
      if (properties) {
        for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
          if (properties[keys[i]] != null)
            this[keys[i]] = properties[keys[i]];
      }
    }
    Signature.prototype.signedBy = null;
    Signature.prototype.signatureType = 0;
    Signature.prototype.signature = $util.newBuffer([]);
    Signature.create = function create2(properties) {
      return new Signature(properties);
    };
    Signature.encode = function encode(message, writer) {
      if (!writer)
        writer = $Writer.create();
      if (message.signedBy != null && Object.hasOwnProperty.call(message, "signedBy"))
        $root.pruntime_rpc.Certificate.encode(
          message.signedBy,
          writer.uint32(10).fork()
        ).ldelim();
      if (message.signatureType != null && Object.hasOwnProperty.call(message, "signatureType"))
        writer.uint32(16).int32(message.signatureType);
      if (message.signature != null && Object.hasOwnProperty.call(message, "signature"))
        writer.uint32(26).bytes(message.signature);
      return writer;
    };
    Signature.encodeDelimited = function encodeDelimited(message, writer) {
      return this.encode(message, writer).ldelim();
    };
    Signature.decode = function decode(reader, length) {
      if (!(reader instanceof $Reader))
        reader = $Reader.create(reader);
      let end = length === void 0 ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.Signature();
      while (reader.pos < end) {
        let tag = reader.uint32();
        switch (tag >>> 3) {
          case 1:
            message.signedBy = $root.pruntime_rpc.Certificate.decode(
              reader,
              reader.uint32()
            );
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
    Signature.decodeDelimited = function decodeDelimited(reader) {
      if (!(reader instanceof $Reader))
        reader = new $Reader(reader);
      return this.decode(reader, reader.uint32());
    };
    Signature.verify = function verify(message) {
      if (typeof message !== "object" || message === null)
        return "object expected";
      if (message.signedBy != null && message.hasOwnProperty("signedBy")) {
        let error = $root.pruntime_rpc.Certificate.verify(message.signedBy);
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
      if (message.signature != null && message.hasOwnProperty("signature")) {
        if (!(message.signature && typeof message.signature.length === "number" || $util.isString(message.signature)))
          return "signature: buffer expected";
      }
      return null;
    };
    Signature.fromObject = function fromObject(object) {
      if (object instanceof $root.pruntime_rpc.Signature)
        return object;
      let message = new $root.pruntime_rpc.Signature();
      if (object.signedBy != null) {
        if (typeof object.signedBy !== "object")
          throw TypeError(".pruntime_rpc.Signature.signedBy: object expected");
        message.signedBy = $root.pruntime_rpc.Certificate.fromObject(
          object.signedBy
        );
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
      if (object.signature != null) {
        if (typeof object.signature === "string")
          $util.base64.decode(
            object.signature,
            message.signature = $util.newBuffer(
              $util.base64.length(object.signature)
            ),
            0
          );
        else if (object.signature.length)
          message.signature = object.signature;
      }
      return message;
    };
    Signature.toObject = function toObject(message, options) {
      if (!options)
        options = {};
      let object = {};
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
        object.signedBy = $root.pruntime_rpc.Certificate.toObject(
          message.signedBy,
          options
        );
      if (message.signatureType != null && message.hasOwnProperty("signatureType"))
        object.signatureType = options.enums === String ? $root.pruntime_rpc.SignatureType[message.signatureType] : message.signatureType;
      if (message.signature != null && message.hasOwnProperty("signature"))
        object.signature = options.bytes === String ? $util.base64.encode(
          message.signature,
          0,
          message.signature.length
        ) : options.bytes === Array ? Array.prototype.slice.call(message.signature) : message.signature;
      return object;
    };
    Signature.prototype.toJSON = function toJSON() {
      return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
    };
    return Signature;
  }();
  pruntime_rpc2.Certificate = function() {
    function Certificate(properties) {
      if (properties) {
        for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
          if (properties[keys[i]] != null)
            this[keys[i]] = properties[keys[i]];
      }
    }
    Certificate.prototype.encodedBody = $util.newBuffer([]);
    Certificate.prototype.signature = null;
    Certificate.create = function create2(properties) {
      return new Certificate(properties);
    };
    Certificate.encode = function encode(message, writer) {
      if (!writer)
        writer = $Writer.create();
      if (message.encodedBody != null && Object.hasOwnProperty.call(message, "encodedBody"))
        writer.uint32(10).bytes(message.encodedBody);
      if (message.signature != null && Object.hasOwnProperty.call(message, "signature"))
        $root.pruntime_rpc.Signature.encode(
          message.signature,
          writer.uint32(18).fork()
        ).ldelim();
      return writer;
    };
    Certificate.encodeDelimited = function encodeDelimited(message, writer) {
      return this.encode(message, writer).ldelim();
    };
    Certificate.decode = function decode(reader, length) {
      if (!(reader instanceof $Reader))
        reader = $Reader.create(reader);
      let end = length === void 0 ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.Certificate();
      while (reader.pos < end) {
        let tag = reader.uint32();
        switch (tag >>> 3) {
          case 1:
            message.encodedBody = reader.bytes();
            break;
          case 2:
            message.signature = $root.pruntime_rpc.Signature.decode(
              reader,
              reader.uint32()
            );
            break;
          default:
            reader.skipType(tag & 7);
            break;
        }
      }
      return message;
    };
    Certificate.decodeDelimited = function decodeDelimited(reader) {
      if (!(reader instanceof $Reader))
        reader = new $Reader(reader);
      return this.decode(reader, reader.uint32());
    };
    Certificate.verify = function verify(message) {
      if (typeof message !== "object" || message === null)
        return "object expected";
      if (message.encodedBody != null && message.hasOwnProperty("encodedBody")) {
        if (!(message.encodedBody && typeof message.encodedBody.length === "number" || $util.isString(message.encodedBody)))
          return "encodedBody: buffer expected";
      }
      if (message.signature != null && message.hasOwnProperty("signature")) {
        let error = $root.pruntime_rpc.Signature.verify(message.signature);
        if (error)
          return "signature." + error;
      }
      return null;
    };
    Certificate.fromObject = function fromObject(object) {
      if (object instanceof $root.pruntime_rpc.Certificate)
        return object;
      let message = new $root.pruntime_rpc.Certificate();
      if (object.encodedBody != null) {
        if (typeof object.encodedBody === "string")
          $util.base64.decode(
            object.encodedBody,
            message.encodedBody = $util.newBuffer(
              $util.base64.length(object.encodedBody)
            ),
            0
          );
        else if (object.encodedBody.length)
          message.encodedBody = object.encodedBody;
      }
      if (object.signature != null) {
        if (typeof object.signature !== "object")
          throw TypeError(
            ".pruntime_rpc.Certificate.signature: object expected"
          );
        message.signature = $root.pruntime_rpc.Signature.fromObject(
          object.signature
        );
      }
      return message;
    };
    Certificate.toObject = function toObject(message, options) {
      if (!options)
        options = {};
      let object = {};
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
        object.encodedBody = options.bytes === String ? $util.base64.encode(
          message.encodedBody,
          0,
          message.encodedBody.length
        ) : options.bytes === Array ? Array.prototype.slice.call(message.encodedBody) : message.encodedBody;
      if (message.signature != null && message.hasOwnProperty("signature"))
        object.signature = $root.pruntime_rpc.Signature.toObject(
          message.signature,
          options
        );
      return object;
    };
    Certificate.prototype.toJSON = function toJSON() {
      return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
    };
    return Certificate;
  }();
  pruntime_rpc2.SignatureType = function() {
    const valuesById = {}, values = Object.create(valuesById);
    values[valuesById[0] = "Ed25519"] = 0;
    values[valuesById[1] = "Sr25519"] = 1;
    values[valuesById[2] = "Ecdsa"] = 2;
    values[valuesById[3] = "Ed25519WrapBytes"] = 3;
    values[valuesById[4] = "Sr25519WrapBytes"] = 4;
    values[valuesById[5] = "EcdsaWrapBytes"] = 5;
    return values;
  }();
  pruntime_rpc2.ContractQueryResponse = function() {
    function ContractQueryResponse(properties) {
      if (properties) {
        for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
          if (properties[keys[i]] != null)
            this[keys[i]] = properties[keys[i]];
      }
    }
    ContractQueryResponse.prototype.encodedEncryptedData = $util.newBuffer([]);
    ContractQueryResponse.create = function create2(properties) {
      return new ContractQueryResponse(properties);
    };
    ContractQueryResponse.encode = function encode(message, writer) {
      if (!writer)
        writer = $Writer.create();
      if (message.encodedEncryptedData != null && Object.hasOwnProperty.call(message, "encodedEncryptedData"))
        writer.uint32(10).bytes(message.encodedEncryptedData);
      return writer;
    };
    ContractQueryResponse.encodeDelimited = function encodeDelimited(message, writer) {
      return this.encode(message, writer).ldelim();
    };
    ContractQueryResponse.decode = function decode(reader, length) {
      if (!(reader instanceof $Reader))
        reader = $Reader.create(reader);
      let end = length === void 0 ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.ContractQueryResponse();
      while (reader.pos < end) {
        let tag = reader.uint32();
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
    ContractQueryResponse.decodeDelimited = function decodeDelimited(reader) {
      if (!(reader instanceof $Reader))
        reader = new $Reader(reader);
      return this.decode(reader, reader.uint32());
    };
    ContractQueryResponse.verify = function verify(message) {
      if (typeof message !== "object" || message === null)
        return "object expected";
      if (message.encodedEncryptedData != null && message.hasOwnProperty("encodedEncryptedData")) {
        if (!(message.encodedEncryptedData && typeof message.encodedEncryptedData.length === "number" || $util.isString(message.encodedEncryptedData)))
          return "encodedEncryptedData: buffer expected";
      }
      return null;
    };
    ContractQueryResponse.fromObject = function fromObject(object) {
      if (object instanceof $root.pruntime_rpc.ContractQueryResponse)
        return object;
      let message = new $root.pruntime_rpc.ContractQueryResponse();
      if (object.encodedEncryptedData != null) {
        if (typeof object.encodedEncryptedData === "string")
          $util.base64.decode(
            object.encodedEncryptedData,
            message.encodedEncryptedData = $util.newBuffer(
              $util.base64.length(object.encodedEncryptedData)
            ),
            0
          );
        else if (object.encodedEncryptedData.length)
          message.encodedEncryptedData = object.encodedEncryptedData;
      }
      return message;
    };
    ContractQueryResponse.toObject = function toObject(message, options) {
      if (!options)
        options = {};
      let object = {};
      if (options.defaults)
        if (options.bytes === String)
          object.encodedEncryptedData = "";
        else {
          object.encodedEncryptedData = [];
          if (options.bytes !== Array)
            object.encodedEncryptedData = $util.newBuffer(
              object.encodedEncryptedData
            );
        }
      if (message.encodedEncryptedData != null && message.hasOwnProperty("encodedEncryptedData"))
        object.encodedEncryptedData = options.bytes === String ? $util.base64.encode(
          message.encodedEncryptedData,
          0,
          message.encodedEncryptedData.length
        ) : options.bytes === Array ? Array.prototype.slice.call(message.encodedEncryptedData) : message.encodedEncryptedData;
      return object;
    };
    ContractQueryResponse.prototype.toJSON = function toJSON() {
      return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
    };
    return ContractQueryResponse;
  }();
  pruntime_rpc2.GetWorkerStateRequest = function() {
    function GetWorkerStateRequest(properties) {
      if (properties) {
        for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
          if (properties[keys[i]] != null)
            this[keys[i]] = properties[keys[i]];
      }
    }
    GetWorkerStateRequest.prototype.publicKey = $util.newBuffer([]);
    GetWorkerStateRequest.create = function create2(properties) {
      return new GetWorkerStateRequest(properties);
    };
    GetWorkerStateRequest.encode = function encode(message, writer) {
      if (!writer)
        writer = $Writer.create();
      if (message.publicKey != null && Object.hasOwnProperty.call(message, "publicKey"))
        writer.uint32(10).bytes(message.publicKey);
      return writer;
    };
    GetWorkerStateRequest.encodeDelimited = function encodeDelimited(message, writer) {
      return this.encode(message, writer).ldelim();
    };
    GetWorkerStateRequest.decode = function decode(reader, length) {
      if (!(reader instanceof $Reader))
        reader = $Reader.create(reader);
      let end = length === void 0 ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.GetWorkerStateRequest();
      while (reader.pos < end) {
        let tag = reader.uint32();
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
    GetWorkerStateRequest.decodeDelimited = function decodeDelimited(reader) {
      if (!(reader instanceof $Reader))
        reader = new $Reader(reader);
      return this.decode(reader, reader.uint32());
    };
    GetWorkerStateRequest.verify = function verify(message) {
      if (typeof message !== "object" || message === null)
        return "object expected";
      if (message.publicKey != null && message.hasOwnProperty("publicKey")) {
        if (!(message.publicKey && typeof message.publicKey.length === "number" || $util.isString(message.publicKey)))
          return "publicKey: buffer expected";
      }
      return null;
    };
    GetWorkerStateRequest.fromObject = function fromObject(object) {
      if (object instanceof $root.pruntime_rpc.GetWorkerStateRequest)
        return object;
      let message = new $root.pruntime_rpc.GetWorkerStateRequest();
      if (object.publicKey != null) {
        if (typeof object.publicKey === "string")
          $util.base64.decode(
            object.publicKey,
            message.publicKey = $util.newBuffer(
              $util.base64.length(object.publicKey)
            ),
            0
          );
        else if (object.publicKey.length)
          message.publicKey = object.publicKey;
      }
      return message;
    };
    GetWorkerStateRequest.toObject = function toObject(message, options) {
      if (!options)
        options = {};
      let object = {};
      if (options.defaults)
        if (options.bytes === String)
          object.publicKey = "";
        else {
          object.publicKey = [];
          if (options.bytes !== Array)
            object.publicKey = $util.newBuffer(object.publicKey);
        }
      if (message.publicKey != null && message.hasOwnProperty("publicKey"))
        object.publicKey = options.bytes === String ? $util.base64.encode(
          message.publicKey,
          0,
          message.publicKey.length
        ) : options.bytes === Array ? Array.prototype.slice.call(message.publicKey) : message.publicKey;
      return object;
    };
    GetWorkerStateRequest.prototype.toJSON = function toJSON() {
      return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
    };
    return GetWorkerStateRequest;
  }();
  pruntime_rpc2.WorkerStat = function() {
    function WorkerStat(properties) {
      if (properties) {
        for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
          if (properties[keys[i]] != null)
            this[keys[i]] = properties[keys[i]];
      }
    }
    WorkerStat.prototype.lastHeartbeatForBlock = 0;
    WorkerStat.prototype.lastHeartbeatAtBlock = 0;
    WorkerStat.prototype.lastGkResponsiveEvent = 0;
    WorkerStat.prototype.lastGkResponsiveEventAtBlock = 0;
    WorkerStat.create = function create2(properties) {
      return new WorkerStat(properties);
    };
    WorkerStat.encode = function encode(message, writer) {
      if (!writer)
        writer = $Writer.create();
      if (message.lastHeartbeatForBlock != null && Object.hasOwnProperty.call(message, "lastHeartbeatForBlock"))
        writer.uint32(8).uint32(message.lastHeartbeatForBlock);
      if (message.lastHeartbeatAtBlock != null && Object.hasOwnProperty.call(message, "lastHeartbeatAtBlock"))
        writer.uint32(16).uint32(message.lastHeartbeatAtBlock);
      if (message.lastGkResponsiveEvent != null && Object.hasOwnProperty.call(message, "lastGkResponsiveEvent"))
        writer.uint32(24).int32(message.lastGkResponsiveEvent);
      if (message.lastGkResponsiveEventAtBlock != null && Object.hasOwnProperty.call(message, "lastGkResponsiveEventAtBlock"))
        writer.uint32(32).uint32(message.lastGkResponsiveEventAtBlock);
      return writer;
    };
    WorkerStat.encodeDelimited = function encodeDelimited(message, writer) {
      return this.encode(message, writer).ldelim();
    };
    WorkerStat.decode = function decode(reader, length) {
      if (!(reader instanceof $Reader))
        reader = $Reader.create(reader);
      let end = length === void 0 ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.WorkerStat();
      while (reader.pos < end) {
        let tag = reader.uint32();
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
    WorkerStat.decodeDelimited = function decodeDelimited(reader) {
      if (!(reader instanceof $Reader))
        reader = new $Reader(reader);
      return this.decode(reader, reader.uint32());
    };
    WorkerStat.verify = function verify(message) {
      if (typeof message !== "object" || message === null)
        return "object expected";
      if (message.lastHeartbeatForBlock != null && message.hasOwnProperty("lastHeartbeatForBlock")) {
        if (!$util.isInteger(message.lastHeartbeatForBlock))
          return "lastHeartbeatForBlock: integer expected";
      }
      if (message.lastHeartbeatAtBlock != null && message.hasOwnProperty("lastHeartbeatAtBlock")) {
        if (!$util.isInteger(message.lastHeartbeatAtBlock))
          return "lastHeartbeatAtBlock: integer expected";
      }
      if (message.lastGkResponsiveEvent != null && message.hasOwnProperty("lastGkResponsiveEvent"))
        switch (message.lastGkResponsiveEvent) {
          default:
            return "lastGkResponsiveEvent: enum value expected";
          case 0:
          case 1:
          case 2:
            break;
        }
      if (message.lastGkResponsiveEventAtBlock != null && message.hasOwnProperty("lastGkResponsiveEventAtBlock")) {
        if (!$util.isInteger(message.lastGkResponsiveEventAtBlock))
          return "lastGkResponsiveEventAtBlock: integer expected";
      }
      return null;
    };
    WorkerStat.fromObject = function fromObject(object) {
      if (object instanceof $root.pruntime_rpc.WorkerStat)
        return object;
      let message = new $root.pruntime_rpc.WorkerStat();
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
    WorkerStat.toObject = function toObject(message, options) {
      if (!options)
        options = {};
      let object = {};
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
    WorkerStat.prototype.toJSON = function toJSON() {
      return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
    };
    return WorkerStat;
  }();
  pruntime_rpc2.WorkerState = function() {
    function WorkerState(properties) {
      this.waitingHeartbeats = [];
      if (properties) {
        for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
          if (properties[keys[i]] != null)
            this[keys[i]] = properties[keys[i]];
      }
    }
    WorkerState.prototype.registered = false;
    WorkerState.prototype.unresponsive = false;
    WorkerState.prototype.benchState = null;
    WorkerState.prototype.miningState = null;
    WorkerState.prototype.waitingHeartbeats = $util.emptyArray;
    WorkerState.prototype.tokenomicInfo = null;
    WorkerState.prototype.stat = null;
    WorkerState.create = function create2(properties) {
      return new WorkerState(properties);
    };
    WorkerState.encode = function encode(message, writer) {
      if (!writer)
        writer = $Writer.create();
      if (message.registered != null && Object.hasOwnProperty.call(message, "registered"))
        writer.uint32(8).bool(message.registered);
      if (message.unresponsive != null && Object.hasOwnProperty.call(message, "unresponsive"))
        writer.uint32(16).bool(message.unresponsive);
      if (message.benchState != null && Object.hasOwnProperty.call(message, "benchState"))
        $root.pruntime_rpc.BenchState.encode(
          message.benchState,
          writer.uint32(26).fork()
        ).ldelim();
      if (message.miningState != null && Object.hasOwnProperty.call(message, "miningState"))
        $root.pruntime_rpc.MiningState.encode(
          message.miningState,
          writer.uint32(34).fork()
        ).ldelim();
      if (message.waitingHeartbeats != null && message.waitingHeartbeats.length) {
        writer.uint32(42).fork();
        for (let i = 0; i < message.waitingHeartbeats.length; ++i)
          writer.uint32(message.waitingHeartbeats[i]);
        writer.ldelim();
      }
      if (message.tokenomicInfo != null && Object.hasOwnProperty.call(message, "tokenomicInfo"))
        $root.pruntime_rpc.TokenomicInfo.encode(
          message.tokenomicInfo,
          writer.uint32(82).fork()
        ).ldelim();
      if (message.stat != null && Object.hasOwnProperty.call(message, "stat"))
        $root.pruntime_rpc.WorkerStat.encode(
          message.stat,
          writer.uint32(90).fork()
        ).ldelim();
      return writer;
    };
    WorkerState.encodeDelimited = function encodeDelimited(message, writer) {
      return this.encode(message, writer).ldelim();
    };
    WorkerState.decode = function decode(reader, length) {
      if (!(reader instanceof $Reader))
        reader = $Reader.create(reader);
      let end = length === void 0 ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.WorkerState();
      while (reader.pos < end) {
        let tag = reader.uint32();
        switch (tag >>> 3) {
          case 1:
            message.registered = reader.bool();
            break;
          case 2:
            message.unresponsive = reader.bool();
            break;
          case 3:
            message.benchState = $root.pruntime_rpc.BenchState.decode(
              reader,
              reader.uint32()
            );
            break;
          case 4:
            message.miningState = $root.pruntime_rpc.MiningState.decode(
              reader,
              reader.uint32()
            );
            break;
          case 5:
            if (!(message.waitingHeartbeats && message.waitingHeartbeats.length))
              message.waitingHeartbeats = [];
            if ((tag & 7) === 2) {
              let end2 = reader.uint32() + reader.pos;
              while (reader.pos < end2)
                message.waitingHeartbeats.push(reader.uint32());
            } else
              message.waitingHeartbeats.push(reader.uint32());
            break;
          case 10:
            message.tokenomicInfo = $root.pruntime_rpc.TokenomicInfo.decode(
              reader,
              reader.uint32()
            );
            break;
          case 11:
            message.stat = $root.pruntime_rpc.WorkerStat.decode(
              reader,
              reader.uint32()
            );
            break;
          default:
            reader.skipType(tag & 7);
            break;
        }
      }
      return message;
    };
    WorkerState.decodeDelimited = function decodeDelimited(reader) {
      if (!(reader instanceof $Reader))
        reader = new $Reader(reader);
      return this.decode(reader, reader.uint32());
    };
    WorkerState.verify = function verify(message) {
      if (typeof message !== "object" || message === null)
        return "object expected";
      if (message.registered != null && message.hasOwnProperty("registered")) {
        if (typeof message.registered !== "boolean")
          return "registered: boolean expected";
      }
      if (message.unresponsive != null && message.hasOwnProperty("unresponsive")) {
        if (typeof message.unresponsive !== "boolean")
          return "unresponsive: boolean expected";
      }
      if (message.benchState != null && message.hasOwnProperty("benchState")) {
        let error = $root.pruntime_rpc.BenchState.verify(message.benchState);
        if (error)
          return "benchState." + error;
      }
      if (message.miningState != null && message.hasOwnProperty("miningState")) {
        let error = $root.pruntime_rpc.MiningState.verify(message.miningState);
        if (error)
          return "miningState." + error;
      }
      if (message.waitingHeartbeats != null && message.hasOwnProperty("waitingHeartbeats")) {
        if (!Array.isArray(message.waitingHeartbeats))
          return "waitingHeartbeats: array expected";
        for (let i = 0; i < message.waitingHeartbeats.length; ++i)
          if (!$util.isInteger(message.waitingHeartbeats[i]))
            return "waitingHeartbeats: integer[] expected";
      }
      if (message.tokenomicInfo != null && message.hasOwnProperty("tokenomicInfo")) {
        let error = $root.pruntime_rpc.TokenomicInfo.verify(
          message.tokenomicInfo
        );
        if (error)
          return "tokenomicInfo." + error;
      }
      if (message.stat != null && message.hasOwnProperty("stat")) {
        let error = $root.pruntime_rpc.WorkerStat.verify(message.stat);
        if (error)
          return "stat." + error;
      }
      return null;
    };
    WorkerState.fromObject = function fromObject(object) {
      if (object instanceof $root.pruntime_rpc.WorkerState)
        return object;
      let message = new $root.pruntime_rpc.WorkerState();
      if (object.registered != null)
        message.registered = Boolean(object.registered);
      if (object.unresponsive != null)
        message.unresponsive = Boolean(object.unresponsive);
      if (object.benchState != null) {
        if (typeof object.benchState !== "object")
          throw TypeError(
            ".pruntime_rpc.WorkerState.benchState: object expected"
          );
        message.benchState = $root.pruntime_rpc.BenchState.fromObject(
          object.benchState
        );
      }
      if (object.miningState != null) {
        if (typeof object.miningState !== "object")
          throw TypeError(
            ".pruntime_rpc.WorkerState.miningState: object expected"
          );
        message.miningState = $root.pruntime_rpc.MiningState.fromObject(
          object.miningState
        );
      }
      if (object.waitingHeartbeats) {
        if (!Array.isArray(object.waitingHeartbeats))
          throw TypeError(
            ".pruntime_rpc.WorkerState.waitingHeartbeats: array expected"
          );
        message.waitingHeartbeats = [];
        for (let i = 0; i < object.waitingHeartbeats.length; ++i)
          message.waitingHeartbeats[i] = object.waitingHeartbeats[i] >>> 0;
      }
      if (object.tokenomicInfo != null) {
        if (typeof object.tokenomicInfo !== "object")
          throw TypeError(
            ".pruntime_rpc.WorkerState.tokenomicInfo: object expected"
          );
        message.tokenomicInfo = $root.pruntime_rpc.TokenomicInfo.fromObject(
          object.tokenomicInfo
        );
      }
      if (object.stat != null) {
        if (typeof object.stat !== "object")
          throw TypeError(".pruntime_rpc.WorkerState.stat: object expected");
        message.stat = $root.pruntime_rpc.WorkerStat.fromObject(object.stat);
      }
      return message;
    };
    WorkerState.toObject = function toObject(message, options) {
      if (!options)
        options = {};
      let object = {};
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
        object.benchState = $root.pruntime_rpc.BenchState.toObject(
          message.benchState,
          options
        );
      if (message.miningState != null && message.hasOwnProperty("miningState"))
        object.miningState = $root.pruntime_rpc.MiningState.toObject(
          message.miningState,
          options
        );
      if (message.waitingHeartbeats && message.waitingHeartbeats.length) {
        object.waitingHeartbeats = [];
        for (let j = 0; j < message.waitingHeartbeats.length; ++j)
          object.waitingHeartbeats[j] = message.waitingHeartbeats[j];
      }
      if (message.tokenomicInfo != null && message.hasOwnProperty("tokenomicInfo"))
        object.tokenomicInfo = $root.pruntime_rpc.TokenomicInfo.toObject(
          message.tokenomicInfo,
          options
        );
      if (message.stat != null && message.hasOwnProperty("stat"))
        object.stat = $root.pruntime_rpc.WorkerStat.toObject(
          message.stat,
          options
        );
      return object;
    };
    WorkerState.prototype.toJSON = function toJSON() {
      return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
    };
    return WorkerState;
  }();
  pruntime_rpc2.HandoverChallenge = function() {
    function HandoverChallenge(properties) {
      if (properties) {
        for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
          if (properties[keys[i]] != null)
            this[keys[i]] = properties[keys[i]];
      }
    }
    HandoverChallenge.prototype.encodedChallenge = $util.newBuffer([]);
    HandoverChallenge.create = function create2(properties) {
      return new HandoverChallenge(properties);
    };
    HandoverChallenge.encode = function encode(message, writer) {
      if (!writer)
        writer = $Writer.create();
      if (message.encodedChallenge != null && Object.hasOwnProperty.call(message, "encodedChallenge"))
        writer.uint32(10).bytes(message.encodedChallenge);
      return writer;
    };
    HandoverChallenge.encodeDelimited = function encodeDelimited(message, writer) {
      return this.encode(message, writer).ldelim();
    };
    HandoverChallenge.decode = function decode(reader, length) {
      if (!(reader instanceof $Reader))
        reader = $Reader.create(reader);
      let end = length === void 0 ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.HandoverChallenge();
      while (reader.pos < end) {
        let tag = reader.uint32();
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
    HandoverChallenge.decodeDelimited = function decodeDelimited(reader) {
      if (!(reader instanceof $Reader))
        reader = new $Reader(reader);
      return this.decode(reader, reader.uint32());
    };
    HandoverChallenge.verify = function verify(message) {
      if (typeof message !== "object" || message === null)
        return "object expected";
      if (message.encodedChallenge != null && message.hasOwnProperty("encodedChallenge")) {
        if (!(message.encodedChallenge && typeof message.encodedChallenge.length === "number" || $util.isString(message.encodedChallenge)))
          return "encodedChallenge: buffer expected";
      }
      return null;
    };
    HandoverChallenge.fromObject = function fromObject(object) {
      if (object instanceof $root.pruntime_rpc.HandoverChallenge)
        return object;
      let message = new $root.pruntime_rpc.HandoverChallenge();
      if (object.encodedChallenge != null) {
        if (typeof object.encodedChallenge === "string")
          $util.base64.decode(
            object.encodedChallenge,
            message.encodedChallenge = $util.newBuffer(
              $util.base64.length(object.encodedChallenge)
            ),
            0
          );
        else if (object.encodedChallenge.length)
          message.encodedChallenge = object.encodedChallenge;
      }
      return message;
    };
    HandoverChallenge.toObject = function toObject(message, options) {
      if (!options)
        options = {};
      let object = {};
      if (options.defaults)
        if (options.bytes === String)
          object.encodedChallenge = "";
        else {
          object.encodedChallenge = [];
          if (options.bytes !== Array)
            object.encodedChallenge = $util.newBuffer(object.encodedChallenge);
        }
      if (message.encodedChallenge != null && message.hasOwnProperty("encodedChallenge"))
        object.encodedChallenge = options.bytes === String ? $util.base64.encode(
          message.encodedChallenge,
          0,
          message.encodedChallenge.length
        ) : options.bytes === Array ? Array.prototype.slice.call(message.encodedChallenge) : message.encodedChallenge;
      return object;
    };
    HandoverChallenge.prototype.toJSON = function toJSON() {
      return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
    };
    return HandoverChallenge;
  }();
  pruntime_rpc2.HandoverChallengeResponse = function() {
    function HandoverChallengeResponse(properties) {
      if (properties) {
        for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
          if (properties[keys[i]] != null)
            this[keys[i]] = properties[keys[i]];
      }
    }
    HandoverChallengeResponse.prototype.encodedChallengeHandler = $util.newBuffer([]);
    HandoverChallengeResponse.prototype.attestation = null;
    HandoverChallengeResponse.create = function create2(properties) {
      return new HandoverChallengeResponse(properties);
    };
    HandoverChallengeResponse.encode = function encode(message, writer) {
      if (!writer)
        writer = $Writer.create();
      if (message.encodedChallengeHandler != null && Object.hasOwnProperty.call(message, "encodedChallengeHandler"))
        writer.uint32(10).bytes(message.encodedChallengeHandler);
      if (message.attestation != null && Object.hasOwnProperty.call(message, "attestation"))
        $root.pruntime_rpc.Attestation.encode(
          message.attestation,
          writer.uint32(18).fork()
        ).ldelim();
      return writer;
    };
    HandoverChallengeResponse.encodeDelimited = function encodeDelimited(message, writer) {
      return this.encode(message, writer).ldelim();
    };
    HandoverChallengeResponse.decode = function decode(reader, length) {
      if (!(reader instanceof $Reader))
        reader = $Reader.create(reader);
      let end = length === void 0 ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.HandoverChallengeResponse();
      while (reader.pos < end) {
        let tag = reader.uint32();
        switch (tag >>> 3) {
          case 1:
            message.encodedChallengeHandler = reader.bytes();
            break;
          case 2:
            message.attestation = $root.pruntime_rpc.Attestation.decode(
              reader,
              reader.uint32()
            );
            break;
          default:
            reader.skipType(tag & 7);
            break;
        }
      }
      return message;
    };
    HandoverChallengeResponse.decodeDelimited = function decodeDelimited(reader) {
      if (!(reader instanceof $Reader))
        reader = new $Reader(reader);
      return this.decode(reader, reader.uint32());
    };
    HandoverChallengeResponse.verify = function verify(message) {
      if (typeof message !== "object" || message === null)
        return "object expected";
      if (message.encodedChallengeHandler != null && message.hasOwnProperty("encodedChallengeHandler")) {
        if (!(message.encodedChallengeHandler && typeof message.encodedChallengeHandler.length === "number" || $util.isString(message.encodedChallengeHandler)))
          return "encodedChallengeHandler: buffer expected";
      }
      if (message.attestation != null && message.hasOwnProperty("attestation")) {
        let error = $root.pruntime_rpc.Attestation.verify(message.attestation);
        if (error)
          return "attestation." + error;
      }
      return null;
    };
    HandoverChallengeResponse.fromObject = function fromObject(object) {
      if (object instanceof $root.pruntime_rpc.HandoverChallengeResponse)
        return object;
      let message = new $root.pruntime_rpc.HandoverChallengeResponse();
      if (object.encodedChallengeHandler != null) {
        if (typeof object.encodedChallengeHandler === "string")
          $util.base64.decode(
            object.encodedChallengeHandler,
            message.encodedChallengeHandler = $util.newBuffer(
              $util.base64.length(object.encodedChallengeHandler)
            ),
            0
          );
        else if (object.encodedChallengeHandler.length)
          message.encodedChallengeHandler = object.encodedChallengeHandler;
      }
      if (object.attestation != null) {
        if (typeof object.attestation !== "object")
          throw TypeError(
            ".pruntime_rpc.HandoverChallengeResponse.attestation: object expected"
          );
        message.attestation = $root.pruntime_rpc.Attestation.fromObject(
          object.attestation
        );
      }
      return message;
    };
    HandoverChallengeResponse.toObject = function toObject(message, options) {
      if (!options)
        options = {};
      let object = {};
      if (options.defaults) {
        if (options.bytes === String)
          object.encodedChallengeHandler = "";
        else {
          object.encodedChallengeHandler = [];
          if (options.bytes !== Array)
            object.encodedChallengeHandler = $util.newBuffer(
              object.encodedChallengeHandler
            );
        }
        object.attestation = null;
      }
      if (message.encodedChallengeHandler != null && message.hasOwnProperty("encodedChallengeHandler"))
        object.encodedChallengeHandler = options.bytes === String ? $util.base64.encode(
          message.encodedChallengeHandler,
          0,
          message.encodedChallengeHandler.length
        ) : options.bytes === Array ? Array.prototype.slice.call(message.encodedChallengeHandler) : message.encodedChallengeHandler;
      if (message.attestation != null && message.hasOwnProperty("attestation"))
        object.attestation = $root.pruntime_rpc.Attestation.toObject(
          message.attestation,
          options
        );
      return object;
    };
    HandoverChallengeResponse.prototype.toJSON = function toJSON() {
      return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
    };
    return HandoverChallengeResponse;
  }();
  pruntime_rpc2.HandoverWorkerKey = function() {
    function HandoverWorkerKey(properties) {
      if (properties) {
        for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
          if (properties[keys[i]] != null)
            this[keys[i]] = properties[keys[i]];
      }
    }
    HandoverWorkerKey.prototype.encodedWorkerKey = $util.newBuffer([]);
    HandoverWorkerKey.prototype.attestation = null;
    HandoverWorkerKey.create = function create2(properties) {
      return new HandoverWorkerKey(properties);
    };
    HandoverWorkerKey.encode = function encode(message, writer) {
      if (!writer)
        writer = $Writer.create();
      if (message.encodedWorkerKey != null && Object.hasOwnProperty.call(message, "encodedWorkerKey"))
        writer.uint32(10).bytes(message.encodedWorkerKey);
      if (message.attestation != null && Object.hasOwnProperty.call(message, "attestation"))
        $root.pruntime_rpc.Attestation.encode(
          message.attestation,
          writer.uint32(18).fork()
        ).ldelim();
      return writer;
    };
    HandoverWorkerKey.encodeDelimited = function encodeDelimited(message, writer) {
      return this.encode(message, writer).ldelim();
    };
    HandoverWorkerKey.decode = function decode(reader, length) {
      if (!(reader instanceof $Reader))
        reader = $Reader.create(reader);
      let end = length === void 0 ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.HandoverWorkerKey();
      while (reader.pos < end) {
        let tag = reader.uint32();
        switch (tag >>> 3) {
          case 1:
            message.encodedWorkerKey = reader.bytes();
            break;
          case 2:
            message.attestation = $root.pruntime_rpc.Attestation.decode(
              reader,
              reader.uint32()
            );
            break;
          default:
            reader.skipType(tag & 7);
            break;
        }
      }
      return message;
    };
    HandoverWorkerKey.decodeDelimited = function decodeDelimited(reader) {
      if (!(reader instanceof $Reader))
        reader = new $Reader(reader);
      return this.decode(reader, reader.uint32());
    };
    HandoverWorkerKey.verify = function verify(message) {
      if (typeof message !== "object" || message === null)
        return "object expected";
      if (message.encodedWorkerKey != null && message.hasOwnProperty("encodedWorkerKey")) {
        if (!(message.encodedWorkerKey && typeof message.encodedWorkerKey.length === "number" || $util.isString(message.encodedWorkerKey)))
          return "encodedWorkerKey: buffer expected";
      }
      if (message.attestation != null && message.hasOwnProperty("attestation")) {
        let error = $root.pruntime_rpc.Attestation.verify(message.attestation);
        if (error)
          return "attestation." + error;
      }
      return null;
    };
    HandoverWorkerKey.fromObject = function fromObject(object) {
      if (object instanceof $root.pruntime_rpc.HandoverWorkerKey)
        return object;
      let message = new $root.pruntime_rpc.HandoverWorkerKey();
      if (object.encodedWorkerKey != null) {
        if (typeof object.encodedWorkerKey === "string")
          $util.base64.decode(
            object.encodedWorkerKey,
            message.encodedWorkerKey = $util.newBuffer(
              $util.base64.length(object.encodedWorkerKey)
            ),
            0
          );
        else if (object.encodedWorkerKey.length)
          message.encodedWorkerKey = object.encodedWorkerKey;
      }
      if (object.attestation != null) {
        if (typeof object.attestation !== "object")
          throw TypeError(
            ".pruntime_rpc.HandoverWorkerKey.attestation: object expected"
          );
        message.attestation = $root.pruntime_rpc.Attestation.fromObject(
          object.attestation
        );
      }
      return message;
    };
    HandoverWorkerKey.toObject = function toObject(message, options) {
      if (!options)
        options = {};
      let object = {};
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
        object.encodedWorkerKey = options.bytes === String ? $util.base64.encode(
          message.encodedWorkerKey,
          0,
          message.encodedWorkerKey.length
        ) : options.bytes === Array ? Array.prototype.slice.call(message.encodedWorkerKey) : message.encodedWorkerKey;
      if (message.attestation != null && message.hasOwnProperty("attestation"))
        object.attestation = $root.pruntime_rpc.Attestation.toObject(
          message.attestation,
          options
        );
      return object;
    };
    HandoverWorkerKey.prototype.toJSON = function toJSON() {
      return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
    };
    return HandoverWorkerKey;
  }();
  pruntime_rpc2.BenchState = function() {
    function BenchState(properties) {
      if (properties) {
        for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
          if (properties[keys[i]] != null)
            this[keys[i]] = properties[keys[i]];
      }
    }
    BenchState.prototype.startBlock = 0;
    BenchState.prototype.startTime = $util.Long ? $util.Long.fromBits(0, 0, true) : 0;
    BenchState.prototype.duration = 0;
    BenchState.create = function create2(properties) {
      return new BenchState(properties);
    };
    BenchState.encode = function encode(message, writer) {
      if (!writer)
        writer = $Writer.create();
      if (message.startBlock != null && Object.hasOwnProperty.call(message, "startBlock"))
        writer.uint32(8).uint32(message.startBlock);
      if (message.startTime != null && Object.hasOwnProperty.call(message, "startTime"))
        writer.uint32(16).uint64(message.startTime);
      if (message.duration != null && Object.hasOwnProperty.call(message, "duration"))
        writer.uint32(32).uint32(message.duration);
      return writer;
    };
    BenchState.encodeDelimited = function encodeDelimited(message, writer) {
      return this.encode(message, writer).ldelim();
    };
    BenchState.decode = function decode(reader, length) {
      if (!(reader instanceof $Reader))
        reader = $Reader.create(reader);
      let end = length === void 0 ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.BenchState();
      while (reader.pos < end) {
        let tag = reader.uint32();
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
    BenchState.decodeDelimited = function decodeDelimited(reader) {
      if (!(reader instanceof $Reader))
        reader = new $Reader(reader);
      return this.decode(reader, reader.uint32());
    };
    BenchState.verify = function verify(message) {
      if (typeof message !== "object" || message === null)
        return "object expected";
      if (message.startBlock != null && message.hasOwnProperty("startBlock")) {
        if (!$util.isInteger(message.startBlock))
          return "startBlock: integer expected";
      }
      if (message.startTime != null && message.hasOwnProperty("startTime")) {
        if (!$util.isInteger(message.startTime) && !(message.startTime && $util.isInteger(message.startTime.low) && $util.isInteger(message.startTime.high)))
          return "startTime: integer|Long expected";
      }
      if (message.duration != null && message.hasOwnProperty("duration")) {
        if (!$util.isInteger(message.duration))
          return "duration: integer expected";
      }
      return null;
    };
    BenchState.fromObject = function fromObject(object) {
      if (object instanceof $root.pruntime_rpc.BenchState)
        return object;
      let message = new $root.pruntime_rpc.BenchState();
      if (object.startBlock != null)
        message.startBlock = object.startBlock >>> 0;
      if (object.startTime != null) {
        if ($util.Long)
          (message.startTime = $util.Long.fromValue(
            object.startTime
          )).unsigned = true;
        else if (typeof object.startTime === "string")
          message.startTime = parseInt(object.startTime, 10);
        else if (typeof object.startTime === "number")
          message.startTime = object.startTime;
        else if (typeof object.startTime === "object")
          message.startTime = new $util.LongBits(
            object.startTime.low >>> 0,
            object.startTime.high >>> 0
          ).toNumber(true);
      }
      if (object.duration != null)
        message.duration = object.duration >>> 0;
      return message;
    };
    BenchState.toObject = function toObject(message, options) {
      if (!options)
        options = {};
      let object = {};
      if (options.defaults) {
        object.startBlock = 0;
        if ($util.Long) {
          let long = new $util.Long(0, 0, true);
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
          object.startTime = options.longs === String ? $util.Long.prototype.toString.call(message.startTime) : options.longs === Number ? new $util.LongBits(
            message.startTime.low >>> 0,
            message.startTime.high >>> 0
          ).toNumber(true) : message.startTime;
      if (message.duration != null && message.hasOwnProperty("duration"))
        object.duration = message.duration;
      return object;
    };
    BenchState.prototype.toJSON = function toJSON() {
      return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
    };
    return BenchState;
  }();
  pruntime_rpc2.MiningState = function() {
    function MiningState(properties) {
      if (properties) {
        for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
          if (properties[keys[i]] != null)
            this[keys[i]] = properties[keys[i]];
      }
    }
    MiningState.prototype.sessionId = 0;
    MiningState.prototype.paused = false;
    MiningState.prototype.startTime = $util.Long ? $util.Long.fromBits(0, 0, true) : 0;
    MiningState.create = function create2(properties) {
      return new MiningState(properties);
    };
    MiningState.encode = function encode(message, writer) {
      if (!writer)
        writer = $Writer.create();
      if (message.sessionId != null && Object.hasOwnProperty.call(message, "sessionId"))
        writer.uint32(8).uint32(message.sessionId);
      if (message.paused != null && Object.hasOwnProperty.call(message, "paused"))
        writer.uint32(16).bool(message.paused);
      if (message.startTime != null && Object.hasOwnProperty.call(message, "startTime"))
        writer.uint32(24).uint64(message.startTime);
      return writer;
    };
    MiningState.encodeDelimited = function encodeDelimited(message, writer) {
      return this.encode(message, writer).ldelim();
    };
    MiningState.decode = function decode(reader, length) {
      if (!(reader instanceof $Reader))
        reader = $Reader.create(reader);
      let end = length === void 0 ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.MiningState();
      while (reader.pos < end) {
        let tag = reader.uint32();
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
    MiningState.decodeDelimited = function decodeDelimited(reader) {
      if (!(reader instanceof $Reader))
        reader = new $Reader(reader);
      return this.decode(reader, reader.uint32());
    };
    MiningState.verify = function verify(message) {
      if (typeof message !== "object" || message === null)
        return "object expected";
      if (message.sessionId != null && message.hasOwnProperty("sessionId")) {
        if (!$util.isInteger(message.sessionId))
          return "sessionId: integer expected";
      }
      if (message.paused != null && message.hasOwnProperty("paused")) {
        if (typeof message.paused !== "boolean")
          return "paused: boolean expected";
      }
      if (message.startTime != null && message.hasOwnProperty("startTime")) {
        if (!$util.isInteger(message.startTime) && !(message.startTime && $util.isInteger(message.startTime.low) && $util.isInteger(message.startTime.high)))
          return "startTime: integer|Long expected";
      }
      return null;
    };
    MiningState.fromObject = function fromObject(object) {
      if (object instanceof $root.pruntime_rpc.MiningState)
        return object;
      let message = new $root.pruntime_rpc.MiningState();
      if (object.sessionId != null)
        message.sessionId = object.sessionId >>> 0;
      if (object.paused != null)
        message.paused = Boolean(object.paused);
      if (object.startTime != null) {
        if ($util.Long)
          (message.startTime = $util.Long.fromValue(
            object.startTime
          )).unsigned = true;
        else if (typeof object.startTime === "string")
          message.startTime = parseInt(object.startTime, 10);
        else if (typeof object.startTime === "number")
          message.startTime = object.startTime;
        else if (typeof object.startTime === "object")
          message.startTime = new $util.LongBits(
            object.startTime.low >>> 0,
            object.startTime.high >>> 0
          ).toNumber(true);
      }
      return message;
    };
    MiningState.toObject = function toObject(message, options) {
      if (!options)
        options = {};
      let object = {};
      if (options.defaults) {
        object.sessionId = 0;
        object.paused = false;
        if ($util.Long) {
          let long = new $util.Long(0, 0, true);
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
          object.startTime = options.longs === String ? $util.Long.prototype.toString.call(message.startTime) : options.longs === Number ? new $util.LongBits(
            message.startTime.low >>> 0,
            message.startTime.high >>> 0
          ).toNumber(true) : message.startTime;
      return object;
    };
    MiningState.prototype.toJSON = function toJSON() {
      return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
    };
    return MiningState;
  }();
  pruntime_rpc2.EchoMessage = function() {
    function EchoMessage(properties) {
      if (properties) {
        for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
          if (properties[keys[i]] != null)
            this[keys[i]] = properties[keys[i]];
      }
    }
    EchoMessage.prototype.echoMsg = $util.newBuffer([]);
    EchoMessage.create = function create2(properties) {
      return new EchoMessage(properties);
    };
    EchoMessage.encode = function encode(message, writer) {
      if (!writer)
        writer = $Writer.create();
      if (message.echoMsg != null && Object.hasOwnProperty.call(message, "echoMsg"))
        writer.uint32(10).bytes(message.echoMsg);
      return writer;
    };
    EchoMessage.encodeDelimited = function encodeDelimited(message, writer) {
      return this.encode(message, writer).ldelim();
    };
    EchoMessage.decode = function decode(reader, length) {
      if (!(reader instanceof $Reader))
        reader = $Reader.create(reader);
      let end = length === void 0 ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.EchoMessage();
      while (reader.pos < end) {
        let tag = reader.uint32();
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
    EchoMessage.decodeDelimited = function decodeDelimited(reader) {
      if (!(reader instanceof $Reader))
        reader = new $Reader(reader);
      return this.decode(reader, reader.uint32());
    };
    EchoMessage.verify = function verify(message) {
      if (typeof message !== "object" || message === null)
        return "object expected";
      if (message.echoMsg != null && message.hasOwnProperty("echoMsg")) {
        if (!(message.echoMsg && typeof message.echoMsg.length === "number" || $util.isString(message.echoMsg)))
          return "echoMsg: buffer expected";
      }
      return null;
    };
    EchoMessage.fromObject = function fromObject(object) {
      if (object instanceof $root.pruntime_rpc.EchoMessage)
        return object;
      let message = new $root.pruntime_rpc.EchoMessage();
      if (object.echoMsg != null) {
        if (typeof object.echoMsg === "string")
          $util.base64.decode(
            object.echoMsg,
            message.echoMsg = $util.newBuffer(
              $util.base64.length(object.echoMsg)
            ),
            0
          );
        else if (object.echoMsg.length)
          message.echoMsg = object.echoMsg;
      }
      return message;
    };
    EchoMessage.toObject = function toObject(message, options) {
      if (!options)
        options = {};
      let object = {};
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
    EchoMessage.prototype.toJSON = function toJSON() {
      return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
    };
    return EchoMessage;
  }();
  pruntime_rpc2.ResponsiveEvent = function() {
    const valuesById = {}, values = Object.create(valuesById);
    values[valuesById[0] = "NoEvent"] = 0;
    values[valuesById[1] = "EnterUnresponsive"] = 1;
    values[valuesById[2] = "ExitUnresponsive"] = 2;
    return values;
  }();
  pruntime_rpc2.AddEndpointRequest = function() {
    function AddEndpointRequest(properties) {
      if (properties) {
        for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
          if (properties[keys[i]] != null)
            this[keys[i]] = properties[keys[i]];
      }
    }
    AddEndpointRequest.prototype.encodedEndpointType = $util.newBuffer([]);
    AddEndpointRequest.prototype.endpoint = "";
    AddEndpointRequest.create = function create2(properties) {
      return new AddEndpointRequest(properties);
    };
    AddEndpointRequest.encode = function encode(message, writer) {
      if (!writer)
        writer = $Writer.create();
      if (message.encodedEndpointType != null && Object.hasOwnProperty.call(message, "encodedEndpointType"))
        writer.uint32(10).bytes(message.encodedEndpointType);
      if (message.endpoint != null && Object.hasOwnProperty.call(message, "endpoint"))
        writer.uint32(18).string(message.endpoint);
      return writer;
    };
    AddEndpointRequest.encodeDelimited = function encodeDelimited(message, writer) {
      return this.encode(message, writer).ldelim();
    };
    AddEndpointRequest.decode = function decode(reader, length) {
      if (!(reader instanceof $Reader))
        reader = $Reader.create(reader);
      let end = length === void 0 ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.AddEndpointRequest();
      while (reader.pos < end) {
        let tag = reader.uint32();
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
    AddEndpointRequest.decodeDelimited = function decodeDelimited(reader) {
      if (!(reader instanceof $Reader))
        reader = new $Reader(reader);
      return this.decode(reader, reader.uint32());
    };
    AddEndpointRequest.verify = function verify(message) {
      if (typeof message !== "object" || message === null)
        return "object expected";
      if (message.encodedEndpointType != null && message.hasOwnProperty("encodedEndpointType")) {
        if (!(message.encodedEndpointType && typeof message.encodedEndpointType.length === "number" || $util.isString(message.encodedEndpointType)))
          return "encodedEndpointType: buffer expected";
      }
      if (message.endpoint != null && message.hasOwnProperty("endpoint")) {
        if (!$util.isString(message.endpoint))
          return "endpoint: string expected";
      }
      return null;
    };
    AddEndpointRequest.fromObject = function fromObject(object) {
      if (object instanceof $root.pruntime_rpc.AddEndpointRequest)
        return object;
      let message = new $root.pruntime_rpc.AddEndpointRequest();
      if (object.encodedEndpointType != null) {
        if (typeof object.encodedEndpointType === "string")
          $util.base64.decode(
            object.encodedEndpointType,
            message.encodedEndpointType = $util.newBuffer(
              $util.base64.length(object.encodedEndpointType)
            ),
            0
          );
        else if (object.encodedEndpointType.length)
          message.encodedEndpointType = object.encodedEndpointType;
      }
      if (object.endpoint != null)
        message.endpoint = String(object.endpoint);
      return message;
    };
    AddEndpointRequest.toObject = function toObject(message, options) {
      if (!options)
        options = {};
      let object = {};
      if (options.defaults) {
        if (options.bytes === String)
          object.encodedEndpointType = "";
        else {
          object.encodedEndpointType = [];
          if (options.bytes !== Array)
            object.encodedEndpointType = $util.newBuffer(
              object.encodedEndpointType
            );
        }
        object.endpoint = "";
      }
      if (message.encodedEndpointType != null && message.hasOwnProperty("encodedEndpointType"))
        object.encodedEndpointType = options.bytes === String ? $util.base64.encode(
          message.encodedEndpointType,
          0,
          message.encodedEndpointType.length
        ) : options.bytes === Array ? Array.prototype.slice.call(message.encodedEndpointType) : message.encodedEndpointType;
      if (message.endpoint != null && message.hasOwnProperty("endpoint"))
        object.endpoint = message.endpoint;
      return object;
    };
    AddEndpointRequest.prototype.toJSON = function toJSON() {
      return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
    };
    return AddEndpointRequest;
  }();
  pruntime_rpc2.GetEndpointResponse = function() {
    function GetEndpointResponse(properties) {
      if (properties) {
        for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
          if (properties[keys[i]] != null)
            this[keys[i]] = properties[keys[i]];
      }
    }
    GetEndpointResponse.prototype.encodedEndpointPayload = null;
    GetEndpointResponse.prototype.signature = null;
    let $oneOfFields;
    Object.defineProperty(
      GetEndpointResponse.prototype,
      "_encodedEndpointPayload",
      {
        get: $util.oneOfGetter($oneOfFields = ["encodedEndpointPayload"]),
        set: $util.oneOfSetter($oneOfFields)
      }
    );
    Object.defineProperty(GetEndpointResponse.prototype, "_signature", {
      get: $util.oneOfGetter($oneOfFields = ["signature"]),
      set: $util.oneOfSetter($oneOfFields)
    });
    GetEndpointResponse.create = function create2(properties) {
      return new GetEndpointResponse(properties);
    };
    GetEndpointResponse.encode = function encode(message, writer) {
      if (!writer)
        writer = $Writer.create();
      if (message.encodedEndpointPayload != null && Object.hasOwnProperty.call(message, "encodedEndpointPayload"))
        writer.uint32(10).bytes(message.encodedEndpointPayload);
      if (message.signature != null && Object.hasOwnProperty.call(message, "signature"))
        writer.uint32(18).bytes(message.signature);
      return writer;
    };
    GetEndpointResponse.encodeDelimited = function encodeDelimited(message, writer) {
      return this.encode(message, writer).ldelim();
    };
    GetEndpointResponse.decode = function decode(reader, length) {
      if (!(reader instanceof $Reader))
        reader = $Reader.create(reader);
      let end = length === void 0 ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.GetEndpointResponse();
      while (reader.pos < end) {
        let tag = reader.uint32();
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
    GetEndpointResponse.decodeDelimited = function decodeDelimited(reader) {
      if (!(reader instanceof $Reader))
        reader = new $Reader(reader);
      return this.decode(reader, reader.uint32());
    };
    GetEndpointResponse.verify = function verify(message) {
      if (typeof message !== "object" || message === null)
        return "object expected";
      let properties = {};
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
    GetEndpointResponse.fromObject = function fromObject(object) {
      if (object instanceof $root.pruntime_rpc.GetEndpointResponse)
        return object;
      let message = new $root.pruntime_rpc.GetEndpointResponse();
      if (object.encodedEndpointPayload != null) {
        if (typeof object.encodedEndpointPayload === "string")
          $util.base64.decode(
            object.encodedEndpointPayload,
            message.encodedEndpointPayload = $util.newBuffer(
              $util.base64.length(object.encodedEndpointPayload)
            ),
            0
          );
        else if (object.encodedEndpointPayload.length)
          message.encodedEndpointPayload = object.encodedEndpointPayload;
      }
      if (object.signature != null) {
        if (typeof object.signature === "string")
          $util.base64.decode(
            object.signature,
            message.signature = $util.newBuffer(
              $util.base64.length(object.signature)
            ),
            0
          );
        else if (object.signature.length)
          message.signature = object.signature;
      }
      return message;
    };
    GetEndpointResponse.toObject = function toObject(message, options) {
      if (!options)
        options = {};
      let object = {};
      if (message.encodedEndpointPayload != null && message.hasOwnProperty("encodedEndpointPayload")) {
        object.encodedEndpointPayload = options.bytes === String ? $util.base64.encode(
          message.encodedEndpointPayload,
          0,
          message.encodedEndpointPayload.length
        ) : options.bytes === Array ? Array.prototype.slice.call(message.encodedEndpointPayload) : message.encodedEndpointPayload;
        if (options.oneofs)
          object._encodedEndpointPayload = "encodedEndpointPayload";
      }
      if (message.signature != null && message.hasOwnProperty("signature")) {
        object.signature = options.bytes === String ? $util.base64.encode(
          message.signature,
          0,
          message.signature.length
        ) : options.bytes === Array ? Array.prototype.slice.call(message.signature) : message.signature;
        if (options.oneofs)
          object._signature = "signature";
      }
      return object;
    };
    GetEndpointResponse.prototype.toJSON = function toJSON() {
      return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
    };
    return GetEndpointResponse;
  }();
  pruntime_rpc2.SignEndpointsRequest = function() {
    function SignEndpointsRequest(properties) {
      if (properties) {
        for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
          if (properties[keys[i]] != null)
            this[keys[i]] = properties[keys[i]];
      }
    }
    SignEndpointsRequest.prototype.encodedEndpoints = $util.newBuffer([]);
    SignEndpointsRequest.create = function create2(properties) {
      return new SignEndpointsRequest(properties);
    };
    SignEndpointsRequest.encode = function encode(message, writer) {
      if (!writer)
        writer = $Writer.create();
      if (message.encodedEndpoints != null && Object.hasOwnProperty.call(message, "encodedEndpoints"))
        writer.uint32(10).bytes(message.encodedEndpoints);
      return writer;
    };
    SignEndpointsRequest.encodeDelimited = function encodeDelimited(message, writer) {
      return this.encode(message, writer).ldelim();
    };
    SignEndpointsRequest.decode = function decode(reader, length) {
      if (!(reader instanceof $Reader))
        reader = $Reader.create(reader);
      let end = length === void 0 ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.SignEndpointsRequest();
      while (reader.pos < end) {
        let tag = reader.uint32();
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
    SignEndpointsRequest.decodeDelimited = function decodeDelimited(reader) {
      if (!(reader instanceof $Reader))
        reader = new $Reader(reader);
      return this.decode(reader, reader.uint32());
    };
    SignEndpointsRequest.verify = function verify(message) {
      if (typeof message !== "object" || message === null)
        return "object expected";
      if (message.encodedEndpoints != null && message.hasOwnProperty("encodedEndpoints")) {
        if (!(message.encodedEndpoints && typeof message.encodedEndpoints.length === "number" || $util.isString(message.encodedEndpoints)))
          return "encodedEndpoints: buffer expected";
      }
      return null;
    };
    SignEndpointsRequest.fromObject = function fromObject(object) {
      if (object instanceof $root.pruntime_rpc.SignEndpointsRequest)
        return object;
      let message = new $root.pruntime_rpc.SignEndpointsRequest();
      if (object.encodedEndpoints != null) {
        if (typeof object.encodedEndpoints === "string")
          $util.base64.decode(
            object.encodedEndpoints,
            message.encodedEndpoints = $util.newBuffer(
              $util.base64.length(object.encodedEndpoints)
            ),
            0
          );
        else if (object.encodedEndpoints.length)
          message.encodedEndpoints = object.encodedEndpoints;
      }
      return message;
    };
    SignEndpointsRequest.toObject = function toObject(message, options) {
      if (!options)
        options = {};
      let object = {};
      if (options.defaults)
        if (options.bytes === String)
          object.encodedEndpoints = "";
        else {
          object.encodedEndpoints = [];
          if (options.bytes !== Array)
            object.encodedEndpoints = $util.newBuffer(object.encodedEndpoints);
        }
      if (message.encodedEndpoints != null && message.hasOwnProperty("encodedEndpoints"))
        object.encodedEndpoints = options.bytes === String ? $util.base64.encode(
          message.encodedEndpoints,
          0,
          message.encodedEndpoints.length
        ) : options.bytes === Array ? Array.prototype.slice.call(message.encodedEndpoints) : message.encodedEndpoints;
      return object;
    };
    SignEndpointsRequest.prototype.toJSON = function toJSON() {
      return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
    };
    return SignEndpointsRequest;
  }();
  pruntime_rpc2.DerivePhalaI2pKeyResponse = function() {
    function DerivePhalaI2pKeyResponse(properties) {
      if (properties) {
        for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
          if (properties[keys[i]] != null)
            this[keys[i]] = properties[keys[i]];
      }
    }
    DerivePhalaI2pKeyResponse.prototype.phalaI2pKey = $util.newBuffer([]);
    DerivePhalaI2pKeyResponse.create = function create2(properties) {
      return new DerivePhalaI2pKeyResponse(properties);
    };
    DerivePhalaI2pKeyResponse.encode = function encode(message, writer) {
      if (!writer)
        writer = $Writer.create();
      if (message.phalaI2pKey != null && Object.hasOwnProperty.call(message, "phalaI2pKey"))
        writer.uint32(10).bytes(message.phalaI2pKey);
      return writer;
    };
    DerivePhalaI2pKeyResponse.encodeDelimited = function encodeDelimited(message, writer) {
      return this.encode(message, writer).ldelim();
    };
    DerivePhalaI2pKeyResponse.decode = function decode(reader, length) {
      if (!(reader instanceof $Reader))
        reader = $Reader.create(reader);
      let end = length === void 0 ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.DerivePhalaI2pKeyResponse();
      while (reader.pos < end) {
        let tag = reader.uint32();
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
    DerivePhalaI2pKeyResponse.decodeDelimited = function decodeDelimited(reader) {
      if (!(reader instanceof $Reader))
        reader = new $Reader(reader);
      return this.decode(reader, reader.uint32());
    };
    DerivePhalaI2pKeyResponse.verify = function verify(message) {
      if (typeof message !== "object" || message === null)
        return "object expected";
      if (message.phalaI2pKey != null && message.hasOwnProperty("phalaI2pKey")) {
        if (!(message.phalaI2pKey && typeof message.phalaI2pKey.length === "number" || $util.isString(message.phalaI2pKey)))
          return "phalaI2pKey: buffer expected";
      }
      return null;
    };
    DerivePhalaI2pKeyResponse.fromObject = function fromObject(object) {
      if (object instanceof $root.pruntime_rpc.DerivePhalaI2pKeyResponse)
        return object;
      let message = new $root.pruntime_rpc.DerivePhalaI2pKeyResponse();
      if (object.phalaI2pKey != null) {
        if (typeof object.phalaI2pKey === "string")
          $util.base64.decode(
            object.phalaI2pKey,
            message.phalaI2pKey = $util.newBuffer(
              $util.base64.length(object.phalaI2pKey)
            ),
            0
          );
        else if (object.phalaI2pKey.length)
          message.phalaI2pKey = object.phalaI2pKey;
      }
      return message;
    };
    DerivePhalaI2pKeyResponse.toObject = function toObject(message, options) {
      if (!options)
        options = {};
      let object = {};
      if (options.defaults)
        if (options.bytes === String)
          object.phalaI2pKey = "";
        else {
          object.phalaI2pKey = [];
          if (options.bytes !== Array)
            object.phalaI2pKey = $util.newBuffer(object.phalaI2pKey);
        }
      if (message.phalaI2pKey != null && message.hasOwnProperty("phalaI2pKey"))
        object.phalaI2pKey = options.bytes === String ? $util.base64.encode(
          message.phalaI2pKey,
          0,
          message.phalaI2pKey.length
        ) : options.bytes === Array ? Array.prototype.slice.call(message.phalaI2pKey) : message.phalaI2pKey;
      return object;
    };
    DerivePhalaI2pKeyResponse.prototype.toJSON = function toJSON() {
      return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
    };
    return DerivePhalaI2pKeyResponse;
  }();
  pruntime_rpc2.TokenomicStat = function() {
    function TokenomicStat(properties) {
      if (properties) {
        for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
          if (properties[keys[i]] != null)
            this[keys[i]] = properties[keys[i]];
      }
    }
    TokenomicStat.prototype.lastPayout = "";
    TokenomicStat.prototype.lastPayoutAtBlock = 0;
    TokenomicStat.prototype.totalPayout = "";
    TokenomicStat.prototype.totalPayoutCount = 0;
    TokenomicStat.prototype.lastSlash = "";
    TokenomicStat.prototype.lastSlashAtBlock = 0;
    TokenomicStat.prototype.totalSlash = "";
    TokenomicStat.prototype.totalSlashCount = 0;
    TokenomicStat.create = function create2(properties) {
      return new TokenomicStat(properties);
    };
    TokenomicStat.encode = function encode(message, writer) {
      if (!writer)
        writer = $Writer.create();
      if (message.lastPayout != null && Object.hasOwnProperty.call(message, "lastPayout"))
        writer.uint32(10).string(message.lastPayout);
      if (message.lastPayoutAtBlock != null && Object.hasOwnProperty.call(message, "lastPayoutAtBlock"))
        writer.uint32(16).uint32(message.lastPayoutAtBlock);
      if (message.totalPayout != null && Object.hasOwnProperty.call(message, "totalPayout"))
        writer.uint32(26).string(message.totalPayout);
      if (message.totalPayoutCount != null && Object.hasOwnProperty.call(message, "totalPayoutCount"))
        writer.uint32(32).uint32(message.totalPayoutCount);
      if (message.lastSlash != null && Object.hasOwnProperty.call(message, "lastSlash"))
        writer.uint32(42).string(message.lastSlash);
      if (message.lastSlashAtBlock != null && Object.hasOwnProperty.call(message, "lastSlashAtBlock"))
        writer.uint32(48).uint32(message.lastSlashAtBlock);
      if (message.totalSlash != null && Object.hasOwnProperty.call(message, "totalSlash"))
        writer.uint32(58).string(message.totalSlash);
      if (message.totalSlashCount != null && Object.hasOwnProperty.call(message, "totalSlashCount"))
        writer.uint32(64).uint32(message.totalSlashCount);
      return writer;
    };
    TokenomicStat.encodeDelimited = function encodeDelimited(message, writer) {
      return this.encode(message, writer).ldelim();
    };
    TokenomicStat.decode = function decode(reader, length) {
      if (!(reader instanceof $Reader))
        reader = $Reader.create(reader);
      let end = length === void 0 ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.TokenomicStat();
      while (reader.pos < end) {
        let tag = reader.uint32();
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
    TokenomicStat.decodeDelimited = function decodeDelimited(reader) {
      if (!(reader instanceof $Reader))
        reader = new $Reader(reader);
      return this.decode(reader, reader.uint32());
    };
    TokenomicStat.verify = function verify(message) {
      if (typeof message !== "object" || message === null)
        return "object expected";
      if (message.lastPayout != null && message.hasOwnProperty("lastPayout")) {
        if (!$util.isString(message.lastPayout))
          return "lastPayout: string expected";
      }
      if (message.lastPayoutAtBlock != null && message.hasOwnProperty("lastPayoutAtBlock")) {
        if (!$util.isInteger(message.lastPayoutAtBlock))
          return "lastPayoutAtBlock: integer expected";
      }
      if (message.totalPayout != null && message.hasOwnProperty("totalPayout")) {
        if (!$util.isString(message.totalPayout))
          return "totalPayout: string expected";
      }
      if (message.totalPayoutCount != null && message.hasOwnProperty("totalPayoutCount")) {
        if (!$util.isInteger(message.totalPayoutCount))
          return "totalPayoutCount: integer expected";
      }
      if (message.lastSlash != null && message.hasOwnProperty("lastSlash")) {
        if (!$util.isString(message.lastSlash))
          return "lastSlash: string expected";
      }
      if (message.lastSlashAtBlock != null && message.hasOwnProperty("lastSlashAtBlock")) {
        if (!$util.isInteger(message.lastSlashAtBlock))
          return "lastSlashAtBlock: integer expected";
      }
      if (message.totalSlash != null && message.hasOwnProperty("totalSlash")) {
        if (!$util.isString(message.totalSlash))
          return "totalSlash: string expected";
      }
      if (message.totalSlashCount != null && message.hasOwnProperty("totalSlashCount")) {
        if (!$util.isInteger(message.totalSlashCount))
          return "totalSlashCount: integer expected";
      }
      return null;
    };
    TokenomicStat.fromObject = function fromObject(object) {
      if (object instanceof $root.pruntime_rpc.TokenomicStat)
        return object;
      let message = new $root.pruntime_rpc.TokenomicStat();
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
    TokenomicStat.toObject = function toObject(message, options) {
      if (!options)
        options = {};
      let object = {};
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
    TokenomicStat.prototype.toJSON = function toJSON() {
      return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
    };
    return TokenomicStat;
  }();
  pruntime_rpc2.TokenomicInfo = function() {
    function TokenomicInfo(properties) {
      if (properties) {
        for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
          if (properties[keys[i]] != null)
            this[keys[i]] = properties[keys[i]];
      }
    }
    TokenomicInfo.prototype.v = "";
    TokenomicInfo.prototype.vInit = "";
    TokenomicInfo.prototype.vDeductible = "";
    TokenomicInfo.prototype.share = "";
    TokenomicInfo.prototype.vUpdateAt = $util.Long ? $util.Long.fromBits(0, 0, true) : 0;
    TokenomicInfo.prototype.vUpdateBlock = 0;
    TokenomicInfo.prototype.iterationLast = $util.Long ? $util.Long.fromBits(0, 0, true) : 0;
    TokenomicInfo.prototype.challengeTimeLast = $util.Long ? $util.Long.fromBits(0, 0, true) : 0;
    TokenomicInfo.prototype.pBench = "";
    TokenomicInfo.prototype.pInstant = "";
    TokenomicInfo.prototype.confidenceLevel = 0;
    TokenomicInfo.prototype.stat = null;
    TokenomicInfo.create = function create2(properties) {
      return new TokenomicInfo(properties);
    };
    TokenomicInfo.encode = function encode(message, writer) {
      if (!writer)
        writer = $Writer.create();
      if (message.v != null && Object.hasOwnProperty.call(message, "v"))
        writer.uint32(10).string(message.v);
      if (message.vInit != null && Object.hasOwnProperty.call(message, "vInit"))
        writer.uint32(18).string(message.vInit);
      if (message.vUpdateAt != null && Object.hasOwnProperty.call(message, "vUpdateAt"))
        writer.uint32(32).uint64(message.vUpdateAt);
      if (message.vUpdateBlock != null && Object.hasOwnProperty.call(message, "vUpdateBlock"))
        writer.uint32(40).uint32(message.vUpdateBlock);
      if (message.iterationLast != null && Object.hasOwnProperty.call(message, "iterationLast"))
        writer.uint32(48).uint64(message.iterationLast);
      if (message.challengeTimeLast != null && Object.hasOwnProperty.call(message, "challengeTimeLast"))
        writer.uint32(56).uint64(message.challengeTimeLast);
      if (message.pBench != null && Object.hasOwnProperty.call(message, "pBench"))
        writer.uint32(66).string(message.pBench);
      if (message.pInstant != null && Object.hasOwnProperty.call(message, "pInstant"))
        writer.uint32(74).string(message.pInstant);
      if (message.confidenceLevel != null && Object.hasOwnProperty.call(message, "confidenceLevel"))
        writer.uint32(80).uint32(message.confidenceLevel);
      if (message.vDeductible != null && Object.hasOwnProperty.call(message, "vDeductible"))
        writer.uint32(154).string(message.vDeductible);
      if (message.share != null && Object.hasOwnProperty.call(message, "share"))
        writer.uint32(162).string(message.share);
      if (message.stat != null && Object.hasOwnProperty.call(message, "stat"))
        $root.pruntime_rpc.TokenomicStat.encode(
          message.stat,
          writer.uint32(170).fork()
        ).ldelim();
      return writer;
    };
    TokenomicInfo.encodeDelimited = function encodeDelimited(message, writer) {
      return this.encode(message, writer).ldelim();
    };
    TokenomicInfo.decode = function decode(reader, length) {
      if (!(reader instanceof $Reader))
        reader = $Reader.create(reader);
      let end = length === void 0 ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.TokenomicInfo();
      while (reader.pos < end) {
        let tag = reader.uint32();
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
            message.stat = $root.pruntime_rpc.TokenomicStat.decode(
              reader,
              reader.uint32()
            );
            break;
          default:
            reader.skipType(tag & 7);
            break;
        }
      }
      return message;
    };
    TokenomicInfo.decodeDelimited = function decodeDelimited(reader) {
      if (!(reader instanceof $Reader))
        reader = new $Reader(reader);
      return this.decode(reader, reader.uint32());
    };
    TokenomicInfo.verify = function verify(message) {
      if (typeof message !== "object" || message === null)
        return "object expected";
      if (message.v != null && message.hasOwnProperty("v")) {
        if (!$util.isString(message.v))
          return "v: string expected";
      }
      if (message.vInit != null && message.hasOwnProperty("vInit")) {
        if (!$util.isString(message.vInit))
          return "vInit: string expected";
      }
      if (message.vDeductible != null && message.hasOwnProperty("vDeductible")) {
        if (!$util.isString(message.vDeductible))
          return "vDeductible: string expected";
      }
      if (message.share != null && message.hasOwnProperty("share")) {
        if (!$util.isString(message.share))
          return "share: string expected";
      }
      if (message.vUpdateAt != null && message.hasOwnProperty("vUpdateAt")) {
        if (!$util.isInteger(message.vUpdateAt) && !(message.vUpdateAt && $util.isInteger(message.vUpdateAt.low) && $util.isInteger(message.vUpdateAt.high)))
          return "vUpdateAt: integer|Long expected";
      }
      if (message.vUpdateBlock != null && message.hasOwnProperty("vUpdateBlock")) {
        if (!$util.isInteger(message.vUpdateBlock))
          return "vUpdateBlock: integer expected";
      }
      if (message.iterationLast != null && message.hasOwnProperty("iterationLast")) {
        if (!$util.isInteger(message.iterationLast) && !(message.iterationLast && $util.isInteger(message.iterationLast.low) && $util.isInteger(message.iterationLast.high)))
          return "iterationLast: integer|Long expected";
      }
      if (message.challengeTimeLast != null && message.hasOwnProperty("challengeTimeLast")) {
        if (!$util.isInteger(message.challengeTimeLast) && !(message.challengeTimeLast && $util.isInteger(message.challengeTimeLast.low) && $util.isInteger(message.challengeTimeLast.high)))
          return "challengeTimeLast: integer|Long expected";
      }
      if (message.pBench != null && message.hasOwnProperty("pBench")) {
        if (!$util.isString(message.pBench))
          return "pBench: string expected";
      }
      if (message.pInstant != null && message.hasOwnProperty("pInstant")) {
        if (!$util.isString(message.pInstant))
          return "pInstant: string expected";
      }
      if (message.confidenceLevel != null && message.hasOwnProperty("confidenceLevel")) {
        if (!$util.isInteger(message.confidenceLevel))
          return "confidenceLevel: integer expected";
      }
      if (message.stat != null && message.hasOwnProperty("stat")) {
        let error = $root.pruntime_rpc.TokenomicStat.verify(message.stat);
        if (error)
          return "stat." + error;
      }
      return null;
    };
    TokenomicInfo.fromObject = function fromObject(object) {
      if (object instanceof $root.pruntime_rpc.TokenomicInfo)
        return object;
      let message = new $root.pruntime_rpc.TokenomicInfo();
      if (object.v != null)
        message.v = String(object.v);
      if (object.vInit != null)
        message.vInit = String(object.vInit);
      if (object.vDeductible != null)
        message.vDeductible = String(object.vDeductible);
      if (object.share != null)
        message.share = String(object.share);
      if (object.vUpdateAt != null) {
        if ($util.Long)
          (message.vUpdateAt = $util.Long.fromValue(
            object.vUpdateAt
          )).unsigned = true;
        else if (typeof object.vUpdateAt === "string")
          message.vUpdateAt = parseInt(object.vUpdateAt, 10);
        else if (typeof object.vUpdateAt === "number")
          message.vUpdateAt = object.vUpdateAt;
        else if (typeof object.vUpdateAt === "object")
          message.vUpdateAt = new $util.LongBits(
            object.vUpdateAt.low >>> 0,
            object.vUpdateAt.high >>> 0
          ).toNumber(true);
      }
      if (object.vUpdateBlock != null)
        message.vUpdateBlock = object.vUpdateBlock >>> 0;
      if (object.iterationLast != null) {
        if ($util.Long)
          (message.iterationLast = $util.Long.fromValue(
            object.iterationLast
          )).unsigned = true;
        else if (typeof object.iterationLast === "string")
          message.iterationLast = parseInt(object.iterationLast, 10);
        else if (typeof object.iterationLast === "number")
          message.iterationLast = object.iterationLast;
        else if (typeof object.iterationLast === "object")
          message.iterationLast = new $util.LongBits(
            object.iterationLast.low >>> 0,
            object.iterationLast.high >>> 0
          ).toNumber(true);
      }
      if (object.challengeTimeLast != null) {
        if ($util.Long)
          (message.challengeTimeLast = $util.Long.fromValue(
            object.challengeTimeLast
          )).unsigned = true;
        else if (typeof object.challengeTimeLast === "string")
          message.challengeTimeLast = parseInt(object.challengeTimeLast, 10);
        else if (typeof object.challengeTimeLast === "number")
          message.challengeTimeLast = object.challengeTimeLast;
        else if (typeof object.challengeTimeLast === "object")
          message.challengeTimeLast = new $util.LongBits(
            object.challengeTimeLast.low >>> 0,
            object.challengeTimeLast.high >>> 0
          ).toNumber(true);
      }
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
    TokenomicInfo.toObject = function toObject(message, options) {
      if (!options)
        options = {};
      let object = {};
      if (options.defaults) {
        object.v = "";
        object.vInit = "";
        if ($util.Long) {
          let long = new $util.Long(0, 0, true);
          object.vUpdateAt = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : long;
        } else
          object.vUpdateAt = options.longs === String ? "0" : 0;
        object.vUpdateBlock = 0;
        if ($util.Long) {
          let long = new $util.Long(0, 0, true);
          object.iterationLast = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : long;
        } else
          object.iterationLast = options.longs === String ? "0" : 0;
        if ($util.Long) {
          let long = new $util.Long(0, 0, true);
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
          object.vUpdateAt = options.longs === String ? $util.Long.prototype.toString.call(message.vUpdateAt) : options.longs === Number ? new $util.LongBits(
            message.vUpdateAt.low >>> 0,
            message.vUpdateAt.high >>> 0
          ).toNumber(true) : message.vUpdateAt;
      if (message.vUpdateBlock != null && message.hasOwnProperty("vUpdateBlock"))
        object.vUpdateBlock = message.vUpdateBlock;
      if (message.iterationLast != null && message.hasOwnProperty("iterationLast"))
        if (typeof message.iterationLast === "number")
          object.iterationLast = options.longs === String ? String(message.iterationLast) : message.iterationLast;
        else
          object.iterationLast = options.longs === String ? $util.Long.prototype.toString.call(message.iterationLast) : options.longs === Number ? new $util.LongBits(
            message.iterationLast.low >>> 0,
            message.iterationLast.high >>> 0
          ).toNumber(true) : message.iterationLast;
      if (message.challengeTimeLast != null && message.hasOwnProperty("challengeTimeLast"))
        if (typeof message.challengeTimeLast === "number")
          object.challengeTimeLast = options.longs === String ? String(message.challengeTimeLast) : message.challengeTimeLast;
        else
          object.challengeTimeLast = options.longs === String ? $util.Long.prototype.toString.call(message.challengeTimeLast) : options.longs === Number ? new $util.LongBits(
            message.challengeTimeLast.low >>> 0,
            message.challengeTimeLast.high >>> 0
          ).toNumber(true) : message.challengeTimeLast;
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
        object.stat = $root.pruntime_rpc.TokenomicStat.toObject(
          message.stat,
          options
        );
      return object;
    };
    TokenomicInfo.prototype.toJSON = function toJSON() {
      return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
    };
    return TokenomicInfo;
  }();
  pruntime_rpc2.NetworkStatus = function() {
    function NetworkStatus(properties) {
      if (properties) {
        for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
          if (properties[keys[i]] != null)
            this[keys[i]] = properties[keys[i]];
      }
    }
    NetworkStatus.prototype.publicRpcPort = null;
    NetworkStatus.prototype.config = null;
    let $oneOfFields;
    Object.defineProperty(NetworkStatus.prototype, "_publicRpcPort", {
      get: $util.oneOfGetter($oneOfFields = ["publicRpcPort"]),
      set: $util.oneOfSetter($oneOfFields)
    });
    Object.defineProperty(NetworkStatus.prototype, "_config", {
      get: $util.oneOfGetter($oneOfFields = ["config"]),
      set: $util.oneOfSetter($oneOfFields)
    });
    NetworkStatus.create = function create2(properties) {
      return new NetworkStatus(properties);
    };
    NetworkStatus.encode = function encode(message, writer) {
      if (!writer)
        writer = $Writer.create();
      if (message.publicRpcPort != null && Object.hasOwnProperty.call(message, "publicRpcPort"))
        writer.uint32(8).uint32(message.publicRpcPort);
      if (message.config != null && Object.hasOwnProperty.call(message, "config"))
        $root.pruntime_rpc.NetworkConfig.encode(
          message.config,
          writer.uint32(18).fork()
        ).ldelim();
      return writer;
    };
    NetworkStatus.encodeDelimited = function encodeDelimited(message, writer) {
      return this.encode(message, writer).ldelim();
    };
    NetworkStatus.decode = function decode(reader, length) {
      if (!(reader instanceof $Reader))
        reader = $Reader.create(reader);
      let end = length === void 0 ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.NetworkStatus();
      while (reader.pos < end) {
        let tag = reader.uint32();
        switch (tag >>> 3) {
          case 1:
            message.publicRpcPort = reader.uint32();
            break;
          case 2:
            message.config = $root.pruntime_rpc.NetworkConfig.decode(
              reader,
              reader.uint32()
            );
            break;
          default:
            reader.skipType(tag & 7);
            break;
        }
      }
      return message;
    };
    NetworkStatus.decodeDelimited = function decodeDelimited(reader) {
      if (!(reader instanceof $Reader))
        reader = new $Reader(reader);
      return this.decode(reader, reader.uint32());
    };
    NetworkStatus.verify = function verify(message) {
      if (typeof message !== "object" || message === null)
        return "object expected";
      let properties = {};
      if (message.publicRpcPort != null && message.hasOwnProperty("publicRpcPort")) {
        properties._publicRpcPort = 1;
        if (!$util.isInteger(message.publicRpcPort))
          return "publicRpcPort: integer expected";
      }
      if (message.config != null && message.hasOwnProperty("config")) {
        properties._config = 1;
        {
          let error = $root.pruntime_rpc.NetworkConfig.verify(message.config);
          if (error)
            return "config." + error;
        }
      }
      return null;
    };
    NetworkStatus.fromObject = function fromObject(object) {
      if (object instanceof $root.pruntime_rpc.NetworkStatus)
        return object;
      let message = new $root.pruntime_rpc.NetworkStatus();
      if (object.publicRpcPort != null)
        message.publicRpcPort = object.publicRpcPort >>> 0;
      if (object.config != null) {
        if (typeof object.config !== "object")
          throw TypeError(
            ".pruntime_rpc.NetworkStatus.config: object expected"
          );
        message.config = $root.pruntime_rpc.NetworkConfig.fromObject(
          object.config
        );
      }
      return message;
    };
    NetworkStatus.toObject = function toObject(message, options) {
      if (!options)
        options = {};
      let object = {};
      if (message.publicRpcPort != null && message.hasOwnProperty("publicRpcPort")) {
        object.publicRpcPort = message.publicRpcPort;
        if (options.oneofs)
          object._publicRpcPort = "publicRpcPort";
      }
      if (message.config != null && message.hasOwnProperty("config")) {
        object.config = $root.pruntime_rpc.NetworkConfig.toObject(
          message.config,
          options
        );
        if (options.oneofs)
          object._config = "config";
      }
      return object;
    };
    NetworkStatus.prototype.toJSON = function toJSON() {
      return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
    };
    return NetworkStatus;
  }();
  pruntime_rpc2.NetworkConfig = function() {
    function NetworkConfig(properties) {
      if (properties) {
        for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
          if (properties[keys[i]] != null)
            this[keys[i]] = properties[keys[i]];
      }
    }
    NetworkConfig.prototype.allProxy = "";
    NetworkConfig.prototype.i2pProxy = "";
    NetworkConfig.create = function create2(properties) {
      return new NetworkConfig(properties);
    };
    NetworkConfig.encode = function encode(message, writer) {
      if (!writer)
        writer = $Writer.create();
      if (message.allProxy != null && Object.hasOwnProperty.call(message, "allProxy"))
        writer.uint32(18).string(message.allProxy);
      if (message.i2pProxy != null && Object.hasOwnProperty.call(message, "i2pProxy"))
        writer.uint32(26).string(message.i2pProxy);
      return writer;
    };
    NetworkConfig.encodeDelimited = function encodeDelimited(message, writer) {
      return this.encode(message, writer).ldelim();
    };
    NetworkConfig.decode = function decode(reader, length) {
      if (!(reader instanceof $Reader))
        reader = $Reader.create(reader);
      let end = length === void 0 ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.NetworkConfig();
      while (reader.pos < end) {
        let tag = reader.uint32();
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
    NetworkConfig.decodeDelimited = function decodeDelimited(reader) {
      if (!(reader instanceof $Reader))
        reader = new $Reader(reader);
      return this.decode(reader, reader.uint32());
    };
    NetworkConfig.verify = function verify(message) {
      if (typeof message !== "object" || message === null)
        return "object expected";
      if (message.allProxy != null && message.hasOwnProperty("allProxy")) {
        if (!$util.isString(message.allProxy))
          return "allProxy: string expected";
      }
      if (message.i2pProxy != null && message.hasOwnProperty("i2pProxy")) {
        if (!$util.isString(message.i2pProxy))
          return "i2pProxy: string expected";
      }
      return null;
    };
    NetworkConfig.fromObject = function fromObject(object) {
      if (object instanceof $root.pruntime_rpc.NetworkConfig)
        return object;
      let message = new $root.pruntime_rpc.NetworkConfig();
      if (object.allProxy != null)
        message.allProxy = String(object.allProxy);
      if (object.i2pProxy != null)
        message.i2pProxy = String(object.i2pProxy);
      return message;
    };
    NetworkConfig.toObject = function toObject(message, options) {
      if (!options)
        options = {};
      let object = {};
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
    NetworkConfig.prototype.toJSON = function toJSON() {
      return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
    };
    return NetworkConfig;
  }();
  pruntime_rpc2.HttpHeader = function() {
    function HttpHeader(properties) {
      if (properties) {
        for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
          if (properties[keys[i]] != null)
            this[keys[i]] = properties[keys[i]];
      }
    }
    HttpHeader.prototype.name = "";
    HttpHeader.prototype.value = "";
    HttpHeader.create = function create2(properties) {
      return new HttpHeader(properties);
    };
    HttpHeader.encode = function encode(message, writer) {
      if (!writer)
        writer = $Writer.create();
      if (message.name != null && Object.hasOwnProperty.call(message, "name"))
        writer.uint32(10).string(message.name);
      if (message.value != null && Object.hasOwnProperty.call(message, "value"))
        writer.uint32(18).string(message.value);
      return writer;
    };
    HttpHeader.encodeDelimited = function encodeDelimited(message, writer) {
      return this.encode(message, writer).ldelim();
    };
    HttpHeader.decode = function decode(reader, length) {
      if (!(reader instanceof $Reader))
        reader = $Reader.create(reader);
      let end = length === void 0 ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.HttpHeader();
      while (reader.pos < end) {
        let tag = reader.uint32();
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
    HttpHeader.decodeDelimited = function decodeDelimited(reader) {
      if (!(reader instanceof $Reader))
        reader = new $Reader(reader);
      return this.decode(reader, reader.uint32());
    };
    HttpHeader.verify = function verify(message) {
      if (typeof message !== "object" || message === null)
        return "object expected";
      if (message.name != null && message.hasOwnProperty("name")) {
        if (!$util.isString(message.name))
          return "name: string expected";
      }
      if (message.value != null && message.hasOwnProperty("value")) {
        if (!$util.isString(message.value))
          return "value: string expected";
      }
      return null;
    };
    HttpHeader.fromObject = function fromObject(object) {
      if (object instanceof $root.pruntime_rpc.HttpHeader)
        return object;
      let message = new $root.pruntime_rpc.HttpHeader();
      if (object.name != null)
        message.name = String(object.name);
      if (object.value != null)
        message.value = String(object.value);
      return message;
    };
    HttpHeader.toObject = function toObject(message, options) {
      if (!options)
        options = {};
      let object = {};
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
    HttpHeader.prototype.toJSON = function toJSON() {
      return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
    };
    return HttpHeader;
  }();
  pruntime_rpc2.HttpRequest = function() {
    function HttpRequest(properties) {
      this.headers = [];
      if (properties) {
        for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
          if (properties[keys[i]] != null)
            this[keys[i]] = properties[keys[i]];
      }
    }
    HttpRequest.prototype.url = "";
    HttpRequest.prototype.method = "";
    HttpRequest.prototype.headers = $util.emptyArray;
    HttpRequest.prototype.body = $util.newBuffer([]);
    HttpRequest.create = function create2(properties) {
      return new HttpRequest(properties);
    };
    HttpRequest.encode = function encode(message, writer) {
      if (!writer)
        writer = $Writer.create();
      if (message.url != null && Object.hasOwnProperty.call(message, "url"))
        writer.uint32(10).string(message.url);
      if (message.method != null && Object.hasOwnProperty.call(message, "method"))
        writer.uint32(18).string(message.method);
      if (message.headers != null && message.headers.length)
        for (let i = 0; i < message.headers.length; ++i)
          $root.pruntime_rpc.HttpHeader.encode(
            message.headers[i],
            writer.uint32(26).fork()
          ).ldelim();
      if (message.body != null && Object.hasOwnProperty.call(message, "body"))
        writer.uint32(34).bytes(message.body);
      return writer;
    };
    HttpRequest.encodeDelimited = function encodeDelimited(message, writer) {
      return this.encode(message, writer).ldelim();
    };
    HttpRequest.decode = function decode(reader, length) {
      if (!(reader instanceof $Reader))
        reader = $Reader.create(reader);
      let end = length === void 0 ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.HttpRequest();
      while (reader.pos < end) {
        let tag = reader.uint32();
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
            message.headers.push(
              $root.pruntime_rpc.HttpHeader.decode(reader, reader.uint32())
            );
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
    HttpRequest.decodeDelimited = function decodeDelimited(reader) {
      if (!(reader instanceof $Reader))
        reader = new $Reader(reader);
      return this.decode(reader, reader.uint32());
    };
    HttpRequest.verify = function verify(message) {
      if (typeof message !== "object" || message === null)
        return "object expected";
      if (message.url != null && message.hasOwnProperty("url")) {
        if (!$util.isString(message.url))
          return "url: string expected";
      }
      if (message.method != null && message.hasOwnProperty("method")) {
        if (!$util.isString(message.method))
          return "method: string expected";
      }
      if (message.headers != null && message.hasOwnProperty("headers")) {
        if (!Array.isArray(message.headers))
          return "headers: array expected";
        for (let i = 0; i < message.headers.length; ++i) {
          let error = $root.pruntime_rpc.HttpHeader.verify(message.headers[i]);
          if (error)
            return "headers." + error;
        }
      }
      if (message.body != null && message.hasOwnProperty("body")) {
        if (!(message.body && typeof message.body.length === "number" || $util.isString(message.body)))
          return "body: buffer expected";
      }
      return null;
    };
    HttpRequest.fromObject = function fromObject(object) {
      if (object instanceof $root.pruntime_rpc.HttpRequest)
        return object;
      let message = new $root.pruntime_rpc.HttpRequest();
      if (object.url != null)
        message.url = String(object.url);
      if (object.method != null)
        message.method = String(object.method);
      if (object.headers) {
        if (!Array.isArray(object.headers))
          throw TypeError(".pruntime_rpc.HttpRequest.headers: array expected");
        message.headers = [];
        for (let i = 0; i < object.headers.length; ++i) {
          if (typeof object.headers[i] !== "object")
            throw TypeError(
              ".pruntime_rpc.HttpRequest.headers: object expected"
            );
          message.headers[i] = $root.pruntime_rpc.HttpHeader.fromObject(
            object.headers[i]
          );
        }
      }
      if (object.body != null) {
        if (typeof object.body === "string")
          $util.base64.decode(
            object.body,
            message.body = $util.newBuffer($util.base64.length(object.body)),
            0
          );
        else if (object.body.length)
          message.body = object.body;
      }
      return message;
    };
    HttpRequest.toObject = function toObject(message, options) {
      if (!options)
        options = {};
      let object = {};
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
        for (let j = 0; j < message.headers.length; ++j)
          object.headers[j] = $root.pruntime_rpc.HttpHeader.toObject(
            message.headers[j],
            options
          );
      }
      if (message.body != null && message.hasOwnProperty("body"))
        object.body = options.bytes === String ? $util.base64.encode(message.body, 0, message.body.length) : options.bytes === Array ? Array.prototype.slice.call(message.body) : message.body;
      return object;
    };
    HttpRequest.prototype.toJSON = function toJSON() {
      return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
    };
    return HttpRequest;
  }();
  pruntime_rpc2.HttpResponse = function() {
    function HttpResponse(properties) {
      this.headers = [];
      if (properties) {
        for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
          if (properties[keys[i]] != null)
            this[keys[i]] = properties[keys[i]];
      }
    }
    HttpResponse.prototype.statusCode = 0;
    HttpResponse.prototype.headers = $util.emptyArray;
    HttpResponse.prototype.body = $util.newBuffer([]);
    HttpResponse.create = function create2(properties) {
      return new HttpResponse(properties);
    };
    HttpResponse.encode = function encode(message, writer) {
      if (!writer)
        writer = $Writer.create();
      if (message.statusCode != null && Object.hasOwnProperty.call(message, "statusCode"))
        writer.uint32(8).uint32(message.statusCode);
      if (message.headers != null && message.headers.length)
        for (let i = 0; i < message.headers.length; ++i)
          $root.pruntime_rpc.HttpHeader.encode(
            message.headers[i],
            writer.uint32(18).fork()
          ).ldelim();
      if (message.body != null && Object.hasOwnProperty.call(message, "body"))
        writer.uint32(26).bytes(message.body);
      return writer;
    };
    HttpResponse.encodeDelimited = function encodeDelimited(message, writer) {
      return this.encode(message, writer).ldelim();
    };
    HttpResponse.decode = function decode(reader, length) {
      if (!(reader instanceof $Reader))
        reader = $Reader.create(reader);
      let end = length === void 0 ? reader.len : reader.pos + length, message = new $root.pruntime_rpc.HttpResponse();
      while (reader.pos < end) {
        let tag = reader.uint32();
        switch (tag >>> 3) {
          case 1:
            message.statusCode = reader.uint32();
            break;
          case 2:
            if (!(message.headers && message.headers.length))
              message.headers = [];
            message.headers.push(
              $root.pruntime_rpc.HttpHeader.decode(reader, reader.uint32())
            );
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
    HttpResponse.decodeDelimited = function decodeDelimited(reader) {
      if (!(reader instanceof $Reader))
        reader = new $Reader(reader);
      return this.decode(reader, reader.uint32());
    };
    HttpResponse.verify = function verify(message) {
      if (typeof message !== "object" || message === null)
        return "object expected";
      if (message.statusCode != null && message.hasOwnProperty("statusCode")) {
        if (!$util.isInteger(message.statusCode))
          return "statusCode: integer expected";
      }
      if (message.headers != null && message.hasOwnProperty("headers")) {
        if (!Array.isArray(message.headers))
          return "headers: array expected";
        for (let i = 0; i < message.headers.length; ++i) {
          let error = $root.pruntime_rpc.HttpHeader.verify(message.headers[i]);
          if (error)
            return "headers." + error;
        }
      }
      if (message.body != null && message.hasOwnProperty("body")) {
        if (!(message.body && typeof message.body.length === "number" || $util.isString(message.body)))
          return "body: buffer expected";
      }
      return null;
    };
    HttpResponse.fromObject = function fromObject(object) {
      if (object instanceof $root.pruntime_rpc.HttpResponse)
        return object;
      let message = new $root.pruntime_rpc.HttpResponse();
      if (object.statusCode != null)
        message.statusCode = object.statusCode >>> 0;
      if (object.headers) {
        if (!Array.isArray(object.headers))
          throw TypeError(".pruntime_rpc.HttpResponse.headers: array expected");
        message.headers = [];
        for (let i = 0; i < object.headers.length; ++i) {
          if (typeof object.headers[i] !== "object")
            throw TypeError(
              ".pruntime_rpc.HttpResponse.headers: object expected"
            );
          message.headers[i] = $root.pruntime_rpc.HttpHeader.fromObject(
            object.headers[i]
          );
        }
      }
      if (object.body != null) {
        if (typeof object.body === "string")
          $util.base64.decode(
            object.body,
            message.body = $util.newBuffer($util.base64.length(object.body)),
            0
          );
        else if (object.body.length)
          message.body = object.body;
      }
      return message;
    };
    HttpResponse.toObject = function toObject(message, options) {
      if (!options)
        options = {};
      let object = {};
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
        for (let j = 0; j < message.headers.length; ++j)
          object.headers[j] = $root.pruntime_rpc.HttpHeader.toObject(
            message.headers[j],
            options
          );
      }
      if (message.body != null && message.hasOwnProperty("body"))
        object.body = options.bytes === String ? $util.base64.encode(message.body, 0, message.body.length) : options.bytes === Array ? Array.prototype.slice.call(message.body) : message.body;
      return object;
    };
    HttpResponse.prototype.toJSON = function toJSON() {
      return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
    };
    return HttpResponse;
  }();
  return pruntime_rpc2;
})();
var google = $root.google = (() => {
  const google2 = {};
  google2.protobuf = function() {
    const protobuf = {};
    protobuf.Empty = function() {
      function Empty(properties) {
        if (properties) {
          for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
            if (properties[keys[i]] != null)
              this[keys[i]] = properties[keys[i]];
        }
      }
      Empty.create = function create2(properties) {
        return new Empty(properties);
      };
      Empty.encode = function encode(message, writer) {
        if (!writer)
          writer = $Writer.create();
        return writer;
      };
      Empty.encodeDelimited = function encodeDelimited(message, writer) {
        return this.encode(message, writer).ldelim();
      };
      Empty.decode = function decode(reader, length) {
        if (!(reader instanceof $Reader))
          reader = $Reader.create(reader);
        let end = length === void 0 ? reader.len : reader.pos + length, message = new $root.google.protobuf.Empty();
        while (reader.pos < end) {
          let tag = reader.uint32();
          switch (tag >>> 3) {
            default:
              reader.skipType(tag & 7);
              break;
          }
        }
        return message;
      };
      Empty.decodeDelimited = function decodeDelimited(reader) {
        if (!(reader instanceof $Reader))
          reader = new $Reader(reader);
        return this.decode(reader, reader.uint32());
      };
      Empty.verify = function verify(message) {
        if (typeof message !== "object" || message === null)
          return "object expected";
        return null;
      };
      Empty.fromObject = function fromObject(object) {
        if (object instanceof $root.google.protobuf.Empty)
          return object;
        return new $root.google.protobuf.Empty();
      };
      Empty.toObject = function toObject() {
        return {};
      };
      Empty.prototype.toJSON = function toJSON() {
        return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
      };
      return Empty;
    }();
    return protobuf;
  }();
  return google2;
})();

// src/create.ts
var createPruntimeApi = (baseURL) => {
  const http = axios.create({
    baseURL,
    headers: {
      "Content-Type": "application/octet-stream"
    },
    responseType: "arraybuffer"
  }).post;
  const pruntimeApi = pruntime_rpc.PhactoryAPI.create(
    async (method, requestData, callback) => {
      var _a;
      try {
        const res = await http(
          `/prpc/PhactoryAPI.${method.name}`,
          new Uint8Array(requestData)
        );
        callback(null, new Uint8Array(res.data));
      } catch (err) {
        if (err instanceof AxiosError && ((_a = err.response) == null ? void 0 : _a.data) instanceof ArrayBuffer) {
          const message = new Uint8Array(err.response.data);
          callback(new Error(prpc.PrpcError.decode(message).message));
        } else {
          throw err;
        }
      }
    }
  );
  return pruntimeApi;
};
async function create({
  api,
  baseURL,
  contractId,
  remotePubkey,
  autoDeposit = false
}) {
  await waitReady();
  const pruntimeApi = createPruntimeApi(baseURL);
  if (!remotePubkey) {
    const info = await pruntimeApi.getInfo({});
    if (!info || !info.publicKey)
      throw new Error("No remote pubkey");
    remotePubkey = hexAddPrefix2(info.publicKey);
  }
  const seed = hexToU8a2(hexAddPrefix2(randomHex(32)));
  const pair = sr25519KeypairFromSeed(seed);
  const [sk, pk] = [pair.slice(0, 64), pair.slice(64)];
  const queryAgreementKey = sr25519Agree(
    hexToU8a2(hexAddPrefix2(remotePubkey)),
    sk
  );
  const contractKey = (await api.query.phalaRegistry.contractKeys(contractId)).toString();
  if (!contractKey) {
    throw new Error(`No contract key for ${contractId}`);
  }
  const commandAgreementKey = sr25519Agree(hexToU8a2(contractKey), sk);
  const createEncryptedData = (data, agreementKey) => {
    const iv = hexAddPrefix2(randomHex(12));
    return {
      iv,
      pubkey: u8aToHex(pk),
      data: hexAddPrefix2(encrypt(data, agreementKey, hexToU8a2(iv)))
    };
  };
  let gasPrice = new BN(0);
  if (autoDeposit) {
    const contractInfo = await api.query.phalaFatContracts.contracts(
      contractId
    );
    const cluster = contractInfo.unwrap().cluster;
    const clusterInfo = await api.query.phalaFatContracts.clusters(
      cluster
    );
    gasPrice = new BN(
      clusterInfo.unwrap().gasPrice
    );
  }
  const query = async (encodedQuery, { certificate, pubkey, secret }) => {
    const encryptedData = createEncryptedData(encodedQuery, queryAgreementKey);
    const encodedEncryptedData = api.createType("EncryptedData", encryptedData).toU8a();
    const signature = {
      signedBy: certificate,
      signatureType: pruntime_rpc.SignatureType.Sr25519,
      signature: sr25519Sign(pubkey, secret, encodedEncryptedData)
    };
    const requestData = {
      encodedEncryptedData,
      signature
    };
    return pruntimeApi.contractQuery(requestData).then((res) => {
      const { encodedEncryptedData: encodedEncryptedData2 } = res;
      const { data: encryptedData2, iv } = api.createType("EncryptedData", encodedEncryptedData2).toJSON();
      const data = decrypt(encryptedData2, queryAgreementKey, iv);
      return hexAddPrefix2(data);
    });
  };
  const sidevmQuery = async (bytes, certificateData) => query(
    api.createType("InkQuery", {
      head: {
        nonce: hexAddPrefix2(randomHex(32)),
        id: contractId
      },
      data: {
        SidevmMessage: bytes
      }
    }).toHex(),
    certificateData
  );
  const instantiate = async (payload, certificateData) => query(
    api.createType("InkQuery", {
      head: {
        nonce: hexAddPrefix2(randomHex(32)),
        id: contractId
      },
      data: {
        InkInstantiate: payload
      }
    }).toHex(),
    certificateData
  );
  const command = ({ contractId: contractId2, payload, deposit }) => {
    const encodedPayload = api.createType("CommandPayload", {
      encrypted: createEncryptedData(payload, commandAgreementKey)
    }).toHex();
    try {
      return api.tx.phalaFatContracts.pushContractMessage(
        contractId2,
        encodedPayload,
        deposit
      );
    } catch (err) {
      return api.tx.phalaMq.pushMessage(
        stringToHex(`phala/contract/${hexStripPrefix2(contractId2)}/command`),
        encodedPayload
      );
    }
  };
  const txContracts = (dest, value, gas, storageDepositLimit, encParams) => {
    let deposit = new BN(0);
    if (autoDeposit) {
      const gasFee = new BN(gas.refTime).mul(gasPrice);
      deposit = new BN(value).add(gasFee).add(new BN(storageDepositLimit || 0));
    }
    return command({
      contractId: dest.toHex(),
      payload: api.createType("InkCommand", {
        InkMessage: {
          nonce: hexAddPrefix2(randomHex(32)),
          message: api.createType("Vec<u8>", encParams).toHex(),
          transfer: value,
          gasLimit: gas.refTime,
          storageDepositLimit
        }
      }).toHex(),
      deposit
    });
  };
  Object.defineProperty(txContracts, "meta", {
    value: { args: [] },
    enumerable: true
  });
  const instantiateWithCode = () => null;
  instantiateWithCode.meta = { args: new Array(6) };
  Object.defineProperty(api.tx, "contracts", {
    value: {
      instantiateWithCode,
      call: txContracts
    },
    enumerable: true
  });
  Object.defineProperty(api.rx.call, "contractsApi", {
    value: {
      call: (origin, dest, value, gasLimit, storageDepositLimit, inputData) => {
        return from(
          query(
            api.createType("InkQuery", {
              head: {
                nonce: hexAddPrefix2(randomHex(32)),
                id: dest
              },
              data: {
                InkMessage: inputData
              }
            }).toHex(),
            origin
          ).then((data) => {
            return api.createType(
              "ContractExecResult",
              api.createType("InkResponse", hexAddPrefix2(data)).toJSON().result.ok.inkMessageReturn
            );
          })
        );
      }
    },
    enumerable: true
  });
  Object.defineProperty(api.call, "contractsApi", {
    value: { call: () => null },
    enumerable: true
  });
  return { api, sidevmQuery, instantiate };
}

// src/certificate.ts
import { hexAddPrefix as hexAddPrefix3, hexToU8a as hexToU8a3, u8aToHex as u8aToHex2 } from "@polkadot/util";
import { decodeAddress } from "@polkadot/util-crypto";
import { sr25519KeypairFromSeed as sr25519KeypairFromSeed2, waitReady as waitReady2 } from "@polkadot/wasm-crypto";
var isUsingSigner = (params) => params.signer !== void 0;
var signCertificate = async (params) => {
  var _a;
  await waitReady2();
  const { api } = params;
  const generatedSeed = hexToU8a3(hexAddPrefix3(randomHex(32)));
  const generatedPair = sr25519KeypairFromSeed2(generatedSeed);
  const [secret, pubkey] = [
    generatedPair.slice(0, 64),
    generatedPair.slice(64)
  ];
  const encodedCertificateBody = api.createType("CertificateBody", {
    pubkey: u8aToHex2(pubkey),
    ttl: 2147483647,
    config_bits: 0
  }).toU8a();
  let signerPubkey;
  let signatureType = params.signatureType;
  let signature;
  if (isUsingSigner(params)) {
    const { account, signer } = params;
    const address = account.address;
    signerPubkey = u8aToHex2(decodeAddress(address));
    if (!signatureType) {
      signatureType = getSignatureTypeFromAccount(account);
    }
    const signerResult = await ((_a = signer.signRaw) == null ? void 0 : _a.call(signer, {
      address,
      data: u8aToHex2(encodedCertificateBody),
      type: "bytes"
    }));
    if (signerResult) {
      signature = hexToU8a3(signerResult.signature);
    } else {
      throw new Error("Failed to sign certificate");
    }
  } else {
    const { pair } = params;
    signerPubkey = u8aToHex2(pair.publicKey);
    if (!signatureType) {
      signatureType = getSignatureTypeFromPair(pair);
    }
    signature = pair.sign(encodedCertificateBody);
  }
  const certificate = {
    encodedBody: encodedCertificateBody,
    signature: {
      signedBy: {
        encodedBody: api.createType("CertificateBody", {
          pubkey: signerPubkey,
          ttl: 2147483647,
          config_bits: 0
        }).toU8a(),
        signature: null
      },
      signatureType,
      signature
    }
  };
  return {
    certificate,
    pubkey,
    secret
  };
};
var getSignatureTypeFromAccount = (account) => {
  const keypairType = account.type || "sr25519";
  const useWrapBytes = account.meta.source === "polkadot-js";
  switch (keypairType) {
    case "sr25519":
      return useWrapBytes ? pruntime_rpc.SignatureType.Sr25519WrapBytes : pruntime_rpc.SignatureType.Sr25519;
    case "ed25519":
      return useWrapBytes ? pruntime_rpc.SignatureType.Ed25519WrapBytes : pruntime_rpc.SignatureType.Ed25519;
    case "ecdsa":
      return useWrapBytes ? pruntime_rpc.SignatureType.EcdsaWrapBytes : pruntime_rpc.SignatureType.Ecdsa;
    default:
      throw new Error("Unsupported keypair type");
  }
};
var getSignatureTypeFromPair = (pair) => {
  switch (pair.type) {
    case "sr25519":
      return pruntime_rpc.SignatureType.Sr25519;
    case "ed25519":
      return pruntime_rpc.SignatureType.Ed25519;
    case "ecdsa":
      return pruntime_rpc.SignatureType.Ecdsa;
    default:
      throw new Error("Unsupported keypair type");
  }
};
export {
  create,
  createPruntimeApi,
  randomHex,
  signCertificate,
  types
};

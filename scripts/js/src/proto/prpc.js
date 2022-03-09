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

        return PrpcError;
    })();

    return prpc;
})();

module.exports = $root;

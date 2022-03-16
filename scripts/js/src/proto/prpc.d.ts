import * as $protobuf from "protobufjs";
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
    }
}

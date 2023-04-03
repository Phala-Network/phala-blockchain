import { RegistryTypes } from "@polkadot/types/types";

export const types: RegistryTypes = {
  ContractId: "H256",
  EcdhPublicKey: "SpCoreSr25519Public",
  ContractQueryHead: {
    id: "ContractId",
    nonce: "[u8; 32]",
  },
  CertificateBody: {
    pubkey: "Vec<u8>",
    ttl: "u32",
    config_bits: "u32",
  },
  EncryptedData: {
    iv: "[u8; 12]",
    pubkey: "EcdhPublicKey",
    data: "Vec<u8>",
  },
  CommandPayload: {
    _enum: {
      Plain: null, // disable plain
      Encrypted: "EncryptedData",
    },
  },
  InkQueryData: {
    _enum: {
      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore
      InkMessage: {
        payload: "Vec<u8>",
        deposit: "u128",
        transfer: "u128",
        estimating: "bool",
      },
      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore
      SidevmMessage: "Vec<u8>",
      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore
      InkInstantiate: {
        codeHash: "H256",
        salt: "Vec<u8>",
        instantiateData: "Vec<u8>",
        deposit: "u128",
        transfer: "u128",
      },
    },
  },
  InkQuery: {
    head: "ContractQueryHead",
    data: "InkQueryData",
  },
  InkQueryError: {
    _enum: {
      BadOrigin: null,
      RuntimeError: "String",
      SidevmNotFound: null,
      NoResponse: null,
      ServiceUnavailable: null,
      Timeout: null,
    },
  },
  InkQueryOk: {
    _enum: {
      InkMessageReturn: "Vec<u8>",
    },
  },
  InkResponse: {
    nonce: "[u8; 32]",
    result: "Result<InkQueryOk, InkQueryError>",
  },
  InkMessage: {
    nonce: "Vec<u8>",
    message: "Vec<u8>",
    transfer: "u128",
    gasLimit: "u64",
    storageDepositLimit: "Option<u128>",
  },
  InkCommand: { _enum: { InkMessage: "InkMessage" } },
};

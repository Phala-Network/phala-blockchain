import type { ApiOptions } from '@polkadot/api/types'
import { TypeRegistry, typeDefinitions } from '@polkadot/types'
import { RegistryTypes } from '@polkadot/types/types'
import SubstrateLookupTypes from '@polkadot/types-augment/lookup/substrate'

export const types = {
  ContractId: 'H256',
  EcdhPublicKey: 'SpCoreSr25519Public',
  ContractQueryHead: {
    id: 'ContractId',
    nonce: '[u8; 32]',
  },
  CertificateBody: {
    pubkey: 'Vec<u8>',
    ttl: 'u32',
    config_bits: 'u32',
  },
  EncryptedData: {
    iv: '[u8; 12]',
    pubkey: 'EcdhPublicKey',
    data: 'Vec<u8>',
  },
  CommandPayload: {
    _enum: {
      Plain: 'InkCommand',
      Encrypted: 'EncryptedData',
    },
  },
  InkQueryData: {
    _enum: {
      InkMessage: {
        payload: 'Vec<u8>',
        deposit: 'u128',
        transfer: 'u128',
        estimating: 'bool',
      },
      SidevmMessage: 'Vec<u8>',
      InkInstantiate: {
        codeHash: 'H256',
        salt: 'Vec<u8>',
        instantiateData: 'Vec<u8>',
        deposit: 'u128',
        transfer: 'u128',
      },
    },
  },
  InkQuery: {
    head: 'ContractQueryHead',
    data: 'InkQueryData',
  },
  InkQueryError: {
    _enum: {
      BadOrigin: null,
      RuntimeError: 'String',
      SidevmNotFound: null,
      NoResponse: null,
      ServiceUnavailable: null,
      Timeout: null,
    },
  },
  InkQueryOk: {
    _enum: {
      InkMessageReturn: 'Vec<u8>',
    },
  },
  InkResponse: {
    nonce: '[u8; 32]',
    result: 'Result<InkQueryOk, InkQueryError>',
  },
  InkMessage: {
    nonce: 'Vec<u8>',
    message: 'Vec<u8>',
    transfer: 'u128',
    gasLimit: 'u64',
    storageDepositLimit: 'Option<u128>',
  },
  InkCommand: { _enum: { InkMessage: 'InkMessage' } },
  ContractError: {
    _enum: {
      InvalidScheduleVersion: null,
      InvalidCallFlags: null,
      OutOfGas: null,
      OutputBufferTooSmall: null,
      TransferFailed: null,
      MaxCallDepthReached: null,
      ContractNotFound: null,
      CodeTooLarge: null,
      CodeNotFound: null,
      OutOfBounds: null,
      DecodingFailed: null,
      ContractTrapped: null,
      ValueTooLarge: null,
      TerminatedWhileReentrant: null,
      InputForwarded: null,
      RandomSubjectTooLong: null,
      TooManyTopics: null,
      NoChainExtension: null,
      DuplicateContract: null,
      TerminatedInConstructor: null,
      ReentranceDenied: null,
      StorageDepositNotEnoughFunds: null,
      StorageDepositLimitExhausted: null,
      CodeInUse: null,
      ContractReverted: null,
      CodeRejected: null,
      Indeterministic: null,
    },
  },
}

export const phalaRegistryTypes = { ...types, ...typeDefinitions, ...SubstrateLookupTypes } as unknown as RegistryTypes

export const phalaTypes = new TypeRegistry()

phalaTypes.register(phalaRegistryTypes)

export function options(options: ApiOptions = {}): ApiOptions {
  return {
    ...options,
    types: {
      ...phalaRegistryTypes,
      ...(options.types || {}),
    } as unknown as ApiOptions['types'],
  }
}

import { U8aFixed } from '@polkadot/types'
import { type AccountId } from '@polkadot/types/interfaces'
import { BN, BN_ZERO, hexAddPrefix, hexToU8a, stringToHex, u8aToHex } from '@polkadot/util'
import { sr25519Agree } from '@polkadot/wasm-crypto'
import { phalaTypes } from '../options'
import { encrypt } from '../utils/aes-256-gcm'
import { randomHex } from '../utils/hex'
import { generatePair } from './certificate'

type ContractAddress = AccountId | string
type LooseNumber = number | string | bigint | BN | null

function encodeNumber(n?: LooseNumber, defaults = BN_ZERO) {
  if (!n) {
    return defaults
  }
  if (typeof n === 'bigint') {
    return n
  }
  return new BN(n)
}

export function InkQueryMessage(
  address: ContractAddress,
  payload: Uint8Array,
  deposit?: LooseNumber,
  transfer?: LooseNumber,
  estimating?: boolean
) {
  return phalaTypes.createType('InkQuery', {
    head: {
      nonce: hexAddPrefix(randomHex(32)),
      id: address,
    },
    data: {
      InkMessage: {
        payload: payload,
        deposit: encodeNumber(deposit),
        transfer: encodeNumber(transfer),
        estimating: !!estimating,
      },
    },
  })
}

export function InkQuerySidevmMessage<T = unknown>(address: ContractAddress, sidevmMessage: T) {
  return phalaTypes.createType('InkQuery', {
    head: {
      nonce: hexAddPrefix(randomHex(32)),
      id: address,
    },
    data: {
      SidevmMessage: stringToHex(JSON.stringify(sidevmMessage)),
    },
  })
}

export function InkQueryInstantiate(
  address: ContractAddress,
  codeHash: string | U8aFixed,
  instantiateData: Uint8Array,
  salt: string,
  deposit?: LooseNumber,
  transfer?: LooseNumber
) {
  return phalaTypes.createType('InkQuery', {
    head: {
      nonce: hexAddPrefix(randomHex(32)),
      id: address,
    },
    data: {
      InkInstantiate: {
        codeHash,
        salt,
        instantiateData: instantiateData,
        deposit: encodeNumber(deposit),
        transfer: encodeNumber(transfer),
      },
    },
  })
}

export function PlainInkCommand(
  address: ContractAddress,
  encParams: Uint8Array,
  value: LooseNumber | undefined,
  gas: { refTime: LooseNumber },
  storageDepositLimit?: LooseNumber
) {
  return phalaTypes.createType('CommandPayload', {
    Plain: phalaTypes.createType('InkCommand', {
      InkMessage: {
        nonce: hexAddPrefix(randomHex(32)),
        // FIXME: unexpected u8a prefix
        message: phalaTypes.createType('Vec<u8>', encParams).toHex(),
        transfer: value,
        gasLimit: gas.refTime,
        storageDepositLimit,
      },
    }),
  })
}

export function EncryptedInkCommand(
  address: string,
  encParams: Uint8Array,
  value: LooseNumber | undefined,
  gas: { refTime: LooseNumber },
  storageDepositLimit?: LooseNumber
) {
  const [sk, pk] = generatePair()
  const commandAgreementKey = sr25519Agree(hexToU8a(address), sk)
  const payload = phalaTypes.createType('InkCommand', {
    InkMessage: {
      nonce: hexAddPrefix(randomHex(32)),
      // FIXME: unexpected u8a prefix
      message: phalaTypes.createType('Vec<u8>', encParams).toHex(),
      transfer: value,
      gasLimit: gas.refTime,
      storageDepositLimit,
    },
  })
  const iv = hexAddPrefix(randomHex(12))
  return phalaTypes.createType('CommandPayload', {
    Encrypted: {
      iv,
      pubkey: u8aToHex(pk),
      data: hexAddPrefix(encrypt(payload.toHex(), commandAgreementKey, hexToU8a(iv))),
    },
  })
}

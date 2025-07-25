import type { ApiPromise } from '@polkadot/api'
import type { ApiTypes, SubmittableExtrinsic } from '@polkadot/api/types'
import type { U256, U64 } from '@polkadot/types-codec'
import { hexToString } from '@polkadot/util'
import type { Account, Address, Hex } from 'viem'
import { type signTypedData } from 'viem/wallet'

// keccak256(b"phala/phat-contract")
const SALT: Readonly<Hex> = '0x0ea813d1592526d672ea2576d7a07914cef2ca301b35c5eed941f7c897512a00'

type SignTypedDataInput = Parameters<typeof signTypedData>[1]

export function createEip712StructedDataSignCertificate(
  account: Account,
  encodedCert: string,
  ttl: number
): SignTypedDataInput {
  return {
    domain: {
      name: 'Phat Query Certificate',
      version: '1',
      salt: SALT,
    },
    message: {
      description:
        'You are signing a Certificate that can be used to query Phat Contracts using your identity without further prompts.',
      timeToLive: `The Certificate will be valid till block ${ttl}.`,
      encodedCert,
    },
    primaryType: 'IssueQueryCertificate',
    types: {
      EIP712Domain: [
        { name: 'name', type: 'string' },
        { name: 'version', type: 'string' },
        { name: 'salt', type: 'bytes32' },
      ],
      IssueQueryCertificate: [
        { name: 'description', type: 'string' },
        { name: 'timeToLive', type: 'string' },
        { name: 'encodedCert', type: 'bytes' },
      ],
    },
    account,
  }
}

export function createEip712StructedDataSignQuery(account: Account, encodedQuery: string): SignTypedDataInput {
  return {
    domain: {
      name: 'Phat Contract Query',
      version: '1',
      salt: SALT,
    },
    message: {
      description: 'You are signing a query request that would be sent to a Phat Contract.',
      encodedQuery: encodedQuery,
    },
    primaryType: 'PhatContractQuery',
    types: {
      EIP712Domain: [
        { name: 'name', type: 'string' },
        { name: 'version', type: 'string' },
        { name: 'salt', type: 'bytes32' },
      ],
      PhatContractQuery: [
        { name: 'description', type: 'string' },
        { name: 'encodedQuery', type: 'bytes' },
      ],
    },
    account,
  }
}

export interface Eip712Domain {
  name: string
  version: string
  chainId: number
  verifyingContract: Address
}

export function createEip712Domain(api: ApiPromise): Eip712Domain {
  try {
    const name = hexToString(api.consts.evmAccountMapping.eip712Name.toString())
    const version = hexToString(api.consts.evmAccountMapping.eip712Version.toString())
    const chainId = (api.consts.evmAccountMapping.eip712ChainID as U256).toNumber()
    const verifyingContract = api.consts.evmAccountMapping.eip712VerifyingContractAddress.toString() as Address
    return {
      name,
      version,
      chainId,
      verifyingContract,
    }
  } catch (_err) {
    throw new Error(
      'Create Eip712Domain object failed, possibly due to the unavailability of the evmAccountMapping pallet.'
    )
  }
}

export interface SubstrateCall {
  who: string
  callData: string
  nonce: number
}

export async function createSubstrateCall<T extends ApiTypes>(
  api: ApiPromise,
  substrateAddress: string,
  extrinsic: SubmittableExtrinsic<T>
) {
  const nonce = await api.query.evmAccountMapping.accountNonce<U64>(substrateAddress)
  return {
    who: substrateAddress,
    callData: extrinsic.inner.toHex(),
    nonce: nonce.toNumber(),
  }
}

/**
 * @params account Account  The viem WalletAccount instance for signging.
 * @params who string       The SS58 formated address of the account.
 * @params callData string  The encoded call data, usually create with `api.tx.foo.bar.inner.toHex()`
 * @params nonce number     The nonce of the account.
 */
export function createEip712StructedDataSubstrateCall(
  domain: Eip712Domain,
  message: SubstrateCall
): Omit<SignTypedDataInput, 'account'> {
  return {
    types: {
      EIP712Domain: [
        {
          name: 'name',
          type: 'string',
        },
        {
          name: 'version',
          type: 'string',
        },
        {
          name: 'chainId',
          type: 'uint256',
        },
        {
          name: 'verifyingContract',
          type: 'address',
        },
      ],
      SubstrateCall: [
        { name: 'who', type: 'string' },
        { name: 'callData', type: 'bytes' },
        { name: 'nonce', type: 'uint64' },
      ],
    },
    primaryType: 'SubstrateCall',
    domain: domain,
    message: { ...message },
  }
}

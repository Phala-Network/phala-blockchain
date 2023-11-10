import { hexToU8a, u8aToHex } from '@polkadot/util'
import { blake2AsU8a, encodeAddress, secp256k1Compress } from '@polkadot/util-crypto'
import { type Account, type Client } from 'viem'
import { hashMessage, recoverPublicKey } from 'viem'
import { type signTypedData } from 'viem/wallet'
import { signMessage } from 'viem/wallet'

// keccak256(b"phala/phat-contract")
const SALT = '0x0ea813d1592526d672ea2576d7a07914cef2ca301b35c5eed941f7c897512a00'

type SignTypedDataInput = Parameters<typeof signTypedData>[1]

/**
 * Get compact formatted ether address for a specified account via a Wallet Client.
 */
export async function etherAddressToCompactPubkey(client: Client, account: Account, msg = 'Allows to access the pubkey address.') {
  const sign = await signMessage(client, { account, message: msg })
  const hash = hashMessage(msg)
  const recovered = await recoverPublicKey({ hash, signature: sign })
  const compactPubkey = u8aToHex(secp256k1Compress(hexToU8a(recovered)))
  return compactPubkey
}

/**
 * Convert an Ethereum address to a Substrate address.
 */
export async function etherAddressToSubstrateAddress(
    client: Client,
    account: Account,
    {
        SS58Prefix = 30,
        msg = undefined,
    } = {}
) {
  const compactPubkey = await etherAddressToCompactPubkey(client, account, msg)
  const substratePubkey = encodeAddress(blake2AsU8a(hexToU8a(compactPubkey)), SS58Prefix)
  return substratePubkey
}

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

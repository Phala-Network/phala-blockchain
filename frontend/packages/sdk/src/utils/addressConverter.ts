import { hexToU8a, stringToU8a, u8aToHex } from '@polkadot/util'
import { blake2AsU8a, decodeAddress, keccak256AsU8a, secp256k1Compress } from '@polkadot/util-crypto'
import type { Account, Hex, TestClient, WalletClient } from 'viem'
import { hashMessage, recoverPublicKey } from 'viem'
import { signMessage } from 'viem/wallet'

/**
 * Recovered pubkey and compressed pubkey.
 */
export async function recoverEvmPubkey(
  client: WalletClient | TestClient,
  account: Account,
  msg = 'Allows to access the pubkey address.'
) {
  const sign = await signMessage(client, { account, message: msg })
  const hash = hashMessage(msg)
  const uncompressed = await recoverPublicKey({ hash, signature: sign })
  const compressed = u8aToHex(secp256k1Compress(hexToU8a(uncompressed)))
  return {
    uncompressed,
    compressed,
    toString: () => uncompressed,
  } as const
}

const EVM_ADDRESS_SUFFIX = stringToU8a('@evm_address')

/**
 * Convert an EVM public key (both compressed & uncompressed are supported) to a Substrate raw address.
 *
 * https://davidederosa.com/basic-blockchain-programming/elliptic-curve-keys/
 */
export function evmPublicKeyToSubstrateRawAddressU8a(hex: string): Uint8Array {
  const pubkey = hexToU8a(hex)
  if (pubkey.length === 65) {
    if (pubkey[0] !== 4) {
      throw new Error('Invalid public key format.')
    }
    const h32 = keccak256AsU8a(pubkey.subarray(1))
    const h20 = h32.subarray(12)
    return new Uint8Array([...h20, ...EVM_ADDRESS_SUFFIX])
  } else if (pubkey.length === 33) {
    return blake2AsU8a(pubkey)
  } else {
    throw new Error('Invalid public key length.')
  }
}

export function substrateRawAddressToEvmAddress(address: string): Hex {
  const substratePubkey = decodeAddress(address)
  if (substratePubkey.length !== 32) {
    throw new Error('Invalid public key length.')
  }
  if (substratePubkey.subarray(20).toString() !== EVM_ADDRESS_SUFFIX.toString()) {
    throw new Error('This Substrate Address is not mapped from an EVM Address.')
  }
  const h20 = substratePubkey.subarray(0, 20)
  return u8aToHex(h20)
}

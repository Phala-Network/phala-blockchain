import { hexToU8a, u8aToHex } from '@polkadot/util'
import { blake2AsU8a, decodeAddress, keccak256AsU8a, secp256k1Compress, secp256k1Expand } from '@polkadot/util-crypto'
import type { Account, Hex, TestClient, WalletClient } from 'viem'
import { hashMessage, recoverPublicKey } from 'viem'
import { signMessage } from 'viem/wallet'

//
// See: https://github.com/Phala-Network/substrate-evm_account_mapping/blob/main/pallets/evm_account_mapping/src/lib.rs
// We keep use same name here avoiding confusion.
//
export type Converters = 'SubstrateAddressConverter' | 'EvmTransparentConverter'

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

// equals to `stringToU8a('@evm_address')`
const EVM_ADDRESS_SUFFIX = new Uint8Array([64, 101, 118, 109, 95, 97, 100, 100, 114, 101, 115, 115])

/**
 * Convert an EVM public key (both compressed & uncompressed are supported) to a Substrate raw address.
 *
 * https://davidederosa.com/basic-blockchain-programming/elliptic-curve-keys/
 */
export function evmPublicKeyToSubstrateRawAddressU8a(
  hex: string,
  converter: Converters = 'EvmTransparentConverter'
): Uint8Array {
  let pubkey = hexToU8a(hex)
  let isCompressedPubkey = false
  const len = pubkey.length
  if (len === 64) {
    throw new Error('Unexpected public key length: it should be 65 bytes or 33 bytes (compressed form).')
  }
  if (len !== 65 && len !== 33) {
    throw new Error('Invalid public key length.')
  }
  if (len === 65 && pubkey[0] !== 4) {
    throw new Error('Invalid public key format: it should 65 bytes and prefix with 0x04.')
  } else if (len === 33) {
    isCompressedPubkey = true
    if (pubkey[0] !== 2 && pubkey[0] !== 3) {
      throw new Error('Invalid compressed public key format: it should 33 bytes and prefix with 0x02 or 0x03.')
    }
  }
  if (converter === 'EvmTransparentConverter') {
    const h32 = keccak256AsU8a(isCompressedPubkey ? secp256k1Expand(pubkey) : pubkey.subarray(1))
    const h20 = h32.subarray(12)
    return new Uint8Array([...h20, ...EVM_ADDRESS_SUFFIX])
  } else if (converter === 'SubstrateAddressConverter') {
    if (!isCompressedPubkey) {
      pubkey = secp256k1Compress(pubkey)
    }
    return blake2AsU8a(pubkey)
  } else {
    throw new Error(`Unknown converter: ${converter}`)
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

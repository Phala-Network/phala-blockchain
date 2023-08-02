import { type Client, type Account } from 'viem';

import { hexToU8a, u8aToHex } from "@polkadot/util";
import { encodeAddress } from "@polkadot/util-crypto";
import { hashMessage, recoverPublicKey } from "viem";
import { signMessage } from 'viem/wallet';
import { secp256k1Compress, blake2AsU8a } from '@polkadot/util-crypto';


/**
 * Get compact formatted ether address for a specified account via a Wallet Client.
 */
export async function etherAddressToCompactPubkey(client: Client, account: Account) {
  const msg = '0x48656c6c6f'
  const sign = await signMessage(client, { account, message: msg })
  const hash = hashMessage(msg)
  const recovered = await recoverPublicKey({ hash, signature:sign })
  const compactPubkey = u8aToHex(secp256k1Compress(hexToU8a(recovered)))
  return compactPubkey
}

/**
 * Convert an Ethereum address to a Substrate address.
 */
export async function etherAddressToSubstrateAddress(client: Client, account: Account) {
  const compactPubkey = await etherAddressToCompactPubkey(client, account)
  const substratePubkey = encodeAddress(blake2AsU8a(hexToU8a(compactPubkey)), 42)
  return substratePubkey
}

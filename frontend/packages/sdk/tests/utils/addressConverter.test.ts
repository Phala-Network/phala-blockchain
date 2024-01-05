import { hexToU8a, stringToU8a, u8aToHex } from '@polkadot/util'
import { encodeAddress, keccak256AsU8a, secp256k1Compress } from '@polkadot/util-crypto'
import { createWalletClient, http } from 'viem'
import { privateKeyToAccount } from 'viem/accounts'
import { mainnet } from 'viem/chains'
import { describe, expect, it } from 'vitest'
import {
  evmPublicKeyToSubstrateRawAddressU8a,
  recoverEvmPubkey,
  substrateRawAddressToEvmAddress,
} from '../../src/utils/addressConverter'

describe('AddressConverter', () => {
  it('supports EvmTransparentConverter `origin = keccak256(pubkey)[12..] + b"@evm_address"`', () => {
    const uncompressed =
      '049df1e69b8b7c2da2efe0069dc141c2cec0317bf3fd135abaeb69ee33801f597024dc8558dbe54a0328ceaa081387a5e1c5749247266fe53dde4ba7ddbf43eae6'

    const public_key = hexToU8a(uncompressed)
    const h32 = keccak256AsU8a(public_key.subarray(1))
    const h20 = h32.subarray(12)
    const suffix = stringToU8a('@evm_address')
    const raw = new Uint8Array([...h20, ...suffix])
    expect(raw).toEqual(hexToU8a('77bb3d64ea13e4f0beafdd5d92508d4643bb09cb4065766d5f61646472657373'))

    const result = evmPublicKeyToSubstrateRawAddressU8a(uncompressed)
    expect(raw).toEqual(result)

    expect(encodeAddress(raw, 42)).toEqual('5EmhBEe8vsSfqYseKctWsaQqNKCF9FFao6Mqa9hNfcdF25oE')
  })

  it('supports EvmTransparentConverter with compressed pubkey', () => {
    const uncompressed =
      '049df1e69b8b7c2da2efe0069dc141c2cec0317bf3fd135abaeb69ee33801f597024dc8558dbe54a0328ceaa081387a5e1c5749247266fe53dde4ba7ddbf43eae6'
    const compressed = '0x029df1e69b8b7c2da2efe0069dc141c2cec0317bf3fd135abaeb69ee33801f5970'

    const compressedPubkey = secp256k1Compress(hexToU8a(uncompressed))
    expect(u8aToHex(compressedPubkey)).toEqual(compressed)

    const fromUncompressed = evmPublicKeyToSubstrateRawAddressU8a(uncompressed)
    const fromCompressed = evmPublicKeyToSubstrateRawAddressU8a(compressed)
    expect(fromUncompressed).toEqual(fromCompressed)
  })

  it('supports SubstrateAddressConverter', () => {
    const hex = '027cf2fa7bfe66adad4149481ff86794ce7e1ab2f7ed615ad3918f91581d2c00f1'
    const result = evmPublicKeyToSubstrateRawAddressU8a(hex, 'SubstrateAddressConverter')
    expect('5DT96geTS2iLpkH8fAhYAAphNpxddKCV36s5ShVFavf1xQiF').toEqual(encodeAddress(result, 42))
  })

  it('supports SubstrateAddressConverter with uncompressed pubkey', () => {
    const uncompressed =
      '049df1e69b8b7c2da2efe0069dc141c2cec0317bf3fd135abaeb69ee33801f597024dc8558dbe54a0328ceaa081387a5e1c5749247266fe53dde4ba7ddbf43eae6'
    const compressed = '0x029df1e69b8b7c2da2efe0069dc141c2cec0317bf3fd135abaeb69ee33801f5970'

    const fromUncompressed = evmPublicKeyToSubstrateRawAddressU8a(uncompressed, 'SubstrateAddressConverter')
    const fromCompressed = evmPublicKeyToSubstrateRawAddressU8a(compressed, 'SubstrateAddressConverter')
    expect(fromUncompressed).toEqual(fromCompressed)
  })

  it('can convert substrate address into evm address', () => {
    const evmAddress = '0x77bb3d64ea13e4f0beafdd5d92508d4643bb09cb'
    const substrateAddress = '5EmhBEe8vsSfqYseKctWsaQqNKCF9FFao6Mqa9hNfcdF25oE'

    const result = substrateRawAddressToEvmAddress(substrateAddress)
    expect(evmAddress).toEqual(result)
  })

  it('can recoverEvmPubkey', async () => {
    const account = privateKeyToAccount('0x415ac5b1b9c3742f85f2536b1eb60a03bf64a590ea896b087182f9c92f41ea12')
    const walletClient = createWalletClient({
      account,
      chain: mainnet,
      transport: http(),
    })
    const recovered = await recoverEvmPubkey(walletClient, account, 'Allows to access the pubkey address.')
    expect(recovered.uncompressed).toEqual(
      '0x047cf2fa7bfe66adad4149481ff86794ce7e1ab2f7ed615ad3918f91581d2c00f1b78639ed0e27ac2990496f3459b2c09ea4d5b3322a0ce7da7ec1fd86f069854c'
    )
    expect(recovered.compressed).toEqual('0x027cf2fa7bfe66adad4149481ff86794ce7e1ab2f7ed615ad3918f91581d2c00f1')
  })
})

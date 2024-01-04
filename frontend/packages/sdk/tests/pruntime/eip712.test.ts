import { hexToU8a, stringToU8a } from '@polkadot/util'
import { encodeAddress, keccak256AsU8a } from '@polkadot/util-crypto'
import { describe, expect, it } from 'vitest'
import { evmPublicKeyToSubstratePubkey, substrateAddressToEvmAddress } from '../../src/pruntime/eip712'

describe('eip712', () => {
  it('can satisfy formula `origin = keccak256(pubkey)[12..] + b"@evm_address"`', () => {
    const hex =
      '049df1e69b8b7c2da2efe0069dc141c2cec0317bf3fd135abaeb69ee33801f597024dc8558dbe54a0328ceaa081387a5e1c5749247266fe53dde4ba7ddbf43eae6'

    const public_key = hexToU8a(hex)
    const h32 = keccak256AsU8a(public_key.subarray(1))
    const h20 = h32.subarray(12)
    const suffix = stringToU8a('@evm_address')
    const raw = new Uint8Array([...h20, ...suffix])

    expect(raw).toEqual(hexToU8a('77bb3d64ea13e4f0beafdd5d92508d4643bb09cb4065766d5f61646472657373'))
    const result = evmPublicKeyToSubstratePubkey(hex)
    expect(raw).toEqual(result)

    expect(encodeAddress(raw)).toEqual('5EmhBEe8vsSfqYseKctWsaQqNKCF9FFao6Mqa9hNfcdF25oE')
  })

  it('can convert compressed pubkey into substrate address', () => {
    const hex = '027cf2fa7bfe66adad4149481ff86794ce7e1ab2f7ed615ad3918f91581d2c00f1'
    const result = evmPublicKeyToSubstratePubkey(hex)
    expect('5DT96geTS2iLpkH8fAhYAAphNpxddKCV36s5ShVFavf1xQiF').toEqual(encodeAddress(result))
  })

  it('can convert substrate address into evm address', () => {
    const evmAddress = '0x77bb3d64ea13e4f0beafdd5d92508d4643bb09cb'
    const substrateAddress = '5EmhBEe8vsSfqYseKctWsaQqNKCF9FFao6Mqa9hNfcdF25oE'

    const result = substrateAddressToEvmAddress(substrateAddress)
    expect(evmAddress).toEqual(result)
  })
})

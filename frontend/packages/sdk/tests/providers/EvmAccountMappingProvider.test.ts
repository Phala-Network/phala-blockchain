import { ApiPromise, WsProvider } from '@polkadot/api'
import type { ApiTypes } from '@polkadot/api/types'
import type { QueryableConsts } from '@polkadot/api/types/consts'
import type { Bytes, U256 } from '@polkadot/types'
import { H160 } from '@polkadot/types/interfaces'
import { hexToU8a, u8aToHex } from '@polkadot/util'
import { encodeAddress, secp256k1Compress } from '@polkadot/util-crypto'
import { createWalletClient, http } from 'viem'
import { generatePrivateKey, privateKeyToAccount } from 'viem/accounts'
import { mainnet } from 'viem/chains'
import { describe, expect, it, vi } from 'vitest'
import { options } from '../../src/options'
import { EvmAccountMappingProvider } from '../../src/providers/EvmAccountMappingProvider'
import { evmPublicKeyToSubstratePubkey } from '../../src/pruntime/eip712'

declare module '@polkadot/api/types/consts' {
  interface AugmentedConsts<ApiType extends ApiTypes> {
    evmAccountMapping: {
      eip712Name: Bytes
      eip712Version: Bytes
      eip712ChainID: U256
      eip712VerifyingContractAddress: H160
    }
  }

  export interface QueryableConsts<ApiType extends ApiTypes> extends AugmentedConsts<ApiType> {}
}

describe.skipIf(!process.env.TEST_RPC_ENDPOINT)('EvmAccountMappingProvider', () => {
  const account = privateKeyToAccount(generatePrivateKey())
  const walletClient = createWalletClient({
    account,
    chain: mainnet,
    transport: http(),
  })
  const rpc = process.env.TEST_RPC_ENDPOINT

  it('throws error if evm_account_mapping pallet is not available', () => {
    expect(async () => {
      const api = await ApiPromise.create(
        options({
          provider: new WsProvider(rpc),
          noInitWarn: true,
        })
      )
      vi.spyOn(api, 'consts', 'get').mockReturnValue({} as QueryableConsts<'promise'>)
      await EvmAccountMappingProvider.create(api, walletClient, account)
    }).rejects.toThrowError()
  })

  it('throws error on unsupport eip712Version', () => {
    expect(async () => {
      const api = await ApiPromise.create(
        options({
          provider: new WsProvider(rpc),
          noInitWarn: true,
        })
      )
      vi.spyOn(api, 'consts', 'get').mockReturnValue({
        evmAccountMapping: {
          eip712Name: api.createType('Bytes', '0x5068616c614e6574776f726b'),
          eip712Version: api.createType('Bytes', '0x30'),
          eip712ChainID: api.createType('u256', 0),
          eip712VerifyingContractAddress: api.createType('H160', ''),
        },
      })
      await EvmAccountMappingProvider.create(api, walletClient, account)
    }).rejects.toThrowError()
  })

  it('handle eip712Version 1 that create mapped address from compressed pubkey', async () => {
    const api = await ApiPromise.create(
      options({
        provider: new WsProvider(rpc),
        noInitWarn: true,
      })
    )
    const cloned = { ...api.consts.evmAccountMapping, eip712Version: api.createType('Bytes', '0x31') }
    vi.spyOn(api, 'consts', 'get').mockReturnValue({
      evmAccountMapping: cloned,
    })
    const provider = await EvmAccountMappingProvider.create(api, walletClient, account)

    const fromUncompressed = evmPublicKeyToSubstratePubkey(account.publicKey)
    expect(encodeAddress(fromUncompressed)).not.toEqual(provider.address)

    const compressed = u8aToHex(secp256k1Compress(hexToU8a(account.publicKey)))
    const fromCompressed = evmPublicKeyToSubstratePubkey(compressed)
    expect(encodeAddress(fromCompressed, 30)).toEqual(provider.address)
  })

  it('handle eip712Version 2 that create mapped address from uncompressed pubkey', async () => {
    const api = await ApiPromise.create(
      options({
        provider: new WsProvider(rpc),
        noInitWarn: true,
      })
    )
    if (api.consts.evmAccountMapping.eip712Version.toString() !== '0x32') {
      const cloned = { ...api.consts.evmAccountMapping, eip712Version: api.createType('Bytes', '0x32') }
      vi.spyOn(api, 'consts', 'get').mockReturnValue({
        evmAccountMapping: cloned,
      })
    }
    const provider = await EvmAccountMappingProvider.create(api, walletClient, account)

    const fromUncompressed = evmPublicKeyToSubstratePubkey(account.publicKey)
    expect(encodeAddress(fromUncompressed, 30)).toEqual(provider.address)

    const compressed = u8aToHex(secp256k1Compress(hexToU8a(account.publicKey)))
    const fromCompressed = evmPublicKeyToSubstratePubkey(compressed)
    expect(encodeAddress(fromCompressed, 30)).not.toEqual(provider.address)
  })
})

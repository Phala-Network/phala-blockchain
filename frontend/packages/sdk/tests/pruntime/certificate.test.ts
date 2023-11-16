import { Keyring } from '@polkadot/api'
import type { KeyringPair } from '@polkadot/keyring/types'
import { TypeRegistry, typeDefinitions } from '@polkadot/types'
import type {
  Registry,
  RegistryTypes,
  Signer,
  SignerPayloadJSON,
  SignerPayloadRaw,
  SignerResult,
} from '@polkadot/types/types'
import { hexToU8a, objectSpread, u8aToHex } from '@polkadot/util'
import { createTestClient, http } from 'viem'
import { privateKeyToAccount } from 'viem/accounts'
import { mainnet } from 'viem/chains'
import { describe, it } from 'vitest'
import { types } from '../../src/options'
import { signCertificate, unstable_signEip712Certificate } from '../../src/pruntime/certificate'
import { etherAddressToCompressedPubkey } from '../../src/pruntime/eip712'

let id = 0

export class SingleAccountSigner implements Signer {
  readonly #keyringPair: KeyringPair
  readonly #registry: Registry
  readonly #signDelay: number

  constructor(registry: Registry, keyringPair: KeyringPair, signDelay = 0) {
    this.#keyringPair = keyringPair
    this.#registry = registry
    this.#signDelay = signDelay
  }

  public async signPayload(payload: SignerPayloadJSON): Promise<SignerResult> {
    if (payload.address !== this.#keyringPair.address) {
      throw new Error('Signer does not have the keyringPair')
    }

    return new Promise((resolve): void => {
      setTimeout((): void => {
        const signed = this.#registry
          .createType('ExtrinsicPayload', payload, { version: payload.version })
          .sign(this.#keyringPair)

        resolve(objectSpread({ id: ++id }, signed))
      }, this.#signDelay)
    })
  }

  public async signRaw({ address, data }: SignerPayloadRaw): Promise<SignerResult> {
    if (address !== this.#keyringPair.address) {
      throw new Error('Signer does not have the keyringPair')
    }

    return new Promise((resolve): void => {
      setTimeout((): void => {
        const signature = u8aToHex(this.#keyringPair.sign(hexToU8a(data)))

        resolve({
          id: ++id,
          signature,
        })
      }, this.#signDelay)
    })
  }
}

describe('sign certificate', async function () {
  const registry = new TypeRegistry()
  registry.register({ ...types, ...typeDefinitions } as unknown as RegistryTypes)
  const keyring = new Keyring({ type: 'sr25519' })
  const pair = keyring.addFromUri('//Alice')

  it('smoking test for sign certificate with keyring pair', async function () {
    await signCertificate({ pair })
  })

  it('smoking test for sign certificate with signer', async function () {
    const mockSigner = new SingleAccountSigner(registry, pair)
    await signCertificate({ account: pair, signer: mockSigner })
  })

  it('smoking test for sign certificate with eip712 wallet client', async function () {
    const account = privateKeyToAccount('0x415ac5b1b9c3742f85f2536b1eb60a03bf64a590ea896b087182f9c92f41ea12')
    const client = createTestClient({ account, chain: mainnet, mode: 'anvil', transport: http() })
    const compressedPubkey = await etherAddressToCompressedPubkey(client, account)
    await unstable_signEip712Certificate({ client, account, compressedPubkey })
  })
})

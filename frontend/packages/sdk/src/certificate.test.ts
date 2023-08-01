import { describe, it } from 'vitest'
import { Keyring, type ApiPromise } from '@polkadot/api'
import { TypeRegistry, typeDefinitions } from '@polkadot/types'
import type { KeyringPair } from '@polkadot/keyring/types'
import type { Registry, RegistryTypes, SignerPayloadJSON, SignerPayloadRaw } from '@polkadot/types/types'
import type { Signer, SignerResult } from '@polkadot/types/types'
import { hexToU8a, objectSpread, u8aToHex } from '@polkadot/util';

import { types } from './lib/types'
import { signCertificate } from './certificate'


let id = 0;

export class SingleAccountSigner implements Signer {
  readonly #keyringPair: KeyringPair;
  readonly #registry: Registry;
  readonly #signDelay: number;

  constructor (registry: Registry, keyringPair: KeyringPair, signDelay = 0) {
    this.#keyringPair = keyringPair;
    this.#registry = registry;
    this.#signDelay = signDelay;
  }

  public async signPayload (payload: SignerPayloadJSON): Promise<SignerResult> {
    if (payload.address !== this.#keyringPair.address) {
      throw new Error('Signer does not have the keyringPair');
    }

    return new Promise((resolve): void => {
      setTimeout((): void => {
        const signed = this.#registry.createType('ExtrinsicPayload', payload, { version: payload.version }).sign(this.#keyringPair);

        resolve(objectSpread({ id: ++id }, signed));
      }, this.#signDelay);
    });
  }

  public async signRaw ({ address, data }: SignerPayloadRaw): Promise<SignerResult> {
    if (address !== this.#keyringPair.address) {
      throw new Error('Signer does not have the keyringPair');
    }

    return new Promise((resolve): void => {
      setTimeout((): void => {
        const signature = u8aToHex(this.#keyringPair.sign(hexToU8a(data)));

        resolve({
          id: ++id,
          signature
        });
      }, this.#signDelay);
    });
  }
}


describe('sign certificate', async function() {
  const registry = new TypeRegistry()
  registry.register({ ...types, ...typeDefinitions } as unknown as RegistryTypes)
  const keyring = new Keyring({ type: 'sr25519' })
  const pair = keyring.addFromUri('//Alice')

  it('smoking test for sign certificate with keyring pair', async function() {
    await signCertificate({ pair, api: (registry as unknown as ApiPromise) })
  })

  it('smoking test for sign certificate with signer', async function() {
    const mockSigner = new SingleAccountSigner(registry, pair)
    await signCertificate({ account: pair, signer: mockSigner, api: (registry as unknown as ApiPromise) })
  })
})

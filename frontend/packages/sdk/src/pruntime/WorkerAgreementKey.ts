import { hexAddPrefix, hexToU8a } from '@polkadot/util'
import { sr25519Agreement, sr25519PairFromSeed } from '@polkadot/util-crypto'
import { type Keypair } from '@polkadot/util-crypto/types'
import { type HexString } from '../types'
import { randomHex } from '../utils/hex'

// Generate a keypair for encryption
// NOTE: each instance only has a pre-generated pair now, it maybe better to generate a new keypair every time encrypting
export class WorkerAgreementKey {
  readonly pair: Keypair
  readonly agreementKey: Uint8Array

  constructor(workerPubkey: string, seed?: HexString) {
    // TODO validate workerPubkey & seed
    if (!workerPubkey) {
      throw new Error("You OnChainRegistry object doesn't setup correctly and no remotePubkey found.")
    }

    if (!seed) {
      seed = randomHex(32)
    }
    this.pair = sr25519PairFromSeed(hexToU8a(hexAddPrefix(seed)))
    this.agreementKey = sr25519Agreement(this.pair.secretKey, hexToU8a(hexAddPrefix(workerPubkey)))
  }

  get publicKey() {
    return this.pair.publicKey
  }

  get secretKey() {
    return this.pair.secretKey
  }
}

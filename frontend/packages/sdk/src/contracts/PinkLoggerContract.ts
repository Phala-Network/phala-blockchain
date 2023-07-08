import type { ApiPromise } from '@polkadot/api'
import type { Text } from '@polkadot/types'
import type { Result } from '@polkadot/types-codec'
import type { KeyringPair } from '@polkadot/keyring/types'
import type { AccountId } from '@polkadot/types/interfaces'
import type { OnChainRegistry } from '../OnChainRegistry'
import type { InkResponse } from '../types'

import { Keyring } from '@polkadot/api'
import { hexAddPrefix, hexToU8a, stringToHex } from '@polkadot/util'
import { sr25519Agree, sr25519KeypairFromSeed } from "@polkadot/wasm-crypto";

import { PinkContractPromise, pinkQuery } from './PinkContract'
import { ContractInitialError } from './Errors'
import logServerAbi from '../abis/log_server.json'
import { signCertificate } from '../certificate'
import { randomHex } from '../lib/hex'


export class PinkLoggerContractPromise extends PinkContractPromise {

  #pair: KeyringPair

  static async create(api: ApiPromise, registry: OnChainRegistry, systemContract: PinkContractPromise, pair?: KeyringPair): Promise<PinkLoggerContractPromise> {
    let _pair: KeyringPair | undefined = pair
    if (!_pair) {
      const keyring = new Keyring({ type: 'sr25519' });
      _pair = keyring.addFromUri('//Alice')
    }
    const cert = await signCertificate({ api, pair: _pair })
    const { output } = await systemContract.query['system::getDriver'](_pair, cert, 'PinkLogger')
    const contractId = (output as Result<Text, any>).asOk.toHex()
    if (!contractId) {
      throw new ContractInitialError('No PinkLogger contract registered in the cluster.')
    }
    const contractKey = await registry.getContractKey(contractId)
    if (!contractKey) {
      throw new ContractInitialError('PinkLogger contract ID is incorrect and not found in the cluster.')
    }
    return new PinkLoggerContractPromise(api, registry, logServerAbi, contractId, contractKey, pair)
  }

  constructor(api: ApiPromise, registry: OnChainRegistry, abi: any, contractId: string, contractKey: string, pair?: KeyringPair) {
    super(api, registry, abi, contractId, contractKey)
    if (!pair) {
      const keyring = new Keyring({ type: 'sr25519' });
      this.#pair = keyring.addFromUri('//Alice')
    } else {
      this.#pair = pair
    }
  }

  async getLog(contractId: AccountId | string, from: number = 0, counts: number = 100) {
    const api = this.api as ApiPromise

    // Generate a keypair for encryption
    // NOTE: each instance only has a pre-generated pair now, it maybe better to generate a new keypair every time encrypting
    const seed = hexToU8a(hexAddPrefix(randomHex(32)));
    const pair = sr25519KeypairFromSeed(seed);
    const [sk, pk] = [pair.slice(0, 64), pair.slice(64)];

    const encodedQuery = api.createType('InkQuery', {
      head: {
        nonce: hexAddPrefix(randomHex(32)),
        id: this.address,
      },
      data: {
        SidevmMessage: stringToHex(JSON.stringify({
          action: 'GetLog',
          contract: contractId,
          from,
          count: counts,
        })),
      },
    })

    const queryAgreementKey = sr25519Agree(
      hexToU8a(hexAddPrefix(this.phatRegistry.remotePubkey)),
      sk
    );

    const cert = await signCertificate({ pair: this.#pair, api });

    const response = await pinkQuery(api, this.phatRegistry.phactory, pk, queryAgreementKey, encodedQuery.toHex(), cert)
    const inkResponse = api.createType<InkResponse>('InkResponse', response)
    if (inkResponse.result.isErr) {
      let error = `[${inkResponse.result.asErr.index}] ${inkResponse.result.asErr.type}`
      if (inkResponse.result.asErr.type === 'RuntimeError') {
        error = `${error}: ${inkResponse.result.asErr.value}`
      }
      throw new Error(error)
    }
    const payload = inkResponse.result.asOk.asInkMessageReturn.toHuman() as string
    return JSON.parse(payload)
  }
}

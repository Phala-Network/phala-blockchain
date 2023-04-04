import type { ApiPromise } from '@polkadot/api'
import type { KeyringPair } from '@polkadot/keyring/types'
import type { bool, Result, Tuple, U128, U32, U64, U8, Vec } from '@polkadot/types'
import type { AccountId } from '@polkadot/types/interfaces'
import type { OnChainRegistry } from '../OnChainRegistry'

import { Keyring } from '@polkadot/api'

import { PinkContractPromise } from './PinkContract'
import { ContractInitialError } from './Errors'
import systemAbi from '../abis/system.json'

// @todo we can added more drivers name here so we can get hinting from IDE.
export type SystemDrivers = ('PinkLogger') & string

export class SystemContractPromise extends PinkContractPromise {

  #pair: KeyringPair

  static async create(api: ApiPromise, registry: OnChainRegistry): Promise<SystemContractPromise> {
    const contractId = registry.clusterInfo?.systemContract
    if (!contractId) {
      throw new ContractInitialError('No system contract registered in the cluster.')
    }
    const contractKey = await registry.getContractKey(contractId)
    if (!contractKey) {
      throw new ContractInitialError('System contract ID is incorrect and not found in the cluster.')
    }
    const systemContract = new SystemContractPromise(api, registry, systemAbi, contractId, contractKey)
    return systemContract
  }

  constructor(api: ApiPromise, registry: OnChainRegistry, abi: any, contractId: string | AccountId, contractKey: string, pair?: KeyringPair) {
    super(api, registry, abi, contractId, contractKey)
    if (!pair) {
      const keyring = new Keyring({ type: 'sr25519' });
      this.#pair = keyring.addFromUri('//Alice')
    } else {
      this.#pair = pair
    }
  }

  /**
   * Get the version of the system contract.
   */
  async version() {
    const response = await this.query['system::version'](this.#pair)
    if (response.output) {
      const output = response.output as Result<Tuple, Vec<U8>>
      return output.asOk.toJSON() as [number, number]
    }
  }

  // @fixme Not yet test.
  async grantAdmin(contractId: AccountId | string) {
    const { gasRequired } = await this.query['system::grantAdmin'](this.#pair, contractId)
    await this.tx['system::grantAdmin']({ gasLimit: gasRequired.refTime.toBn() }, this.#pair, contractId).signAndSend(this.#pair)
  }

  // @fixme Not yet test.
  async setDriver(name: SystemDrivers, contractId: AccountId | string) {
    const { gasRequired } = await this.query['system::setDriver'](this.#pair, name, contractId)
    await this.tx['system::set_driver']({ gasLimit: gasRequired.refTime.toBn() }, this.#pair, name, contractId).signAndSend(this.#pair)
  }

  // @fixme Not yet test.
  async getDriver(name: SystemDrivers) {
    const response = await this.query['system::getDriver'](this.#pair, name)
    if (response.output) {
      const output = response.output as Result<AccountId, Vec<U8>>
      return output.asOk.toHex()
    }
  }

  // @fixme Not yet test.
  async deploySidevmTo(contractId: AccountId | string, codeHash: string) {
    await this.query['system::deploySidevmTo'](this.#pair, contractId, codeHash)
  }

  // @fixme Not yet test.
  async stopSidevmAt(contractId: AccountId | string) {
    await this.query['system::stopSidevmAt'](this.#pair, contractId)
  }

  // @fixme Not yet test.
  async setHook(hook: unknown, contractId: AccountId | string, selector: string, gasLimit: U64 | number) {
    const { gasRequired } = await this.query['system::setHook'](this.#pair, hook, contractId, selector, gasLimit)
    await this.tx['system::setHook']({ gasLimit: gasRequired.refTime.toBn() }, this.#pair, hook, contractId, selector, gasLimit).signAndSend(this.#pair)
  }

  // @fixme Not yet test.
  async setContractWeight(contractId: AccountId | string, weight: U32 | number) {
    await this.query['system::setContractWeight'](this.#pair, contractId, weight)
  }

  async totalBalanceOf(accountId: AccountId | string) {
    const response = await this.query['system::totalBalanceOf'](this.#pair, accountId)
    if (response.output) {
      const output = response.output as Result<U128, Vec<U8>>
      return output.asOk.toBn()
    }
  }

  async freeBalanceOf(accountId: AccountId | string) {
    const response = await this.query['system::freeBlanceOf'](this.#pair, accountId)
    if (response.output) {
      const output = response.output as Result<U128, Vec<U8>>
      return output.asOk.toBn()
    }
  }

  // @fixme Not yet test.
  async isAdmin(contractId: AccountId | string) {
    const response = await this.query['system::isAdmin'](this.#pair, contractId)
    if (response.output) {
      const output = response.output as Result<bool, Vec<U8>>
      return output.asOk.toPrimitive()
    }
  }

  async upgradeSystemContract() {
    const { gasRequired } = await this.query['system::upgradeSystemContract'](this.#pair)
    await this.tx['system::upgradeSystemContract']({ gasLimit: gasRequired.refTime.toBn() }, this.#pair).signAndSend(this.#pair)
  }

  // @fixme Not yet test.
  async changeDeposit(contractId: AccountId | string, deposit: U128 | number) {
    const { gasRequired } = await this.query['system::changeDeposit'](this.#pair, contractId, deposit)
    await this.tx['system::changeDeposit']({ gasLimit: gasRequired.refTime.toBn() }, this.#pair, contractId, deposit).signAndSend(this.#pair)
  }
}

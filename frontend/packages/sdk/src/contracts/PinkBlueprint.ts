import type { DecorateMethod, ApiTypes } from '@polkadot/api/types';
import type { AccountId, Hash, ContractInstantiateResult } from '@polkadot/types/interfaces';
import type { ISubmittableResult } from '@polkadot/types/types';
import type { AbiMessage, AbiConstructor, BlueprintOptions, ContractCallOutcome } from '@polkadot/api-contract/types';
import type { ContractCallResult, MessageMeta, MapConstructorExec } from '@polkadot/api-contract/base/types';
import type { ApiPromise } from '@polkadot/api';
import type { KeyringPair } from '@polkadot/keyring/types';

import type { OnChainRegistry } from '../OnChainRegistry';
import type { InkResponse, InkQueryError, AbiLike } from '../types';
import type { CertificateData } from '../certificate';

import { SubmittableResult } from '@polkadot/api';
import { ApiBase } from '@polkadot/api/base';
import { BN_ZERO, isUndefined, hexAddPrefix, u8aToHex, hexToU8a } from '@polkadot/util';
import { createBluePrintTx, withMeta } from '@polkadot/api-contract/base/util';
import { sr25519Agree, sr25519KeypairFromSeed, sr25519Sign } from "@polkadot/wasm-crypto";
import crypto from 'crypto';
import { from } from 'rxjs';

import { Abi } from '@polkadot/api-contract/Abi';
import { toPromiseMethod } from '@polkadot/api';
import { CodecMap, Option } from '@polkadot/types';

import { pruntime_rpc as pruntimeRpc } from "../proto";
import { decrypt, encrypt } from "../lib/aes-256-gcm";
import { randomHex } from "../lib/hex";
import assert from '../lib/assert';
import { PinkContractQueryOptions, PinkContractPromise } from './PinkContract';


interface ContractInkQuery<ApiType extends ApiTypes> extends MessageMeta {
  (origin: string | AccountId | Uint8Array, ...params: unknown[]): ContractCallResult<ApiType, ContractCallOutcome>;
}

interface MapMessageInkQuery<ApiType extends ApiTypes> {
  [message: string]: ContractInkQuery<ApiType>;
}

interface PinkContractInstantiateResult extends ContractInstantiateResult {
  salt: string;
}

interface IEncryptedData extends CodecMap {
  data: Uint8Array
  iv: Uint8Array
}

function hex(b: string | Uint8Array) {
  if (typeof b != "string") {
    b = Buffer.from(b).toString('hex');
  }
  if (!b.startsWith('0x')) {
    return '0x' + b;
  } else {
    return b;
  }
}

function createEncryptedData(pk: Uint8Array, data: string, agreementKey: Uint8Array) {
  const iv = hexAddPrefix(randomHex(12));
  return {
    iv,
    pubkey: u8aToHex(pk),
    data: hexAddPrefix(encrypt(data, agreementKey, hexToU8a(iv))),
  };
};

async function pinkQuery(
  api: ApiPromise,
  pruntimeApi: pruntimeRpc.PhactoryAPI,
  pk: Uint8Array,
  queryAgreementKey: Uint8Array,
  encodedQuery: string,
  { certificate, pubkey, secret }: CertificateData
) {
  // Encrypt the ContractQuery.
  const encryptedData = createEncryptedData(pk, encodedQuery, queryAgreementKey);
  const encodedEncryptedData = api
    .createType("EncryptedData", encryptedData)
    .toU8a();

  // Sign the encrypted data.
  const signature: pruntimeRpc.ISignature = {
    signedBy: certificate,
    signatureType: pruntimeRpc.SignatureType.Sr25519,
    signature: sr25519Sign(pubkey, secret, encodedEncryptedData),
  };

  // Send request.
  const requestData = {
    encodedEncryptedData,
    signature,
  };
  return pruntimeApi.contractQuery(requestData).then((res) => {
    const { encodedEncryptedData } = res;
    const encryptedData = api.createType<IEncryptedData>("EncryptedData", encodedEncryptedData)
    const data = decrypt(encryptedData.data.toString(), queryAgreementKey, encryptedData.iv);
    return hexAddPrefix(data);
  });
};

function createQuery(meta: AbiMessage, fn: (origin: string | AccountId | Uint8Array, options: PinkContractQueryOptions, params: unknown[]) => ContractCallResult<'promise', ContractCallOutcome>): ContractInkQuery<'promise'> {
  return withMeta(meta, (origin: string | AccountId | Uint8Array, options: PinkContractQueryOptions, ...params: unknown[]): ContractCallResult<'promise', ContractCallOutcome> =>
    fn(origin, options, params)
  );
}

export class PinkBlueprintSubmittableResult extends SubmittableResult {
  readonly registry: OnChainRegistry;
  readonly abi: Abi;
  readonly contractId?: string;

  #isFinalized: boolean = false;
  #contract?: PinkContractPromise;

  constructor (result: ISubmittableResult, abi: Abi, registry: OnChainRegistry, contractId?: string) {
    super(result);

    this.registry = registry;
    this.abi = abi;
    this.contractId = contractId;
  }

  async waitFinalized(timeout: number = 120_000) {
    if (this.#isFinalized) {
      return
    }

    if (this.isInBlock || this.isFinalized) {
      let contractId: string | undefined
      for (const event of this.events) {
        if (event.event.method === 'Instantiating') {
          // tired of TS complaining about the type of event.event.data.contract
          // @ts-ignore
          contractId = event.event.data.contract.toString();
          break;
        }
      }
      if (!contractId) {
        throw new Error('Failed to find contract ID in events, maybe instantiate failed.')
      }

      const t0 = new Date().getTime();
      while (true) {
        const result1 = (await this.registry.api.query.phalaPhatContracts.clusterContracts(this.registry.clusterId)) as unknown as Text[]
        const contractIds = result1.map(i => i.toString())
        if (contractIds.indexOf(contractId) !== -1) {
          const result2 = (await this.registry.api.query.phalaRegistry.contractKeys(contractId)) as unknown as Option<any>
          if (result2.isSome) {
            this.#isFinalized = true
            if (this.contractId) {
              const contractKey = await this.registry.getContractKeyOrFail(this.contractId)
              this.#contract = new PinkContractPromise(this.registry.api, this.registry, this.abi, this.contractId, contractKey)
            }
            return
          }
        }

        const t1 = new Date().getTime();
        if (t1 - t0 > timeout) {
          throw new Error('Timeout')
        }
        await new Promise(resolve => setTimeout(resolve, 1000));
      }
    }
    throw new Error(`instantiate failed for ${this.abi.info.source.wasmHash.toString()}`)
  }

  get contract() {
    return this.#contract
  }
}

export class PinkBlueprintPromise {

  readonly abi: Abi;
  readonly api: ApiBase<'promise'>;
  readonly phatRegistry: OnChainRegistry;

  protected readonly _decorateMethod: DecorateMethod<'promise'>;

  /**
   * @description The on-chain code hash for this blueprint
   */
  readonly codeHash: Hash;

  readonly #query: MapMessageInkQuery<'promise'> = {};
  readonly #tx: MapConstructorExec<'promise'> = {};

  constructor (api: ApiBase<'promise'>, phatRegistry: OnChainRegistry, abi: AbiLike, codeHash: string | Hash | Uint8Array) {
    if (!api || !api.isConnected || !api.tx) {
      throw new Error('Your API has not been initialized correctly and is not connected to a chain');
    }
    if (!phatRegistry.isReady()) {
      throw new Error('Your phatRegistry has not been initialized correctly.');
    }

    this.abi = abi instanceof Abi
      ? abi
      : new Abi(abi, api.registry.getChainProperties());
    this.api = api;
    this._decorateMethod = toPromiseMethod;
    this.phatRegistry = phatRegistry

    this.codeHash = this.api.registry.createType('Hash', codeHash);

    this.abi.constructors.forEach((c): void => {
      if (isUndefined(this.#tx[c.method])) {
        this.#tx[c.method] = createBluePrintTx(c, (o, p) => this.#deploy(c, o, p));
        this.#query[c.method] = createQuery(c, (f, o, p) => this.#estimateGas(c, o, p).send(f));
      }
    });
  }

  public get tx (): MapConstructorExec<'promise'> {
    return this.#tx;
  }

  public get query (): MapMessageInkQuery<'promise'> {
    return this.#query;
  }

  #deploy = (
    constructorOrId: AbiConstructor | string | number,
    { gasLimit = BN_ZERO, storageDepositLimit = null, value = BN_ZERO, salt }: BlueprintOptions,
    params: unknown[]
  ) => {
    if (!salt) {
      salt = hex(crypto.randomBytes(4))
    }
    const codeHash = this.abi.info.source.wasmHash.toString()
    return this.api.tx.phalaPhatContracts.instantiateContract(
      { 'WasmCode': codeHash },
      this.abi.findConstructor(constructorOrId).toU8a(params),
      salt,
      this.phatRegistry.clusterId,
      0,  // not transfer any token to the contract during initialization
      gasLimit,
      storageDepositLimit,
      0
    ).withResultTransform((result: ISubmittableResult) => {
      let maybeContactId: string | undefined
      const instantiateEvent = result.events.filter(i => i.event.method === 'Instantiating')[0]
      if (instantiateEvent) {
        const contractId = (instantiateEvent.event.data as any).contract
        if (contractId) {
          maybeContactId = contractId.toString()
        }
      }
      return new PinkBlueprintSubmittableResult(result, this.abi, this.phatRegistry, maybeContactId)
    });
  };

  #estimateGas = (
    constructorOrId: AbiConstructor | string | number,
    options: PinkContractQueryOptions,
    params: unknown[]
  ) => {
    const api = this.api as ApiPromise

    // Generate a keypair for encryption
    // NOTE: each instance only has a pre-generated pair now, it maybe better to generate a new keypair every time encrypting
    const seed = hexToU8a(hexAddPrefix(randomHex(32)));
    const pair = sr25519KeypairFromSeed(seed);
    const [sk, pk] = [pair.slice(0, 64), pair.slice(64)];
    const { cert } = options

    const queryAgreementKey = sr25519Agree(
      hexToU8a(hexAddPrefix(this.phatRegistry.remotePubkey)),
      sk
    );

    const inkQueryInternal = async (origin: string | AccountId | Uint8Array) => {
      if (typeof origin === 'string') {
        assert(origin === cert.address, 'origin must be the same as the certificate address')
      } else if (origin.hasOwnProperty('verify') && origin.hasOwnProperty('adddress')) {
        throw new Error('Contract query expected AccountId as first parameter but since we got signer object here.')
      } else {
        assert(origin.toString() === cert.address, 'origin must be the same as the certificate address')
      }
      const salt = hex(crypto.randomBytes(4))
      const payload = api.createType("InkQuery", {
        head: {
          nonce: hexAddPrefix(randomHex(32)),
          id: this.phatRegistry.clusterInfo?.systemContract!,
        },
        data: {
          InkInstantiate: {
            codeHash: this.abi.info.source.wasmHash,
            salt,
            instantiateData: this.abi.findConstructor(constructorOrId).toU8a(params),
            deposit: 0,
            transfer: 0,
          },
        },
      });
      const rawResponse = await pinkQuery(api, this.phatRegistry.phactory, pk, queryAgreementKey, payload.toHex(), cert);
      const response = api.createType<InkResponse>("InkResponse", rawResponse);
      if (response.result.isErr) {
        return api.createType<InkQueryError>(
          "InkQueryError",
          response.result.asErr.toHex()
        )
      }
      const result = api.createType<ContractInstantiateResult>(
        "ContractInstantiateResult",
        response.result.asOk.asInkMessageReturn.toHex()
      );
      (result as PinkContractInstantiateResult).salt = salt;
      return result;
    }

    return {
      send: this._decorateMethod((origin: string | AccountId | Uint8Array) => from(inkQueryInternal(origin)))
    };
  };
}

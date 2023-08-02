import type { Bytes } from '@polkadot/types';
import type { Codec, IEnum, Registry, ISubmittableResult  } from '@polkadot/types/types';
import type { SubmittableExtrinsic } from '@polkadot/api/submittable/types';
import type { AccountId, ContractExecResult, EventRecord } from '@polkadot/types/interfaces';
import type { ApiPromise } from '@polkadot/api';
import type { ApiBase } from '@polkadot/api/base';
import type { AbiMessage, ContractOptions, ContractCallOutcome, DecodedEvent } from '@polkadot/api-contract/types';
import type { ContractCallResult, ContractCallSend, MessageMeta, ContractTx, MapMessageTx } from '@polkadot/api-contract/base/types';
import type { DecorateMethod, ApiTypes } from '@polkadot/api/types';

import type { OnChainRegistry } from '../OnChainRegistry';
import type { AbiLike } from '../types';
import type { CertificateData } from '../certificate';

import { Abi } from '@polkadot/api-contract/Abi';
import { toPromiseMethod } from '@polkadot/api';
import { ContractSubmittableResult } from '@polkadot/api-contract/base/Contract';
import { applyOnEvent } from '@polkadot/api-contract/util';
import { withMeta, convertWeight } from '@polkadot/api-contract/base/util'
import { BN, BN_ZERO, hexAddPrefix, u8aToHex, hexToU8a } from '@polkadot/util';
import { sr25519Agree, sr25519KeypairFromSeed, sr25519Sign } from "@polkadot/wasm-crypto";
import { from } from 'rxjs';

import { pruntime_rpc as pruntimeRpc } from "../proto";
import { decrypt, encrypt } from "../lib/aes-256-gcm";
import { randomHex } from "../lib/hex";
import assert from '../lib/assert';


export type PinkContractCallOutcome<ResultType> = {
  output: ResultType
} & Omit<ContractCallOutcome, 'output'>;

export interface ILooseResult<O, E extends Codec = Codec> extends IEnum {
    readonly asErr: E;
    readonly asOk: O;
    readonly isErr: boolean;
    readonly isOk: boolean;
}

export interface ContractInkQuery<ApiType extends ApiTypes> extends MessageMeta {
  <ResultType = Codec, ErrType extends Codec = Codec>(origin: string | AccountId | Uint8Array, ...params: unknown[]): ContractCallResult<
    ApiType, PinkContractCallOutcome<ILooseResult<ResultType, ErrType>>
  >;
}

export interface MapMessageInkQuery<ApiType extends ApiTypes> {
  [message: string]: ContractInkQuery<ApiType>;
}

export interface PinkContractQueryOptions {
  cert: CertificateData
}

class PinkContractSubmittableResult extends ContractSubmittableResult {

  readonly #registry: OnChainRegistry

  #isFinalized: boolean = false

  constructor(registry: OnChainRegistry, result: ISubmittableResult, contractEvents?: DecodedEvent[]) {
    super(result, contractEvents)
    this.#registry = registry
  }

  async waitFinalized(timeout: number = 120_000) {
    if (this.#isFinalized) {
      return
    }

    if (this.isInBlock || this.isFinalized) {
      const codeHash = this.status.asInBlock.toString()
      const block = await this.#registry.api.rpc.chain.getBlock(codeHash)
      const chainHeight = block.block.header.number.toNumber()

      const t0 = new Date().getTime();
      while (true) {
        const result = await this.#registry.phactory.getInfo({})
        if (result.blocknum > chainHeight) {
          this.#isFinalized = true
          return
        }

        const t1 = new Date().getTime();
        if (t1 - t0 > timeout) {
          throw new Error('Timeout')
        }
        await new Promise(resolve => setTimeout(resolve, 1000));
      }
    }
    throw new Error('Contract transaction submit failed.')
  }
}


function createQuery(meta: AbiMessage, fn: (origin: string | AccountId | Uint8Array, options: PinkContractQueryOptions, params: unknown[]) => ContractCallResult<'promise', ContractCallOutcome>): ContractInkQuery<'promise'> {
  return withMeta(meta, (origin: string | AccountId | Uint8Array, options: PinkContractQueryOptions, ...params: unknown[]): ContractCallResult<'promise', ContractCallOutcome> =>
    fn(origin, options, params)
  );
}

function createTx(meta: AbiMessage, fn: (options: ContractOptions, params: unknown[]) => SubmittableExtrinsic<'promise'>): ContractTx<'promise'> {
  return withMeta(meta, (options: ContractOptions, ...params: unknown[]): SubmittableExtrinsic<'promise'> =>
    fn(options, params)
  );
}

function createEncryptedData(pk: Uint8Array, data: string, agreementKey: Uint8Array) {
  const iv = hexAddPrefix(randomHex(12));
  return {
    iv,
    pubkey: u8aToHex(pk),
    data: hexAddPrefix(encrypt(data, agreementKey, hexToU8a(iv))),
  };
};

export async function pinkQuery(
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

  const res = await pruntimeApi.contractQuery(requestData)

  const { data: encryptedResult, iv } = api.createType("EncryptedData", res.encodedEncryptedData).toJSON() as {
    iv: string;
    data: string;
  };
  const data = decrypt(encryptedResult, queryAgreementKey, iv);
  return hexAddPrefix(data);
};


export class PinkContractPromise {

  readonly abi: Abi;
  readonly api: ApiBase<'promise'>;
  readonly address: AccountId;
  readonly contractKey: string;
  readonly phatRegistry: OnChainRegistry;

  protected readonly _decorateMethod: DecorateMethod<'promise'>;

  readonly #query: MapMessageInkQuery<'promise'> = {};
  readonly #tx: MapMessageTx<'promise'> = {};

  constructor (api: ApiBase<'promise'>, phatRegistry: OnChainRegistry, abi: AbiLike, address: string | AccountId, contractKey: string) {
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

    this.address = this.registry.createType('AccountId', address);
    this.contractKey = contractKey

    this.abi.messages.forEach((m): void => {
      if (m.isMutating) {
        this.#tx[m.method] = createTx(m, (o, p) => this.#inkCommand(m, o, p));
        this.#query[m.method] = createQuery(m, (f, c, p) => this.#inkQuery(true, m, c, p).send(f));
      } else {
        this.#query[m.method] = createQuery(m, (f, c, p) => this.#inkQuery(false, m, c, p).send(f));
      }
    });
  }

  public get registry (): Registry {
    return this.api.registry;
  }

  public get query (): MapMessageInkQuery<'promise'> {
    return this.#query;
  }

  public get tx (): MapMessageTx<'promise'> {
    return this.#tx;
  }

  #inkQuery = (isEstimating: boolean, messageOrId: AbiMessage | string | number, options: PinkContractQueryOptions, params: unknown[]): ContractCallSend<'promise'> => {
    const message = this.abi.findMessage(messageOrId);
    const api = this.api as ApiPromise

    const { cert } = options

    // Generate a keypair for encryption
    // NOTE: each instance only has a pre-generated pair now, it maybe better to generate a new keypair every time encrypting
    const seed = hexToU8a(hexAddPrefix(randomHex(32)));
    const pair = sr25519KeypairFromSeed(seed);
    const [sk, pk] = [pair.slice(0, 64), pair.slice(64)];

    const queryAgreementKey = sr25519Agree(
      hexToU8a(hexAddPrefix(this.phatRegistry.remotePubkey)),
      sk
    );

    const inkQueryInternal = async (origin: string | AccountId | Uint8Array): Promise<ContractCallOutcome> => {

      if (typeof origin === 'string') {
        assert(origin === cert.address, 'origin must be the same as the certificate address')
      } else if (origin.hasOwnProperty('verify') && origin.hasOwnProperty('adddress')) {
        throw new Error('Contract query expected AccountId as first parameter but since we got signer object here.')
      } else {
        assert(origin.toString() === cert.address, 'origin must be the same as the certificate address')
      }

      const payload = api.createType("InkQuery", {
        head: {
          nonce: hexAddPrefix(randomHex(32)),
          id: this.address,
        },
        data: {
          InkMessage: {
            payload: message.toU8a(params),
            deposit: 0,
            transfer: null,
            estimating: isEstimating,
          }
        },
      });
      const data = await pinkQuery(api, this.phatRegistry.phactory, pk, queryAgreementKey, payload.toHex(), cert);
      const { debugMessage, gasConsumed, gasRequired, result, storageDeposit } = api.createType<ContractExecResult>(
        "ContractExecResult",
        (
          api.createType("InkResponse", data).toJSON() as {
            result: { ok: { inkMessageReturn: string } };
          }
        ).result.ok.inkMessageReturn
      );
      return {
        debugMessage: debugMessage,
        gasConsumed: gasConsumed,
        gasRequired: gasRequired && !convertWeight(gasRequired).v1Weight.isZero() ? gasRequired : gasConsumed,
        output: result.isOk && message.returnType
          ? this.abi.registry.createTypeUnsafe(message.returnType.lookupName || message.returnType.type, [result.asOk.data.toU8a(true)], { isPedantic: true })
          : null,
        result,
        storageDeposit
      }
    }

    return {
      send: this._decorateMethod((origin: string | AccountId | Uint8Array) => from(inkQueryInternal(origin)))
    };
  };

  #inkCommand = (messageOrId: AbiMessage | string | number, { gasLimit = BN_ZERO, storageDepositLimit = null, value = BN_ZERO }: ContractOptions, params: unknown[]): SubmittableExtrinsic<'promise'> => {
    const api = this.api as ApiPromise

    // Generate a keypair for encryption
    // NOTE: each instance only has a pre-generated pair now, it maybe better to generate a new keypair every time encrypting
    const seed = hexToU8a(hexAddPrefix(randomHex(32)));
    const pair = sr25519KeypairFromSeed(seed);
    const [sk, pk] = [pair.slice(0, 64), pair.slice(64)];

    const commandAgreementKey = sr25519Agree(hexToU8a(this.contractKey), sk);

    const inkCommandInternal = (dest: AccountId, value: BN, gas: { refTime: BN }, storageDepositLimit: BN | undefined, encParams: Uint8Array) => {
      // @ts-ignore
      const payload = api.createType("InkCommand", {
        InkMessage: {
          nonce: hexAddPrefix(randomHex(32)),
          // FIXME: unexpected u8a prefix
          message: api.createType("Vec<u8>", encParams).toHex(),
          transfer: value,
          gasLimit: gas.refTime,
          storageDepositLimit,
        },
      });
      const encodedPayload = api
        .createType("CommandPayload", {
          encrypted: createEncryptedData(pk, payload.toHex(), commandAgreementKey),
        })
        .toHex();
      let deposit = new BN(0);
      const gasFee = new BN(gas.refTime).mul(this.phatRegistry.gasPrice);
      deposit = new BN(value).add(gasFee).add(new BN(storageDepositLimit || 0));

      return api.tx.phalaPhatContracts.pushContractMessage(
        dest,
        encodedPayload,
        deposit
      );
    }

    return inkCommandInternal(
      this.address,
      // @ts-ignore
      value,
      convertWeight(gasLimit).v2Weight,
      storageDepositLimit,
      this.abi.findMessage(messageOrId).toU8a(params)
    ).withResultTransform((result: ISubmittableResult) => {
      return new PinkContractSubmittableResult(
        this.phatRegistry,
        result,
        applyOnEvent(result, ['ContractEmitted', 'ContractExecution'], (records: EventRecord[]) => {
          return records
            .map(({ event: { data: [, data] } }): DecodedEvent | null => {
              try {
                return this.abi.decodeEvent(data as Bytes);
              } catch (error) {
                console.error(`Unable to decode contract event: ${(error as Error).message}`);
                return null;
              }
            })
            .filter((decoded): decoded is DecodedEvent => !!decoded)
        })
      )
    });
  };
}

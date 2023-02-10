import type { SubmittableExtrinsic } from '@polkadot/api/submittable/types';
import type { AccountId, ContractExecResult, EventRecord, Weight, WeightV2 } from '@polkadot/types/interfaces';
import type { ApiPromise } from '@polkadot/api';
import type { AbiMessage, ContractOptions, ContractCallOutcome } from '@polkadot/api-contract/types';
import type { ContractCallResult, ContractCallSend, ContractQuery, ContractTx, MapMessageQuery, MapMessageTx } from '@polkadot/api-contract/base/types';
import type { Registry } from '@polkadot/types/types';
import type { ApiTypes, DecorateMethod } from '@polkadot/api/types';

import { pruntime_rpc as pruntimeRpc, pruntime_rpc } from "../proto";

import { Abi } from '@polkadot/api-contract/Abi';
import { toPromiseMethod } from '@polkadot/api';
import { ApiBase } from '@polkadot/api/base';
import { Base } from '@polkadot/api-contract/base/Base'
import { withMeta, convertWeight } from '@polkadot/api-contract/base/util'
import { BN, BN_HUNDRED, BN_ONE, BN_ZERO, isUndefined, logger, hexAddPrefix, u8aToHex, hexToU8a } from '@polkadot/util';
import {
  sr25519Agree,
  sr25519KeypairFromSeed,
  sr25519Sign,
  waitReady,
} from "@polkadot/wasm-crypto";
import { from } from 'rxjs';

import { signCertificate, CertificateData } from '../certificate';
import { decrypt, encrypt } from "../lib/aes-256-gcm";
import { randomHex } from "../lib/hex";

function createQuery(meta: AbiMessage, fn: (origin: string | AccountId | Uint8Array, params: unknown[]) => ContractCallResult<'promise', ContractCallOutcome>): ContractQuery<'promise'> {
  return withMeta(meta, (origin: string | AccountId | Uint8Array, ...params: unknown[]): ContractCallResult<'promise', ContractCallOutcome> =>
    fn(origin, params)
  );
}

function createTx(meta: AbiMessage, fn: (options: ContractOptions, params: unknown[]) => SubmittableExtrinsic<'promise'>): ContractTx<'promise'> {
  return withMeta(meta, (options: ContractOptions, ...params: unknown[]): SubmittableExtrinsic<'promise'> =>
    fn(options, params)
  );
}

// const createEncryptedData: CreateEncryptedData = (data, agreementKey) => {
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
  pruntimeApi: pruntime_rpc.PhactoryAPI,
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
    const { data: encryptedData, iv } = api
      .createType("EncryptedData", encodedEncryptedData)
      .toJSON() as {
      iv: string;
      data: string;
    };
    const data = decrypt(encryptedData, queryAgreementKey, iv);
    return hexAddPrefix(data);
  });
};


export class PinkContractPromise {

  readonly abi: Abi;
  readonly api: ApiBase<'promise'>;
  readonly address: AccountId;

  protected readonly _decorateMethod: DecorateMethod<'promise'>;
  protected readonly _isWeightV1: boolean;

  readonly #query: MapMessageQuery<'promise'> = {};
  readonly #tx: MapMessageTx<'promise'> = {};

  readonly #phactory: pruntimeRpc.PhactoryAPI;
  readonly #remotePubkey: string;

  constructor (api: ApiPromise, phactory: pruntimeRpc.PhactoryAPI, remotePubkey: string, abi: string | Record<string, unknown> | Abi, address: string | AccountId) {
    if (!api || !api.isConnected || !api.tx) {
      throw new Error('Your API has not been initialized correctly and is not connected to a chain');
    }

    this.abi = abi instanceof Abi
      ? abi
      : new Abi(abi, api.registry.getChainProperties());
    this.api = api;
    this._decorateMethod = toPromiseMethod;
    this._isWeightV1 = !api.registry.createType<WeightV2>('Weight').proofSize;

    this.address = this.registry.createType('AccountId', address);

    this.#phactory = phactory;
    this.#remotePubkey = remotePubkey;

    this.abi.messages.forEach((m): void => {
      if (m.isMutating) {
        // @ts-ignore
        this.#tx[m.method] = createTx(m, () => {})
      } else {
        this.#query[m.method] = createQuery(m, (f, p) => this.#inkQuery(m, p).send(f));
      }
    });
  }

  public get registry (): Registry {
    return this.api.registry;
  }

  public get query (): MapMessageQuery<'promise'> {
    return this.#query;
  }

  public get tx (): MapMessageTx<'promise'> {
    return this.#tx;
  }

  #inkQuery = (messageOrId: AbiMessage | string | number, params: unknown[]): ContractCallSend<'promise'> => {
    const message = this.abi.findMessage(messageOrId);
    const api = this.api as ApiPromise

    // Generate a keypair for encryption
    // NOTE: each instance only has a pre-generated pair now, it maybe better to generate a new keypair every time encrypting
    const seed = hexToU8a(hexAddPrefix(randomHex(32)));
    const pair = sr25519KeypairFromSeed(seed);
    const [sk, pk] = [pair.slice(0, 64), pair.slice(64)];

    const queryAgreementKey = sr25519Agree(
      hexToU8a(hexAddPrefix(this.#remotePubkey)),
      sk
    );

    const asyncMethod = async (origin: string | AccountId | Uint8Array): Promise<ContractCallOutcome> => {
      // @ts-ignore
      const cert = await signCertificate({ pair: origin, api });
      const query = api.createType("InkQuery", {
        head: {
          nonce: hexAddPrefix(randomHex(32)),
          id: this.address,
        },
        data: {
          InkMessage: message.toU8a(params),
        },
      });
      const data = await pinkQuery(api, this.#phactory, pk, queryAgreementKey, query.toHex(), cert);
      const { debugMessage, gasConsumed, gasRequired, result, storageDeposit } = api.createType<ContractExecResult>(
        "ContractExecResult",
        (
          api.createType("InkResponse", hexAddPrefix(data)).toJSON() as {
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
      send: this._decorateMethod((origin: string | AccountId | Uint8Array) => from(asyncMethod(origin)))
    };
  };
}

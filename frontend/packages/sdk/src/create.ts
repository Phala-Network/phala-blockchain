import type { ApiPromise } from "@polkadot/api";
import type { SubmittableExtrinsic } from "@polkadot/api/types";
import type { Bytes, Compact, Option, Result, u8, u64, Vec } from "@polkadot/types-codec";
import type { IMap, IEnum } from '@polkadot/types-codec/types';
import { AccountId, ContractInstantiateResult } from "@polkadot/types/interfaces";
import type { Codec } from "@polkadot/types/types";
import {
  BN,
  hexAddPrefix,
  hexStripPrefix,
  hexToU8a,
  stringToHex,
  u8aToHex,
} from "@polkadot/util";
import {
  sr25519Agree,
  sr25519KeypairFromSeed,
  sr25519Sign,
  waitReady,
} from "@polkadot/wasm-crypto";
import { from } from "rxjs";
import { fetch } from "undici";
import type { CertificateData } from "./certificate";
import { decrypt, encrypt } from "./lib/aes-256-gcm";
import { randomHex } from "./lib/hex";
import { prpc, pruntime_rpc as pruntimeRpc } from "./proto";

export type QueryFn = (
  encodedQuery: string,
  certificateData: CertificateData
) => Promise<string>;

export type SidevmQuery<T = string> = (
  bytes: Bytes,
  certificateData: CertificateData
) => Promise<T>;

type EncryptedData = {
  iv: string;
  pubkey: string;
  data: string;
};

type CreateEncryptedData = (
  data: string,
  agreementKey: Uint8Array
) => EncryptedData;

export type CommandFn = (params: {
  contractId: string;
  payload: string;
  deposit: BN;
}) => SubmittableExtrinsic<"promise">;

export interface PhalaInstance {
  query: QueryFn;
  command: CommandFn;
}

export interface ContractExecResultWeightV2 extends Codec {
  gasConsumedV2?: {
    refTime: Compact<u64>;
    proofSize: Compact<u64>;
  };
  gasConsumed?: u64;
  gasRequiredV2?: {
    refTime: Compact<u64>;
    proofSize: Compact<u64>;
  };
  gasRequired?: u64;
}

export interface CreateFnOptions {
  api: ApiPromise;
  baseURL: string;
  contractId: string;
  remotePubkey?: string;
  autoDeposit?: boolean;
}

export interface CreateFnResult {
  api: ApiPromise;
  sidevmQuery: SidevmQuery;
  instantiate: SidevmQuery<ContractInstantiateResult | InkQueryError>;
}

export interface ContractInfo {
  cluster: string;
  codeIndex: {
    wasmCode: string;
  };
  deployer: AccountId;
  pubkey: string;
}

export interface ClusterInfo {
  owner: AccountId;
  // @fixme
  permission: "Public" | string;
  systemContract?: string;
  workers: string[];
  gasPrice: BN;
}

export interface InkQueryOk extends IEnum {
  asInkMessageReturn: Vec<u8>
}

export interface InkQueryError extends IEnum {
  BadOrigin: null
  RuntimeError: string
}

export interface InkResponse extends IMap {
  nonce: Vec<u8>;
  result: Result<InkQueryOk, InkQueryError>;
}

export const createPruntimeApi = (baseURL: string) => {
  // Create a http client prepared for protobuf
  const pruntimeApi = pruntimeRpc.PhactoryAPI.create(
    async (method, requestData, callback) => {
      try {
        const resp = await fetch(
          `${baseURL}/prpc/PhactoryAPI.${method.name}`,
          {
            method: 'POST',
            headers:{
              "Content-Type": "application/octet-stream",
            },
            body: new Uint8Array(requestData),
          }
        )
        const buffer = await (await resp.blob()).arrayBuffer()
        callback(null, new Uint8Array(buffer));
      } catch (err) {
        // todo fixme
        console.log('Error:', err)
        // if (
        //   err instanceof AxiosError &&
        //   err.response?.data instanceof ArrayBuffer
        // ) {
        //   const message = new Uint8Array(err.response.data);
        //   callback(new Error(prpc.PrpcError.decode(message).message));
        // } else {
        //   throw err;
        // }
        throw err;
      }
    }
  );

  return pruntimeApi;
};

export async function create({
  api,
  baseURL,
  contractId,
  remotePubkey,
  autoDeposit = false,
}: CreateFnOptions): Promise<CreateFnResult> {
  await waitReady();

  const pruntimeApi = createPruntimeApi(baseURL);

  if (!remotePubkey) {
    // Get public key from remote for encrypting
    const info = await pruntimeApi.getInfo({});
    if (!info || !info.publicKey) throw new Error("No remote pubkey");
    remotePubkey = hexAddPrefix(info.publicKey);
  }

  // Generate a keypair for encryption
  // NOTE: each instance only has a pre-generated pair now, it maybe better to generate a new keypair every time encrypting
  const seed = hexToU8a(hexAddPrefix(randomHex(32)));
  const pair = sr25519KeypairFromSeed(seed);
  const [sk, pk] = [pair.slice(0, 64), pair.slice(64)];

  const queryAgreementKey = sr25519Agree(
    hexToU8a(hexAddPrefix(remotePubkey)),
    sk
  );
  const contractKey = (
    await api.query.phalaRegistry.contractKeys(contractId)
  ).toString();

  if (!contractKey) {
    throw new Error(`No contract key for ${contractId}`);
  }

  const commandAgreementKey = sr25519Agree(hexToU8a(contractKey), sk);

  const createEncryptedData: CreateEncryptedData = (data, agreementKey) => {
    const iv = hexAddPrefix(randomHex(12));
    return {
      iv,
      pubkey: u8aToHex(pk),
      data: hexAddPrefix(encrypt(data, agreementKey, hexToU8a(iv))),
    };
  };

  let gasPrice = new BN(0);
  if (autoDeposit) {
    const contractInfo = (await api.query.phalaPhatContracts.contracts(
      contractId
    )) as Option<Codec>;
    const cluster = (contractInfo.unwrap() as unknown as ContractInfo).cluster;
    const clusterInfo = (await api.query.phalaPhatContracts.clusters(
      cluster
    )) as Option<Codec>;
    gasPrice = new BN(
      (clusterInfo.unwrap() as unknown as ClusterInfo).gasPrice
    );
  }

  const query: QueryFn = async (
    encodedQuery,
    { certificate, pubkey, secret }
  ) => {
    // Encrypt the ContractQuery.
    const encryptedData = createEncryptedData(encodedQuery, queryAgreementKey);
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

  const sidevmQuery: SidevmQuery = async (bytes, certificateData) =>
    query(
      api
        .createType("InkQuery", {
          head: {
            nonce: hexAddPrefix(randomHex(32)),
            id: contractId,
          },
          data: {
            SidevmMessage: bytes,
          },
        })
        .toHex(),
      certificateData
    );

  const instantiate: SidevmQuery<ContractInstantiateResult | InkQueryError> = async (payload, certificateData) => {
    const instantiateReturns = await query(
      api
        .createType("InkQuery", {
          head: {
            nonce: hexAddPrefix(randomHex(32)),
            id: contractId,
          },
          data: {
            InkInstantiate: payload,
          },
        })
        .toHex(),
      certificateData
    );
    const response = api.createType<InkResponse>('InkResponse', instantiateReturns);
    if (response.result.isErr) {
      // @FIXME not yet check the branch
      return api.createType<InkQueryError>('InkQueryError', response.result.asErr.toHex())
    }
    return api.createType<ContractInstantiateResult>(
      'ContractInstantiateResult',
      response.result.asOk.asInkMessageReturn.toHex()
    );
  };

  const command: CommandFn = ({ contractId, payload, deposit }) => {
    const encodedPayload = api
      .createType("CommandPayload", {
        encrypted: createEncryptedData(payload, commandAgreementKey),
      })
      .toHex();

    try {
      return api.tx.phalaPhatContracts.pushContractMessage(
        contractId,
        encodedPayload,
        deposit
      );
    } catch (err) {
      return api.tx.phalaMq.pushMessage(
        stringToHex(`phala/contract/${hexStripPrefix(contractId)}/command`),
        encodedPayload
      );
    }
  };

  const txContracts = (
    dest: AccountId,
    value: BN,
    gas: { refTime: BN },
    storageDepositLimit: BN | undefined,
    encParams: Uint8Array
  ) => {
    let deposit = new BN(0);
    if (autoDeposit) {
      const gasFee = new BN(gas.refTime).mul(gasPrice);
      deposit = new BN(value).add(gasFee).add(new BN(storageDepositLimit || 0));
    }
    return command({
      contractId: dest.toHex(),
      payload: api
        .createType("InkCommand", {
          InkMessage: {
            nonce: hexAddPrefix(randomHex(32)),
            // FIXME: unexpected u8a prefix
            message: api.createType("Vec<u8>", encParams).toHex(),
            transfer: value,
            gasLimit: gas.refTime,
            storageDepositLimit,
          },
        })
        .toHex(),
      deposit,
    });
  };

  Object.defineProperty(txContracts, "meta", {
    value: { args: [] },
    enumerable: true,
  });

  const instantiateWithCode = () => null;
  instantiateWithCode.meta = { args: new Array(6) };

  Object.defineProperty(api.tx, "contracts", {
    value: {
      instantiateWithCode,
      call: txContracts,
    },
    enumerable: true,
  });

  Object.defineProperty(api.rx.call, "contractsApi", {
    value: {
      call: (
        origin: CertificateData,
        dest: AccountId,
        value: unknown,
        gasLimit: unknown,
        storageDepositLimit: unknown,
        inputData: Bytes
      ) => {
        return from(
          query(
            api
              .createType("InkQuery", {
                head: {
                  nonce: hexAddPrefix(randomHex(32)),
                  id: dest,
                },
                data: {
                  InkMessage: {
                    payload: inputData,
                    deposit: 0,
                    transfer: value,
                    estimating: false,
                  },
                },
              })
              .toHex(),
            origin
          ).then((data) => {
            return api.createType(
              "ContractExecResult",
              (
                api.createType("InkResponse", hexAddPrefix(data)).toJSON() as {
                  result: { ok: { inkMessageReturn: string } };
                }
              ).result.ok.inkMessageReturn
            );
          })
        );
      },
    },
    enumerable: true,
  });

  Object.defineProperty(api.call, "contractsApi", {
    value: { call: () => null },
    enumerable: true,
  });

  return { api, sidevmQuery, instantiate };
}

export async function ready() {
  await waitReady();
}

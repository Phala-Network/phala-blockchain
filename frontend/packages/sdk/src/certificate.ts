import type { ApiPromise } from "@polkadot/api";
import type { Signer as InjectedSigner } from "@polkadot/api/types";
import type { InjectedAccountWithMeta } from "@polkadot/extension-inject/types";
import type { KeyringPair } from "@polkadot/keyring/types";
import type { Signer } from "@polkadot/types/types";

import { hexAddPrefix, hexToU8a, u8aToHex } from "@polkadot/util";
import { decodeAddress } from "@polkadot/util-crypto";
import { sr25519KeypairFromSeed, waitReady } from "@polkadot/wasm-crypto";
import { randomHex } from "./lib/hex";
import { pruntime_rpc as pruntimeRpc } from "./proto";

export type CertificateData = {
  certificate: pruntimeRpc.ICertificate;
  pubkey: Uint8Array;
  secret: Uint8Array;
};

interface CertificateBaseParams {
  api: ApiPromise;
  signatureType?: pruntimeRpc.SignatureType;
}

interface CertificateParamsWithSigner extends CertificateBaseParams {
  signer: Signer | InjectedSigner;
  account: InjectedAccountWithMeta;
}

interface CertificateParamsWithPair extends CertificateBaseParams {
  pair: KeyringPair;
}

type CertificateParams =
  | CertificateParamsWithSigner
  | CertificateParamsWithPair;

const isUsingSigner = (
  params: CertificateParams
): params is CertificateParamsWithSigner =>
  (params as CertificateParamsWithSigner).signer !== undefined;

export const signCertificate = async (
  params: CertificateParams
): Promise<CertificateData> => {
  await waitReady();
  const { api } = params;
  const generatedSeed = hexToU8a(hexAddPrefix(randomHex(32)));
  const generatedPair = sr25519KeypairFromSeed(generatedSeed);
  const [secret, pubkey] = [
    generatedPair.slice(0, 64),
    generatedPair.slice(64),
  ];

  const encodedCertificateBody = api
    .createType("CertificateBody", {
      pubkey: u8aToHex(pubkey),
      ttl: 0x7fffffff, // FIXME: max ttl is not safe
      config_bits: 0,
    })
    .toU8a();

  let signerPubkey: string;
  let signatureType = params.signatureType;
  let signature: Uint8Array;
  if (isUsingSigner(params)) {
    const { account, signer } = params;
    const address = account.address;
    signerPubkey = u8aToHex(decodeAddress(address));
    if (!signatureType) {
      signatureType = getSignatureTypeFromAccount(account);
    }
    const signerResult = await signer.signRaw?.({
      address,
      data: u8aToHex(encodedCertificateBody),
      type: "bytes",
    });
    if (signerResult) {
      signature = hexToU8a(signerResult.signature);
    } else {
      throw new Error("Failed to sign certificate");
    }
  } else {
    const { pair } = params;
    signerPubkey = u8aToHex(pair.publicKey);
    if (!signatureType) {
      signatureType = getSignatureTypeFromPair(pair);
    }
    signature = pair.sign(encodedCertificateBody);
  }

  const certificate: pruntimeRpc.ICertificate = {
    encodedBody: encodedCertificateBody,
    signature: {
      signedBy: {
        encodedBody: api
          .createType("CertificateBody", {
            pubkey: signerPubkey,
            ttl: 0x7fffffff, // FIXME: max ttl is not safe
            config_bits: 0,
          })
          .toU8a(),
        signature: null,
      },
      signatureType,
      signature,
    },
  };

  return {
    certificate,
    pubkey,
    secret,
  };
};

const getSignatureTypeFromAccount = (account: InjectedAccountWithMeta) => {
  const keypairType = account.type || "sr25519";
  switch (keypairType) {
    case "sr25519":
      return pruntimeRpc.SignatureType.Sr25519WrapBytes;
    case "ed25519":
      return pruntimeRpc.SignatureType.Ed25519WrapBytes;
    case "ecdsa":
      return pruntimeRpc.SignatureType.EcdsaWrapBytes;
  }
};

const getSignatureTypeFromPair = (pair: KeyringPair) => {
  switch (pair.type) {
    case "sr25519":
      return pruntimeRpc.SignatureType.Sr25519;
    case "ed25519":
      return pruntimeRpc.SignatureType.Ed25519;
    case "ecdsa":
      return pruntimeRpc.SignatureType.Ecdsa;
    default:
      throw new Error("Unsupported keypair type");
  }
};

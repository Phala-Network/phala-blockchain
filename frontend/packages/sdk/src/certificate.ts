import type { ApiPromise } from "@polkadot/api";
import type { Signer as InjectedSigner } from "@polkadot/api/types";
import type { KeyringPair } from "@polkadot/keyring/types";
import type { Signer } from "@polkadot/types/types";
import { type Client, type Account } from 'viem';

import { hexAddPrefix, hexToU8a, u8aToHex } from "@polkadot/util";
import { decodeAddress } from "@polkadot/util-crypto";
import { KeypairType } from "@polkadot/util-crypto/types";
import { sr25519KeypairFromSeed, waitReady } from "@polkadot/wasm-crypto";
import { TypeRegistry } from "@polkadot/types";
import { signTypedData } from 'viem/wallet';

import { randomHex } from "./lib/hex";
import { pruntime_rpc as pruntimeRpc } from "./proto";
import { phalaRegistryTypes } from './options';

interface InjectedAccount {
  address: string;
  genesisHash?: string | null;
  name?: string
  type?: KeypairType;
}

export type CertificateData = {
  address: string;
  certificate: pruntimeRpc.ICertificate;
  pubkey: Uint8Array;
  secret: Uint8Array;
};

interface CertificateBaseParams {
  api?: ApiPromise;
  signatureType?: pruntimeRpc.SignatureType;
  ttl?: number
}

interface CertificateParamsWithSigner extends CertificateBaseParams {
  signer: Signer | InjectedSigner;
  account: InjectedAccount;
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


const phalaTypes = new TypeRegistry()

phalaTypes.register(phalaRegistryTypes)


function generatePair(): [Uint8Array, Uint8Array] {
  const generatedSeed = hexToU8a(hexAddPrefix(randomHex(32)));
  const generatedPair = sr25519KeypairFromSeed(generatedSeed);
  return [
    generatedPair.slice(0, 64),
    generatedPair.slice(64),
  ];
}


function getSignatureTypeFromAccount(account: KeyringPair | InjectedAccount) {
  const keypairType = account.type || "sr25519";
  switch (keypairType) {
    case "sr25519":
      return pruntimeRpc.SignatureType.Sr25519WrapBytes;
    case "ed25519":
      return pruntimeRpc.SignatureType.Ed25519WrapBytes;
    case "ecdsa":
      return pruntimeRpc.SignatureType.EcdsaWrapBytes;
  }
}


function getSignatureTypeFromPair(pair: KeyringPair) {
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
}


function createEip712StructedDataQueryCertificate(finalValidBlock: number, encodedCert: string, account: Account, salt: `0x${string}`): Parameters<typeof signTypedData>[1] {
  return {
    domain: {
      name: "Phat Query Certificate",
      version: '1',
      salt,
    },
    message: {
      finalValidBlock,
      encodedCert,
    },
    primaryType: 'IssueQueryCertificate',
    types: {
      EIP712Domain: [
        { name: 'name', type: 'string' },
        { name: 'version', type: 'string' },
        { name: 'salt', type: 'bytes32' },
      ],
      IssueQueryCertificate: [
        { name: 'finalValidBlock', type: 'uint32' },
        { name: 'encodedCert', type: 'bytes' },
      ],
    },
    account,
  }
}


function CertificateBody(pubkey: string, ttl: number, config_bits: number = 0) {
  const created = phalaTypes.createType(
    "CertificateBody",
    { pubkey, ttl, config_bits }
  )
  return created.toU8a()
}


export async function signCertificate(params: CertificateParams): Promise<CertificateData> {
  await waitReady();
  if (params.api) {
    console.warn('signCertificate not longer need pass the ApiPromise as parameter, it will remove from type hint in the next.')
  }
  // FIXME: max ttl is not safe
  let { signatureType, ttl = 0x7fffffff } = params;
  const [secret, pubkey] = generatePair()
  const encodedCertificateBody = CertificateBody(u8aToHex(pubkey), ttl)

  let signerPubkey: string;
  let signature: Uint8Array;
  let address: string;
  if (isUsingSigner(params)) {
    const { account, signer } = params;
    address = account.address;
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
    address = pair.address;
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
        encodedBody: CertificateBody(signerPubkey, ttl),
        signature: null,
      },
      signatureType,
      signature,
    },
  };

  return { address, certificate, pubkey, secret };
}


export async function unstable_signEip712Certificate({ client, account, compactPubkey, ttl = 0x7fffffff }: { client: Client, account: Account, compactPubkey: string, ttl?: number }): Promise<CertificateData> {
  await waitReady();
  // const salt = `0x${randomHex(32)}` as `0x${string}`
  // console.log('salt', salt, salt.length)
  const salt = '0x0ea813d1592526d672ea2576d7a07914cef2ca301b35c5eed941f7c897512a00'
  const [secret, pubkey] = generatePair()
  const address = account.address || account
  const eip712Cert = CertificateBody(u8aToHex(pubkey), ttl)
  // It will pop up a window to ask for confirm, so it might failed.
  const signature = await signTypedData(client, createEip712StructedDataQueryCertificate(ttl, u8aToHex(eip712Cert), account, salt))
  const rootCert = CertificateBody(compactPubkey, ttl)
  const certificate: pruntimeRpc.ICertificate = {
    encodedBody: eip712Cert,
    signature: {
      signedBy: {
        encodedBody: rootCert,
        signature: null,
      },
      signatureType: pruntimeRpc.SignatureType.Eip712,
      signature: hexToU8a(signature),
    },
  };
  return { address, certificate, pubkey, secret } as const;
}

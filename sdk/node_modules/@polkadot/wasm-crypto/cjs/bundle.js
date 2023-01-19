"use strict";

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.blake2b = exports.bip39Validate = exports.bip39ToSeed = exports.bip39ToMiniSecret = exports.bip39ToEntropy = exports.bip39Generate = void 0;
Object.defineProperty(exports, "bridge", {
  enumerable: true,
  get: function () {
    return _init.bridge;
  }
});
exports.hmacSha512 = exports.hmacSha256 = exports.ed25519Verify = exports.ed25519Sign = exports.ed25519KeypairFromSeed = void 0;
exports.isReady = isReady;
exports.keccak512 = exports.keccak256 = void 0;
Object.defineProperty(exports, "packageInfo", {
  enumerable: true,
  get: function () {
    return _packageInfo.packageInfo;
  }
});
exports.vrfVerify = exports.vrfSign = exports.twox = exports.sr25519Verify = exports.sr25519Sign = exports.sr25519KeypairFromSeed = exports.sr25519DerivePublicSoft = exports.sr25519DeriveKeypairSoft = exports.sr25519DeriveKeypairHard = exports.sr25519Agree = exports.sha512 = exports.sha256 = exports.secp256k1Sign = exports.secp256k1Recover = exports.secp256k1FromSeed = exports.secp256k1Expand = exports.secp256k1Compress = exports.scrypt = exports.pbkdf2 = void 0;
exports.waitReady = waitReady;

var _init = require("./init");

var _packageInfo = require("./packageInfo");

// Copyright 2019-2022 @polkadot/wasm-crypto authors & contributors
// SPDX-License-Identifier: Apache-2.0

/**
 * @internal
 * @description
 * This create an extenal interface function from the signature, all the while checking
 * the actual bridge wasm interface to ensure it has been initialized.
 *
 * This means that we can call it
 *
 *   withWasm(wasm: WasmCryptoInstance, a: number, b: string) => Uint8Array
 *
 * and in this case it will create an interface function with the signarure
 *
 *   (a: number, b: string) => Uint8Array
 */
function withWasm(fn) {
  return function () {
    if (!_init.bridge.wasm) {
      throw new Error('The WASM interface has not been initialized. Ensure that you wait for the initialization Promise with waitReady() from @polkadot/wasm-crypto (or cryptoWaitReady() from @polkadot/util-crypto) before attempting to use WASM-only interfaces.');
    }

    for (var _len = arguments.length, params = new Array(_len), _key = 0; _key < _len; _key++) {
      params[_key] = arguments[_key];
    }

    return fn(_init.bridge.wasm, ...params);
  };
}

const bip39Generate = withWasm((wasm, words) => {
  wasm.ext_bip39_generate(8, words);
  return _init.bridge.resultString();
});
exports.bip39Generate = bip39Generate;
const bip39ToEntropy = withWasm((wasm, phrase) => {
  wasm.ext_bip39_to_entropy(8, ..._init.bridge.allocString(phrase));
  return _init.bridge.resultU8a();
});
exports.bip39ToEntropy = bip39ToEntropy;
const bip39ToMiniSecret = withWasm((wasm, phrase, password) => {
  wasm.ext_bip39_to_mini_secret(8, ..._init.bridge.allocString(phrase), ..._init.bridge.allocString(password));
  return _init.bridge.resultU8a();
});
exports.bip39ToMiniSecret = bip39ToMiniSecret;
const bip39ToSeed = withWasm((wasm, phrase, password) => {
  wasm.ext_bip39_to_seed(8, ..._init.bridge.allocString(phrase), ..._init.bridge.allocString(password));
  return _init.bridge.resultU8a();
});
exports.bip39ToSeed = bip39ToSeed;
const bip39Validate = withWasm((wasm, phrase) => {
  const ret = wasm.ext_bip39_validate(..._init.bridge.allocString(phrase));
  return ret !== 0;
});
exports.bip39Validate = bip39Validate;
const ed25519KeypairFromSeed = withWasm((wasm, seed) => {
  wasm.ext_ed_from_seed(8, ..._init.bridge.allocU8a(seed));
  return _init.bridge.resultU8a();
});
exports.ed25519KeypairFromSeed = ed25519KeypairFromSeed;
const ed25519Sign = withWasm((wasm, pubkey, seckey, message) => {
  wasm.ext_ed_sign(8, ..._init.bridge.allocU8a(pubkey), ..._init.bridge.allocU8a(seckey), ..._init.bridge.allocU8a(message));
  return _init.bridge.resultU8a();
});
exports.ed25519Sign = ed25519Sign;
const ed25519Verify = withWasm((wasm, signature, message, pubkey) => {
  const ret = wasm.ext_ed_verify(..._init.bridge.allocU8a(signature), ..._init.bridge.allocU8a(message), ..._init.bridge.allocU8a(pubkey));
  return ret !== 0;
});
exports.ed25519Verify = ed25519Verify;
const secp256k1FromSeed = withWasm((wasm, seckey) => {
  wasm.ext_secp_from_seed(8, ..._init.bridge.allocU8a(seckey));
  return _init.bridge.resultU8a();
});
exports.secp256k1FromSeed = secp256k1FromSeed;
const secp256k1Compress = withWasm((wasm, pubkey) => {
  wasm.ext_secp_pub_compress(8, ..._init.bridge.allocU8a(pubkey));
  return _init.bridge.resultU8a();
});
exports.secp256k1Compress = secp256k1Compress;
const secp256k1Expand = withWasm((wasm, pubkey) => {
  wasm.ext_secp_pub_expand(8, ..._init.bridge.allocU8a(pubkey));
  return _init.bridge.resultU8a();
});
exports.secp256k1Expand = secp256k1Expand;
const secp256k1Recover = withWasm((wasm, msgHash, sig, recovery) => {
  wasm.ext_secp_recover(8, ..._init.bridge.allocU8a(msgHash), ..._init.bridge.allocU8a(sig), recovery);
  return _init.bridge.resultU8a();
});
exports.secp256k1Recover = secp256k1Recover;
const secp256k1Sign = withWasm((wasm, msgHash, seckey) => {
  wasm.ext_secp_sign(8, ..._init.bridge.allocU8a(msgHash), ..._init.bridge.allocU8a(seckey));
  return _init.bridge.resultU8a();
});
exports.secp256k1Sign = secp256k1Sign;
const sr25519DeriveKeypairHard = withWasm((wasm, pair, cc) => {
  wasm.ext_sr_derive_keypair_hard(8, ..._init.bridge.allocU8a(pair), ..._init.bridge.allocU8a(cc));
  return _init.bridge.resultU8a();
});
exports.sr25519DeriveKeypairHard = sr25519DeriveKeypairHard;
const sr25519DeriveKeypairSoft = withWasm((wasm, pair, cc) => {
  wasm.ext_sr_derive_keypair_soft(8, ..._init.bridge.allocU8a(pair), ..._init.bridge.allocU8a(cc));
  return _init.bridge.resultU8a();
});
exports.sr25519DeriveKeypairSoft = sr25519DeriveKeypairSoft;
const sr25519DerivePublicSoft = withWasm((wasm, pubkey, cc) => {
  wasm.ext_sr_derive_public_soft(8, ..._init.bridge.allocU8a(pubkey), ..._init.bridge.allocU8a(cc));
  return _init.bridge.resultU8a();
});
exports.sr25519DerivePublicSoft = sr25519DerivePublicSoft;
const sr25519KeypairFromSeed = withWasm((wasm, seed) => {
  wasm.ext_sr_from_seed(8, ..._init.bridge.allocU8a(seed));
  return _init.bridge.resultU8a();
});
exports.sr25519KeypairFromSeed = sr25519KeypairFromSeed;
const sr25519Sign = withWasm((wasm, pubkey, secret, message) => {
  wasm.ext_sr_sign(8, ..._init.bridge.allocU8a(pubkey), ..._init.bridge.allocU8a(secret), ..._init.bridge.allocU8a(message));
  return _init.bridge.resultU8a();
});
exports.sr25519Sign = sr25519Sign;
const sr25519Verify = withWasm((wasm, signature, message, pubkey) => {
  const ret = wasm.ext_sr_verify(..._init.bridge.allocU8a(signature), ..._init.bridge.allocU8a(message), ..._init.bridge.allocU8a(pubkey));
  return ret !== 0;
});
exports.sr25519Verify = sr25519Verify;
const sr25519Agree = withWasm((wasm, pubkey, secret) => {
  wasm.ext_sr_agree(8, ..._init.bridge.allocU8a(pubkey), ..._init.bridge.allocU8a(secret));
  return _init.bridge.resultU8a();
});
exports.sr25519Agree = sr25519Agree;
const vrfSign = withWasm((wasm, secret, context, message, extra) => {
  wasm.ext_vrf_sign(8, ..._init.bridge.allocU8a(secret), ..._init.bridge.allocU8a(context), ..._init.bridge.allocU8a(message), ..._init.bridge.allocU8a(extra));
  return _init.bridge.resultU8a();
});
exports.vrfSign = vrfSign;
const vrfVerify = withWasm((wasm, pubkey, context, message, extra, outAndProof) => {
  const ret = wasm.ext_vrf_verify(..._init.bridge.allocU8a(pubkey), ..._init.bridge.allocU8a(context), ..._init.bridge.allocU8a(message), ..._init.bridge.allocU8a(extra), ..._init.bridge.allocU8a(outAndProof));
  return ret !== 0;
});
exports.vrfVerify = vrfVerify;
const blake2b = withWasm((wasm, data, key, size) => {
  wasm.ext_blake2b(8, ..._init.bridge.allocU8a(data), ..._init.bridge.allocU8a(key), size);
  return _init.bridge.resultU8a();
});
exports.blake2b = blake2b;
const hmacSha256 = withWasm((wasm, key, data) => {
  wasm.ext_hmac_sha256(8, ..._init.bridge.allocU8a(key), ..._init.bridge.allocU8a(data));
  return _init.bridge.resultU8a();
});
exports.hmacSha256 = hmacSha256;
const hmacSha512 = withWasm((wasm, key, data) => {
  wasm.ext_hmac_sha512(8, ..._init.bridge.allocU8a(key), ..._init.bridge.allocU8a(data));
  return _init.bridge.resultU8a();
});
exports.hmacSha512 = hmacSha512;
const keccak256 = withWasm((wasm, data) => {
  wasm.ext_keccak256(8, ..._init.bridge.allocU8a(data));
  return _init.bridge.resultU8a();
});
exports.keccak256 = keccak256;
const keccak512 = withWasm((wasm, data) => {
  wasm.ext_keccak512(8, ..._init.bridge.allocU8a(data));
  return _init.bridge.resultU8a();
});
exports.keccak512 = keccak512;
const pbkdf2 = withWasm((wasm, data, salt, rounds) => {
  wasm.ext_pbkdf2(8, ..._init.bridge.allocU8a(data), ..._init.bridge.allocU8a(salt), rounds);
  return _init.bridge.resultU8a();
});
exports.pbkdf2 = pbkdf2;
const scrypt = withWasm((wasm, password, salt, log2n, r, p) => {
  wasm.ext_scrypt(8, ..._init.bridge.allocU8a(password), ..._init.bridge.allocU8a(salt), log2n, r, p);
  return _init.bridge.resultU8a();
});
exports.scrypt = scrypt;
const sha256 = withWasm((wasm, data) => {
  wasm.ext_sha256(8, ..._init.bridge.allocU8a(data));
  return _init.bridge.resultU8a();
});
exports.sha256 = sha256;
const sha512 = withWasm((wasm, data) => {
  wasm.ext_sha512(8, ..._init.bridge.allocU8a(data));
  return _init.bridge.resultU8a();
});
exports.sha512 = sha512;
const twox = withWasm((wasm, data, rounds) => {
  wasm.ext_twox(8, ..._init.bridge.allocU8a(data), rounds);
  return _init.bridge.resultU8a();
});
exports.twox = twox;

function isReady() {
  return !!_init.bridge.wasm;
}

async function waitReady() {
  try {
    const wasm = await (0, _init.initBridge)();
    return !!wasm;
  } catch {
    return false;
  }
}
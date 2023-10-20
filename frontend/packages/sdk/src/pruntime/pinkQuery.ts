import { type CodecMap } from '@polkadot/types'
import { hexAddPrefix, hexToU8a, u8aToHex } from '@polkadot/util'
import { sr25519Sign } from '@polkadot/util-crypto'
import { phalaTypes } from '../options'
import { decrypt, encrypt } from '../utils/aes-256-gcm'
import { randomHex } from '../utils/hex'
import { type CertificateData } from './certificate'
import { pruntime_rpc as pruntimeRpc } from './proto'

interface IEncryptedData extends CodecMap {
  data: Uint8Array
  iv: Uint8Array
}

function createEncryptedData(pk: Uint8Array, data: string, agreementKey: Uint8Array) {
  const iv = hexAddPrefix(randomHex(12))
  return {
    iv,
    pubkey: u8aToHex(pk),
    data: hexAddPrefix(encrypt(data, agreementKey, hexToU8a(iv))),
  }
}

export async function pinkQuery(
  pruntimeApi: pruntimeRpc.PhactoryAPI,
  pk: Uint8Array,
  queryAgreementKey: Uint8Array,
  encodedQuery: string,
  { certificate, pubkey, secret }: CertificateData
) {
  // Encrypt the ContractQuery.
  const encryptedData = createEncryptedData(pk, encodedQuery, queryAgreementKey)
  const encodedEncryptedData = phalaTypes.createType('EncryptedData', encryptedData).toU8a()

  // Sign the encrypted data.
  const signature: pruntimeRpc.ISignature = {
    signedBy: certificate,
    signatureType: pruntimeRpc.SignatureType.Sr25519,
    // signature: sr25519Sign(pubkey, secret, encodedEncryptedData),
    signature: sr25519Sign(encodedEncryptedData, { publicKey: pubkey, secretKey: secret }),
  }

  // Send request.
  const requestData = {
    encodedEncryptedData,
    signature,
  }

  const res = await pruntimeApi.contractQuery(requestData)

  const { data: encryptedResult, iv } = phalaTypes.createType<IEncryptedData>('EncryptedData', res.encodedEncryptedData)
  const data = decrypt(encryptedResult.toString(), queryAgreementKey, iv)
  return hexAddPrefix(data)
}

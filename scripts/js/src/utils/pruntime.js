const { pruntime_rpc: { PhactoryAPI } } = require('../proto/pruntime_rpc');
const axios = require('axios').default;

function createPRuntimeApi(endpoint, debug=console.log, warn=console.warn) {
    return PhactoryAPI.create(
        async (method, requestData, callback) => {
            const url = `${endpoint}/prpc/PhactoryAPI.${method.name}`
            debug({ url, requestData }, 'Sending HTTP request...')
            try {
                const res = await axios.post(url, requestData, {
                    headers: {
                        'Content-Type': 'application/octet-stream',
                    },
                    responseType: 'arraybuffer',
                });

                const buffer = await res.data;
                if (res.status === 200) {
                    callback(null, buffer)
                } else {
                    const errPb = prpc.PrpcError.decode(buffer)
                    warn(prpc.PrpcError.toObject(errPb))
                    callback(new Error(errPb.message))
                }
            } catch (e) {
                warn(e)
                callback(e)
            }
        },
        false,
        false
    )
};

// ---- copied from js-sdk ----

const {createCipheriv, createDecipheriv} = require('crypto')
const {hexToU8a, hexAddPrefix, hexStripPrefix} = require('@polkadot/util')

const ALGO = 'aes-256-gcm'
const AUTH_TAG_LENGTH = 32

const toU8a = (param) => {
  if (typeof param === 'string') {
    param = hexAddPrefix(param)
    return hexToU8a(param)
  }

  return param
}

const encrypt = (data, key, iv) => {
  data = hexStripPrefix(data)
  const cipher = createCipheriv(ALGO, toU8a(key), toU8a(iv))
  const enc = cipher.update(data, 'hex', 'hex')
  cipher.final()
  return `${enc}${cipher.getAuthTag().toString('hex')}`
}

const decrypt = (enc, key, iv) => {
  enc = hexStripPrefix(enc)
  const decipher = createDecipheriv(ALGO, toU8a(key), toU8a(iv))
  const authTag = hexToU8a(hexAddPrefix(enc.slice(-AUTH_TAG_LENGTH)))
  decipher.setAuthTag(authTag)
  const data = decipher.update(enc.slice(0, -AUTH_TAG_LENGTH), 'hex', 'hex')
  decipher.final()
  return data
}

module.exports = { createPRuntimeApi, crypto: { encrypt, decrypt } };

import { Buffer } from 'buffer'
import { hexAddPrefix, hexStripPrefix, hexToU8a } from '@polkadot/util'
import { createCipheriv, createDecipheriv } from 'browserify-cipher'

const ALGO = 'aes-256-gcm'
const AUTH_TAG_LENGTH = 32

type Param = Uint8Array | string

const toU8a = (param: Param): Uint8Array => {
  if (typeof param === 'string') {
    param = hexAddPrefix(param)
    return hexToU8a(param)
  }

  return param
}

export const encrypt = (data: string, key: Param, iv: Param): string => {
  data = hexStripPrefix(data)
  const cipher = createCipheriv(ALGO, toU8a(key), Buffer.from(toU8a(iv)) as unknown as string)
  const enc = cipher.update(data, 'hex', 'hex')
  cipher.final()
  return `${enc}${cipher.getAuthTag().toString('hex')}`
}

export const decrypt = (enc: string, key: Param, iv: Param): string => {
  enc = hexStripPrefix(enc)
  const decipher = createDecipheriv(ALGO, toU8a(key), Buffer.from(toU8a(iv)) as unknown as string)
  const authTag = hexToU8a(hexAddPrefix(enc.slice(-AUTH_TAG_LENGTH)))
  decipher.setAuthTag(authTag)
  const data = decipher.update(enc.slice(0, -AUTH_TAG_LENGTH), 'hex', 'hex')
  decipher.final()
  return data
}

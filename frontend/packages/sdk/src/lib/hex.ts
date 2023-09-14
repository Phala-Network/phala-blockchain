import { randomBytes } from 'crypto-browserify'

export const randomHex = (size = 12): string => randomBytes(size).toString('hex')

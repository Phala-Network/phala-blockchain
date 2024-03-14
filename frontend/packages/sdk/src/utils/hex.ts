import randomBytes from 'randombytes'
import { type HexString } from '../types'

export const randomHex = (size = 12): HexString => randomBytes(size).toString('hex')

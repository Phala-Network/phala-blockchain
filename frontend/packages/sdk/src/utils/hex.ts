import randomBytes from 'randombytes'

export const randomHex = (size = 12): string => randomBytes(size).toString('hex')

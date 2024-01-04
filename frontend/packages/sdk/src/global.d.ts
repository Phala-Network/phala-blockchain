declare module 'browserify-cipher' {
  export { createCipheriv, createDecipheriv } from 'crypto'
}

declare module 'randombytes' {
  export { randombytes as default } from 'crypto'
}

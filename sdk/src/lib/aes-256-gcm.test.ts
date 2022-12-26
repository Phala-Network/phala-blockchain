import {encrypt, decrypt} from './aes-256-gcm'

const data = '675abfa9aff26fbf3f4a0bd91f513c40644571f86aa2c18d2d284ad68f17fc97'
const key = '8a3ae1de0dddb21d1dce3647d66d488ce9dfd0f0f4bdad4766e931aef7e35656'
const encryptedData =
  'ba10a8bd942fddc0d3acc5c20e33fb114c292d3521efed516e7e7dc444a92a3f69d69dd07a003cb8160067953d79fad4'
const iv = '989e2eaba6f775ef660ccdd3'

test('aes-256-gcm encrypt', () => {
  expect(encrypt(data, key, iv)).toBe(encryptedData)
})

test('aes-256-gcm decrypt', () => {
  expect(decrypt(encryptedData, key, iv)).toBe(data)
})

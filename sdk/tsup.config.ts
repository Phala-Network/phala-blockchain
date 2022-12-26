import {defineConfig} from 'tsup'

export default defineConfig({
  entry: ['src/index.ts'],
  dts: true,
  format: ['cjs', 'esm'],
  ignoreWatch: ['*.test.ts'],
  target: 'node16',
  clean: true,
})

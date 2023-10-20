import { defineConfig } from 'tsup'

export default defineConfig({
  entry: ['src/index.ts'],
  outDir: './dist/node',
  dts: true,
  format: ['cjs', 'esm'],
  ignoreWatch: ['*.test.ts'],
  target: 'node16',
  clean: true,
  noExternal: ['protobufjs'],
  metafile: true,
})

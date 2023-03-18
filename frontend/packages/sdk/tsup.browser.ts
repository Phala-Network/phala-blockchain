import { NodeGlobalsPolyfillPlugin } from "@esbuild-plugins/node-globals-polyfill";
import { NodeModulesPolyfillPlugin } from "@esbuild-plugins/node-modules-polyfill";
// import nodePolyfills from 'rollup-plugin-node-polyfills';
import { defineConfig } from "tsup";
import { Plugin, Loader } from 'esbuild'

const ReplaceModulesPlugin = (modules: {
  name: string
  contents: string
  loader?: Loader
}[]): Plugin => {
  return {
    name: 'replaceModules',
    setup(build) {

      modules.forEach(moduleItem => {
        const { name, contents, loader = 'js' } = moduleItem
        const filter = new RegExp(`^${name}\\/?(.+)?`)

        build.onResolve({ filter }, args => {
          return {
            path: args.path,
            namespace: name,
          }
        })

        build.onLoad({ filter, namespace: name }, () => {
          return {
            contents,
            loader,
          }
        })
      })
    },
  }
}

export default defineConfig({
  esbuildPlugins: [
    NodeModulesPolyfillPlugin(),
    NodeGlobalsPolyfillPlugin({
      buffer: true,
    }),
    ReplaceModulesPlugin([{
      name: 'undici',
      contents: 'export const fetch = global.fetch || window.fetch;'
    }])
  ],
  entry: ["src/index.ts"],
  outDir: "./dist/browser",
  dts: true,
  format: ["cjs", "esm"],
  ignoreWatch: ["*.test.ts"],
  target: "node16",
  clean: true,
  platform: "browser",
  noExternal: ["crypto-browserify", "protobufjs", "randomBytes", "undici"],
});

import nodeResolve from "@rollup/plugin-node-resolve";
import commonjs from "@rollup/plugin-commonjs";
import json from "@rollup/plugin-json";
import { terser } from "rollup-plugin-terser";

export default {
    output: {
        dir: "dist",
        format: "cjs",
        exports: "default",
    },
    plugins: [nodeResolve({exportConditions: ['node']}), commonjs(), json(), terser()],
};

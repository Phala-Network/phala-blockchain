"use strict";

Object.defineProperty(exports, "__esModule", {
  value: true
});

var _wasm = require("./wasm");

Object.keys(_wasm).forEach(function (key) {
  if (key === "default" || key === "__esModule") return;
  if (key in exports && exports[key] === _wasm[key]) return;
  Object.defineProperty(exports, key, {
    enumerable: true,
    get: function () {
      return _wasm[key];
    }
  });
});
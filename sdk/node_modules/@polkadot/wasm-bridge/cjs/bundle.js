"use strict";

Object.defineProperty(exports, "__esModule", {
  value: true
});

var _bridge = require("./bridge");

Object.keys(_bridge).forEach(function (key) {
  if (key === "default" || key === "__esModule") return;
  if (key in exports && exports[key] === _bridge[key]) return;
  Object.defineProperty(exports, key, {
    enumerable: true,
    get: function () {
      return _bridge[key];
    }
  });
});

var _init = require("./init");

Object.keys(_init).forEach(function (key) {
  if (key === "default" || key === "__esModule") return;
  if (key in exports && exports[key] === _init[key]) return;
  Object.defineProperty(exports, key, {
    enumerable: true,
    get: function () {
      return _init[key];
    }
  });
});
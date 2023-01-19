"use strict";

Object.defineProperty(exports, "__esModule", {
  value: true
});
Object.defineProperty(exports, "naclBoxPairFromSecret", {
  enumerable: true,
  get: function () {
    return _fromSecret.naclBoxPairFromSecret;
  }
});
Object.defineProperty(exports, "naclDecrypt", {
  enumerable: true,
  get: function () {
    return _decrypt.naclDecrypt;
  }
});
Object.defineProperty(exports, "naclEncrypt", {
  enumerable: true,
  get: function () {
    return _encrypt.naclEncrypt;
  }
});
Object.defineProperty(exports, "naclOpen", {
  enumerable: true,
  get: function () {
    return _open.naclOpen;
  }
});
Object.defineProperty(exports, "naclSeal", {
  enumerable: true,
  get: function () {
    return _seal.naclSeal;
  }
});
var _decrypt = require("./decrypt");
var _encrypt = require("./encrypt");
var _fromSecret = require("./box/fromSecret");
var _open = require("./open");
var _seal = require("./seal");
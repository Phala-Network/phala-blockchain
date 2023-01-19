"use strict";

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.memoize = memoize;
var _stringify = require("./stringify");
// Copyright 2017-2022 @polkadot/util authors & contributors
// SPDX-License-Identifier: Apache-2.0

function defaultGetId() {
  return 'none';
}

/**
 * @name memoize
 * @description Memomize the function with a specific instanceId
 */
// eslint-disable-next-line @typescript-eslint/no-explicit-any
function memoize(fn) {
  let {
    getInstanceId = defaultGetId
  } = arguments.length > 1 && arguments[1] !== undefined ? arguments[1] : {};
  const cache = {};
  const memoized = function () {
    for (var _len = arguments.length, args = new Array(_len), _key = 0; _key < _len; _key++) {
      args[_key] = arguments[_key];
    }
    const stringParams = (0, _stringify.stringify)(args);
    const instanceId = getInstanceId();
    if (!cache[instanceId]) {
      cache[instanceId] = {};
    }
    if (cache[instanceId][stringParams] === undefined) {
      cache[instanceId][stringParams] = fn(...args);
    }
    return cache[instanceId][stringParams];
  };
  memoized.unmemoize = function () {
    for (var _len2 = arguments.length, args = new Array(_len2), _key2 = 0; _key2 < _len2; _key2++) {
      args[_key2] = arguments[_key2];
    }
    const stringParams = (0, _stringify.stringify)(args);
    const instanceId = getInstanceId();
    if (cache[instanceId] && cache[instanceId][stringParams] !== undefined) {
      delete cache[instanceId][stringParams];
    }
  };
  return memoized;
}
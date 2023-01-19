"use strict";

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.createCmp = createCmp;
// Copyright 2017-2022 @polkadot/util authors & contributors
// SPDX-License-Identifier: Apache-2.0

/** @internal */
function createCmp(cmp) {
  return function () {
    for (var _len = arguments.length, items = new Array(_len), _key = 0; _key < _len; _key++) {
      items[_key] = arguments[_key];
    }
    if (items.length === 0) {
      throw new Error('Must provide one or more arguments');
    }
    let result = items[0];
    for (let i = 1; i < items.length; i++) {
      if (cmp(items[i], result)) {
        result = items[i];
      }
    }
    return result;
  };
}
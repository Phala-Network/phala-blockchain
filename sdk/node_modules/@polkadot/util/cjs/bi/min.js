"use strict";

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.nMin = exports.nMax = void 0;
var _helpers = require("./helpers");
// Copyright 2017-2022 @polkadot/util authors & contributors
// SPDX-License-Identifier: Apache-2.0

/**
 * @name nMax
 * @summary Finds and returns the highest value in an array of bigint.
 */
const nMax = (0, _helpers.createCmp)((a, b) => a > b);

/**
 * @name nMin
 * @summary Finds and returns the lowest value in an array of bigint.
 */
exports.nMax = nMax;
const nMin = (0, _helpers.createCmp)((a, b) => a < b);
exports.nMin = nMin;
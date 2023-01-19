"use strict";

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.bnMin = exports.bnMax = void 0;
var _helpers = require("../bi/helpers");
// Copyright 2017-2022 @polkadot/util authors & contributors
// SPDX-License-Identifier: Apache-2.0

/**
 * @name bnMax
 * @summary Finds and returns the highest value in an array of BNs.
 * @example
 * <BR>
 *
 * ```javascript
 * import BN from 'bn.js';
 * import { bnMax } from '@polkadot/util';
 *
 * bnMax([new BN(1), new BN(3), new BN(2)]).toString(); // => '3'
 * ```
 */
const bnMax = (0, _helpers.createCmp)((a, b) => a.gt(b));

/**
 * @name bnMin
 * @summary Finds and returns the smallest value in an array of BNs.
 * @example
 * <BR>
 *
 * ```javascript
 * import BN from 'bn.js';
 * import { bnMin } from '@polkadot/util';
 *
 * bnMin([new BN(1), new BN(3), new BN(2)]).toString(); // => '1'
 * ```
 */
exports.bnMax = bnMax;
const bnMin = (0, _helpers.createCmp)((a, b) => a.lt(b));
exports.bnMin = bnMin;
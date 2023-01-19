"use strict";

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.isCompact = void 0;
var _helpers = require("./helpers");
// Copyright 2017-2022 @polkadot/util authors & contributors
// SPDX-License-Identifier: Apache-2.0

/**
 * @name isCompact
 * @summary Tests for SCALE-Compact-like object instance.
 */
const isCompact = (0, _helpers.isOnObject)('toBigInt', 'toBn', 'toNumber', 'unwrap');
exports.isCompact = isCompact;
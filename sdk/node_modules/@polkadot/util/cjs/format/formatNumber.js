"use strict";

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.formatNumber = formatNumber;
var _toBn = require("../bn/toBn");
var _formatDecimal = require("./formatDecimal");
// Copyright 2017-2022 @polkadot/util authors & contributors
// SPDX-License-Identifier: Apache-2.0

/**
 * @name formatNumber
 * @description Formats a number into string format with thousand seperators
 */
function formatNumber(value) {
  return (0, _formatDecimal.formatDecimal)((0, _toBn.bnToBn)(value).toString());
}
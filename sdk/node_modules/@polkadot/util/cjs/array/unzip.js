"use strict";

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.arrayUnzip = arrayUnzip;
// Copyright 2017-2022 @polkadot/util authors & contributors
// SPDX-License-Identifier: Apache-2.0

/**
 * @name arrayUnzip
 * @description Splits a single [K, V][] into [K[], V[]]
 */
function arrayUnzip(entries) {
  const keys = new Array(entries.length);
  const values = new Array(entries.length);
  for (let i = 0; i < entries.length; i++) {
    [keys[i], values[i]] = entries[i];
  }
  return [keys, values];
}
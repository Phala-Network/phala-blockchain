// Copyright 2017-2022 @polkadot/util authors & contributors
// SPDX-License-Identifier: Apache-2.0

import { createCmp } from "./helpers.js";

/**
 * @name nMax
 * @summary Finds and returns the highest value in an array of bigint.
 */
export const nMax = createCmp((a, b) => a > b);

/**
 * @name nMin
 * @summary Finds and returns the lowest value in an array of bigint.
 */
export const nMin = createCmp((a, b) => a < b);
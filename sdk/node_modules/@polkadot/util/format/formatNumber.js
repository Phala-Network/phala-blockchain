// Copyright 2017-2022 @polkadot/util authors & contributors
// SPDX-License-Identifier: Apache-2.0

import { bnToBn } from "../bn/toBn.js";
import { formatDecimal } from "./formatDecimal.js";

/**
 * @name formatNumber
 * @description Formats a number into string format with thousand seperators
 */
export function formatNumber(value) {
  return formatDecimal(bnToBn(value).toString());
}
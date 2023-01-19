// Copyright 2017-2022 @polkadot/util authors & contributors
// SPDX-License-Identifier: Apache-2.0

import { isOnObject } from "./helpers.js";
/**
 * @name isCompact
 * @summary Tests for SCALE-Compact-like object instance.
 */
export const isCompact = isOnObject('toBigInt', 'toBn', 'toNumber', 'unwrap');
// Copyright 2017-2022 @polkadot/util authors & contributors
// SPDX-License-Identifier: Apache-2.0

// The regex patterns below were copied as-is from the ip-regex package 5.0.0,
// https://github.com/sindresorhus/ip-regex/blob/a2a44dfa7f776528158c2a5ff9d8a1be435ec1b9/index.js#L1
//
// MIT License
// Copyright (c) Sindre Sorhus <sindresorhus@gmail.com> (https://sindresorhus.com)
//
// Changes made:
//  - boundary support option has been dropped
//  - only exact matching is used (isIp always passed exact: true)
//  - the newest ip-regex is ESM-only, we compile to all platforms
const v4 = '(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)(?:\\.(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)){3}';
const v6s = '[a-fA-F\\d]{1,4}';
const v6 = `
(?:
(?:${v6s}:){7}(?:${v6s}|:)|                                    // 1:2:3:4:5:6:7::  1:2:3:4:5:6:7:8
(?:${v6s}:){6}(?:${v4}|:${v6s}|:)|                             // 1:2:3:4:5:6::    1:2:3:4:5:6::8   1:2:3:4:5:6::8  1:2:3:4:5:6::1.2.3.4
(?:${v6s}:){5}(?::${v4}|(?::${v6s}){1,2}|:)|                   // 1:2:3:4:5::      1:2:3:4:5::7:8   1:2:3:4:5::8    1:2:3:4:5::7:1.2.3.4
(?:${v6s}:){4}(?:(?::${v6s}){0,1}:${v4}|(?::${v6s}){1,3}|:)| // 1:2:3:4::        1:2:3:4::6:7:8   1:2:3:4::8      1:2:3:4::6:7:1.2.3.4
(?:${v6s}:){3}(?:(?::${v6s}){0,2}:${v4}|(?::${v6s}){1,4}|:)| // 1:2:3::          1:2:3::5:6:7:8   1:2:3::8        1:2:3::5:6:7:1.2.3.4
(?:${v6s}:){2}(?:(?::${v6s}){0,3}:${v4}|(?::${v6s}){1,5}|:)| // 1:2::            1:2::4:5:6:7:8   1:2::8          1:2::4:5:6:7:1.2.3.4
(?:${v6s}:){1}(?:(?::${v6s}){0,4}:${v4}|(?::${v6s}){1,6}|:)| // 1::              1::3:4:5:6:7:8   1::8            1::3:4:5:6:7:1.2.3.4
(?::(?:(?::${v6s}){0,5}:${v4}|(?::${v6s}){1,7}|:))             // ::2:3:4:5:6:7:8  ::2:3:4:5:6:7:8  ::8             ::1.2.3.4
)(?:%[0-9a-zA-Z]{1,})?                                             // %eth0            %1
`.replace(/\s*\/\/.*$/gm, '').replace(/\n/g, '').trim();
const v46Exact = new RegExp(`(?:^${v4}$)|(?:^${v6}$)`);
const v4exact = new RegExp(`^${v4}$`);
const v6exact = new RegExp(`^${v6}$`);

/**
 * @name isIp
 * @summary Tests if the value is a valid IP address
 * @description
 * Checks to see if the value is a valid IP address. Optionally check for either v4/v6
 * @example
 * <BR>
 *
 * ```javascript
 * import { isIp } from '@polkadot/util';
 *
 * isIp('192.168.0.1')); // => true
 * isIp('1:2:3:4:5:6:7:8'); // => true
 * isIp('192.168.0.1', 'v6')); // => false
 * isIp('1:2:3:4:5:6:7:8', 'v4'); // => false
 * ```
 */
export function isIp(value, type) {
  switch (type) {
    case 'v4':
      return v4exact.test(value);
    case 'v6':
      return v6exact.test(value);
    default:
      return v46Exact.test(value);
  }
}
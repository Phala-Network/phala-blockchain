"use strict";

var _browser = require("./browser");
var _node = require("./node");
// Copyright 2017-2022 @polkadot/x-textencoder authors & contributors
// SPDX-License-Identifier: Apache-2.0

console.log(new _browser.TextEncoder().encode('abc'));
console.log(new _node.TextEncoder().encode('abc'));
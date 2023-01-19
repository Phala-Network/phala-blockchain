"use strict";

var _browser = require("./browser");
var _node = require("./node");
// Copyright 2017-2022 @polkadot/x-textdecoder authors & contributors
// SPDX-License-Identifier: Apache-2.0

console.log(new _browser.TextDecoder('utf-8').decode(new Uint8Array([1, 2, 3])));
console.log(new _node.TextDecoder('utf-8').decode(new Uint8Array([1, 2, 3])));
"use strict";

var _interopRequireDefault = require("@babel/runtime/helpers/interopRequireDefault");
var _detectOther = _interopRequireDefault(require("./detectOther"));
var _packageInfo = require("./packageInfo");
var _versionDetect = require("./versionDetect");
// Copyright 2017-2022 @polkadot/util authors & contributors
// SPDX-License-Identifier: Apache-2.0

(0, _versionDetect.detectPackage)(_packageInfo.packageInfo, null, _detectOther.default);
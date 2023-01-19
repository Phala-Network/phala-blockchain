"use strict";

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.detectPackage = detectPackage;
var _xGlobal = require("@polkadot/x-global");
var _function = require("./is/function");
// Copyright 2017-2022 @polkadot/util authors & contributors
// SPDX-License-Identifier: Apache-2.0

const DEDUPE = 'Either remove and explicitly install matching versions or dedupe using your package manager.\nThe following conflicting packages were found:';

/** @internal */
function getEntry(name) {
  const _global = _xGlobal.xglobal;
  if (!_global.__polkadotjs) {
    _global.__polkadotjs = {};
  }
  if (!_global.__polkadotjs[name]) {
    _global.__polkadotjs[name] = [];
  }
  return _global.__polkadotjs[name];
}

/** @internal */
function formatDisplay(all, fmt) {
  let max = 0;
  for (let i = 0; i < all.length; i++) {
    max = Math.max(max, all[i].version.length);
  }
  return all.map(d => `\t${fmt(d.version.padEnd(max), d).join('\t')}`).join('\n');
}

/** @internal */
function formatInfo(version, _ref) {
  let {
    name
  } = _ref;
  return [version, name];
}

/** @internal */
function formatVersion(version, _ref2) {
  let {
    path,
    type
  } = _ref2;
  let extracted;
  if (path && path.length >= 5) {
    const nmIndex = path.indexOf('node_modules');
    extracted = nmIndex === -1 ? path : path.substring(nmIndex);
  } else {
    extracted = '<unknown>';
  }
  return [`${`${type || ''}`.padStart(3)} ${version}`, extracted];
}

/** @internal */
function getPath(infoPath, pathOrFn) {
  if (infoPath) {
    return infoPath;
  } else if ((0, _function.isFunction)(pathOrFn)) {
    try {
      return pathOrFn() || '';
    } catch (error) {
      return '';
    }
  }
  return pathOrFn || '';
}

/** @internal */
function warn(pre, all, fmt) {
  console.warn(`${pre}\n${DEDUPE}\n${formatDisplay(all, fmt)}`);
}

/**
 * @name detectPackage
 * @summary Checks that a specific package is only imported once
 * @description A `@polkadot/*` version detection utility, checking for one occurence of a package in addition to checking for ddependency versions.
 */
function detectPackage(_ref3, pathOrFn) {
  let {
    name,
    path,
    type,
    version
  } = _ref3;
  let deps = arguments.length > 2 && arguments[2] !== undefined ? arguments[2] : [];
  if (!name.startsWith('@polkadot')) {
    throw new Error(`Invalid package descriptor ${name}`);
  }
  const entry = getEntry(name);
  entry.push({
    path: getPath(path, pathOrFn),
    type,
    version
  });
  if (entry.length !== 1) {
    warn(`${name} has multiple versions, ensure that there is only one installed.`, entry, formatVersion);
  } else {
    const mismatches = deps.filter(d => d && d.version !== version);
    if (mismatches.length) {
      warn(`${name} requires direct dependencies exactly matching version ${version}.`, mismatches, formatInfo);
    }
  }
}
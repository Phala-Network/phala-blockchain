(function (global, factory) {
  typeof exports === 'object' && typeof module !== 'undefined' ? factory(exports, require('@polkadot/util')) :
  typeof define === 'function' && define.amd ? define(['exports', '@polkadot/util'], factory) :
  (global = typeof globalThis !== 'undefined' ? globalThis : global || self, factory(global.polkadotUtilCrypto = {}, global.polkadotUtil));
})(this, (function (exports, util) { 'use strict';

  const global = typeof globalThis !== "undefined" ? globalThis : typeof self !== "undefined" ? self : window;

  const packageInfo$3 = {
    name: '@polkadot/x-global',
    path: (({ url: (typeof document === 'undefined' && typeof location === 'undefined' ? new (require('u' + 'rl').URL)('file:' + __filename).href : typeof document === 'undefined' ? location.href : (document.currentScript && document.currentScript.src || new URL('bundle-polkadot-util-crypto.js', document.baseURI).href)) }) && (typeof document === 'undefined' && typeof location === 'undefined' ? new (require('u' + 'rl').URL)('file:' + __filename).href : typeof document === 'undefined' ? location.href : (document.currentScript && document.currentScript.src || new URL('bundle-polkadot-util-crypto.js', document.baseURI).href))) ? new URL((typeof document === 'undefined' && typeof location === 'undefined' ? new (require('u' + 'rl').URL)('file:' + __filename).href : typeof document === 'undefined' ? location.href : (document.currentScript && document.currentScript.src || new URL('bundle-polkadot-util-crypto.js', document.baseURI).href))).pathname.substring(0, new URL((typeof document === 'undefined' && typeof location === 'undefined' ? new (require('u' + 'rl').URL)('file:' + __filename).href : typeof document === 'undefined' ? location.href : (document.currentScript && document.currentScript.src || new URL('bundle-polkadot-util-crypto.js', document.baseURI).href))).pathname.lastIndexOf('/') + 1) : 'auto',
    type: 'esm',
    version: '10.1.12'
  };

  function evaluateThis(fn) {
    return fn('return this');
  }
  const xglobal = typeof globalThis !== 'undefined' ? globalThis : typeof global !== 'undefined' ? global : typeof self !== 'undefined' ? self : typeof window !== 'undefined' ? window : evaluateThis(Function);
  function extractGlobal(name, fallback) {
    return typeof xglobal[name] === 'undefined' ? fallback : xglobal[name];
  }
  function exposeGlobal(name, fallback) {
    if (typeof xglobal[name] === 'undefined') {
      xglobal[name] = fallback;
    }
  }

  const build = /*#__PURE__*/Object.freeze({
    __proto__: null,
    xglobal: xglobal,
    extractGlobal: extractGlobal,
    exposeGlobal: exposeGlobal,
    packageInfo: packageInfo$3
  });

  const BigInt$1 = typeof xglobal.BigInt === 'function' && typeof xglobal.BigInt.asIntN === 'function' ? xglobal.BigInt : () => Number.NaN;

  exposeGlobal('BigInt', BigInt$1);

  const crypto$1 = {};

  const crypto$2 = /*#__PURE__*/Object.freeze({
    __proto__: null,
    default: crypto$1
  });

  /*! noble-secp256k1 - MIT License (c) 2019 Paul Miller (paulmillr.com) */
  const _0n$1 = BigInt(0);
  const _1n$1 = BigInt(1);
  const _2n$1 = BigInt(2);
  const _3n = BigInt(3);
  const _8n = BigInt(8);
  const CURVE = Object.freeze({
      a: _0n$1,
      b: BigInt(7),
      P: BigInt('0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f'),
      n: BigInt('0xfffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141'),
      h: _1n$1,
      Gx: BigInt('55066263022277343669578718895168534326250603453777594175500187360389116729240'),
      Gy: BigInt('32670510020758816978083085130507043184471273380659243275938904335757337482424'),
      beta: BigInt('0x7ae96a2b657c07106e64479eac3434e99cf0497512f58995c1396c28719501ee'),
  });
  function weistrass(x) {
      const { a, b } = CURVE;
      const x2 = mod(x * x);
      const x3 = mod(x2 * x);
      return mod(x3 + a * x + b);
  }
  const USE_ENDOMORPHISM = CURVE.a === _0n$1;
  class ShaError extends Error {
      constructor(message) {
          super(message);
      }
  }
  class JacobianPoint {
      constructor(x, y, z) {
          this.x = x;
          this.y = y;
          this.z = z;
      }
      static fromAffine(p) {
          if (!(p instanceof Point)) {
              throw new TypeError('JacobianPoint#fromAffine: expected Point');
          }
          return new JacobianPoint(p.x, p.y, _1n$1);
      }
      static toAffineBatch(points) {
          const toInv = invertBatch(points.map((p) => p.z));
          return points.map((p, i) => p.toAffine(toInv[i]));
      }
      static normalizeZ(points) {
          return JacobianPoint.toAffineBatch(points).map(JacobianPoint.fromAffine);
      }
      equals(other) {
          if (!(other instanceof JacobianPoint))
              throw new TypeError('JacobianPoint expected');
          const { x: X1, y: Y1, z: Z1 } = this;
          const { x: X2, y: Y2, z: Z2 } = other;
          const Z1Z1 = mod(Z1 * Z1);
          const Z2Z2 = mod(Z2 * Z2);
          const U1 = mod(X1 * Z2Z2);
          const U2 = mod(X2 * Z1Z1);
          const S1 = mod(mod(Y1 * Z2) * Z2Z2);
          const S2 = mod(mod(Y2 * Z1) * Z1Z1);
          return U1 === U2 && S1 === S2;
      }
      negate() {
          return new JacobianPoint(this.x, mod(-this.y), this.z);
      }
      double() {
          const { x: X1, y: Y1, z: Z1 } = this;
          const A = mod(X1 * X1);
          const B = mod(Y1 * Y1);
          const C = mod(B * B);
          const x1b = X1 + B;
          const D = mod(_2n$1 * (mod(x1b * x1b) - A - C));
          const E = mod(_3n * A);
          const F = mod(E * E);
          const X3 = mod(F - _2n$1 * D);
          const Y3 = mod(E * (D - X3) - _8n * C);
          const Z3 = mod(_2n$1 * Y1 * Z1);
          return new JacobianPoint(X3, Y3, Z3);
      }
      add(other) {
          if (!(other instanceof JacobianPoint))
              throw new TypeError('JacobianPoint expected');
          const { x: X1, y: Y1, z: Z1 } = this;
          const { x: X2, y: Y2, z: Z2 } = other;
          if (X2 === _0n$1 || Y2 === _0n$1)
              return this;
          if (X1 === _0n$1 || Y1 === _0n$1)
              return other;
          const Z1Z1 = mod(Z1 * Z1);
          const Z2Z2 = mod(Z2 * Z2);
          const U1 = mod(X1 * Z2Z2);
          const U2 = mod(X2 * Z1Z1);
          const S1 = mod(mod(Y1 * Z2) * Z2Z2);
          const S2 = mod(mod(Y2 * Z1) * Z1Z1);
          const H = mod(U2 - U1);
          const r = mod(S2 - S1);
          if (H === _0n$1) {
              if (r === _0n$1) {
                  return this.double();
              }
              else {
                  return JacobianPoint.ZERO;
              }
          }
          const HH = mod(H * H);
          const HHH = mod(H * HH);
          const V = mod(U1 * HH);
          const X3 = mod(r * r - HHH - _2n$1 * V);
          const Y3 = mod(r * (V - X3) - S1 * HHH);
          const Z3 = mod(Z1 * Z2 * H);
          return new JacobianPoint(X3, Y3, Z3);
      }
      subtract(other) {
          return this.add(other.negate());
      }
      multiplyUnsafe(scalar) {
          const P0 = JacobianPoint.ZERO;
          if (typeof scalar === 'bigint' && scalar === _0n$1)
              return P0;
          let n = normalizeScalar(scalar);
          if (n === _1n$1)
              return this;
          if (!USE_ENDOMORPHISM) {
              let p = P0;
              let d = this;
              while (n > _0n$1) {
                  if (n & _1n$1)
                      p = p.add(d);
                  d = d.double();
                  n >>= _1n$1;
              }
              return p;
          }
          let { k1neg, k1, k2neg, k2 } = splitScalarEndo(n);
          let k1p = P0;
          let k2p = P0;
          let d = this;
          while (k1 > _0n$1 || k2 > _0n$1) {
              if (k1 & _1n$1)
                  k1p = k1p.add(d);
              if (k2 & _1n$1)
                  k2p = k2p.add(d);
              d = d.double();
              k1 >>= _1n$1;
              k2 >>= _1n$1;
          }
          if (k1neg)
              k1p = k1p.negate();
          if (k2neg)
              k2p = k2p.negate();
          k2p = new JacobianPoint(mod(k2p.x * CURVE.beta), k2p.y, k2p.z);
          return k1p.add(k2p);
      }
      precomputeWindow(W) {
          const windows = USE_ENDOMORPHISM ? 128 / W + 1 : 256 / W + 1;
          const points = [];
          let p = this;
          let base = p;
          for (let window = 0; window < windows; window++) {
              base = p;
              points.push(base);
              for (let i = 1; i < 2 ** (W - 1); i++) {
                  base = base.add(p);
                  points.push(base);
              }
              p = base.double();
          }
          return points;
      }
      wNAF(n, affinePoint) {
          if (!affinePoint && this.equals(JacobianPoint.BASE))
              affinePoint = Point.BASE;
          const W = (affinePoint && affinePoint._WINDOW_SIZE) || 1;
          if (256 % W) {
              throw new Error('Point#wNAF: Invalid precomputation window, must be power of 2');
          }
          let precomputes = affinePoint && pointPrecomputes.get(affinePoint);
          if (!precomputes) {
              precomputes = this.precomputeWindow(W);
              if (affinePoint && W !== 1) {
                  precomputes = JacobianPoint.normalizeZ(precomputes);
                  pointPrecomputes.set(affinePoint, precomputes);
              }
          }
          let p = JacobianPoint.ZERO;
          let f = JacobianPoint.ZERO;
          const windows = 1 + (USE_ENDOMORPHISM ? 128 / W : 256 / W);
          const windowSize = 2 ** (W - 1);
          const mask = BigInt(2 ** W - 1);
          const maxNumber = 2 ** W;
          const shiftBy = BigInt(W);
          for (let window = 0; window < windows; window++) {
              const offset = window * windowSize;
              let wbits = Number(n & mask);
              n >>= shiftBy;
              if (wbits > windowSize) {
                  wbits -= maxNumber;
                  n += _1n$1;
              }
              if (wbits === 0) {
                  let pr = precomputes[offset];
                  if (window % 2)
                      pr = pr.negate();
                  f = f.add(pr);
              }
              else {
                  let cached = precomputes[offset + Math.abs(wbits) - 1];
                  if (wbits < 0)
                      cached = cached.negate();
                  p = p.add(cached);
              }
          }
          return { p, f };
      }
      multiply(scalar, affinePoint) {
          let n = normalizeScalar(scalar);
          let point;
          let fake;
          if (USE_ENDOMORPHISM) {
              const { k1neg, k1, k2neg, k2 } = splitScalarEndo(n);
              let { p: k1p, f: f1p } = this.wNAF(k1, affinePoint);
              let { p: k2p, f: f2p } = this.wNAF(k2, affinePoint);
              if (k1neg)
                  k1p = k1p.negate();
              if (k2neg)
                  k2p = k2p.negate();
              k2p = new JacobianPoint(mod(k2p.x * CURVE.beta), k2p.y, k2p.z);
              point = k1p.add(k2p);
              fake = f1p.add(f2p);
          }
          else {
              const { p, f } = this.wNAF(n, affinePoint);
              point = p;
              fake = f;
          }
          return JacobianPoint.normalizeZ([point, fake])[0];
      }
      toAffine(invZ = invert(this.z)) {
          const { x, y, z } = this;
          const iz1 = invZ;
          const iz2 = mod(iz1 * iz1);
          const iz3 = mod(iz2 * iz1);
          const ax = mod(x * iz2);
          const ay = mod(y * iz3);
          const zz = mod(z * iz1);
          if (zz !== _1n$1)
              throw new Error('invZ was invalid');
          return new Point(ax, ay);
      }
  }
  JacobianPoint.BASE = new JacobianPoint(CURVE.Gx, CURVE.Gy, _1n$1);
  JacobianPoint.ZERO = new JacobianPoint(_0n$1, _1n$1, _0n$1);
  const pointPrecomputes = new WeakMap();
  class Point {
      constructor(x, y) {
          this.x = x;
          this.y = y;
      }
      _setWindowSize(windowSize) {
          this._WINDOW_SIZE = windowSize;
          pointPrecomputes.delete(this);
      }
      hasEvenY() {
          return this.y % _2n$1 === _0n$1;
      }
      static fromCompressedHex(bytes) {
          const isShort = bytes.length === 32;
          const x = bytesToNumber(isShort ? bytes : bytes.subarray(1));
          if (!isValidFieldElement(x))
              throw new Error('Point is not on curve');
          const y2 = weistrass(x);
          let y = sqrtMod(y2);
          const isYOdd = (y & _1n$1) === _1n$1;
          if (isShort) {
              if (isYOdd)
                  y = mod(-y);
          }
          else {
              const isFirstByteOdd = (bytes[0] & 1) === 1;
              if (isFirstByteOdd !== isYOdd)
                  y = mod(-y);
          }
          const point = new Point(x, y);
          point.assertValidity();
          return point;
      }
      static fromUncompressedHex(bytes) {
          const x = bytesToNumber(bytes.subarray(1, 33));
          const y = bytesToNumber(bytes.subarray(33, 65));
          const point = new Point(x, y);
          point.assertValidity();
          return point;
      }
      static fromHex(hex) {
          const bytes = ensureBytes(hex);
          const len = bytes.length;
          const header = bytes[0];
          if (len === 32 || (len === 33 && (header === 0x02 || header === 0x03))) {
              return this.fromCompressedHex(bytes);
          }
          if (len === 65 && header === 0x04)
              return this.fromUncompressedHex(bytes);
          throw new Error(`Point.fromHex: received invalid point. Expected 32-33 compressed bytes or 65 uncompressed bytes, not ${len}`);
      }
      static fromPrivateKey(privateKey) {
          return Point.BASE.multiply(normalizePrivateKey(privateKey));
      }
      static fromSignature(msgHash, signature, recovery) {
          msgHash = ensureBytes(msgHash);
          const h = truncateHash(msgHash);
          const { r, s } = normalizeSignature(signature);
          if (recovery !== 0 && recovery !== 1) {
              throw new Error('Cannot recover signature: invalid recovery bit');
          }
          const prefix = recovery & 1 ? '03' : '02';
          const R = Point.fromHex(prefix + numTo32bStr(r));
          const { n } = CURVE;
          const rinv = invert(r, n);
          const u1 = mod(-h * rinv, n);
          const u2 = mod(s * rinv, n);
          const Q = Point.BASE.multiplyAndAddUnsafe(R, u1, u2);
          if (!Q)
              throw new Error('Cannot recover signature: point at infinify');
          Q.assertValidity();
          return Q;
      }
      toRawBytes(isCompressed = false) {
          return hexToBytes(this.toHex(isCompressed));
      }
      toHex(isCompressed = false) {
          const x = numTo32bStr(this.x);
          if (isCompressed) {
              const prefix = this.hasEvenY() ? '02' : '03';
              return `${prefix}${x}`;
          }
          else {
              return `04${x}${numTo32bStr(this.y)}`;
          }
      }
      toHexX() {
          return this.toHex(true).slice(2);
      }
      toRawX() {
          return this.toRawBytes(true).slice(1);
      }
      assertValidity() {
          const msg = 'Point is not on elliptic curve';
          const { x, y } = this;
          if (!isValidFieldElement(x) || !isValidFieldElement(y))
              throw new Error(msg);
          const left = mod(y * y);
          const right = weistrass(x);
          if (mod(left - right) !== _0n$1)
              throw new Error(msg);
      }
      equals(other) {
          return this.x === other.x && this.y === other.y;
      }
      negate() {
          return new Point(this.x, mod(-this.y));
      }
      double() {
          return JacobianPoint.fromAffine(this).double().toAffine();
      }
      add(other) {
          return JacobianPoint.fromAffine(this).add(JacobianPoint.fromAffine(other)).toAffine();
      }
      subtract(other) {
          return this.add(other.negate());
      }
      multiply(scalar) {
          return JacobianPoint.fromAffine(this).multiply(scalar, this).toAffine();
      }
      multiplyAndAddUnsafe(Q, a, b) {
          const P = JacobianPoint.fromAffine(this);
          const aP = a === _0n$1 || a === _1n$1 || this !== Point.BASE ? P.multiplyUnsafe(a) : P.multiply(a);
          const bQ = JacobianPoint.fromAffine(Q).multiplyUnsafe(b);
          const sum = aP.add(bQ);
          return sum.equals(JacobianPoint.ZERO) ? undefined : sum.toAffine();
      }
  }
  Point.BASE = new Point(CURVE.Gx, CURVE.Gy);
  Point.ZERO = new Point(_0n$1, _0n$1);
  function sliceDER(s) {
      return Number.parseInt(s[0], 16) >= 8 ? '00' + s : s;
  }
  function parseDERInt(data) {
      if (data.length < 2 || data[0] !== 0x02) {
          throw new Error(`Invalid signature integer tag: ${bytesToHex(data)}`);
      }
      const len = data[1];
      const res = data.subarray(2, len + 2);
      if (!len || res.length !== len) {
          throw new Error(`Invalid signature integer: wrong length`);
      }
      if (res[0] === 0x00 && res[1] <= 0x7f) {
          throw new Error('Invalid signature integer: trailing length');
      }
      return { data: bytesToNumber(res), left: data.subarray(len + 2) };
  }
  function parseDERSignature(data) {
      if (data.length < 2 || data[0] != 0x30) {
          throw new Error(`Invalid signature tag: ${bytesToHex(data)}`);
      }
      if (data[1] !== data.length - 2) {
          throw new Error('Invalid signature: incorrect length');
      }
      const { data: r, left: sBytes } = parseDERInt(data.subarray(2));
      const { data: s, left: rBytesLeft } = parseDERInt(sBytes);
      if (rBytesLeft.length) {
          throw new Error(`Invalid signature: left bytes after parsing: ${bytesToHex(rBytesLeft)}`);
      }
      return { r, s };
  }
  class Signature {
      constructor(r, s) {
          this.r = r;
          this.s = s;
          this.assertValidity();
      }
      static fromCompact(hex) {
          const arr = hex instanceof Uint8Array;
          const name = 'Signature.fromCompact';
          if (typeof hex !== 'string' && !arr)
              throw new TypeError(`${name}: Expected string or Uint8Array`);
          const str = arr ? bytesToHex(hex) : hex;
          if (str.length !== 128)
              throw new Error(`${name}: Expected 64-byte hex`);
          return new Signature(hexToNumber(str.slice(0, 64)), hexToNumber(str.slice(64, 128)));
      }
      static fromDER(hex) {
          const arr = hex instanceof Uint8Array;
          if (typeof hex !== 'string' && !arr)
              throw new TypeError(`Signature.fromDER: Expected string or Uint8Array`);
          const { r, s } = parseDERSignature(arr ? hex : hexToBytes(hex));
          return new Signature(r, s);
      }
      static fromHex(hex) {
          return this.fromDER(hex);
      }
      assertValidity() {
          const { r, s } = this;
          if (!isWithinCurveOrder(r))
              throw new Error('Invalid Signature: r must be 0 < r < n');
          if (!isWithinCurveOrder(s))
              throw new Error('Invalid Signature: s must be 0 < s < n');
      }
      hasHighS() {
          const HALF = CURVE.n >> _1n$1;
          return this.s > HALF;
      }
      normalizeS() {
          return this.hasHighS() ? new Signature(this.r, CURVE.n - this.s) : this;
      }
      toDERRawBytes(isCompressed = false) {
          return hexToBytes(this.toDERHex(isCompressed));
      }
      toDERHex(isCompressed = false) {
          const sHex = sliceDER(numberToHexUnpadded(this.s));
          if (isCompressed)
              return sHex;
          const rHex = sliceDER(numberToHexUnpadded(this.r));
          const rLen = numberToHexUnpadded(rHex.length / 2);
          const sLen = numberToHexUnpadded(sHex.length / 2);
          const length = numberToHexUnpadded(rHex.length / 2 + sHex.length / 2 + 4);
          return `30${length}02${rLen}${rHex}02${sLen}${sHex}`;
      }
      toRawBytes() {
          return this.toDERRawBytes();
      }
      toHex() {
          return this.toDERHex();
      }
      toCompactRawBytes() {
          return hexToBytes(this.toCompactHex());
      }
      toCompactHex() {
          return numTo32bStr(this.r) + numTo32bStr(this.s);
      }
  }
  function concatBytes(...arrays) {
      if (!arrays.every((b) => b instanceof Uint8Array))
          throw new Error('Uint8Array list expected');
      if (arrays.length === 1)
          return arrays[0];
      const length = arrays.reduce((a, arr) => a + arr.length, 0);
      const result = new Uint8Array(length);
      for (let i = 0, pad = 0; i < arrays.length; i++) {
          const arr = arrays[i];
          result.set(arr, pad);
          pad += arr.length;
      }
      return result;
  }
  const hexes = Array.from({ length: 256 }, (v, i) => i.toString(16).padStart(2, '0'));
  function bytesToHex(uint8a) {
      if (!(uint8a instanceof Uint8Array))
          throw new Error('Expected Uint8Array');
      let hex = '';
      for (let i = 0; i < uint8a.length; i++) {
          hex += hexes[uint8a[i]];
      }
      return hex;
  }
  const POW_2_256 = BigInt('0x10000000000000000000000000000000000000000000000000000000000000000');
  function numTo32bStr(num) {
      if (typeof num !== 'bigint')
          throw new Error('Expected bigint');
      if (!(_0n$1 <= num && num < POW_2_256))
          throw new Error('Expected number < 2^256');
      return num.toString(16).padStart(64, '0');
  }
  function numTo32b(num) {
      const b = hexToBytes(numTo32bStr(num));
      if (b.length !== 32)
          throw new Error('Error: expected 32 bytes');
      return b;
  }
  function numberToHexUnpadded(num) {
      const hex = num.toString(16);
      return hex.length & 1 ? `0${hex}` : hex;
  }
  function hexToNumber(hex) {
      if (typeof hex !== 'string') {
          throw new TypeError('hexToNumber: expected string, got ' + typeof hex);
      }
      return BigInt(`0x${hex}`);
  }
  function hexToBytes(hex) {
      if (typeof hex !== 'string') {
          throw new TypeError('hexToBytes: expected string, got ' + typeof hex);
      }
      if (hex.length % 2)
          throw new Error('hexToBytes: received invalid unpadded hex' + hex.length);
      const array = new Uint8Array(hex.length / 2);
      for (let i = 0; i < array.length; i++) {
          const j = i * 2;
          const hexByte = hex.slice(j, j + 2);
          const byte = Number.parseInt(hexByte, 16);
          if (Number.isNaN(byte) || byte < 0)
              throw new Error('Invalid byte sequence');
          array[i] = byte;
      }
      return array;
  }
  function bytesToNumber(bytes) {
      return hexToNumber(bytesToHex(bytes));
  }
  function ensureBytes(hex) {
      return hex instanceof Uint8Array ? Uint8Array.from(hex) : hexToBytes(hex);
  }
  function normalizeScalar(num) {
      if (typeof num === 'number' && Number.isSafeInteger(num) && num > 0)
          return BigInt(num);
      if (typeof num === 'bigint' && isWithinCurveOrder(num))
          return num;
      throw new TypeError('Expected valid private scalar: 0 < scalar < curve.n');
  }
  function mod(a, b = CURVE.P) {
      const result = a % b;
      return result >= _0n$1 ? result : b + result;
  }
  function pow2(x, power) {
      const { P } = CURVE;
      let res = x;
      while (power-- > _0n$1) {
          res *= res;
          res %= P;
      }
      return res;
  }
  function sqrtMod(x) {
      const { P } = CURVE;
      const _6n = BigInt(6);
      const _11n = BigInt(11);
      const _22n = BigInt(22);
      const _23n = BigInt(23);
      const _44n = BigInt(44);
      const _88n = BigInt(88);
      const b2 = (x * x * x) % P;
      const b3 = (b2 * b2 * x) % P;
      const b6 = (pow2(b3, _3n) * b3) % P;
      const b9 = (pow2(b6, _3n) * b3) % P;
      const b11 = (pow2(b9, _2n$1) * b2) % P;
      const b22 = (pow2(b11, _11n) * b11) % P;
      const b44 = (pow2(b22, _22n) * b22) % P;
      const b88 = (pow2(b44, _44n) * b44) % P;
      const b176 = (pow2(b88, _88n) * b88) % P;
      const b220 = (pow2(b176, _44n) * b44) % P;
      const b223 = (pow2(b220, _3n) * b3) % P;
      const t1 = (pow2(b223, _23n) * b22) % P;
      const t2 = (pow2(t1, _6n) * b2) % P;
      return pow2(t2, _2n$1);
  }
  function invert(number, modulo = CURVE.P) {
      if (number === _0n$1 || modulo <= _0n$1) {
          throw new Error(`invert: expected positive integers, got n=${number} mod=${modulo}`);
      }
      let a = mod(number, modulo);
      let b = modulo;
      let x = _0n$1, u = _1n$1;
      while (a !== _0n$1) {
          const q = b / a;
          const r = b % a;
          const m = x - u * q;
          b = a, a = r, x = u, u = m;
      }
      const gcd = b;
      if (gcd !== _1n$1)
          throw new Error('invert: does not exist');
      return mod(x, modulo);
  }
  function invertBatch(nums, p = CURVE.P) {
      const scratch = new Array(nums.length);
      const lastMultiplied = nums.reduce((acc, num, i) => {
          if (num === _0n$1)
              return acc;
          scratch[i] = acc;
          return mod(acc * num, p);
      }, _1n$1);
      const inverted = invert(lastMultiplied, p);
      nums.reduceRight((acc, num, i) => {
          if (num === _0n$1)
              return acc;
          scratch[i] = mod(acc * scratch[i], p);
          return mod(acc * num, p);
      }, inverted);
      return scratch;
  }
  const divNearest = (a, b) => (a + b / _2n$1) / b;
  const ENDO = {
      a1: BigInt('0x3086d221a7d46bcde86c90e49284eb15'),
      b1: -_1n$1 * BigInt('0xe4437ed6010e88286f547fa90abfe4c3'),
      a2: BigInt('0x114ca50f7a8e2f3f657c1108d9d44cfd8'),
      b2: BigInt('0x3086d221a7d46bcde86c90e49284eb15'),
      POW_2_128: BigInt('0x100000000000000000000000000000000'),
  };
  function splitScalarEndo(k) {
      const { n } = CURVE;
      const { a1, b1, a2, b2, POW_2_128 } = ENDO;
      const c1 = divNearest(b2 * k, n);
      const c2 = divNearest(-b1 * k, n);
      let k1 = mod(k - c1 * a1 - c2 * a2, n);
      let k2 = mod(-c1 * b1 - c2 * b2, n);
      const k1neg = k1 > POW_2_128;
      const k2neg = k2 > POW_2_128;
      if (k1neg)
          k1 = n - k1;
      if (k2neg)
          k2 = n - k2;
      if (k1 > POW_2_128 || k2 > POW_2_128) {
          throw new Error('splitScalarEndo: Endomorphism failed, k=' + k);
      }
      return { k1neg, k1, k2neg, k2 };
  }
  function truncateHash(hash) {
      const { n } = CURVE;
      const byteLength = hash.length;
      const delta = byteLength * 8 - 256;
      let h = bytesToNumber(hash);
      if (delta > 0)
          h = h >> BigInt(delta);
      if (h >= n)
          h -= n;
      return h;
  }
  let _sha256Sync;
  let _hmacSha256Sync;
  class HmacDrbg {
      constructor() {
          this.v = new Uint8Array(32).fill(1);
          this.k = new Uint8Array(32).fill(0);
          this.counter = 0;
      }
      hmac(...values) {
          return utils$1.hmacSha256(this.k, ...values);
      }
      hmacSync(...values) {
          return _hmacSha256Sync(this.k, ...values);
      }
      checkSync() {
          if (typeof _hmacSha256Sync !== 'function')
              throw new ShaError('hmacSha256Sync needs to be set');
      }
      incr() {
          if (this.counter >= 1000)
              throw new Error('Tried 1,000 k values for sign(), all were invalid');
          this.counter += 1;
      }
      async reseed(seed = new Uint8Array()) {
          this.k = await this.hmac(this.v, Uint8Array.from([0x00]), seed);
          this.v = await this.hmac(this.v);
          if (seed.length === 0)
              return;
          this.k = await this.hmac(this.v, Uint8Array.from([0x01]), seed);
          this.v = await this.hmac(this.v);
      }
      reseedSync(seed = new Uint8Array()) {
          this.checkSync();
          this.k = this.hmacSync(this.v, Uint8Array.from([0x00]), seed);
          this.v = this.hmacSync(this.v);
          if (seed.length === 0)
              return;
          this.k = this.hmacSync(this.v, Uint8Array.from([0x01]), seed);
          this.v = this.hmacSync(this.v);
      }
      async generate() {
          this.incr();
          this.v = await this.hmac(this.v);
          return this.v;
      }
      generateSync() {
          this.checkSync();
          this.incr();
          this.v = this.hmacSync(this.v);
          return this.v;
      }
  }
  function isWithinCurveOrder(num) {
      return _0n$1 < num && num < CURVE.n;
  }
  function isValidFieldElement(num) {
      return _0n$1 < num && num < CURVE.P;
  }
  function kmdToSig(kBytes, m, d) {
      const k = bytesToNumber(kBytes);
      if (!isWithinCurveOrder(k))
          return;
      const { n } = CURVE;
      const q = Point.BASE.multiply(k);
      const r = mod(q.x, n);
      if (r === _0n$1)
          return;
      const s = mod(invert(k, n) * mod(m + d * r, n), n);
      if (s === _0n$1)
          return;
      const sig = new Signature(r, s);
      const recovery = (q.x === sig.r ? 0 : 2) | Number(q.y & _1n$1);
      return { sig, recovery };
  }
  function normalizePrivateKey(key) {
      let num;
      if (typeof key === 'bigint') {
          num = key;
      }
      else if (typeof key === 'number' && Number.isSafeInteger(key) && key > 0) {
          num = BigInt(key);
      }
      else if (typeof key === 'string') {
          if (key.length !== 64)
              throw new Error('Expected 32 bytes of private key');
          num = hexToNumber(key);
      }
      else if (key instanceof Uint8Array) {
          if (key.length !== 32)
              throw new Error('Expected 32 bytes of private key');
          num = bytesToNumber(key);
      }
      else {
          throw new TypeError('Expected valid private key');
      }
      if (!isWithinCurveOrder(num))
          throw new Error('Expected private key: 0 < key < n');
      return num;
  }
  function normalizeSignature(signature) {
      if (signature instanceof Signature) {
          signature.assertValidity();
          return signature;
      }
      try {
          return Signature.fromDER(signature);
      }
      catch (error) {
          return Signature.fromCompact(signature);
      }
  }
  function getPublicKey(privateKey, isCompressed = false) {
      return Point.fromPrivateKey(privateKey).toRawBytes(isCompressed);
  }
  function recoverPublicKey(msgHash, signature, recovery, isCompressed = false) {
      return Point.fromSignature(msgHash, signature, recovery).toRawBytes(isCompressed);
  }
  function bits2int(bytes) {
      const slice = bytes.length > 32 ? bytes.slice(0, 32) : bytes;
      return bytesToNumber(slice);
  }
  function bits2octets(bytes) {
      const z1 = bits2int(bytes);
      const z2 = mod(z1, CURVE.n);
      return int2octets(z2 < _0n$1 ? z1 : z2);
  }
  function int2octets(num) {
      return numTo32b(num);
  }
  function initSigArgs(msgHash, privateKey, extraEntropy) {
      if (msgHash == null)
          throw new Error(`sign: expected valid message hash, not "${msgHash}"`);
      const h1 = ensureBytes(msgHash);
      const d = normalizePrivateKey(privateKey);
      const seedArgs = [int2octets(d), bits2octets(h1)];
      if (extraEntropy != null) {
          if (extraEntropy === true)
              extraEntropy = utils$1.randomBytes(32);
          const e = ensureBytes(extraEntropy);
          if (e.length !== 32)
              throw new Error('sign: Expected 32 bytes of extra data');
          seedArgs.push(e);
      }
      const seed = concatBytes(...seedArgs);
      const m = bits2int(h1);
      return { seed, m, d };
  }
  function finalizeSig(recSig, opts) {
      let { sig, recovery } = recSig;
      const { canonical, der, recovered } = Object.assign({ canonical: true, der: true }, opts);
      if (canonical && sig.hasHighS()) {
          sig = sig.normalizeS();
          recovery ^= 1;
      }
      const hashed = der ? sig.toDERRawBytes() : sig.toCompactRawBytes();
      return recovered ? [hashed, recovery] : hashed;
  }
  function signSync(msgHash, privKey, opts = {}) {
      const { seed, m, d } = initSigArgs(msgHash, privKey, opts.extraEntropy);
      let sig;
      const drbg = new HmacDrbg();
      drbg.reseedSync(seed);
      while (!(sig = kmdToSig(drbg.generateSync(), m, d)))
          drbg.reseedSync();
      return finalizeSig(sig, opts);
  }
  Point.BASE._setWindowSize(8);
  const crypto = {
      node: crypto$2,
      web: typeof self === 'object' && 'crypto' in self ? self.crypto : undefined,
  };
  const TAGGED_HASH_PREFIXES = {};
  const utils$1 = {
      bytesToHex,
      hexToBytes,
      concatBytes,
      mod,
      invert,
      isValidPrivateKey(privateKey) {
          try {
              normalizePrivateKey(privateKey);
              return true;
          }
          catch (error) {
              return false;
          }
      },
      _bigintTo32Bytes: numTo32b,
      _normalizePrivateKey: normalizePrivateKey,
      hashToPrivateKey: (hash) => {
          hash = ensureBytes(hash);
          if (hash.length < 40 || hash.length > 1024)
              throw new Error('Expected 40-1024 bytes of private key as per FIPS 186');
          const num = mod(bytesToNumber(hash), CURVE.n - _1n$1) + _1n$1;
          return numTo32b(num);
      },
      randomBytes: (bytesLength = 32) => {
          if (crypto.web) {
              return crypto.web.getRandomValues(new Uint8Array(bytesLength));
          }
          else if (crypto.node) {
              const { randomBytes } = crypto.node;
              return Uint8Array.from(randomBytes(bytesLength));
          }
          else {
              throw new Error("The environment doesn't have randomBytes function");
          }
      },
      randomPrivateKey: () => {
          return utils$1.hashToPrivateKey(utils$1.randomBytes(40));
      },
      sha256: async (...messages) => {
          if (crypto.web) {
              const buffer = await crypto.web.subtle.digest('SHA-256', concatBytes(...messages));
              return new Uint8Array(buffer);
          }
          else if (crypto.node) {
              const { createHash } = crypto.node;
              const hash = createHash('sha256');
              messages.forEach((m) => hash.update(m));
              return Uint8Array.from(hash.digest());
          }
          else {
              throw new Error("The environment doesn't have sha256 function");
          }
      },
      hmacSha256: async (key, ...messages) => {
          if (crypto.web) {
              const ckey = await crypto.web.subtle.importKey('raw', key, { name: 'HMAC', hash: { name: 'SHA-256' } }, false, ['sign']);
              const message = concatBytes(...messages);
              const buffer = await crypto.web.subtle.sign('HMAC', ckey, message);
              return new Uint8Array(buffer);
          }
          else if (crypto.node) {
              const { createHmac } = crypto.node;
              const hash = createHmac('sha256', key);
              messages.forEach((m) => hash.update(m));
              return Uint8Array.from(hash.digest());
          }
          else {
              throw new Error("The environment doesn't have hmac-sha256 function");
          }
      },
      sha256Sync: undefined,
      hmacSha256Sync: undefined,
      taggedHash: async (tag, ...messages) => {
          let tagP = TAGGED_HASH_PREFIXES[tag];
          if (tagP === undefined) {
              const tagH = await utils$1.sha256(Uint8Array.from(tag, (c) => c.charCodeAt(0)));
              tagP = concatBytes(tagH, tagH);
              TAGGED_HASH_PREFIXES[tag] = tagP;
          }
          return utils$1.sha256(tagP, ...messages);
      },
      taggedHashSync: (tag, ...messages) => {
          if (typeof _sha256Sync !== 'function')
              throw new ShaError('sha256Sync is undefined, you need to set it');
          let tagP = TAGGED_HASH_PREFIXES[tag];
          if (tagP === undefined) {
              const tagH = _sha256Sync(Uint8Array.from(tag, (c) => c.charCodeAt(0)));
              tagP = concatBytes(tagH, tagH);
              TAGGED_HASH_PREFIXES[tag] = tagP;
          }
          return _sha256Sync(tagP, ...messages);
      },
      precompute(windowSize = 8, point = Point.BASE) {
          const cached = point === Point.BASE ? point : new Point(point.x, point.y);
          cached._setWindowSize(windowSize);
          cached.multiply(_3n);
          return cached;
      },
  };
  Object.defineProperties(utils$1, {
      sha256Sync: {
          configurable: false,
          get() {
              return _sha256Sync;
          },
          set(val) {
              if (!_sha256Sync)
                  _sha256Sync = val;
          },
      },
      hmacSha256Sync: {
          configurable: false,
          get() {
              return _hmacSha256Sync;
          },
          set(val) {
              if (!_hmacSha256Sync)
                  _hmacSha256Sync = val;
          },
      },
  });

  var commonjsGlobal = typeof globalThis !== 'undefined' ? globalThis : typeof window !== 'undefined' ? window : typeof global !== 'undefined' ? global : typeof self !== 'undefined' ? self : {};

  function getDefaultExportFromCjs (x) {
  	return x && x.__esModule && Object.prototype.hasOwnProperty.call(x, 'default') ? x['default'] : x;
  }

  function getAugmentedNamespace(n) {
    var f = n.default;
  	if (typeof f == "function") {
  		var a = function () {
  			return f.apply(this, arguments);
  		};
  		a.prototype = f.prototype;
    } else a = {};
    Object.defineProperty(a, '__esModule', {value: true});
  	Object.keys(n).forEach(function (k) {
  		var d = Object.getOwnPropertyDescriptor(n, k);
  		Object.defineProperty(a, k, d.get ? d : {
  			enumerable: true,
  			get: function () {
  				return n[k];
  			}
  		});
  	});
  	return a;
  }

  var browser = {};

  const require$$0$1 = /*@__PURE__*/getAugmentedNamespace(build);

  var packageInfo$2 = {};

  Object.defineProperty(packageInfo$2, "__esModule", {
    value: true
  });
  packageInfo$2.packageInfo = void 0;
  const packageInfo$1 = {
    name: '@polkadot/x-randomvalues',
    path: typeof __dirname === 'string' ? __dirname : 'auto',
    type: 'cjs',
    version: '10.1.12'
  };
  packageInfo$2.packageInfo = packageInfo$1;

  (function (exports) {
  	Object.defineProperty(exports, "__esModule", {
  	  value: true
  	});
  	exports.getRandomValues = getRandomValues;
  	Object.defineProperty(exports, "packageInfo", {
  	  enumerable: true,
  	  get: function () {
  	    return _packageInfo.packageInfo;
  	  }
  	});
  	var _xGlobal = require$$0$1;
  	var _packageInfo = packageInfo$2;
  	function getRandomValues(arr) {
  	  return _xGlobal.xglobal.crypto.getRandomValues(arr);
  	}
  } (browser));
  getDefaultExportFromCjs(browser);

  const DEFAULT_CRYPTO = {
    getRandomValues: browser.getRandomValues
  };
  const DEFAULT_SELF = {
    crypto: DEFAULT_CRYPTO
  };
  class Wbg {
    #bridge;
    constructor(bridge) {
      this.#bridge = bridge;
    }
    abort = () => {
      throw new Error('abort');
    };
    __wbindgen_is_undefined = idx => {
      return this.#bridge.getObject(idx) === undefined;
    };
    __wbindgen_throw = (ptr, len) => {
      throw new Error(this.#bridge.getString(ptr, len));
    };
    __wbg_self_1b7a39e3a92c949c = () => {
      return this.#bridge.addObject(DEFAULT_SELF);
    };
    __wbg_require_604837428532a733 = (ptr, len) => {
      throw new Error(`Unable to require ${this.#bridge.getString(ptr, len)}`);
    };
    __wbg_crypto_968f1772287e2df0 = _idx => {
      return this.#bridge.addObject(DEFAULT_CRYPTO);
    };
    __wbg_getRandomValues_a3d34b4fee3c2869 = _idx => {
      return this.#bridge.addObject(DEFAULT_CRYPTO.getRandomValues);
    };
    __wbg_getRandomValues_f5e14ab7ac8e995d = (_arg0, ptr, len) => {
      DEFAULT_CRYPTO.getRandomValues(this.#bridge.getU8a(ptr, len));
    };
    __wbg_randomFillSync_d5bd2d655fdf256a = (_idx, _ptr, _len) => {
      throw new Error('randomFillsync is not available');
    };
    __wbindgen_object_drop_ref = idx => {
      this.#bridge.takeObject(idx);
    };
  }

  class Bridge {
    #cachegetInt32;
    #cachegetUint8;
    #createWasm;
    #heap;
    #heapNext;
    #wasm;
    #wasmError;
    #wasmPromise;
    #wbg;
    #type;
    constructor(createWasm) {
      this.#createWasm = createWasm;
      this.#cachegetInt32 = null;
      this.#cachegetUint8 = null;
      this.#heap = new Array(32).fill(undefined).concat(undefined, null, true, false);
      this.#heapNext = this.#heap.length;
      this.#type = 'none';
      this.#wasm = null;
      this.#wasmError = null;
      this.#wasmPromise = null;
      this.#wbg = { ...new Wbg(this)
      };
    }
    get error() {
      return this.#wasmError;
    }
    get type() {
      return this.#type;
    }
    get wasm() {
      return this.#wasm;
    }
    async init(createWasm) {
      if (!this.#wasmPromise || createWasm) {
        this.#wasmPromise = (createWasm || this.#createWasm)(this.#wbg);
      }
      const {
        error,
        type,
        wasm
      } = await this.#wasmPromise;
      this.#type = type;
      this.#wasm = wasm;
      this.#wasmError = error;
      return this.#wasm;
    }
    getObject(idx) {
      return this.#heap[idx];
    }
    dropObject(idx) {
      if (idx < 36) {
        return;
      }
      this.#heap[idx] = this.#heapNext;
      this.#heapNext = idx;
    }
    takeObject(idx) {
      const ret = this.getObject(idx);
      this.dropObject(idx);
      return ret;
    }
    addObject(obj) {
      if (this.#heapNext === this.#heap.length) {
        this.#heap.push(this.#heap.length + 1);
      }
      const idx = this.#heapNext;
      this.#heapNext = this.#heap[idx];
      this.#heap[idx] = obj;
      return idx;
    }
    getInt32() {
      if (this.#cachegetInt32 === null || this.#cachegetInt32.buffer !== this.#wasm.memory.buffer) {
        this.#cachegetInt32 = new Int32Array(this.#wasm.memory.buffer);
      }
      return this.#cachegetInt32;
    }
    getUint8() {
      if (this.#cachegetUint8 === null || this.#cachegetUint8.buffer !== this.#wasm.memory.buffer) {
        this.#cachegetUint8 = new Uint8Array(this.#wasm.memory.buffer);
      }
      return this.#cachegetUint8;
    }
    getU8a(ptr, len) {
      return this.getUint8().subarray(ptr / 1, ptr / 1 + len);
    }
    getString(ptr, len) {
      return util.u8aToString(this.getU8a(ptr, len));
    }
    allocU8a(arg) {
      const ptr = this.#wasm.__wbindgen_malloc(arg.length * 1);
      this.getUint8().set(arg, ptr / 1);
      return [ptr, arg.length];
    }
    allocString(arg) {
      return this.allocU8a(util.stringToU8a(arg));
    }
    resultU8a() {
      const r0 = this.getInt32()[8 / 4 + 0];
      const r1 = this.getInt32()[8 / 4 + 1];
      const ret = this.getU8a(r0, r1).slice();
      this.#wasm.__wbindgen_free(r0, r1 * 1);
      return ret;
    }
    resultString() {
      return util.u8aToString(this.resultU8a());
    }
  }

  function createWasmFn(root, wasmBytes, asmFn) {
    return async wbg => {
      const result = {
        error: null,
        type: 'none',
        wasm: null
      };
      try {
        if (!wasmBytes || !wasmBytes.length) {
          throw new Error('No WebAssembly provided for initialization');
        } else if (typeof WebAssembly !== 'object' || typeof WebAssembly.instantiate !== 'function') {
          throw new Error('WebAssembly is not available in your environment');
        }
        const source = await WebAssembly.instantiate(wasmBytes, {
          wbg
        });
        result.wasm = source.instance.exports;
        result.type = 'wasm';
      } catch (error) {
        if (typeof asmFn === 'function') {
          result.wasm = asmFn(wbg);
          result.type = 'asm';
        } else {
          result.error = `FATAL: Unable to initialize @polkadot/wasm-${root}:: ${error.message}`;
          console.error(result.error);
        }
      }
      return result;
    };
  }

  const chr = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/';
  const map = new Array(256);
  for (let i = 0; i < chr.length; i++) {
    map[chr.charCodeAt(i)] = i;
  }
  function base64Decode$1(data, out) {
    const len = out.length;
    let byte = 0;
    let bits = 0;
    let pos = -1;
    for (let i = 0; pos < len; i++) {
      byte = byte << 6 | map[data.charCodeAt(i)];
      if ((bits += 6) >= 8) {
        out[++pos] = byte >>> (bits -= 8) & 0xff;
      }
    }
    return out;
  }

  const u8 = Uint8Array,
        u16 = Uint16Array,
        u32$1 = Uint32Array;
  const clim = new u8([16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15]);
  const fleb = new u8([0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0,
  0, 0,
  0]);
  const fdeb = new u8([0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13, 13,
  0, 0]);
  const freb = (eb, start) => {
    const b = new u16(31);
    for (let i = 0; i < 31; ++i) {
      b[i] = start += 1 << eb[i - 1];
    }
    const r = new u32$1(b[30]);
    for (let i = 1; i < 30; ++i) {
      for (let j = b[i]; j < b[i + 1]; ++j) {
        r[j] = j - b[i] << 5 | i;
      }
    }
    return [b, r];
  };
  const [fl, revfl] = freb(fleb, 2);
  fl[28] = 258, revfl[258] = 28;
  const [fd] = freb(fdeb, 0);
  const rev = new u16(32768);
  for (let i = 0; i < 32768; ++i) {
    let x = (i & 0xAAAA) >>> 1 | (i & 0x5555) << 1;
    x = (x & 0xCCCC) >>> 2 | (x & 0x3333) << 2;
    x = (x & 0xF0F0) >>> 4 | (x & 0x0F0F) << 4;
    rev[i] = ((x & 0xFF00) >>> 8 | (x & 0x00FF) << 8) >>> 1;
  }
  const hMap = (cd, mb, r) => {
    const s = cd.length;
    let i = 0;
    const l = new u16(mb);
    for (; i < s; ++i) ++l[cd[i] - 1];
    const le = new u16(mb);
    for (i = 0; i < mb; ++i) {
      le[i] = le[i - 1] + l[i - 1] << 1;
    }
    let co;
    if (r) {
      co = new u16(1 << mb);
      const rvb = 15 - mb;
      for (i = 0; i < s; ++i) {
        if (cd[i]) {
          const sv = i << 4 | cd[i];
          const r = mb - cd[i];
          let v = le[cd[i] - 1]++ << r;
          for (const m = v | (1 << r) - 1; v <= m; ++v) {
            co[rev[v] >>> rvb] = sv;
          }
        }
      }
    } else {
      co = new u16(s);
      for (i = 0; i < s; ++i) co[i] = rev[le[cd[i] - 1]++] >>> 15 - cd[i];
    }
    return co;
  };
  const flt = new u8(288);
  for (let i = 0; i < 144; ++i) flt[i] = 8;
  for (let i = 144; i < 256; ++i) flt[i] = 9;
  for (let i = 256; i < 280; ++i) flt[i] = 7;
  for (let i = 280; i < 288; ++i) flt[i] = 8;
  const fdt = new u8(32);
  for (let i = 0; i < 32; ++i) fdt[i] = 5;
  const flrm = hMap(flt, 9, 1);
  const fdrm = hMap(fdt, 5, 1);
  const bits = (d, p, m) => {
    const o = p >>> 3;
    return (d[o] | d[o + 1] << 8) >>> (p & 7) & m;
  };
  const bits16 = (d, p) => {
    const o = p >>> 3;
    return (d[o] | d[o + 1] << 8 | d[o + 2] << 16) >>> (p & 7);
  };
  const shft = p => (p >>> 3) + (p & 7 && 1);
  const slc = (v, s, e) => {
    if (s == null || s < 0) s = 0;
    if (e == null || e > v.length) e = v.length;
    const n = new (v instanceof u16 ? u16 : v instanceof u32$1 ? u32$1 : u8)(e - s);
    n.set(v.subarray(s, e));
    return n;
  };
  const max = a => {
    let m = a[0];
    for (let i = 1; i < a.length; ++i) {
      if (a[i] > m) m = a[i];
    }
    return m;
  };
  const inflt = (dat, buf, st) => {
    const noSt = !st || st.i;
    if (!st) st = {};
    const sl = dat.length;
    const noBuf = !buf || !noSt;
    if (!buf) buf = new u8(sl * 3);
    const cbuf = l => {
      let bl = buf.length;
      if (l > bl) {
        const nbuf = new u8(Math.max(bl << 1, l));
        nbuf.set(buf);
        buf = nbuf;
      }
    };
    let final = st.f || 0,
        pos = st.p || 0,
        bt = st.b || 0,
        lm = st.l,
        dm = st.d,
        lbt = st.m,
        dbt = st.n;
    if (final && !lm) return buf;
    const tbts = sl << 3;
    do {
      if (!lm) {
        st.f = final = bits(dat, pos, 1);
        const type = bits(dat, pos + 1, 3);
        pos += 3;
        if (!type) {
          const s = shft(pos) + 4,
                l = dat[s - 4] | dat[s - 3] << 8,
                t = s + l;
          if (t > sl) {
            if (noSt) throw 'unexpected EOF';
            break;
          }
          if (noBuf) cbuf(bt + l);
          buf.set(dat.subarray(s, t), bt);
          st.b = bt += l, st.p = pos = t << 3;
          continue;
        } else if (type == 1) lm = flrm, dm = fdrm, lbt = 9, dbt = 5;else if (type == 2) {
          const hLit = bits(dat, pos, 31) + 257,
                hcLen = bits(dat, pos + 10, 15) + 4;
          const tl = hLit + bits(dat, pos + 5, 31) + 1;
          pos += 14;
          const ldt = new u8(tl);
          const clt = new u8(19);
          for (let i = 0; i < hcLen; ++i) {
            clt[clim[i]] = bits(dat, pos + i * 3, 7);
          }
          pos += hcLen * 3;
          const clb = max(clt),
                clbmsk = (1 << clb) - 1;
          if (!noSt && pos + tl * (clb + 7) > tbts) break;
          const clm = hMap(clt, clb, 1);
          for (let i = 0; i < tl;) {
            const r = clm[bits(dat, pos, clbmsk)];
            pos += r & 15;
            const s = r >>> 4;
            if (s < 16) {
              ldt[i++] = s;
            } else {
              let c = 0,
                  n = 0;
              if (s == 16) n = 3 + bits(dat, pos, 3), pos += 2, c = ldt[i - 1];else if (s == 17) n = 3 + bits(dat, pos, 7), pos += 3;else if (s == 18) n = 11 + bits(dat, pos, 127), pos += 7;
              while (n--) ldt[i++] = c;
            }
          }
          const lt = ldt.subarray(0, hLit),
                dt = ldt.subarray(hLit);
          lbt = max(lt);
          dbt = max(dt);
          lm = hMap(lt, lbt, 1);
          dm = hMap(dt, dbt, 1);
        } else throw 'invalid block type';
        if (pos > tbts) throw 'unexpected EOF';
      }
      if (noBuf) cbuf(bt + 131072);
      const lms = (1 << lbt) - 1,
            dms = (1 << dbt) - 1;
      const mxa = lbt + dbt + 18;
      while (noSt || pos + mxa < tbts) {
        const c = lm[bits16(dat, pos) & lms],
              sym = c >>> 4;
        pos += c & 15;
        if (pos > tbts) throw 'unexpected EOF';
        if (!c) throw 'invalid length/literal';
        if (sym < 256) buf[bt++] = sym;else if (sym == 256) {
          lm = undefined;
          break;
        } else {
          let add = sym - 254;
          if (sym > 264) {
            const i = sym - 257,
                  b = fleb[i];
            add = bits(dat, pos, (1 << b) - 1) + fl[i];
            pos += b;
          }
          const d = dm[bits16(dat, pos) & dms],
                dsym = d >>> 4;
          if (!d) throw 'invalid distance';
          pos += d & 15;
          let dt = fd[dsym];
          if (dsym > 3) {
            const b = fdeb[dsym];
            dt += bits16(dat, pos) & (1 << b) - 1, pos += b;
          }
          if (pos > tbts) throw 'unexpected EOF';
          if (noBuf) cbuf(bt + 131072);
          const end = bt + add;
          for (; bt < end; bt += 4) {
            buf[bt] = buf[bt - dt];
            buf[bt + 1] = buf[bt + 1 - dt];
            buf[bt + 2] = buf[bt + 2 - dt];
            buf[bt + 3] = buf[bt + 3 - dt];
          }
          bt = end;
        }
      }
      st.l = lm, st.p = pos, st.b = bt;
      if (lm) final = 1, st.m = lbt, st.d = dm, st.n = dbt;
    } while (!final);
    return bt == buf.length ? buf : slc(buf, 0, bt);
  };
  const zlv = d => {
    if ((d[0] & 15) != 8 || d[0] >>> 4 > 7 || (d[0] << 8 | d[1]) % 31) throw 'invalid zlib data';
    if (d[1] & 32) throw 'invalid zlib data: preset dictionaries not supported';
  };
  function unzlibSync(data, out) {
    return inflt((zlv(data), data.subarray(2, -4)), out);
  }

  const lenIn = 171116;
  const lenOut = 339508;
  const bytes$1 = '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';
  var bytes_1 = { bytes: bytes$1, lenIn, lenOut };

  const wasmBytes = unzlibSync(base64Decode$1(bytes_1.bytes, new Uint8Array(bytes_1.lenIn)), new Uint8Array(bytes_1.lenOut));

  const createWasm = createWasmFn('crypto', wasmBytes, null);

  const bridge = new Bridge(createWasm);
  async function initBridge(createWasm) {
    return bridge.init(createWasm);
  }

  function withWasm(fn) {
    return (...params) => {
      if (!bridge.wasm) {
        throw new Error('The WASM interface has not been initialized. Ensure that you wait for the initialization Promise with waitReady() from @polkadot/wasm-crypto (or cryptoWaitReady() from @polkadot/util-crypto) before attempting to use WASM-only interfaces.');
      }
      return fn(bridge.wasm, ...params);
    };
  }
  const bip39Generate = withWasm((wasm, words) => {
    wasm.ext_bip39_generate(8, words);
    return bridge.resultString();
  });
  const bip39ToEntropy = withWasm((wasm, phrase) => {
    wasm.ext_bip39_to_entropy(8, ...bridge.allocString(phrase));
    return bridge.resultU8a();
  });
  const bip39ToMiniSecret = withWasm((wasm, phrase, password) => {
    wasm.ext_bip39_to_mini_secret(8, ...bridge.allocString(phrase), ...bridge.allocString(password));
    return bridge.resultU8a();
  });
  const bip39ToSeed = withWasm((wasm, phrase, password) => {
    wasm.ext_bip39_to_seed(8, ...bridge.allocString(phrase), ...bridge.allocString(password));
    return bridge.resultU8a();
  });
  const bip39Validate = withWasm((wasm, phrase) => {
    const ret = wasm.ext_bip39_validate(...bridge.allocString(phrase));
    return ret !== 0;
  });
  const ed25519KeypairFromSeed = withWasm((wasm, seed) => {
    wasm.ext_ed_from_seed(8, ...bridge.allocU8a(seed));
    return bridge.resultU8a();
  });
  const ed25519Sign$1 = withWasm((wasm, pubkey, seckey, message) => {
    wasm.ext_ed_sign(8, ...bridge.allocU8a(pubkey), ...bridge.allocU8a(seckey), ...bridge.allocU8a(message));
    return bridge.resultU8a();
  });
  const ed25519Verify$1 = withWasm((wasm, signature, message, pubkey) => {
    const ret = wasm.ext_ed_verify(...bridge.allocU8a(signature), ...bridge.allocU8a(message), ...bridge.allocU8a(pubkey));
    return ret !== 0;
  });
  const secp256k1FromSeed = withWasm((wasm, seckey) => {
    wasm.ext_secp_from_seed(8, ...bridge.allocU8a(seckey));
    return bridge.resultU8a();
  });
  const secp256k1Compress$1 = withWasm((wasm, pubkey) => {
    wasm.ext_secp_pub_compress(8, ...bridge.allocU8a(pubkey));
    return bridge.resultU8a();
  });
  const secp256k1Expand$1 = withWasm((wasm, pubkey) => {
    wasm.ext_secp_pub_expand(8, ...bridge.allocU8a(pubkey));
    return bridge.resultU8a();
  });
  const secp256k1Recover$1 = withWasm((wasm, msgHash, sig, recovery) => {
    wasm.ext_secp_recover(8, ...bridge.allocU8a(msgHash), ...bridge.allocU8a(sig), recovery);
    return bridge.resultU8a();
  });
  const secp256k1Sign$1 = withWasm((wasm, msgHash, seckey) => {
    wasm.ext_secp_sign(8, ...bridge.allocU8a(msgHash), ...bridge.allocU8a(seckey));
    return bridge.resultU8a();
  });
  const sr25519DeriveKeypairHard = withWasm((wasm, pair, cc) => {
    wasm.ext_sr_derive_keypair_hard(8, ...bridge.allocU8a(pair), ...bridge.allocU8a(cc));
    return bridge.resultU8a();
  });
  const sr25519DeriveKeypairSoft = withWasm((wasm, pair, cc) => {
    wasm.ext_sr_derive_keypair_soft(8, ...bridge.allocU8a(pair), ...bridge.allocU8a(cc));
    return bridge.resultU8a();
  });
  const sr25519DerivePublicSoft = withWasm((wasm, pubkey, cc) => {
    wasm.ext_sr_derive_public_soft(8, ...bridge.allocU8a(pubkey), ...bridge.allocU8a(cc));
    return bridge.resultU8a();
  });
  const sr25519KeypairFromSeed = withWasm((wasm, seed) => {
    wasm.ext_sr_from_seed(8, ...bridge.allocU8a(seed));
    return bridge.resultU8a();
  });
  const sr25519Sign$1 = withWasm((wasm, pubkey, secret, message) => {
    wasm.ext_sr_sign(8, ...bridge.allocU8a(pubkey), ...bridge.allocU8a(secret), ...bridge.allocU8a(message));
    return bridge.resultU8a();
  });
  const sr25519Verify$1 = withWasm((wasm, signature, message, pubkey) => {
    const ret = wasm.ext_sr_verify(...bridge.allocU8a(signature), ...bridge.allocU8a(message), ...bridge.allocU8a(pubkey));
    return ret !== 0;
  });
  const sr25519Agree = withWasm((wasm, pubkey, secret) => {
    wasm.ext_sr_agree(8, ...bridge.allocU8a(pubkey), ...bridge.allocU8a(secret));
    return bridge.resultU8a();
  });
  const vrfSign = withWasm((wasm, secret, context, message, extra) => {
    wasm.ext_vrf_sign(8, ...bridge.allocU8a(secret), ...bridge.allocU8a(context), ...bridge.allocU8a(message), ...bridge.allocU8a(extra));
    return bridge.resultU8a();
  });
  const vrfVerify = withWasm((wasm, pubkey, context, message, extra, outAndProof) => {
    const ret = wasm.ext_vrf_verify(...bridge.allocU8a(pubkey), ...bridge.allocU8a(context), ...bridge.allocU8a(message), ...bridge.allocU8a(extra), ...bridge.allocU8a(outAndProof));
    return ret !== 0;
  });
  const blake2b$1 = withWasm((wasm, data, key, size) => {
    wasm.ext_blake2b(8, ...bridge.allocU8a(data), ...bridge.allocU8a(key), size);
    return bridge.resultU8a();
  });
  const hmacSha256 = withWasm((wasm, key, data) => {
    wasm.ext_hmac_sha256(8, ...bridge.allocU8a(key), ...bridge.allocU8a(data));
    return bridge.resultU8a();
  });
  const hmacSha512 = withWasm((wasm, key, data) => {
    wasm.ext_hmac_sha512(8, ...bridge.allocU8a(key), ...bridge.allocU8a(data));
    return bridge.resultU8a();
  });
  const keccak256 = withWasm((wasm, data) => {
    wasm.ext_keccak256(8, ...bridge.allocU8a(data));
    return bridge.resultU8a();
  });
  const keccak512 = withWasm((wasm, data) => {
    wasm.ext_keccak512(8, ...bridge.allocU8a(data));
    return bridge.resultU8a();
  });
  const pbkdf2$1 = withWasm((wasm, data, salt, rounds) => {
    wasm.ext_pbkdf2(8, ...bridge.allocU8a(data), ...bridge.allocU8a(salt), rounds);
    return bridge.resultU8a();
  });
  const scrypt$1 = withWasm((wasm, password, salt, log2n, r, p) => {
    wasm.ext_scrypt(8, ...bridge.allocU8a(password), ...bridge.allocU8a(salt), log2n, r, p);
    return bridge.resultU8a();
  });
  const sha256$1 = withWasm((wasm, data) => {
    wasm.ext_sha256(8, ...bridge.allocU8a(data));
    return bridge.resultU8a();
  });
  const sha512$1 = withWasm((wasm, data) => {
    wasm.ext_sha512(8, ...bridge.allocU8a(data));
    return bridge.resultU8a();
  });
  const twox = withWasm((wasm, data, rounds) => {
    wasm.ext_twox(8, ...bridge.allocU8a(data), rounds);
    return bridge.resultU8a();
  });
  function isReady() {
    return !!bridge.wasm;
  }
  async function waitReady() {
    try {
      const wasm = await initBridge();
      return !!wasm;
    } catch {
      return false;
    }
  }

  const cryptoIsReady = isReady;
  function cryptoWaitReady() {
    return waitReady().then(() => {
      if (!isReady()) {
        throw new Error('Unable to initialize @polkadot/util-crypto');
      }
      return true;
    }).catch(() => false);
  }

  function number(n) {
      if (!Number.isSafeInteger(n) || n < 0)
          throw new Error(`Wrong positive integer: ${n}`);
  }
  function bool(b) {
      if (typeof b !== 'boolean')
          throw new Error(`Expected boolean, not ${b}`);
  }
  function bytes(b, ...lengths) {
      if (!(b instanceof Uint8Array))
          throw new TypeError('Expected Uint8Array');
      if (lengths.length > 0 && !lengths.includes(b.length))
          throw new TypeError(`Expected Uint8Array of length ${lengths}, not of length=${b.length}`);
  }
  function hash(hash) {
      if (typeof hash !== 'function' || typeof hash.create !== 'function')
          throw new Error('Hash should be wrapped by utils.wrapConstructor');
      number(hash.outputLen);
      number(hash.blockLen);
  }
  function exists(instance, checkFinished = true) {
      if (instance.destroyed)
          throw new Error('Hash instance has been destroyed');
      if (checkFinished && instance.finished)
          throw new Error('Hash#digest() has already been called');
  }
  function output(out, instance) {
      bytes(out);
      const min = instance.outputLen;
      if (out.length < min) {
          throw new Error(`digestInto() expects output buffer of length at least ${min}`);
      }
  }
  const assert = {
      number,
      bool,
      bytes,
      hash,
      exists,
      output,
  };

  /*! noble-hashes - MIT License (c) 2022 Paul Miller (paulmillr.com) */
  const u32 = (arr) => new Uint32Array(arr.buffer, arr.byteOffset, Math.floor(arr.byteLength / 4));
  const createView = (arr) => new DataView(arr.buffer, arr.byteOffset, arr.byteLength);
  const rotr = (word, shift) => (word << (32 - shift)) | (word >>> shift);
  const isLE = new Uint8Array(new Uint32Array([0x11223344]).buffer)[0] === 0x44;
  if (!isLE)
      throw new Error('Non little-endian hardware is not supported');
  Array.from({ length: 256 }, (v, i) => i.toString(16).padStart(2, '0'));
  function utf8ToBytes(str) {
      if (typeof str !== 'string') {
          throw new TypeError(`utf8ToBytes expected string, got ${typeof str}`);
      }
      return new TextEncoder().encode(str);
  }
  function toBytes(data) {
      if (typeof data === 'string')
          data = utf8ToBytes(data);
      if (!(data instanceof Uint8Array))
          throw new TypeError(`Expected input type is Uint8Array (got ${typeof data})`);
      return data;
  }
  class Hash {
      clone() {
          return this._cloneInto();
      }
  }
  const isPlainObject = (obj) => Object.prototype.toString.call(obj) === '[object Object]' && obj.constructor === Object;
  function checkOpts(defaults, opts) {
      if (opts !== undefined && (typeof opts !== 'object' || !isPlainObject(opts)))
          throw new TypeError('Options should be object or undefined');
      const merged = Object.assign(defaults, opts);
      return merged;
  }
  function wrapConstructor(hashConstructor) {
      const hashC = (message) => hashConstructor().update(toBytes(message)).digest();
      const tmp = hashConstructor();
      hashC.outputLen = tmp.outputLen;
      hashC.blockLen = tmp.blockLen;
      hashC.create = () => hashConstructor();
      return hashC;
  }
  function wrapConstructorWithOpts(hashCons) {
      const hashC = (msg, opts) => hashCons(opts).update(toBytes(msg)).digest();
      const tmp = hashCons({});
      hashC.outputLen = tmp.outputLen;
      hashC.blockLen = tmp.blockLen;
      hashC.create = (opts) => hashCons(opts);
      return hashC;
  }

  class HMAC extends Hash {
      constructor(hash, _key) {
          super();
          this.finished = false;
          this.destroyed = false;
          assert.hash(hash);
          const key = toBytes(_key);
          this.iHash = hash.create();
          if (typeof this.iHash.update !== 'function')
              throw new TypeError('Expected instance of class which extends utils.Hash');
          this.blockLen = this.iHash.blockLen;
          this.outputLen = this.iHash.outputLen;
          const blockLen = this.blockLen;
          const pad = new Uint8Array(blockLen);
          pad.set(key.length > blockLen ? hash.create().update(key).digest() : key);
          for (let i = 0; i < pad.length; i++)
              pad[i] ^= 0x36;
          this.iHash.update(pad);
          this.oHash = hash.create();
          for (let i = 0; i < pad.length; i++)
              pad[i] ^= 0x36 ^ 0x5c;
          this.oHash.update(pad);
          pad.fill(0);
      }
      update(buf) {
          assert.exists(this);
          this.iHash.update(buf);
          return this;
      }
      digestInto(out) {
          assert.exists(this);
          assert.bytes(out, this.outputLen);
          this.finished = true;
          this.iHash.digestInto(out);
          this.oHash.update(out);
          this.oHash.digestInto(out);
          this.destroy();
      }
      digest() {
          const out = new Uint8Array(this.oHash.outputLen);
          this.digestInto(out);
          return out;
      }
      _cloneInto(to) {
          to || (to = Object.create(Object.getPrototypeOf(this), {}));
          const { oHash, iHash, finished, destroyed, blockLen, outputLen } = this;
          to = to;
          to.finished = finished;
          to.destroyed = destroyed;
          to.blockLen = blockLen;
          to.outputLen = outputLen;
          to.oHash = oHash._cloneInto(to.oHash);
          to.iHash = iHash._cloneInto(to.iHash);
          return to;
      }
      destroy() {
          this.destroyed = true;
          this.oHash.destroy();
          this.iHash.destroy();
      }
  }
  const hmac = (hash, key, message) => new HMAC(hash, key).update(message).digest();
  hmac.create = (hash, key) => new HMAC(hash, key);

  function setBigUint64(view, byteOffset, value, isLE) {
      if (typeof view.setBigUint64 === 'function')
          return view.setBigUint64(byteOffset, value, isLE);
      const _32n = BigInt(32);
      const _u32_max = BigInt(0xffffffff);
      const wh = Number((value >> _32n) & _u32_max);
      const wl = Number(value & _u32_max);
      const h = isLE ? 4 : 0;
      const l = isLE ? 0 : 4;
      view.setUint32(byteOffset + h, wh, isLE);
      view.setUint32(byteOffset + l, wl, isLE);
  }
  class SHA2 extends Hash {
      constructor(blockLen, outputLen, padOffset, isLE) {
          super();
          this.blockLen = blockLen;
          this.outputLen = outputLen;
          this.padOffset = padOffset;
          this.isLE = isLE;
          this.finished = false;
          this.length = 0;
          this.pos = 0;
          this.destroyed = false;
          this.buffer = new Uint8Array(blockLen);
          this.view = createView(this.buffer);
      }
      update(data) {
          assert.exists(this);
          const { view, buffer, blockLen } = this;
          data = toBytes(data);
          const len = data.length;
          for (let pos = 0; pos < len;) {
              const take = Math.min(blockLen - this.pos, len - pos);
              if (take === blockLen) {
                  const dataView = createView(data);
                  for (; blockLen <= len - pos; pos += blockLen)
                      this.process(dataView, pos);
                  continue;
              }
              buffer.set(data.subarray(pos, pos + take), this.pos);
              this.pos += take;
              pos += take;
              if (this.pos === blockLen) {
                  this.process(view, 0);
                  this.pos = 0;
              }
          }
          this.length += data.length;
          this.roundClean();
          return this;
      }
      digestInto(out) {
          assert.exists(this);
          assert.output(out, this);
          this.finished = true;
          const { buffer, view, blockLen, isLE } = this;
          let { pos } = this;
          buffer[pos++] = 0b10000000;
          this.buffer.subarray(pos).fill(0);
          if (this.padOffset > blockLen - pos) {
              this.process(view, 0);
              pos = 0;
          }
          for (let i = pos; i < blockLen; i++)
              buffer[i] = 0;
          setBigUint64(view, blockLen - 8, BigInt(this.length * 8), isLE);
          this.process(view, 0);
          const oview = createView(out);
          this.get().forEach((v, i) => oview.setUint32(4 * i, v, isLE));
      }
      digest() {
          const { buffer, outputLen } = this;
          this.digestInto(buffer);
          const res = buffer.slice(0, outputLen);
          this.destroy();
          return res;
      }
      _cloneInto(to) {
          to || (to = new this.constructor());
          to.set(...this.get());
          const { blockLen, buffer, length, finished, destroyed, pos } = this;
          to.length = length;
          to.pos = pos;
          to.finished = finished;
          to.destroyed = destroyed;
          if (length % blockLen)
              to.buffer.set(buffer);
          return to;
      }
  }

  const Chi = (a, b, c) => (a & b) ^ (~a & c);
  const Maj = (a, b, c) => (a & b) ^ (a & c) ^ (b & c);
  const SHA256_K = new Uint32Array([
      0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
      0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
      0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
      0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
      0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
      0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
      0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
      0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2
  ]);
  const IV$1 = new Uint32Array([
      0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19
  ]);
  const SHA256_W = new Uint32Array(64);
  class SHA256 extends SHA2 {
      constructor() {
          super(64, 32, 8, false);
          this.A = IV$1[0] | 0;
          this.B = IV$1[1] | 0;
          this.C = IV$1[2] | 0;
          this.D = IV$1[3] | 0;
          this.E = IV$1[4] | 0;
          this.F = IV$1[5] | 0;
          this.G = IV$1[6] | 0;
          this.H = IV$1[7] | 0;
      }
      get() {
          const { A, B, C, D, E, F, G, H } = this;
          return [A, B, C, D, E, F, G, H];
      }
      set(A, B, C, D, E, F, G, H) {
          this.A = A | 0;
          this.B = B | 0;
          this.C = C | 0;
          this.D = D | 0;
          this.E = E | 0;
          this.F = F | 0;
          this.G = G | 0;
          this.H = H | 0;
      }
      process(view, offset) {
          for (let i = 0; i < 16; i++, offset += 4)
              SHA256_W[i] = view.getUint32(offset, false);
          for (let i = 16; i < 64; i++) {
              const W15 = SHA256_W[i - 15];
              const W2 = SHA256_W[i - 2];
              const s0 = rotr(W15, 7) ^ rotr(W15, 18) ^ (W15 >>> 3);
              const s1 = rotr(W2, 17) ^ rotr(W2, 19) ^ (W2 >>> 10);
              SHA256_W[i] = (s1 + SHA256_W[i - 7] + s0 + SHA256_W[i - 16]) | 0;
          }
          let { A, B, C, D, E, F, G, H } = this;
          for (let i = 0; i < 64; i++) {
              const sigma1 = rotr(E, 6) ^ rotr(E, 11) ^ rotr(E, 25);
              const T1 = (H + sigma1 + Chi(E, F, G) + SHA256_K[i] + SHA256_W[i]) | 0;
              const sigma0 = rotr(A, 2) ^ rotr(A, 13) ^ rotr(A, 22);
              const T2 = (sigma0 + Maj(A, B, C)) | 0;
              H = G;
              G = F;
              F = E;
              E = (D + T1) | 0;
              D = C;
              C = B;
              B = A;
              A = (T1 + T2) | 0;
          }
          A = (A + this.A) | 0;
          B = (B + this.B) | 0;
          C = (C + this.C) | 0;
          D = (D + this.D) | 0;
          E = (E + this.E) | 0;
          F = (F + this.F) | 0;
          G = (G + this.G) | 0;
          H = (H + this.H) | 0;
          this.set(A, B, C, D, E, F, G, H);
      }
      roundClean() {
          SHA256_W.fill(0);
      }
      destroy() {
          this.set(0, 0, 0, 0, 0, 0, 0, 0);
          this.buffer.fill(0);
      }
  }
  const sha256 = wrapConstructor(() => new SHA256());

  const U32_MASK64 = BigInt(2 ** 32 - 1);
  const _32n$1 = BigInt(32);
  function fromBig(n, le = false) {
      if (le)
          return { h: Number(n & U32_MASK64), l: Number((n >> _32n$1) & U32_MASK64) };
      return { h: Number((n >> _32n$1) & U32_MASK64) | 0, l: Number(n & U32_MASK64) | 0 };
  }
  function split(lst, le = false) {
      let Ah = new Uint32Array(lst.length);
      let Al = new Uint32Array(lst.length);
      for (let i = 0; i < lst.length; i++) {
          const { h, l } = fromBig(lst[i], le);
          [Ah[i], Al[i]] = [h, l];
      }
      return [Ah, Al];
  }
  const toBig = (h, l) => (BigInt(h >>> 0) << _32n$1) | BigInt(l >>> 0);
  const shrSH = (h, l, s) => h >>> s;
  const shrSL = (h, l, s) => (h << (32 - s)) | (l >>> s);
  const rotrSH = (h, l, s) => (h >>> s) | (l << (32 - s));
  const rotrSL = (h, l, s) => (h << (32 - s)) | (l >>> s);
  const rotrBH = (h, l, s) => (h << (64 - s)) | (l >>> (s - 32));
  const rotrBL = (h, l, s) => (h >>> (s - 32)) | (l << (64 - s));
  const rotr32H = (h, l) => l;
  const rotr32L = (h, l) => h;
  const rotlSH = (h, l, s) => (h << s) | (l >>> (32 - s));
  const rotlSL = (h, l, s) => (l << s) | (h >>> (32 - s));
  const rotlBH = (h, l, s) => (l << (s - 32)) | (h >>> (64 - s));
  const rotlBL = (h, l, s) => (h << (s - 32)) | (l >>> (64 - s));
  function add(Ah, Al, Bh, Bl) {
      const l = (Al >>> 0) + (Bl >>> 0);
      return { h: (Ah + Bh + ((l / 2 ** 32) | 0)) | 0, l: l | 0 };
  }
  const add3L = (Al, Bl, Cl) => (Al >>> 0) + (Bl >>> 0) + (Cl >>> 0);
  const add3H = (low, Ah, Bh, Ch) => (Ah + Bh + Ch + ((low / 2 ** 32) | 0)) | 0;
  const add4L = (Al, Bl, Cl, Dl) => (Al >>> 0) + (Bl >>> 0) + (Cl >>> 0) + (Dl >>> 0);
  const add4H = (low, Ah, Bh, Ch, Dh) => (Ah + Bh + Ch + Dh + ((low / 2 ** 32) | 0)) | 0;
  const add5L = (Al, Bl, Cl, Dl, El) => (Al >>> 0) + (Bl >>> 0) + (Cl >>> 0) + (Dl >>> 0) + (El >>> 0);
  const add5H = (low, Ah, Bh, Ch, Dh, Eh) => (Ah + Bh + Ch + Dh + Eh + ((low / 2 ** 32) | 0)) | 0;
  const u64 = {
      fromBig, split, toBig,
      shrSH, shrSL,
      rotrSH, rotrSL, rotrBH, rotrBL,
      rotr32H, rotr32L,
      rotlSH, rotlSL, rotlBH, rotlBL,
      add, add3L, add3H, add4L, add4H, add5H, add5L,
  };

  const [SHA512_Kh, SHA512_Kl] = u64.split([
      '0x428a2f98d728ae22', '0x7137449123ef65cd', '0xb5c0fbcfec4d3b2f', '0xe9b5dba58189dbbc',
      '0x3956c25bf348b538', '0x59f111f1b605d019', '0x923f82a4af194f9b', '0xab1c5ed5da6d8118',
      '0xd807aa98a3030242', '0x12835b0145706fbe', '0x243185be4ee4b28c', '0x550c7dc3d5ffb4e2',
      '0x72be5d74f27b896f', '0x80deb1fe3b1696b1', '0x9bdc06a725c71235', '0xc19bf174cf692694',
      '0xe49b69c19ef14ad2', '0xefbe4786384f25e3', '0x0fc19dc68b8cd5b5', '0x240ca1cc77ac9c65',
      '0x2de92c6f592b0275', '0x4a7484aa6ea6e483', '0x5cb0a9dcbd41fbd4', '0x76f988da831153b5',
      '0x983e5152ee66dfab', '0xa831c66d2db43210', '0xb00327c898fb213f', '0xbf597fc7beef0ee4',
      '0xc6e00bf33da88fc2', '0xd5a79147930aa725', '0x06ca6351e003826f', '0x142929670a0e6e70',
      '0x27b70a8546d22ffc', '0x2e1b21385c26c926', '0x4d2c6dfc5ac42aed', '0x53380d139d95b3df',
      '0x650a73548baf63de', '0x766a0abb3c77b2a8', '0x81c2c92e47edaee6', '0x92722c851482353b',
      '0xa2bfe8a14cf10364', '0xa81a664bbc423001', '0xc24b8b70d0f89791', '0xc76c51a30654be30',
      '0xd192e819d6ef5218', '0xd69906245565a910', '0xf40e35855771202a', '0x106aa07032bbd1b8',
      '0x19a4c116b8d2d0c8', '0x1e376c085141ab53', '0x2748774cdf8eeb99', '0x34b0bcb5e19b48a8',
      '0x391c0cb3c5c95a63', '0x4ed8aa4ae3418acb', '0x5b9cca4f7763e373', '0x682e6ff3d6b2b8a3',
      '0x748f82ee5defb2fc', '0x78a5636f43172f60', '0x84c87814a1f0ab72', '0x8cc702081a6439ec',
      '0x90befffa23631e28', '0xa4506cebde82bde9', '0xbef9a3f7b2c67915', '0xc67178f2e372532b',
      '0xca273eceea26619c', '0xd186b8c721c0c207', '0xeada7dd6cde0eb1e', '0xf57d4f7fee6ed178',
      '0x06f067aa72176fba', '0x0a637dc5a2c898a6', '0x113f9804bef90dae', '0x1b710b35131c471b',
      '0x28db77f523047d84', '0x32caab7b40c72493', '0x3c9ebe0a15c9bebc', '0x431d67c49c100d4c',
      '0x4cc5d4becb3e42b6', '0x597f299cfc657e2a', '0x5fcb6fab3ad6faec', '0x6c44198c4a475817'
  ].map(n => BigInt(n)));
  const SHA512_W_H = new Uint32Array(80);
  const SHA512_W_L = new Uint32Array(80);
  class SHA512 extends SHA2 {
      constructor() {
          super(128, 64, 16, false);
          this.Ah = 0x6a09e667 | 0;
          this.Al = 0xf3bcc908 | 0;
          this.Bh = 0xbb67ae85 | 0;
          this.Bl = 0x84caa73b | 0;
          this.Ch = 0x3c6ef372 | 0;
          this.Cl = 0xfe94f82b | 0;
          this.Dh = 0xa54ff53a | 0;
          this.Dl = 0x5f1d36f1 | 0;
          this.Eh = 0x510e527f | 0;
          this.El = 0xade682d1 | 0;
          this.Fh = 0x9b05688c | 0;
          this.Fl = 0x2b3e6c1f | 0;
          this.Gh = 0x1f83d9ab | 0;
          this.Gl = 0xfb41bd6b | 0;
          this.Hh = 0x5be0cd19 | 0;
          this.Hl = 0x137e2179 | 0;
      }
      get() {
          const { Ah, Al, Bh, Bl, Ch, Cl, Dh, Dl, Eh, El, Fh, Fl, Gh, Gl, Hh, Hl } = this;
          return [Ah, Al, Bh, Bl, Ch, Cl, Dh, Dl, Eh, El, Fh, Fl, Gh, Gl, Hh, Hl];
      }
      set(Ah, Al, Bh, Bl, Ch, Cl, Dh, Dl, Eh, El, Fh, Fl, Gh, Gl, Hh, Hl) {
          this.Ah = Ah | 0;
          this.Al = Al | 0;
          this.Bh = Bh | 0;
          this.Bl = Bl | 0;
          this.Ch = Ch | 0;
          this.Cl = Cl | 0;
          this.Dh = Dh | 0;
          this.Dl = Dl | 0;
          this.Eh = Eh | 0;
          this.El = El | 0;
          this.Fh = Fh | 0;
          this.Fl = Fl | 0;
          this.Gh = Gh | 0;
          this.Gl = Gl | 0;
          this.Hh = Hh | 0;
          this.Hl = Hl | 0;
      }
      process(view, offset) {
          for (let i = 0; i < 16; i++, offset += 4) {
              SHA512_W_H[i] = view.getUint32(offset);
              SHA512_W_L[i] = view.getUint32((offset += 4));
          }
          for (let i = 16; i < 80; i++) {
              const W15h = SHA512_W_H[i - 15] | 0;
              const W15l = SHA512_W_L[i - 15] | 0;
              const s0h = u64.rotrSH(W15h, W15l, 1) ^ u64.rotrSH(W15h, W15l, 8) ^ u64.shrSH(W15h, W15l, 7);
              const s0l = u64.rotrSL(W15h, W15l, 1) ^ u64.rotrSL(W15h, W15l, 8) ^ u64.shrSL(W15h, W15l, 7);
              const W2h = SHA512_W_H[i - 2] | 0;
              const W2l = SHA512_W_L[i - 2] | 0;
              const s1h = u64.rotrSH(W2h, W2l, 19) ^ u64.rotrBH(W2h, W2l, 61) ^ u64.shrSH(W2h, W2l, 6);
              const s1l = u64.rotrSL(W2h, W2l, 19) ^ u64.rotrBL(W2h, W2l, 61) ^ u64.shrSL(W2h, W2l, 6);
              const SUMl = u64.add4L(s0l, s1l, SHA512_W_L[i - 7], SHA512_W_L[i - 16]);
              const SUMh = u64.add4H(SUMl, s0h, s1h, SHA512_W_H[i - 7], SHA512_W_H[i - 16]);
              SHA512_W_H[i] = SUMh | 0;
              SHA512_W_L[i] = SUMl | 0;
          }
          let { Ah, Al, Bh, Bl, Ch, Cl, Dh, Dl, Eh, El, Fh, Fl, Gh, Gl, Hh, Hl } = this;
          for (let i = 0; i < 80; i++) {
              const sigma1h = u64.rotrSH(Eh, El, 14) ^ u64.rotrSH(Eh, El, 18) ^ u64.rotrBH(Eh, El, 41);
              const sigma1l = u64.rotrSL(Eh, El, 14) ^ u64.rotrSL(Eh, El, 18) ^ u64.rotrBL(Eh, El, 41);
              const CHIh = (Eh & Fh) ^ (~Eh & Gh);
              const CHIl = (El & Fl) ^ (~El & Gl);
              const T1ll = u64.add5L(Hl, sigma1l, CHIl, SHA512_Kl[i], SHA512_W_L[i]);
              const T1h = u64.add5H(T1ll, Hh, sigma1h, CHIh, SHA512_Kh[i], SHA512_W_H[i]);
              const T1l = T1ll | 0;
              const sigma0h = u64.rotrSH(Ah, Al, 28) ^ u64.rotrBH(Ah, Al, 34) ^ u64.rotrBH(Ah, Al, 39);
              const sigma0l = u64.rotrSL(Ah, Al, 28) ^ u64.rotrBL(Ah, Al, 34) ^ u64.rotrBL(Ah, Al, 39);
              const MAJh = (Ah & Bh) ^ (Ah & Ch) ^ (Bh & Ch);
              const MAJl = (Al & Bl) ^ (Al & Cl) ^ (Bl & Cl);
              Hh = Gh | 0;
              Hl = Gl | 0;
              Gh = Fh | 0;
              Gl = Fl | 0;
              Fh = Eh | 0;
              Fl = El | 0;
              ({ h: Eh, l: El } = u64.add(Dh | 0, Dl | 0, T1h | 0, T1l | 0));
              Dh = Ch | 0;
              Dl = Cl | 0;
              Ch = Bh | 0;
              Cl = Bl | 0;
              Bh = Ah | 0;
              Bl = Al | 0;
              const All = u64.add3L(T1l, sigma0l, MAJl);
              Ah = u64.add3H(All, T1h, sigma0h, MAJh);
              Al = All | 0;
          }
          ({ h: Ah, l: Al } = u64.add(this.Ah | 0, this.Al | 0, Ah | 0, Al | 0));
          ({ h: Bh, l: Bl } = u64.add(this.Bh | 0, this.Bl | 0, Bh | 0, Bl | 0));
          ({ h: Ch, l: Cl } = u64.add(this.Ch | 0, this.Cl | 0, Ch | 0, Cl | 0));
          ({ h: Dh, l: Dl } = u64.add(this.Dh | 0, this.Dl | 0, Dh | 0, Dl | 0));
          ({ h: Eh, l: El } = u64.add(this.Eh | 0, this.El | 0, Eh | 0, El | 0));
          ({ h: Fh, l: Fl } = u64.add(this.Fh | 0, this.Fl | 0, Fh | 0, Fl | 0));
          ({ h: Gh, l: Gl } = u64.add(this.Gh | 0, this.Gl | 0, Gh | 0, Gl | 0));
          ({ h: Hh, l: Hl } = u64.add(this.Hh | 0, this.Hl | 0, Hh | 0, Hl | 0));
          this.set(Ah, Al, Bh, Bl, Ch, Cl, Dh, Dl, Eh, El, Fh, Fl, Gh, Gl, Hh, Hl);
      }
      roundClean() {
          SHA512_W_H.fill(0);
          SHA512_W_L.fill(0);
      }
      destroy() {
          this.buffer.fill(0);
          this.set(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0);
      }
  }
  class SHA512_256 extends SHA512 {
      constructor() {
          super();
          this.Ah = 0x22312194 | 0;
          this.Al = 0xfc2bf72c | 0;
          this.Bh = 0x9f555fa3 | 0;
          this.Bl = 0xc84c64c2 | 0;
          this.Ch = 0x2393b86b | 0;
          this.Cl = 0x6f53b151 | 0;
          this.Dh = 0x96387719 | 0;
          this.Dl = 0x5940eabd | 0;
          this.Eh = 0x96283ee2 | 0;
          this.El = 0xa88effe3 | 0;
          this.Fh = 0xbe5e1e25 | 0;
          this.Fl = 0x53863992 | 0;
          this.Gh = 0x2b0199fc | 0;
          this.Gl = 0x2c85b8aa | 0;
          this.Hh = 0x0eb72ddc | 0;
          this.Hl = 0x81c52ca2 | 0;
          this.outputLen = 32;
      }
  }
  class SHA384 extends SHA512 {
      constructor() {
          super();
          this.Ah = 0xcbbb9d5d | 0;
          this.Al = 0xc1059ed8 | 0;
          this.Bh = 0x629a292a | 0;
          this.Bl = 0x367cd507 | 0;
          this.Ch = 0x9159015a | 0;
          this.Cl = 0x3070dd17 | 0;
          this.Dh = 0x152fecd8 | 0;
          this.Dl = 0xf70e5939 | 0;
          this.Eh = 0x67332667 | 0;
          this.El = 0xffc00b31 | 0;
          this.Fh = 0x8eb44a87 | 0;
          this.Fl = 0x68581511 | 0;
          this.Gh = 0xdb0c2e0d | 0;
          this.Gl = 0x64f98fa7 | 0;
          this.Hh = 0x47b5481d | 0;
          this.Hl = 0xbefa4fa4 | 0;
          this.outputLen = 48;
      }
  }
  const sha512 = wrapConstructor(() => new SHA512());
  wrapConstructor(() => new SHA512_256());
  wrapConstructor(() => new SHA384());

  const JS_HASH = {
    256: sha256,
    512: sha512
  };
  const WA_MHAC = {
    256: hmacSha256,
    512: hmacSha512
  };
  function createSha(bitLength) {
    return (key, data, onlyJs) => hmacShaAsU8a(key, data, bitLength, onlyJs);
  }
  function hmacShaAsU8a(key, data, bitLength = 256, onlyJs) {
    const u8aKey = util.u8aToU8a(key);
    return !util.hasBigInt || !onlyJs && isReady() ? WA_MHAC[bitLength](u8aKey, data) : hmac(JS_HASH[bitLength], u8aKey, data);
  }
  const hmacSha256AsU8a = createSha(256);
  const hmacSha512AsU8a = createSha(512);

  utils$1.hmacSha256Sync = (key, ...messages) => hmacSha256AsU8a(key, util.u8aConcat(...messages));
  cryptoWaitReady().catch(() => {
  });

  const packageInfo = {
    name: '@polkadot/util-crypto',
    path: (({ url: (typeof document === 'undefined' && typeof location === 'undefined' ? new (require('u' + 'rl').URL)('file:' + __filename).href : typeof document === 'undefined' ? location.href : (document.currentScript && document.currentScript.src || new URL('bundle-polkadot-util-crypto.js', document.baseURI).href)) }) && (typeof document === 'undefined' && typeof location === 'undefined' ? new (require('u' + 'rl').URL)('file:' + __filename).href : typeof document === 'undefined' ? location.href : (document.currentScript && document.currentScript.src || new URL('bundle-polkadot-util-crypto.js', document.baseURI).href))) ? new URL((typeof document === 'undefined' && typeof location === 'undefined' ? new (require('u' + 'rl').URL)('file:' + __filename).href : typeof document === 'undefined' ? location.href : (document.currentScript && document.currentScript.src || new URL('bundle-polkadot-util-crypto.js', document.baseURI).href))).pathname.substring(0, new URL((typeof document === 'undefined' && typeof location === 'undefined' ? new (require('u' + 'rl').URL)('file:' + __filename).href : typeof document === 'undefined' ? location.href : (document.currentScript && document.currentScript.src || new URL('bundle-polkadot-util-crypto.js', document.baseURI).href))).pathname.lastIndexOf('/') + 1) : 'auto',
    type: 'esm',
    version: '10.1.12'
  };

  /*! scure-base - MIT License (c) 2022 Paul Miller (paulmillr.com) */
  function assertNumber(n) {
      if (!Number.isSafeInteger(n))
          throw new Error(`Wrong integer: ${n}`);
  }
  function chain(...args) {
      const wrap = (a, b) => (c) => a(b(c));
      const encode = Array.from(args)
          .reverse()
          .reduce((acc, i) => (acc ? wrap(acc, i.encode) : i.encode), undefined);
      const decode = args.reduce((acc, i) => (acc ? wrap(acc, i.decode) : i.decode), undefined);
      return { encode, decode };
  }
  function alphabet(alphabet) {
      return {
          encode: (digits) => {
              if (!Array.isArray(digits) || (digits.length && typeof digits[0] !== 'number'))
                  throw new Error('alphabet.encode input should be an array of numbers');
              return digits.map((i) => {
                  assertNumber(i);
                  if (i < 0 || i >= alphabet.length)
                      throw new Error(`Digit index outside alphabet: ${i} (alphabet: ${alphabet.length})`);
                  return alphabet[i];
              });
          },
          decode: (input) => {
              if (!Array.isArray(input) || (input.length && typeof input[0] !== 'string'))
                  throw new Error('alphabet.decode input should be array of strings');
              return input.map((letter) => {
                  if (typeof letter !== 'string')
                      throw new Error(`alphabet.decode: not string element=${letter}`);
                  const index = alphabet.indexOf(letter);
                  if (index === -1)
                      throw new Error(`Unknown letter: "${letter}". Allowed: ${alphabet}`);
                  return index;
              });
          },
      };
  }
  function join(separator = '') {
      if (typeof separator !== 'string')
          throw new Error('join separator should be string');
      return {
          encode: (from) => {
              if (!Array.isArray(from) || (from.length && typeof from[0] !== 'string'))
                  throw new Error('join.encode input should be array of strings');
              for (let i of from)
                  if (typeof i !== 'string')
                      throw new Error(`join.encode: non-string input=${i}`);
              return from.join(separator);
          },
          decode: (to) => {
              if (typeof to !== 'string')
                  throw new Error('join.decode input should be string');
              return to.split(separator);
          },
      };
  }
  function padding(bits, chr = '=') {
      assertNumber(bits);
      if (typeof chr !== 'string')
          throw new Error('padding chr should be string');
      return {
          encode(data) {
              if (!Array.isArray(data) || (data.length && typeof data[0] !== 'string'))
                  throw new Error('padding.encode input should be array of strings');
              for (let i of data)
                  if (typeof i !== 'string')
                      throw new Error(`padding.encode: non-string input=${i}`);
              while ((data.length * bits) % 8)
                  data.push(chr);
              return data;
          },
          decode(input) {
              if (!Array.isArray(input) || (input.length && typeof input[0] !== 'string'))
                  throw new Error('padding.encode input should be array of strings');
              for (let i of input)
                  if (typeof i !== 'string')
                      throw new Error(`padding.decode: non-string input=${i}`);
              let end = input.length;
              if ((end * bits) % 8)
                  throw new Error('Invalid padding: string should have whole number of bytes');
              for (; end > 0 && input[end - 1] === chr; end--) {
                  if (!(((end - 1) * bits) % 8))
                      throw new Error('Invalid padding: string has too much padding');
              }
              return input.slice(0, end);
          },
      };
  }
  function normalize$1(fn) {
      if (typeof fn !== 'function')
          throw new Error('normalize fn should be function');
      return { encode: (from) => from, decode: (to) => fn(to) };
  }
  function convertRadix(data, from, to) {
      if (from < 2)
          throw new Error(`convertRadix: wrong from=${from}, base cannot be less than 2`);
      if (to < 2)
          throw new Error(`convertRadix: wrong to=${to}, base cannot be less than 2`);
      if (!Array.isArray(data))
          throw new Error('convertRadix: data should be array');
      if (!data.length)
          return [];
      let pos = 0;
      const res = [];
      const digits = Array.from(data);
      digits.forEach((d) => {
          assertNumber(d);
          if (d < 0 || d >= from)
              throw new Error(`Wrong integer: ${d}`);
      });
      while (true) {
          let carry = 0;
          let done = true;
          for (let i = pos; i < digits.length; i++) {
              const digit = digits[i];
              const digitBase = from * carry + digit;
              if (!Number.isSafeInteger(digitBase) ||
                  (from * carry) / from !== carry ||
                  digitBase - digit !== from * carry) {
                  throw new Error('convertRadix: carry overflow');
              }
              carry = digitBase % to;
              digits[i] = Math.floor(digitBase / to);
              if (!Number.isSafeInteger(digits[i]) || digits[i] * to + carry !== digitBase)
                  throw new Error('convertRadix: carry overflow');
              if (!done)
                  continue;
              else if (!digits[i])
                  pos = i;
              else
                  done = false;
          }
          res.push(carry);
          if (done)
              break;
      }
      for (let i = 0; i < data.length - 1 && data[i] === 0; i++)
          res.push(0);
      return res.reverse();
  }
  const gcd = (a, b) => (!b ? a : gcd(b, a % b));
  const radix2carry = (from, to) => from + (to - gcd(from, to));
  function convertRadix2(data, from, to, padding) {
      if (!Array.isArray(data))
          throw new Error('convertRadix2: data should be array');
      if (from <= 0 || from > 32)
          throw new Error(`convertRadix2: wrong from=${from}`);
      if (to <= 0 || to > 32)
          throw new Error(`convertRadix2: wrong to=${to}`);
      if (radix2carry(from, to) > 32) {
          throw new Error(`convertRadix2: carry overflow from=${from} to=${to} carryBits=${radix2carry(from, to)}`);
      }
      let carry = 0;
      let pos = 0;
      const mask = 2 ** to - 1;
      const res = [];
      for (const n of data) {
          assertNumber(n);
          if (n >= 2 ** from)
              throw new Error(`convertRadix2: invalid data word=${n} from=${from}`);
          carry = (carry << from) | n;
          if (pos + from > 32)
              throw new Error(`convertRadix2: carry overflow pos=${pos} from=${from}`);
          pos += from;
          for (; pos >= to; pos -= to)
              res.push(((carry >> (pos - to)) & mask) >>> 0);
          carry &= 2 ** pos - 1;
      }
      carry = (carry << (to - pos)) & mask;
      if (!padding && pos >= from)
          throw new Error('Excess padding');
      if (!padding && carry)
          throw new Error(`Non-zero padding: ${carry}`);
      if (padding && pos > 0)
          res.push(carry >>> 0);
      return res;
  }
  function radix(num) {
      assertNumber(num);
      return {
          encode: (bytes) => {
              if (!(bytes instanceof Uint8Array))
                  throw new Error('radix.encode input should be Uint8Array');
              return convertRadix(Array.from(bytes), 2 ** 8, num);
          },
          decode: (digits) => {
              if (!Array.isArray(digits) || (digits.length && typeof digits[0] !== 'number'))
                  throw new Error('radix.decode input should be array of strings');
              return Uint8Array.from(convertRadix(digits, num, 2 ** 8));
          },
      };
  }
  function radix2(bits, revPadding = false) {
      assertNumber(bits);
      if (bits <= 0 || bits > 32)
          throw new Error('radix2: bits should be in (0..32]');
      if (radix2carry(8, bits) > 32 || radix2carry(bits, 8) > 32)
          throw new Error('radix2: carry overflow');
      return {
          encode: (bytes) => {
              if (!(bytes instanceof Uint8Array))
                  throw new Error('radix2.encode input should be Uint8Array');
              return convertRadix2(Array.from(bytes), 8, bits, !revPadding);
          },
          decode: (digits) => {
              if (!Array.isArray(digits) || (digits.length && typeof digits[0] !== 'number'))
                  throw new Error('radix2.decode input should be array of strings');
              return Uint8Array.from(convertRadix2(digits, bits, 8, revPadding));
          },
      };
  }
  function unsafeWrapper(fn) {
      if (typeof fn !== 'function')
          throw new Error('unsafeWrapper fn should be function');
      return function (...args) {
          try {
              return fn.apply(null, args);
          }
          catch (e) { }
      };
  }
  function checksum(len, fn) {
      assertNumber(len);
      if (typeof fn !== 'function')
          throw new Error('checksum fn should be function');
      return {
          encode(data) {
              if (!(data instanceof Uint8Array))
                  throw new Error('checksum.encode: input should be Uint8Array');
              const checksum = fn(data).slice(0, len);
              const res = new Uint8Array(data.length + len);
              res.set(data);
              res.set(checksum, data.length);
              return res;
          },
          decode(data) {
              if (!(data instanceof Uint8Array))
                  throw new Error('checksum.decode: input should be Uint8Array');
              const payload = data.slice(0, -len);
              const newChecksum = fn(payload).slice(0, len);
              const oldChecksum = data.slice(-len);
              for (let i = 0; i < len; i++)
                  if (newChecksum[i] !== oldChecksum[i])
                      throw new Error('Invalid checksum');
              return payload;
          },
      };
  }
  const utils = { alphabet, chain, checksum, radix, radix2, join, padding };
  const base16 = chain(radix2(4), alphabet('0123456789ABCDEF'), join(''));
  const base32 = chain(radix2(5), alphabet('ABCDEFGHIJKLMNOPQRSTUVWXYZ234567'), padding(5), join(''));
  chain(radix2(5), alphabet('0123456789ABCDEFGHIJKLMNOPQRSTUV'), padding(5), join(''));
  chain(radix2(5), alphabet('0123456789ABCDEFGHJKMNPQRSTVWXYZ'), join(''), normalize$1((s) => s.toUpperCase().replace(/O/g, '0').replace(/[IL]/g, '1')));
  const base64 = chain(radix2(6), alphabet('ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/'), padding(6), join(''));
  const base64url = chain(radix2(6), alphabet('ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_'), padding(6), join(''));
  const genBase58 = (abc) => chain(radix(58), alphabet(abc), join(''));
  const base58 = genBase58('123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz');
  genBase58('123456789abcdefghijkmnopqrstuvwxyzABCDEFGHJKLMNPQRSTUVWXYZ');
  genBase58('rpshnaf39wBUDNEGHJKLM4PQRST7VWXYZ2bcdeCg65jkm8oFqi1tuvAxyz');
  const XMR_BLOCK_LEN = [0, 2, 3, 5, 6, 7, 9, 10, 11];
  const base58xmr = {
      encode(data) {
          let res = '';
          for (let i = 0; i < data.length; i += 8) {
              const block = data.subarray(i, i + 8);
              res += base58.encode(block).padStart(XMR_BLOCK_LEN[block.length], '1');
          }
          return res;
      },
      decode(str) {
          let res = [];
          for (let i = 0; i < str.length; i += 11) {
              const slice = str.slice(i, i + 11);
              const blockLen = XMR_BLOCK_LEN.indexOf(slice.length);
              const block = base58.decode(slice);
              for (let j = 0; j < block.length - blockLen; j++) {
                  if (block[j] !== 0)
                      throw new Error('base58xmr: wrong padding');
              }
              res = res.concat(Array.from(block.slice(block.length - blockLen)));
          }
          return Uint8Array.from(res);
      },
  };
  const BECH_ALPHABET = chain(alphabet('qpzry9x8gf2tvdw0s3jn54khce6mua7l'), join(''));
  const POLYMOD_GENERATORS = [0x3b6a57b2, 0x26508e6d, 0x1ea119fa, 0x3d4233dd, 0x2a1462b3];
  function bech32Polymod(pre) {
      const b = pre >> 25;
      let chk = (pre & 0x1ffffff) << 5;
      for (let i = 0; i < POLYMOD_GENERATORS.length; i++) {
          if (((b >> i) & 1) === 1)
              chk ^= POLYMOD_GENERATORS[i];
      }
      return chk;
  }
  function bechChecksum(prefix, words, encodingConst = 1) {
      const len = prefix.length;
      let chk = 1;
      for (let i = 0; i < len; i++) {
          const c = prefix.charCodeAt(i);
          if (c < 33 || c > 126)
              throw new Error(`Invalid prefix (${prefix})`);
          chk = bech32Polymod(chk) ^ (c >> 5);
      }
      chk = bech32Polymod(chk);
      for (let i = 0; i < len; i++)
          chk = bech32Polymod(chk) ^ (prefix.charCodeAt(i) & 0x1f);
      for (let v of words)
          chk = bech32Polymod(chk) ^ v;
      for (let i = 0; i < 6; i++)
          chk = bech32Polymod(chk);
      chk ^= encodingConst;
      return BECH_ALPHABET.encode(convertRadix2([chk % 2 ** 30], 30, 5, false));
  }
  function genBech32(encoding) {
      const ENCODING_CONST = encoding === 'bech32' ? 1 : 0x2bc830a3;
      const _words = radix2(5);
      const fromWords = _words.decode;
      const toWords = _words.encode;
      const fromWordsUnsafe = unsafeWrapper(fromWords);
      function encode(prefix, words, limit = 90) {
          if (typeof prefix !== 'string')
              throw new Error(`bech32.encode prefix should be string, not ${typeof prefix}`);
          if (!Array.isArray(words) || (words.length && typeof words[0] !== 'number'))
              throw new Error(`bech32.encode words should be array of numbers, not ${typeof words}`);
          const actualLength = prefix.length + 7 + words.length;
          if (limit !== false && actualLength > limit)
              throw new TypeError(`Length ${actualLength} exceeds limit ${limit}`);
          prefix = prefix.toLowerCase();
          return `${prefix}1${BECH_ALPHABET.encode(words)}${bechChecksum(prefix, words, ENCODING_CONST)}`;
      }
      function decode(str, limit = 90) {
          if (typeof str !== 'string')
              throw new Error(`bech32.decode input should be string, not ${typeof str}`);
          if (str.length < 8 || (limit !== false && str.length > limit))
              throw new TypeError(`Wrong string length: ${str.length} (${str}). Expected (8..${limit})`);
          const lowered = str.toLowerCase();
          if (str !== lowered && str !== str.toUpperCase())
              throw new Error(`String must be lowercase or uppercase`);
          str = lowered;
          const sepIndex = str.lastIndexOf('1');
          if (sepIndex === 0 || sepIndex === -1)
              throw new Error(`Letter "1" must be present between prefix and data only`);
          const prefix = str.slice(0, sepIndex);
          const _words = str.slice(sepIndex + 1);
          if (_words.length < 6)
              throw new Error('Data must be at least 6 characters long');
          const words = BECH_ALPHABET.decode(_words).slice(0, -6);
          const sum = bechChecksum(prefix, words, ENCODING_CONST);
          if (!_words.endsWith(sum))
              throw new Error(`Invalid checksum in ${str}: expected "${sum}"`);
          return { prefix, words };
      }
      const decodeUnsafe = unsafeWrapper(decode);
      function decodeToBytes(str) {
          const { prefix, words } = decode(str, false);
          return { prefix, words, bytes: fromWords(words) };
      }
      return { encode, decode, decodeToBytes, decodeUnsafe, fromWords, fromWordsUnsafe, toWords };
  }
  genBech32('bech32');
  genBech32('bech32m');
  const utf8 = {
      encode: (data) => new TextDecoder().decode(data),
      decode: (str) => new TextEncoder().encode(str),
  };
  const hex = chain(radix2(4), alphabet('0123456789abcdef'), join(''), normalize$1((s) => {
      if (typeof s !== 'string' || s.length % 2)
          throw new TypeError(`hex.decode: expected string, got ${typeof s} with length ${s.length}`);
      return s.toLowerCase();
  }));
  const CODERS = {
      utf8, hex, base16, base32, base64, base64url, base58, base58xmr
  };
`Invalid encoding type. Available types: ${Object.keys(CODERS).join(', ')}`;

  function createDecode({
    coder,
    ipfs
  }, validate) {
    return (value, ipfsCompat) => {
      validate(value, ipfsCompat);
      return coder.decode(ipfs && ipfsCompat ? value.substring(1) : value);
    };
  }
  function createEncode({
    coder,
    ipfs
  }) {
    return (value, ipfsCompat) => {
      const out = coder.encode(util.u8aToU8a(value));
      return ipfs && ipfsCompat ? `${ipfs}${out}` : out;
    };
  }
  function createIs(validate) {
    return (value, ipfsCompat) => {
      try {
        return validate(value, ipfsCompat);
      } catch (error) {
        return false;
      }
    };
  }
  function createValidate({
    chars,
    ipfs,
    type
  }) {
    return (value, ipfsCompat) => {
      if (!value || typeof value !== 'string') {
        throw new Error(`Expected non-null, non-empty ${type} string input`);
      }
      if (ipfs && ipfsCompat && value[0] !== ipfs) {
        throw new Error(`Expected ipfs-compatible ${type} to start with '${ipfs}'`);
      }
      for (let i = ipfsCompat ? 1 : 0; i < value.length; i++) {
        if (!(chars.includes(value[i]) || value[i] === '=' && (i === value.length - 1 || !chars.includes(value[i + 1])))) {
          throw new Error(`Invalid ${type} character "${value[i]}" (0x${value.charCodeAt(i).toString(16)}) at index ${i}`);
        }
      }
      return true;
    };
  }

  const config$2 = {
    chars: '123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz',
    coder: base58,
    ipfs: 'z',
    type: 'base58'
  };
  const base58Validate = createValidate(config$2);
  const base58Decode = createDecode(config$2, base58Validate);
  const base58Encode = createEncode(config$2);
  const isBase58 = createIs(base58Validate);

  const SIGMA = new Uint8Array([
      0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
      14, 10, 4, 8, 9, 15, 13, 6, 1, 12, 0, 2, 11, 7, 5, 3,
      11, 8, 12, 0, 5, 2, 15, 13, 10, 14, 3, 6, 7, 1, 9, 4,
      7, 9, 3, 1, 13, 12, 11, 14, 2, 6, 5, 10, 4, 0, 15, 8,
      9, 0, 5, 7, 2, 4, 10, 15, 14, 1, 11, 12, 6, 8, 3, 13,
      2, 12, 6, 10, 0, 11, 8, 3, 4, 13, 7, 5, 15, 14, 1, 9,
      12, 5, 1, 15, 14, 13, 4, 10, 0, 7, 6, 3, 9, 2, 8, 11,
      13, 11, 7, 14, 12, 1, 3, 9, 5, 0, 15, 4, 8, 6, 2, 10,
      6, 15, 14, 9, 11, 3, 0, 8, 12, 2, 13, 7, 1, 4, 10, 5,
      10, 2, 8, 4, 7, 6, 1, 5, 15, 11, 9, 14, 3, 12, 13, 0,
      0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
      14, 10, 4, 8, 9, 15, 13, 6, 1, 12, 0, 2, 11, 7, 5, 3,
  ]);
  class BLAKE2 extends Hash {
      constructor(blockLen, outputLen, opts = {}, keyLen, saltLen, persLen) {
          super();
          this.blockLen = blockLen;
          this.outputLen = outputLen;
          this.length = 0;
          this.pos = 0;
          this.finished = false;
          this.destroyed = false;
          assert.number(blockLen);
          assert.number(outputLen);
          assert.number(keyLen);
          if (outputLen < 0 || outputLen > keyLen)
              throw new Error('Blake2: outputLen bigger than keyLen');
          if (opts.key !== undefined && (opts.key.length < 1 || opts.key.length > keyLen))
              throw new Error(`Key should be up 1..${keyLen} byte long or undefined`);
          if (opts.salt !== undefined && opts.salt.length !== saltLen)
              throw new Error(`Salt should be ${saltLen} byte long or undefined`);
          if (opts.personalization !== undefined && opts.personalization.length !== persLen)
              throw new Error(`Personalization should be ${persLen} byte long or undefined`);
          this.buffer32 = u32((this.buffer = new Uint8Array(blockLen)));
      }
      update(data) {
          assert.exists(this);
          const { blockLen, buffer, buffer32 } = this;
          data = toBytes(data);
          const len = data.length;
          for (let pos = 0; pos < len;) {
              if (this.pos === blockLen) {
                  this.compress(buffer32, 0, false);
                  this.pos = 0;
              }
              const take = Math.min(blockLen - this.pos, len - pos);
              const dataOffset = data.byteOffset + pos;
              if (take === blockLen && !(dataOffset % 4) && pos + take < len) {
                  const data32 = new Uint32Array(data.buffer, dataOffset, Math.floor((len - pos) / 4));
                  for (let pos32 = 0; pos + blockLen < len; pos32 += buffer32.length, pos += blockLen) {
                      this.length += blockLen;
                      this.compress(data32, pos32, false);
                  }
                  continue;
              }
              buffer.set(data.subarray(pos, pos + take), this.pos);
              this.pos += take;
              this.length += take;
              pos += take;
          }
          return this;
      }
      digestInto(out) {
          assert.exists(this);
          assert.output(out, this);
          const { pos, buffer32 } = this;
          this.finished = true;
          this.buffer.subarray(pos).fill(0);
          this.compress(buffer32, 0, true);
          const out32 = u32(out);
          this.get().forEach((v, i) => (out32[i] = v));
      }
      digest() {
          const { buffer, outputLen } = this;
          this.digestInto(buffer);
          const res = buffer.slice(0, outputLen);
          this.destroy();
          return res;
      }
      _cloneInto(to) {
          const { buffer, length, finished, destroyed, outputLen, pos } = this;
          to || (to = new this.constructor({ dkLen: outputLen }));
          to.set(...this.get());
          to.length = length;
          to.finished = finished;
          to.destroyed = destroyed;
          to.outputLen = outputLen;
          to.buffer.set(buffer);
          to.pos = pos;
          return to;
      }
  }

  const IV = new Uint32Array([
      0xf3bcc908, 0x6a09e667, 0x84caa73b, 0xbb67ae85, 0xfe94f82b, 0x3c6ef372, 0x5f1d36f1, 0xa54ff53a,
      0xade682d1, 0x510e527f, 0x2b3e6c1f, 0x9b05688c, 0xfb41bd6b, 0x1f83d9ab, 0x137e2179, 0x5be0cd19
  ]);
  const BUF = new Uint32Array(32);
  function G1(a, b, c, d, msg, x) {
      const Xl = msg[x], Xh = msg[x + 1];
      let Al = BUF[2 * a], Ah = BUF[2 * a + 1];
      let Bl = BUF[2 * b], Bh = BUF[2 * b + 1];
      let Cl = BUF[2 * c], Ch = BUF[2 * c + 1];
      let Dl = BUF[2 * d], Dh = BUF[2 * d + 1];
      let ll = u64.add3L(Al, Bl, Xl);
      Ah = u64.add3H(ll, Ah, Bh, Xh);
      Al = ll | 0;
      ({ Dh, Dl } = { Dh: Dh ^ Ah, Dl: Dl ^ Al });
      ({ Dh, Dl } = { Dh: u64.rotr32H(Dh, Dl), Dl: u64.rotr32L(Dh, Dl) });
      ({ h: Ch, l: Cl } = u64.add(Ch, Cl, Dh, Dl));
      ({ Bh, Bl } = { Bh: Bh ^ Ch, Bl: Bl ^ Cl });
      ({ Bh, Bl } = { Bh: u64.rotrSH(Bh, Bl, 24), Bl: u64.rotrSL(Bh, Bl, 24) });
      (BUF[2 * a] = Al), (BUF[2 * a + 1] = Ah);
      (BUF[2 * b] = Bl), (BUF[2 * b + 1] = Bh);
      (BUF[2 * c] = Cl), (BUF[2 * c + 1] = Ch);
      (BUF[2 * d] = Dl), (BUF[2 * d + 1] = Dh);
  }
  function G2(a, b, c, d, msg, x) {
      const Xl = msg[x], Xh = msg[x + 1];
      let Al = BUF[2 * a], Ah = BUF[2 * a + 1];
      let Bl = BUF[2 * b], Bh = BUF[2 * b + 1];
      let Cl = BUF[2 * c], Ch = BUF[2 * c + 1];
      let Dl = BUF[2 * d], Dh = BUF[2 * d + 1];
      let ll = u64.add3L(Al, Bl, Xl);
      Ah = u64.add3H(ll, Ah, Bh, Xh);
      Al = ll | 0;
      ({ Dh, Dl } = { Dh: Dh ^ Ah, Dl: Dl ^ Al });
      ({ Dh, Dl } = { Dh: u64.rotrSH(Dh, Dl, 16), Dl: u64.rotrSL(Dh, Dl, 16) });
      ({ h: Ch, l: Cl } = u64.add(Ch, Cl, Dh, Dl));
      ({ Bh, Bl } = { Bh: Bh ^ Ch, Bl: Bl ^ Cl });
      ({ Bh, Bl } = { Bh: u64.rotrBH(Bh, Bl, 63), Bl: u64.rotrBL(Bh, Bl, 63) });
      (BUF[2 * a] = Al), (BUF[2 * a + 1] = Ah);
      (BUF[2 * b] = Bl), (BUF[2 * b + 1] = Bh);
      (BUF[2 * c] = Cl), (BUF[2 * c + 1] = Ch);
      (BUF[2 * d] = Dl), (BUF[2 * d + 1] = Dh);
  }
  class BLAKE2b extends BLAKE2 {
      constructor(opts = {}) {
          super(128, opts.dkLen === undefined ? 64 : opts.dkLen, opts, 64, 16, 16);
          this.v0l = IV[0] | 0;
          this.v0h = IV[1] | 0;
          this.v1l = IV[2] | 0;
          this.v1h = IV[3] | 0;
          this.v2l = IV[4] | 0;
          this.v2h = IV[5] | 0;
          this.v3l = IV[6] | 0;
          this.v3h = IV[7] | 0;
          this.v4l = IV[8] | 0;
          this.v4h = IV[9] | 0;
          this.v5l = IV[10] | 0;
          this.v5h = IV[11] | 0;
          this.v6l = IV[12] | 0;
          this.v6h = IV[13] | 0;
          this.v7l = IV[14] | 0;
          this.v7h = IV[15] | 0;
          const keyLength = opts.key ? opts.key.length : 0;
          this.v0l ^= this.outputLen | (keyLength << 8) | (0x01 << 16) | (0x01 << 24);
          if (opts.salt) {
              const salt = u32(toBytes(opts.salt));
              this.v4l ^= salt[0];
              this.v4h ^= salt[1];
              this.v5l ^= salt[2];
              this.v5h ^= salt[3];
          }
          if (opts.personalization) {
              const pers = u32(toBytes(opts.personalization));
              this.v6l ^= pers[0];
              this.v6h ^= pers[1];
              this.v7l ^= pers[2];
              this.v7h ^= pers[3];
          }
          if (opts.key) {
              const tmp = new Uint8Array(this.blockLen);
              tmp.set(toBytes(opts.key));
              this.update(tmp);
          }
      }
      get() {
          let { v0l, v0h, v1l, v1h, v2l, v2h, v3l, v3h, v4l, v4h, v5l, v5h, v6l, v6h, v7l, v7h } = this;
          return [v0l, v0h, v1l, v1h, v2l, v2h, v3l, v3h, v4l, v4h, v5l, v5h, v6l, v6h, v7l, v7h];
      }
      set(v0l, v0h, v1l, v1h, v2l, v2h, v3l, v3h, v4l, v4h, v5l, v5h, v6l, v6h, v7l, v7h) {
          this.v0l = v0l | 0;
          this.v0h = v0h | 0;
          this.v1l = v1l | 0;
          this.v1h = v1h | 0;
          this.v2l = v2l | 0;
          this.v2h = v2h | 0;
          this.v3l = v3l | 0;
          this.v3h = v3h | 0;
          this.v4l = v4l | 0;
          this.v4h = v4h | 0;
          this.v5l = v5l | 0;
          this.v5h = v5h | 0;
          this.v6l = v6l | 0;
          this.v6h = v6h | 0;
          this.v7l = v7l | 0;
          this.v7h = v7h | 0;
      }
      compress(msg, offset, isLast) {
          this.get().forEach((v, i) => (BUF[i] = v));
          BUF.set(IV, 16);
          let { h, l } = u64.fromBig(BigInt(this.length));
          BUF[24] = IV[8] ^ l;
          BUF[25] = IV[9] ^ h;
          if (isLast) {
              BUF[28] = ~BUF[28];
              BUF[29] = ~BUF[29];
          }
          let j = 0;
          const s = SIGMA;
          for (let i = 0; i < 12; i++) {
              G1(0, 4, 8, 12, msg, offset + 2 * s[j++]);
              G2(0, 4, 8, 12, msg, offset + 2 * s[j++]);
              G1(1, 5, 9, 13, msg, offset + 2 * s[j++]);
              G2(1, 5, 9, 13, msg, offset + 2 * s[j++]);
              G1(2, 6, 10, 14, msg, offset + 2 * s[j++]);
              G2(2, 6, 10, 14, msg, offset + 2 * s[j++]);
              G1(3, 7, 11, 15, msg, offset + 2 * s[j++]);
              G2(3, 7, 11, 15, msg, offset + 2 * s[j++]);
              G1(0, 5, 10, 15, msg, offset + 2 * s[j++]);
              G2(0, 5, 10, 15, msg, offset + 2 * s[j++]);
              G1(1, 6, 11, 12, msg, offset + 2 * s[j++]);
              G2(1, 6, 11, 12, msg, offset + 2 * s[j++]);
              G1(2, 7, 8, 13, msg, offset + 2 * s[j++]);
              G2(2, 7, 8, 13, msg, offset + 2 * s[j++]);
              G1(3, 4, 9, 14, msg, offset + 2 * s[j++]);
              G2(3, 4, 9, 14, msg, offset + 2 * s[j++]);
          }
          this.v0l ^= BUF[0] ^ BUF[16];
          this.v0h ^= BUF[1] ^ BUF[17];
          this.v1l ^= BUF[2] ^ BUF[18];
          this.v1h ^= BUF[3] ^ BUF[19];
          this.v2l ^= BUF[4] ^ BUF[20];
          this.v2h ^= BUF[5] ^ BUF[21];
          this.v3l ^= BUF[6] ^ BUF[22];
          this.v3h ^= BUF[7] ^ BUF[23];
          this.v4l ^= BUF[8] ^ BUF[24];
          this.v4h ^= BUF[9] ^ BUF[25];
          this.v5l ^= BUF[10] ^ BUF[26];
          this.v5h ^= BUF[11] ^ BUF[27];
          this.v6l ^= BUF[12] ^ BUF[28];
          this.v6h ^= BUF[13] ^ BUF[29];
          this.v7l ^= BUF[14] ^ BUF[30];
          this.v7h ^= BUF[15] ^ BUF[31];
          BUF.fill(0);
      }
      destroy() {
          this.destroyed = true;
          this.buffer32.fill(0);
          this.set(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0);
      }
  }
  const blake2b = wrapConstructorWithOpts((opts) => new BLAKE2b(opts));

  function createAsHex(fn) {
    return (...args) => util.u8aToHex(fn(...args));
  }
  function createBitHasher(bitLength, fn) {
    return (data, onlyJs) => fn(data, bitLength, onlyJs);
  }
  function createDualHasher(wa, js) {
    return (value, bitLength = 256, onlyJs) => {
      const u8a = util.u8aToU8a(value);
      return !util.hasBigInt || !onlyJs && isReady() ? wa[bitLength](u8a) : js[bitLength](u8a);
    };
  }

  function blake2AsU8a(data, bitLength = 256, key, onlyJs) {
    const byteLength = Math.ceil(bitLength / 8);
    const u8a = util.u8aToU8a(data);
    return !util.hasBigInt || !onlyJs && isReady() ? blake2b$1(u8a, util.u8aToU8a(key), byteLength) : blake2b(u8a, {
      dkLen: byteLength,
      key: key || undefined
    });
  }
  const blake2AsHex = createAsHex(blake2AsU8a);

  const SS58_PREFIX = util.stringToU8a('SS58PRE');
  function sshash(key) {
    return blake2AsU8a(util.u8aConcat(SS58_PREFIX, key), 512);
  }

  function checkAddressChecksum(decoded) {
    const ss58Length = decoded[0] & 0b01000000 ? 2 : 1;
    const ss58Decoded = ss58Length === 1 ? decoded[0] : (decoded[0] & 0b00111111) << 2 | decoded[1] >> 6 | (decoded[1] & 0b00111111) << 8;
    const isPublicKey = [34 + ss58Length, 35 + ss58Length].includes(decoded.length);
    const length = decoded.length - (isPublicKey ? 2 : 1);
    const hash = sshash(decoded.subarray(0, length));
    const isValid = (decoded[0] & 0b10000000) === 0 && ![46, 47].includes(decoded[0]) && (isPublicKey ? decoded[decoded.length - 2] === hash[0] && decoded[decoded.length - 1] === hash[1] : decoded[decoded.length - 1] === hash[0]);
    return [isValid, length, ss58Length, ss58Decoded];
  }

  const knownSubstrate = [
  	{
  		"prefix": 0,
  		"network": "polkadot",
  		"displayName": "Polkadot Relay Chain",
  		"symbols": [
  			"DOT"
  		],
  		"decimals": [
  			10
  		],
  		"standardAccount": "*25519",
  		"website": "https://polkadot.network"
  	},
  	{
  		"prefix": 1,
  		"network": "BareSr25519",
  		"displayName": "Bare 32-bit Schnorr/Ristretto (S/R 25519) public key.",
  		"symbols": [],
  		"decimals": [],
  		"standardAccount": "Sr25519",
  		"website": null
  	},
  	{
  		"prefix": 2,
  		"network": "kusama",
  		"displayName": "Kusama Relay Chain",
  		"symbols": [
  			"KSM"
  		],
  		"decimals": [
  			12
  		],
  		"standardAccount": "*25519",
  		"website": "https://kusama.network"
  	},
  	{
  		"prefix": 3,
  		"network": "BareEd25519",
  		"displayName": "Bare 32-bit Ed25519 public key.",
  		"symbols": [],
  		"decimals": [],
  		"standardAccount": "Ed25519",
  		"website": null
  	},
  	{
  		"prefix": 4,
  		"network": "katalchain",
  		"displayName": "Katal Chain",
  		"symbols": [],
  		"decimals": [],
  		"standardAccount": "*25519",
  		"website": null
  	},
  	{
  		"prefix": 5,
  		"network": "astar",
  		"displayName": "Astar Network",
  		"symbols": [
  			"ASTR"
  		],
  		"decimals": [
  			18
  		],
  		"standardAccount": "*25519",
  		"website": "https://astar.network"
  	},
  	{
  		"prefix": 6,
  		"network": "bifrost",
  		"displayName": "Bifrost",
  		"symbols": [
  			"BNC"
  		],
  		"decimals": [
  			12
  		],
  		"standardAccount": "*25519",
  		"website": "https://bifrost.finance/"
  	},
  	{
  		"prefix": 7,
  		"network": "edgeware",
  		"displayName": "Edgeware",
  		"symbols": [
  			"EDG"
  		],
  		"decimals": [
  			18
  		],
  		"standardAccount": "*25519",
  		"website": "https://edgewa.re"
  	},
  	{
  		"prefix": 8,
  		"network": "karura",
  		"displayName": "Karura",
  		"symbols": [
  			"KAR"
  		],
  		"decimals": [
  			12
  		],
  		"standardAccount": "*25519",
  		"website": "https://karura.network/"
  	},
  	{
  		"prefix": 9,
  		"network": "reynolds",
  		"displayName": "Laminar Reynolds Canary",
  		"symbols": [
  			"REY"
  		],
  		"decimals": [
  			18
  		],
  		"standardAccount": "*25519",
  		"website": "http://laminar.network/"
  	},
  	{
  		"prefix": 10,
  		"network": "acala",
  		"displayName": "Acala",
  		"symbols": [
  			"ACA"
  		],
  		"decimals": [
  			12
  		],
  		"standardAccount": "*25519",
  		"website": "https://acala.network/"
  	},
  	{
  		"prefix": 11,
  		"network": "laminar",
  		"displayName": "Laminar",
  		"symbols": [
  			"LAMI"
  		],
  		"decimals": [
  			18
  		],
  		"standardAccount": "*25519",
  		"website": "http://laminar.network/"
  	},
  	{
  		"prefix": 12,
  		"network": "polymesh",
  		"displayName": "Polymesh",
  		"symbols": [
  			"POLYX"
  		],
  		"decimals": [
  			6
  		],
  		"standardAccount": "*25519",
  		"website": "https://polymath.network/"
  	},
  	{
  		"prefix": 13,
  		"network": "integritee",
  		"displayName": "Integritee",
  		"symbols": [
  			"TEER"
  		],
  		"decimals": [
  			12
  		],
  		"standardAccount": "*25519",
  		"website": "https://integritee.network"
  	},
  	{
  		"prefix": 14,
  		"network": "totem",
  		"displayName": "Totem",
  		"symbols": [
  			"TOTEM"
  		],
  		"decimals": [
  			0
  		],
  		"standardAccount": "*25519",
  		"website": "https://totemaccounting.com"
  	},
  	{
  		"prefix": 15,
  		"network": "synesthesia",
  		"displayName": "Synesthesia",
  		"symbols": [
  			"SYN"
  		],
  		"decimals": [
  			12
  		],
  		"standardAccount": "*25519",
  		"website": "https://synesthesia.network/"
  	},
  	{
  		"prefix": 16,
  		"network": "kulupu",
  		"displayName": "Kulupu",
  		"symbols": [
  			"KLP"
  		],
  		"decimals": [
  			12
  		],
  		"standardAccount": "*25519",
  		"website": "https://kulupu.network/"
  	},
  	{
  		"prefix": 17,
  		"network": "dark",
  		"displayName": "Dark Mainnet",
  		"symbols": [],
  		"decimals": [],
  		"standardAccount": "*25519",
  		"website": null
  	},
  	{
  		"prefix": 18,
  		"network": "darwinia",
  		"displayName": "Darwinia Network",
  		"symbols": [
  			"RING",
  			"KTON"
  		],
  		"decimals": [
  			9,
  			9
  		],
  		"standardAccount": "*25519",
  		"website": "https://darwinia.network/"
  	},
  	{
  		"prefix": 19,
  		"network": "watr",
  		"displayName": "Watr Protocol",
  		"symbols": [
  			"WATR"
  		],
  		"decimals": [
  			18
  		],
  		"standardAccount": "*25519",
  		"website": "https://www.watr.org"
  	},
  	{
  		"prefix": 20,
  		"network": "stafi",
  		"displayName": "Stafi",
  		"symbols": [
  			"FIS"
  		],
  		"decimals": [
  			12
  		],
  		"standardAccount": "*25519",
  		"website": "https://stafi.io"
  	},
  	{
  		"prefix": 22,
  		"network": "dock-pos-mainnet",
  		"displayName": "Dock Mainnet",
  		"symbols": [
  			"DCK"
  		],
  		"decimals": [
  			6
  		],
  		"standardAccount": "*25519",
  		"website": "https://dock.io"
  	},
  	{
  		"prefix": 23,
  		"network": "shift",
  		"displayName": "ShiftNrg",
  		"symbols": [],
  		"decimals": [],
  		"standardAccount": "*25519",
  		"website": null
  	},
  	{
  		"prefix": 24,
  		"network": "zero",
  		"displayName": "ZERO",
  		"symbols": [
  			"ZERO"
  		],
  		"decimals": [
  			18
  		],
  		"standardAccount": "*25519",
  		"website": "https://zero.io"
  	},
  	{
  		"prefix": 25,
  		"network": "zero-alphaville",
  		"displayName": "ZERO Alphaville",
  		"symbols": [
  			"ZERO"
  		],
  		"decimals": [
  			18
  		],
  		"standardAccount": "*25519",
  		"website": "https://zero.io"
  	},
  	{
  		"prefix": 26,
  		"network": "jupiter",
  		"displayName": "Jupiter",
  		"symbols": [
  			"jDOT"
  		],
  		"decimals": [
  			10
  		],
  		"standardAccount": "*25519",
  		"website": "https://jupiter.patract.io"
  	},
  	{
  		"prefix": 27,
  		"network": "kabocha",
  		"displayName": "Kabocha",
  		"symbols": [
  			"KAB"
  		],
  		"decimals": [
  			12
  		],
  		"standardAccount": "*25519",
  		"website": "https://kabocha.network"
  	},
  	{
  		"prefix": 28,
  		"network": "subsocial",
  		"displayName": "Subsocial",
  		"symbols": [],
  		"decimals": [],
  		"standardAccount": "*25519",
  		"website": null
  	},
  	{
  		"prefix": 29,
  		"network": "cord",
  		"displayName": "CORD Network",
  		"symbols": [
  			"DHI",
  			"WAY"
  		],
  		"decimals": [
  			12,
  			12
  		],
  		"standardAccount": "*25519",
  		"website": "https://cord.network/"
  	},
  	{
  		"prefix": 30,
  		"network": "phala",
  		"displayName": "Phala Network",
  		"symbols": [
  			"PHA"
  		],
  		"decimals": [
  			12
  		],
  		"standardAccount": "*25519",
  		"website": "https://phala.network"
  	},
  	{
  		"prefix": 31,
  		"network": "litentry",
  		"displayName": "Litentry Network",
  		"symbols": [
  			"LIT"
  		],
  		"decimals": [
  			12
  		],
  		"standardAccount": "*25519",
  		"website": "https://litentry.com/"
  	},
  	{
  		"prefix": 32,
  		"network": "robonomics",
  		"displayName": "Robonomics",
  		"symbols": [
  			"XRT"
  		],
  		"decimals": [
  			9
  		],
  		"standardAccount": "*25519",
  		"website": "https://robonomics.network"
  	},
  	{
  		"prefix": 33,
  		"network": "datahighway",
  		"displayName": "DataHighway",
  		"symbols": [],
  		"decimals": [],
  		"standardAccount": "*25519",
  		"website": null
  	},
  	{
  		"prefix": 34,
  		"network": "ares",
  		"displayName": "Ares Protocol",
  		"symbols": [
  			"ARES"
  		],
  		"decimals": [
  			12
  		],
  		"standardAccount": "*25519",
  		"website": "https://www.aresprotocol.com/"
  	},
  	{
  		"prefix": 35,
  		"network": "vln",
  		"displayName": "Valiu Liquidity Network",
  		"symbols": [
  			"USDv"
  		],
  		"decimals": [
  			15
  		],
  		"standardAccount": "*25519",
  		"website": "https://valiu.com/"
  	},
  	{
  		"prefix": 36,
  		"network": "centrifuge",
  		"displayName": "Centrifuge Chain",
  		"symbols": [
  			"CFG"
  		],
  		"decimals": [
  			18
  		],
  		"standardAccount": "*25519",
  		"website": "https://centrifuge.io/"
  	},
  	{
  		"prefix": 37,
  		"network": "nodle",
  		"displayName": "Nodle Chain",
  		"symbols": [
  			"NODL"
  		],
  		"decimals": [
  			11
  		],
  		"standardAccount": "*25519",
  		"website": "https://nodle.io/"
  	},
  	{
  		"prefix": 38,
  		"network": "kilt",
  		"displayName": "KILT Spiritnet",
  		"symbols": [
  			"KILT"
  		],
  		"decimals": [
  			15
  		],
  		"standardAccount": "*25519",
  		"website": "https://kilt.io/"
  	},
  	{
  		"prefix": 39,
  		"network": "mathchain",
  		"displayName": "MathChain mainnet",
  		"symbols": [
  			"MATH"
  		],
  		"decimals": [
  			18
  		],
  		"standardAccount": "*25519",
  		"website": "https://mathwallet.org"
  	},
  	{
  		"prefix": 40,
  		"network": "mathchain-testnet",
  		"displayName": "MathChain testnet",
  		"symbols": [
  			"MATH"
  		],
  		"decimals": [
  			18
  		],
  		"standardAccount": "*25519",
  		"website": "https://mathwallet.org"
  	},
  	{
  		"prefix": 41,
  		"network": "poli",
  		"displayName": "Polimec Chain",
  		"symbols": [],
  		"decimals": [],
  		"standardAccount": "*25519",
  		"website": "https://polimec.io/"
  	},
  	{
  		"prefix": 42,
  		"network": "substrate",
  		"displayName": "Substrate",
  		"symbols": [],
  		"decimals": [],
  		"standardAccount": "*25519",
  		"website": "https://substrate.io/"
  	},
  	{
  		"prefix": 43,
  		"network": "BareSecp256k1",
  		"displayName": "Bare 32-bit ECDSA SECP-256k1 public key.",
  		"symbols": [],
  		"decimals": [],
  		"standardAccount": "secp256k1",
  		"website": null
  	},
  	{
  		"prefix": 44,
  		"network": "chainx",
  		"displayName": "ChainX",
  		"symbols": [
  			"PCX"
  		],
  		"decimals": [
  			8
  		],
  		"standardAccount": "*25519",
  		"website": "https://chainx.org/"
  	},
  	{
  		"prefix": 45,
  		"network": "uniarts",
  		"displayName": "UniArts Network",
  		"symbols": [
  			"UART",
  			"UINK"
  		],
  		"decimals": [
  			12,
  			12
  		],
  		"standardAccount": "*25519",
  		"website": "https://uniarts.me"
  	},
  	{
  		"prefix": 46,
  		"network": "reserved46",
  		"displayName": "This prefix is reserved.",
  		"symbols": [],
  		"decimals": [],
  		"standardAccount": null,
  		"website": null
  	},
  	{
  		"prefix": 47,
  		"network": "reserved47",
  		"displayName": "This prefix is reserved.",
  		"symbols": [],
  		"decimals": [],
  		"standardAccount": null,
  		"website": null
  	},
  	{
  		"prefix": 48,
  		"network": "neatcoin",
  		"displayName": "Neatcoin Mainnet",
  		"symbols": [
  			"NEAT"
  		],
  		"decimals": [
  			12
  		],
  		"standardAccount": "*25519",
  		"website": "https://neatcoin.org"
  	},
  	{
  		"prefix": 49,
  		"network": "picasso",
  		"displayName": "Picasso",
  		"symbols": [
  			"PICA"
  		],
  		"decimals": [
  			12
  		],
  		"standardAccount": "*25519",
  		"website": "https://picasso.composable.finance"
  	},
  	{
  		"prefix": 50,
  		"network": "composable",
  		"displayName": "Composable",
  		"symbols": [
  			"LAYR"
  		],
  		"decimals": [
  			12
  		],
  		"standardAccount": "*25519",
  		"website": "https://composable.finance"
  	},
  	{
  		"prefix": 51,
  		"network": "oak",
  		"displayName": "OAK Network",
  		"symbols": [
  			"OAK",
  			"TUR"
  		],
  		"decimals": [
  			10,
  			10
  		],
  		"standardAccount": "*25519",
  		"website": "https://oak.tech"
  	},
  	{
  		"prefix": 52,
  		"network": "KICO",
  		"displayName": "KICO",
  		"symbols": [
  			"KICO"
  		],
  		"decimals": [
  			14
  		],
  		"standardAccount": "*25519",
  		"website": "https://dico.io"
  	},
  	{
  		"prefix": 53,
  		"network": "DICO",
  		"displayName": "DICO",
  		"symbols": [
  			"DICO"
  		],
  		"decimals": [
  			14
  		],
  		"standardAccount": "*25519",
  		"website": "https://dico.io"
  	},
  	{
  		"prefix": 54,
  		"network": "cere",
  		"displayName": "Cere Network",
  		"symbols": [
  			"CERE"
  		],
  		"decimals": [
  			10
  		],
  		"standardAccount": "*25519",
  		"website": "https://cere.network"
  	},
  	{
  		"prefix": 55,
  		"network": "xxnetwork",
  		"displayName": "xx network",
  		"symbols": [
  			"XX"
  		],
  		"decimals": [
  			9
  		],
  		"standardAccount": "*25519",
  		"website": "https://xx.network"
  	},
  	{
  		"prefix": 56,
  		"network": "pendulum",
  		"displayName": "Pendulum chain",
  		"symbols": [
  			"PEN"
  		],
  		"decimals": [
  			12
  		],
  		"standardAccount": "*25519",
  		"website": "https://pendulumchain.org/"
  	},
  	{
  		"prefix": 57,
  		"network": "amplitude",
  		"displayName": "Amplitude chain",
  		"symbols": [
  			"AMPE"
  		],
  		"decimals": [
  			12
  		],
  		"standardAccount": "*25519",
  		"website": "https://pendulumchain.org/"
  	},
  	{
  		"prefix": 63,
  		"network": "hydradx",
  		"displayName": "HydraDX",
  		"symbols": [
  			"HDX"
  		],
  		"decimals": [
  			12
  		],
  		"standardAccount": "*25519",
  		"website": "https://hydradx.io"
  	},
  	{
  		"prefix": 65,
  		"network": "aventus",
  		"displayName": "AvN Mainnet",
  		"symbols": [
  			"AVT"
  		],
  		"decimals": [
  			18
  		],
  		"standardAccount": "*25519",
  		"website": "https://aventus.io"
  	},
  	{
  		"prefix": 66,
  		"network": "crust",
  		"displayName": "Crust Network",
  		"symbols": [
  			"CRU"
  		],
  		"decimals": [
  			12
  		],
  		"standardAccount": "*25519",
  		"website": "https://crust.network"
  	},
  	{
  		"prefix": 67,
  		"network": "genshiro",
  		"displayName": "Genshiro Network",
  		"symbols": [
  			"GENS",
  			"EQD",
  			"LPT0"
  		],
  		"decimals": [
  			9,
  			9,
  			9
  		],
  		"standardAccount": "*25519",
  		"website": "https://genshiro.equilibrium.io"
  	},
  	{
  		"prefix": 68,
  		"network": "equilibrium",
  		"displayName": "Equilibrium Network",
  		"symbols": [
  			"EQ"
  		],
  		"decimals": [
  			9
  		],
  		"standardAccount": "*25519",
  		"website": "https://equilibrium.io"
  	},
  	{
  		"prefix": 69,
  		"network": "sora",
  		"displayName": "SORA Network",
  		"symbols": [
  			"XOR"
  		],
  		"decimals": [
  			18
  		],
  		"standardAccount": "*25519",
  		"website": "https://sora.org"
  	},
  	{
  		"prefix": 71,
  		"network": "p3d",
  		"displayName": "3DP network",
  		"symbols": [
  			"P3D"
  		],
  		"decimals": [
  			12
  		],
  		"standardAccount": "*25519",
  		"website": "https://3dpass.org"
  	},
  	{
  		"prefix": 72,
  		"network": "p3dt",
  		"displayName": "3DP test network",
  		"symbols": [
  			"P3Dt"
  		],
  		"decimals": [
  			12
  		],
  		"standardAccount": "*25519",
  		"website": "https://3dpass.org"
  	},
  	{
  		"prefix": 73,
  		"network": "zeitgeist",
  		"displayName": "Zeitgeist",
  		"symbols": [
  			"ZTG"
  		],
  		"decimals": [
  			10
  		],
  		"standardAccount": "*25519",
  		"website": "https://zeitgeist.pm"
  	},
  	{
  		"prefix": 77,
  		"network": "manta",
  		"displayName": "Manta network",
  		"symbols": [
  			"MANTA"
  		],
  		"decimals": [
  			18
  		],
  		"standardAccount": "*25519",
  		"website": "https://manta.network"
  	},
  	{
  		"prefix": 78,
  		"network": "calamari",
  		"displayName": "Calamari: Manta Canary Network",
  		"symbols": [
  			"KMA"
  		],
  		"decimals": [
  			12
  		],
  		"standardAccount": "*25519",
  		"website": "https://manta.network"
  	},
  	{
  		"prefix": 88,
  		"network": "polkadex",
  		"displayName": "Polkadex Mainnet",
  		"symbols": [
  			"PDEX"
  		],
  		"decimals": [
  			12
  		],
  		"standardAccount": "*25519",
  		"website": "https://polkadex.trade"
  	},
  	{
  		"prefix": 89,
  		"network": "polkadexparachain",
  		"displayName": "Polkadex Parachain",
  		"symbols": [
  			"PDEX"
  		],
  		"decimals": [
  			12
  		],
  		"standardAccount": "*25519",
  		"website": "https://polkadex.trade"
  	},
  	{
  		"prefix": 90,
  		"network": "frequency",
  		"displayName": "Frequency",
  		"symbols": [
  			"FRQCY"
  		],
  		"decimals": [
  			8
  		],
  		"standardAccount": "*25519",
  		"website": "https://www.frequency.xyz"
  	},
  	{
  		"prefix": 92,
  		"network": "anmol",
  		"displayName": "Anmol Network",
  		"symbols": [
  			"ANML"
  		],
  		"decimals": [
  			18
  		],
  		"standardAccount": "*25519",
  		"website": "https://anmol.network/"
  	},
  	{
  		"prefix": 93,
  		"network": "fragnova",
  		"displayName": "Fragnova Network",
  		"symbols": [
  			"NOVA"
  		],
  		"decimals": [
  			12
  		],
  		"standardAccount": "*25519",
  		"website": "https://fragnova.com"
  	},
  	{
  		"prefix": 98,
  		"network": "polkasmith",
  		"displayName": "PolkaSmith Canary Network",
  		"symbols": [
  			"PKS"
  		],
  		"decimals": [
  			18
  		],
  		"standardAccount": "*25519",
  		"website": "https://polkafoundry.com"
  	},
  	{
  		"prefix": 99,
  		"network": "polkafoundry",
  		"displayName": "PolkaFoundry Network",
  		"symbols": [
  			"PKF"
  		],
  		"decimals": [
  			18
  		],
  		"standardAccount": "*25519",
  		"website": "https://polkafoundry.com"
  	},
  	{
  		"prefix": 100,
  		"network": "ibtida",
  		"displayName": "Anmol Network Ibtida Canary network",
  		"symbols": [
  			"IANML"
  		],
  		"decimals": [
  			18
  		],
  		"standardAccount": "*25519",
  		"website": "https://anmol.network/"
  	},
  	{
  		"prefix": 101,
  		"network": "origintrail-parachain",
  		"displayName": "OriginTrail Parachain",
  		"symbols": [
  			"OTP"
  		],
  		"decimals": [
  			12
  		],
  		"standardAccount": "*25519",
  		"website": "https://parachain.origintrail.io/"
  	},
  	{
  		"prefix": 105,
  		"network": "pontem-network",
  		"displayName": "Pontem Network",
  		"symbols": [
  			"PONT"
  		],
  		"decimals": [
  			10
  		],
  		"standardAccount": "*25519",
  		"website": "https://pontem.network"
  	},
  	{
  		"prefix": 110,
  		"network": "heiko",
  		"displayName": "Heiko",
  		"symbols": [
  			"HKO"
  		],
  		"decimals": [
  			12
  		],
  		"standardAccount": "*25519",
  		"website": "https://parallel.fi/"
  	},
  	{
  		"prefix": 113,
  		"network": "integritee-incognito",
  		"displayName": "Integritee Incognito",
  		"symbols": [],
  		"decimals": [],
  		"standardAccount": "*25519",
  		"website": "https://integritee.network"
  	},
  	{
  		"prefix": 117,
  		"network": "tinker",
  		"displayName": "Tinker",
  		"symbols": [
  			"TNKR"
  		],
  		"decimals": [
  			12
  		],
  		"standardAccount": "*25519",
  		"website": "https://invarch.network"
  	},
  	{
  		"prefix": 126,
  		"network": "joystream",
  		"displayName": "Joystream",
  		"symbols": [
  			"JOY"
  		],
  		"decimals": [
  			10
  		],
  		"standardAccount": "*25519",
  		"website": "https://www.joystream.org"
  	},
  	{
  		"prefix": 128,
  		"network": "clover",
  		"displayName": "Clover Finance",
  		"symbols": [
  			"CLV"
  		],
  		"decimals": [
  			18
  		],
  		"standardAccount": "*25519",
  		"website": "https://clover.finance"
  	},
  	{
  		"prefix": 129,
  		"network": "dorafactory-polkadot",
  		"displayName": "Dorafactory Polkadot Network",
  		"symbols": [
  			"DORA"
  		],
  		"decimals": [
  			12
  		],
  		"standardAccount": "*25519",
  		"website": "https://dorafactory.org"
  	},
  	{
  		"prefix": 131,
  		"network": "litmus",
  		"displayName": "Litmus Network",
  		"symbols": [
  			"LIT"
  		],
  		"decimals": [
  			12
  		],
  		"standardAccount": "*25519",
  		"website": "https://litentry.com/"
  	},
  	{
  		"prefix": 136,
  		"network": "altair",
  		"displayName": "Altair",
  		"symbols": [
  			"AIR"
  		],
  		"decimals": [
  			18
  		],
  		"standardAccount": "*25519",
  		"website": "https://centrifuge.io/"
  	},
  	{
  		"prefix": 137,
  		"network": "vara",
  		"displayName": "Vara Network",
  		"symbols": [
  			"VARA"
  		],
  		"decimals": [
  			12
  		],
  		"standardAccount": "*25519",
  		"website": "https://vara-network.io/"
  	},
  	{
  		"prefix": 172,
  		"network": "parallel",
  		"displayName": "Parallel",
  		"symbols": [
  			"PARA"
  		],
  		"decimals": [
  			12
  		],
  		"standardAccount": "*25519",
  		"website": "https://parallel.fi/"
  	},
  	{
  		"prefix": 252,
  		"network": "social-network",
  		"displayName": "Social Network",
  		"symbols": [
  			"NET"
  		],
  		"decimals": [
  			18
  		],
  		"standardAccount": "*25519",
  		"website": "https://social.network"
  	},
  	{
  		"prefix": 255,
  		"network": "quartz_mainnet",
  		"displayName": "QUARTZ by UNIQUE",
  		"symbols": [
  			"QTZ"
  		],
  		"decimals": [
  			18
  		],
  		"standardAccount": "*25519",
  		"website": "https://unique.network"
  	},
  	{
  		"prefix": 268,
  		"network": "pioneer_network",
  		"displayName": "Pioneer Network by Bit.Country",
  		"symbols": [
  			"NEER"
  		],
  		"decimals": [
  			18
  		],
  		"standardAccount": "*25519",
  		"website": "https://bit.country"
  	},
  	{
  		"prefix": 420,
  		"network": "sora_kusama_para",
  		"displayName": "SORA Kusama Parachain",
  		"symbols": [
  			"XOR"
  		],
  		"decimals": [
  			18
  		],
  		"standardAccount": "*25519",
  		"website": "https://sora.org"
  	},
  	{
  		"prefix": 789,
  		"network": "geek",
  		"displayName": "GEEK Network",
  		"symbols": [
  			"GEEK"
  		],
  		"decimals": [
  			18
  		],
  		"standardAccount": "*25519",
  		"website": "https://geek.gl"
  	},
  	{
  		"prefix": 1110,
  		"network": "efinity",
  		"displayName": "Efinity",
  		"symbols": [
  			"EFI"
  		],
  		"decimals": [
  			18
  		],
  		"standardAccount": "*25519",
  		"website": "https://efinity.io/"
  	},
  	{
  		"prefix": 1221,
  		"network": "peaq",
  		"displayName": "Peaq Network",
  		"symbols": [
  			"PEAQ"
  		],
  		"decimals": [
  			18
  		],
  		"standardAccount": "Sr25519",
  		"website": "https://www.peaq.network/"
  	},
  	{
  		"prefix": 1222,
  		"network": "apex",
  		"displayName": "Apex Network",
  		"symbols": [
  			"APEX"
  		],
  		"decimals": [
  			18
  		],
  		"standardAccount": "Sr25519",
  		"website": "https://www.peaq.network/"
  	},
  	{
  		"prefix": 1284,
  		"network": "moonbeam",
  		"displayName": "Moonbeam",
  		"symbols": [
  			"GLMR"
  		],
  		"decimals": [
  			18
  		],
  		"standardAccount": "secp256k1",
  		"website": "https://moonbeam.network"
  	},
  	{
  		"prefix": 1285,
  		"network": "moonriver",
  		"displayName": "Moonriver",
  		"symbols": [
  			"MOVR"
  		],
  		"decimals": [
  			18
  		],
  		"standardAccount": "secp256k1",
  		"website": "https://moonbeam.network"
  	},
  	{
  		"prefix": 1328,
  		"network": "ajuna",
  		"displayName": "Ajuna Network",
  		"symbols": [
  			"AJUN"
  		],
  		"decimals": [
  			12
  		],
  		"standardAccount": "*25519",
  		"website": "https://ajuna.io"
  	},
  	{
  		"prefix": 1337,
  		"network": "bajun",
  		"displayName": "Bajun Network",
  		"symbols": [
  			"BAJU"
  		],
  		"decimals": [
  			12
  		],
  		"standardAccount": "*25519",
  		"website": "https://ajuna.io"
  	},
  	{
  		"prefix": 1985,
  		"network": "seals",
  		"displayName": "Seals Network",
  		"symbols": [
  			"SEAL"
  		],
  		"decimals": [
  			9
  		],
  		"standardAccount": "*25519",
  		"website": "https://seals.app"
  	},
  	{
  		"prefix": 2007,
  		"network": "kapex",
  		"displayName": "Kapex",
  		"symbols": [
  			"KAPEX"
  		],
  		"decimals": [
  			12
  		],
  		"standardAccount": "*25519",
  		"website": "https://totemaccounting.com"
  	},
  	{
  		"prefix": 2009,
  		"network": "cloudwalk_mainnet",
  		"displayName": "CloudWalk Network Mainnet",
  		"symbols": [
  			"CWN"
  		],
  		"decimals": [
  			18
  		],
  		"standardAccount": "*25519",
  		"website": "https://explorer.mainnet.cloudwalk.io"
  	},
  	{
  		"prefix": 2032,
  		"network": "interlay",
  		"displayName": "Interlay",
  		"symbols": [
  			"INTR"
  		],
  		"decimals": [
  			10
  		],
  		"standardAccount": "*25519",
  		"website": "https://interlay.io/"
  	},
  	{
  		"prefix": 2092,
  		"network": "kintsugi",
  		"displayName": "Kintsugi",
  		"symbols": [
  			"KINT"
  		],
  		"decimals": [
  			12
  		],
  		"standardAccount": "*25519",
  		"website": "https://interlay.io/"
  	},
  	{
  		"prefix": 2106,
  		"network": "bitgreen",
  		"displayName": "Bitgreen",
  		"symbols": [
  			"BBB"
  		],
  		"decimals": [
  			18
  		],
  		"standardAccount": "*25519",
  		"website": "https://bitgreen.org/"
  	},
  	{
  		"prefix": 2112,
  		"network": "chainflip",
  		"displayName": "Chainflip",
  		"symbols": [
  			"FLIP"
  		],
  		"decimals": [
  			18
  		],
  		"standardAccount": "*25519",
  		"website": "https://chainflip.io/"
  	},
  	{
  		"prefix": 2114,
  		"network": "Turing",
  		"displayName": "Turing Network",
  		"symbols": [
  			"TUR"
  		],
  		"decimals": [
  			10
  		],
  		"standardAccount": "*25519",
  		"website": "https://oak.tech/turing/home/"
  	},
  	{
  		"prefix": 2207,
  		"network": "SNOW",
  		"displayName": "SNOW: ICE Canary Network",
  		"symbols": [
  			"ICZ"
  		],
  		"decimals": [
  			18
  		],
  		"standardAccount": "*25519",
  		"website": "https://icenetwork.io"
  	},
  	{
  		"prefix": 2208,
  		"network": "ICE",
  		"displayName": "ICE Network",
  		"symbols": [
  			"ICY"
  		],
  		"decimals": [
  			18
  		],
  		"standardAccount": "*25519",
  		"website": "https://icenetwork.io"
  	},
  	{
  		"prefix": 2254,
  		"network": "subspace_testnet",
  		"displayName": "Subspace testnet",
  		"symbols": [
  			"tSSC"
  		],
  		"decimals": [
  			18
  		],
  		"standardAccount": "*25519",
  		"website": "https://subspace.network"
  	},
  	{
  		"prefix": 3000,
  		"network": "hashed",
  		"displayName": "Hashed Network",
  		"symbols": [
  			"HASH"
  		],
  		"decimals": [
  			18
  		],
  		"standardAccount": "*25519",
  		"website": "https://hashed.network"
  	},
  	{
  		"prefix": 4000,
  		"network": "luhn",
  		"displayName": "Luhn Network",
  		"symbols": [
  			"LUHN"
  		],
  		"decimals": [
  			18
  		],
  		"standardAccount": "*25519",
  		"website": "https://luhn.network"
  	},
  	{
  		"prefix": 4006,
  		"network": "tangle",
  		"displayName": "Tangle Network",
  		"symbols": [
  			"TNT"
  		],
  		"decimals": [
  			18
  		],
  		"standardAccount": "*25519",
  		"website": "https://www.webb.tools/"
  	},
  	{
  		"prefix": 4450,
  		"network": "g1",
  		"displayName": "Ğ1",
  		"symbols": [
  			"G1"
  		],
  		"decimals": [
  			2
  		],
  		"standardAccount": "*25519",
  		"website": "https://duniter.org"
  	},
  	{
  		"prefix": 5234,
  		"network": "humanode",
  		"displayName": "Humanode Network",
  		"symbols": [
  			"HMND"
  		],
  		"decimals": [
  			18
  		],
  		"standardAccount": "*25519",
  		"website": "https://humanode.io"
  	},
  	{
  		"prefix": 6094,
  		"network": "subspace",
  		"displayName": "Subspace",
  		"symbols": [
  			"SSC"
  		],
  		"decimals": [
  			18
  		],
  		"standardAccount": "*25519",
  		"website": "https://subspace.network"
  	},
  	{
  		"prefix": 7007,
  		"network": "tidefi",
  		"displayName": "Tidefi",
  		"symbols": [
  			"TDFY"
  		],
  		"decimals": [
  			12
  		],
  		"standardAccount": "*25519",
  		"website": "https://tidefi.com"
  	},
  	{
  		"prefix": 7013,
  		"network": "gm",
  		"displayName": "GM",
  		"symbols": [
  			"FREN",
  			"GM",
  			"GN"
  		],
  		"decimals": [
  			12,
  			0,
  			0
  		],
  		"standardAccount": "*25519",
  		"website": "https://gmordie.com"
  	},
  	{
  		"prefix": 7391,
  		"network": "unique_mainnet",
  		"displayName": "Unique Network",
  		"symbols": [
  			"UNQ"
  		],
  		"decimals": [
  			18
  		],
  		"standardAccount": "*25519",
  		"website": "https://unique.network"
  	},
  	{
  		"prefix": 8883,
  		"network": "sapphire_mainnet",
  		"displayName": "Sapphire by Unique",
  		"symbols": [
  			"QTZ"
  		],
  		"decimals": [
  			18
  		],
  		"standardAccount": "*25519",
  		"website": "https://unique.network"
  	},
  	{
  		"prefix": 9807,
  		"network": "dentnet",
  		"displayName": "DENTNet",
  		"symbols": [
  			"DENTX"
  		],
  		"decimals": [
  			18
  		],
  		"standardAccount": "*25519",
  		"website": "https://www.dentnet.io"
  	},
  	{
  		"prefix": 9935,
  		"network": "t3rn",
  		"displayName": "t3rn",
  		"symbols": [
  			"TRN"
  		],
  		"decimals": [
  			12
  		],
  		"standardAccount": "*25519",
  		"website": "https://t3rn.io/"
  	},
  	{
  		"prefix": 10041,
  		"network": "basilisk",
  		"displayName": "Basilisk",
  		"symbols": [
  			"BSX"
  		],
  		"decimals": [
  			12
  		],
  		"standardAccount": "*25519",
  		"website": "https://bsx.fi"
  	},
  	{
  		"prefix": 11330,
  		"network": "cess-testnet",
  		"displayName": "CESS Testnet",
  		"symbols": [
  			"TCESS"
  		],
  		"decimals": [
  			12
  		],
  		"standardAccount": "*25519",
  		"website": "https://cess.cloud"
  	},
  	{
  		"prefix": 11331,
  		"network": "cess",
  		"displayName": "CESS",
  		"symbols": [
  			"CESS"
  		],
  		"decimals": [
  			12
  		],
  		"standardAccount": "*25519",
  		"website": "https://cess.cloud"
  	},
  	{
  		"prefix": 11820,
  		"network": "contextfree",
  		"displayName": "Automata ContextFree",
  		"symbols": [
  			"CTX"
  		],
  		"decimals": [
  			18
  		],
  		"standardAccount": "*25519",
  		"website": "https://ata.network"
  	},
  	{
  		"prefix": 12191,
  		"network": "nftmart",
  		"displayName": "NFTMart",
  		"symbols": [
  			"NMT"
  		],
  		"decimals": [
  			12
  		],
  		"standardAccount": "*25519",
  		"website": "https://nftmart.io"
  	}
  ];

  const knownGenesis = {
    acala: ['0xfc41b9bd8ef8fe53d58c7ea67c794c7ec9a73daf05e6d54b14ff6342c99ba64c'],
    'aleph-node': ['0x70255b4d28de0fc4e1a193d7e175ad1ccef431598211c55538f1018651a0344e'],
    astar: ['0x9eb76c5184c4ab8679d2d5d819fdf90b9c001403e9e17da2e14b6d8aec4029c6'],
    basilisk: ['0xa85cfb9b9fd4d622a5b28289a02347af987d8f73fa3108450e2b4a11c1ce5755'],
    bifrost: ['0x262e1b2ad728475fd6fe88e62d34c200abe6fd693931ddad144059b1eb884e5b'],
    'bifrost-kusama': ['0x9f28c6a68e0fc9646eff64935684f6eeeece527e37bbe1f213d22caa1d9d6bed'],
    centrifuge: ['0xb3db41421702df9a7fcac62b53ffeac85f7853cc4e689e0b93aeb3db18c09d82', '0x67dddf2673b69e5f875f6f25277495834398eafd67f492e09f3f3345e003d1b5'],
    composable: ['0xdaab8df776eb52ec604a5df5d388bb62a050a0aaec4556a64265b9d42755552d'],
    darwinia: ['0xe71578b37a7c799b0ab4ee87ffa6f059a6b98f71f06fb8c84a8d88013a548ad6'],
    'dock-mainnet': ['0x6bfe24dca2a3be10f22212678ac13a6446ec764103c0f3471c71609eac384aae', '0xf73467c6544aa68df2ee546b135f955c46b90fa627e9b5d7935f41061bb8a5a9'],
    edgeware: ['0x742a2ca70c2fda6cee4f8df98d64c4c670a052d9568058982dad9d5a7a135c5b'],
    equilibrium: ['0x6f1a800de3daff7f5e037ddf66ab22ce03ab91874debeddb1086f5f7dbd48925'],
    genshiro: ['0x9b8cefc0eb5c568b527998bdd76c184e2b76ae561be76e4667072230217ea243'],
    hydradx: ['0xafdc188f45c71dacbaa0b62e16a91f726c7b8699a9748cdf715459de6b7f366d',
    '0xd2a620c27ec5cbc5621ff9a522689895074f7cca0d08e7134a7804e1a3ba86fc',
    '0x10af6e84234477d84dc572bac0789813b254aa490767ed06fb9591191d1073f9',
    '0x3d75507dd46301767e601265791da1d9cb47b6ebc94e87347b635e5bf58bd047',
    '0x0ed32bfcab4a83517fac88f2aa7cbc2f88d3ab93be9a12b6188a036bf8a943c2'
    ],
    'interlay-parachain': ['0xbf88efe70e9e0e916416e8bed61f2b45717f517d7f3523e33c7b001e5ffcbc72'],
    karura: ['0xbaf5aabe40646d11f0ee8abbdc64f4a4b7674925cba08e4a05ff9ebed6e2126b'],
    khala: ['0xd43540ba6d3eb4897c28a77d48cb5b729fea37603cbbfc7a86a73b72adb3be8d'],
    kulupu: ['0xf7a99d3cb92853d00d5275c971c132c074636256583fee53b3bbe60d7b8769ba'],
    kusama: ['0xb0a8d493285c2df73290dfb7e61f870f17b41801197a149ca93654499ea3dafe',
    '0xe3777fa922cafbff200cadeaea1a76bd7898ad5b89f7848999058b50e715f636',
    '0x3fd7b9eb6a00376e5be61f01abb429ffb0b104be05eaff4d458da48fcd425baf'
    ],
    'nodle-para': ['0x97da7ede98d7bad4e36b4d734b6055425a3be036da2a332ea5a7037656427a21'],
    parallel: ['0xe61a41c53f5dcd0beb09df93b34402aada44cb05117b71059cce40a2723a4e97'],
    phala: ['0x1bb969d85965e4bb5a651abbedf21a54b6b31a21f66b5401cc3f1e286268d736'],
    picasso: ['0xe8e7f0f4c4f5a00720b4821dbfddefea7490bcf0b19009961cc46957984e2c1c'],
    polkadex: ['0x3920bcb4960a1eef5580cd5367ff3f430eef052774f78468852f7b9cb39f8a3c'],
    polkadot: ['0x91b171bb158e2d3848fa23a9f1c25182fb8e20313b2c1eb49219da7a70ce90c3'],
    polymesh: ['0x6fbd74e5e1d0a61d52ccfe9d4adaed16dd3a7caa37c6bc4d0c2fa12e8b2f4063'],
    rococo: ['0x6408de7737c59c238890533af25896a2c20608d8b380bb01029acb392781063e', '0xaaf2cd1b74b5f726895921259421b534124726263982522174147046b8827897', '0x037f5f3c8e67b314062025fc886fcd6238ea25a4a9b45dce8d246815c9ebe770', '0xc196f81260cf1686172b47a79cf002120735d7cb0eb1474e8adce56618456fff', '0xf6e9983c37baf68846fedafe21e56718790e39fb1c582abc408b81bc7b208f9a', '0x5fce687da39305dfe682b117f0820b319348e8bb37eb16cf34acbf6a202de9d9', '0xe7c3d5edde7db964317cd9b51a3a059d7cd99f81bdbce14990047354334c9779', '0x1611e1dbf0405379b861e2e27daa90f480b2e6d3682414a80835a52e8cb8a215', '0x343442f12fa715489a8714e79a7b264ea88c0d5b8c66b684a7788a516032f6b9', '0x78bcd530c6b3a068bc17473cf5d2aff9c287102bed9af3ae3c41c33b9d6c6147', '0x47381ee0697153d64404fc578392c8fd5cba9073391908f46c888498415647bd', '0x19c0e4fa8ab75f5ac7865e0b8f74ff91eb9a100d336f423cd013a8befba40299'],
    sora: ['0x7e4e32d0feafd4f9c9414b0be86373f9a1efa904809b683453a9af6856d38ad5'],
    stafi: ['0x290a4149f09ea0e402c74c1c7e96ae4239588577fe78932f94f5404c68243d80'],
    statemine: ['0x48239ef607d7928874027a43a67689209727dfb3d3dc5e5b03a39bdc2eda771a'],
    statemint: ['0x68d56f15f85d3136970ec16946040bc1752654e906147f7e43e9d539d7c3de2f'],
    subsocial: ['0x0bd72c1c305172e1275278aaeb3f161e02eccb7a819e63f62d47bd53a28189f8'],
    unique: ['0x84322d9cddbf35088f1e54e9a85c967a41a56a4f43445768125e61af166c7d31'],
    vtb: ['0x286bc8414c7000ce1d6ee6a834e29a54c1784814b76243eb77ed0b2c5573c60f', '0x7483b89572fb2bd687c7b9a93b242d0b237f9aba463aba07ec24503931038aaa'],
    westend: ['0xe143f23803ac50e8f6f8e62695d1ce9e4e1d68aa36c1cd2cfd15340213f3423e'],
    xxnetwork: ['0x50dd5d206917bf10502c68fb4d18a59fc8aa31586f4e8856b493e43544aa82aa']
  };

  const knownIcon = {
    centrifuge: 'polkadot',
    kusama: 'polkadot',
    polkadot: 'polkadot',
    sora: 'polkadot',
    statemine: 'polkadot',
    statemint: 'polkadot',
    westmint: 'polkadot'
  };

  const knownLedger = {
    acala: 0x00000313,
    'aleph-node': 0x00000283,
    astar: 0x0000032a,
    bifrost: 0x00000314,
    'bifrost-kusama': 0x00000314,
    centrifuge: 0x000002eb,
    composable: 0x00000162,
    darwinia: 0x00000162,
    'dock-mainnet': 0x00000252,
    edgeware: 0x0000020b,
    equilibrium: 0x05f5e0fd,
    genshiro: 0x05f5e0fc,
    hydradx: 0x00000162,
    'interlay-parachain': 0x00000162,
    karura: 0x000002ae,
    khala: 0x000001b2,
    kusama: 0x000001b2,
    'nodle-para': 0x000003eb,
    parallel: 0x00000162,
    phala: 0x00000162,
    polkadex: 0x0000031f,
    polkadot: 0x00000162,
    polymesh: 0x00000253,
    sora: 0x00000269,
    stafi: 0x0000038b,
    statemine: 0x000001b2,
    statemint: 0x00000162,
    unique: 0x00000162,
    vtb: 0x000002b6,
    xxnetwork: 0x000007a3
  };

  const knownTestnet = {
    '': true,
    'cess-testnet': true,
    'dock-testnet': true,
    jupiter: true,
    'mathchain-testnet': true,
    p3dt: true,
    subspace_testnet: true,
    'zero-alphaville': true
  };

  const UNSORTED = [0, 2, 42];
  const TESTNETS = ['testnet'];
  function toExpanded(o) {
    const network = o.network || '';
    const nameParts = network.replace(/_/g, '-').split('-');
    const n = o;
    n.slip44 = knownLedger[network];
    n.hasLedgerSupport = !!n.slip44;
    n.genesisHash = knownGenesis[network] || [];
    n.icon = knownIcon[network] || 'substrate';
    n.isTestnet = !!knownTestnet[network] || TESTNETS.includes(nameParts[nameParts.length - 1]);
    n.isIgnored = n.isTestnet || !(o.standardAccount && o.decimals && o.decimals.length && o.symbols && o.symbols.length) && o.prefix !== 42;
    return n;
  }
  function filterSelectable({
    genesisHash,
    prefix
  }) {
    return !!genesisHash.length || prefix === 42;
  }
  function filterAvailable(n) {
    return !n.isIgnored && !!n.network;
  }
  function sortNetworks(a, b) {
    const isUnSortedA = UNSORTED.includes(a.prefix);
    const isUnSortedB = UNSORTED.includes(b.prefix);
    return isUnSortedA === isUnSortedB ? isUnSortedA ? 0 : a.displayName.localeCompare(b.displayName) : isUnSortedA ? -1 : 1;
  }
  const allNetworks = knownSubstrate.map(toExpanded);
  const availableNetworks = allNetworks.filter(filterAvailable).sort(sortNetworks);
  const selectableNetworks = availableNetworks.filter(filterSelectable);

  const defaults = {
    allowedDecodedLengths: [1, 2, 4, 8, 32, 33],
    allowedEncodedLengths: [3, 4, 6, 10, 35, 36, 37, 38],
    allowedPrefix: availableNetworks.map(({
      prefix
    }) => prefix),
    prefix: 42
  };

  function decodeAddress(encoded, ignoreChecksum, ss58Format = -1) {
    if (!encoded) {
      throw new Error('Invalid empty address passed');
    }
    if (util.isU8a(encoded) || util.isHex(encoded)) {
      return util.u8aToU8a(encoded);
    }
    try {
      const decoded = base58Decode(encoded);
      if (!defaults.allowedEncodedLengths.includes(decoded.length)) {
        throw new Error('Invalid decoded address length');
      }
      const [isValid, endPos, ss58Length, ss58Decoded] = checkAddressChecksum(decoded);
      if (!isValid && !ignoreChecksum) {
        throw new Error('Invalid decoded address checksum');
      } else if (ss58Format !== -1 && ss58Format !== ss58Decoded) {
        throw new Error(`Expected ss58Format ${ss58Format}, received ${ss58Decoded}`);
      }
      return decoded.slice(ss58Length, endPos);
    } catch (error) {
      throw new Error(`Decoding ${encoded}: ${error.message}`);
    }
  }

  function addressToEvm(address, ignoreChecksum) {
    return decodeAddress(address, ignoreChecksum).subarray(0, 20);
  }

  function checkAddress(address, prefix) {
    let decoded;
    try {
      decoded = base58Decode(address);
    } catch (error) {
      return [false, error.message];
    }
    const [isValid,,, ss58Decoded] = checkAddressChecksum(decoded);
    if (ss58Decoded !== prefix) {
      return [false, `Prefix mismatch, expected ${prefix}, found ${ss58Decoded}`];
    } else if (!defaults.allowedEncodedLengths.includes(decoded.length)) {
      return [false, 'Invalid decoded address length'];
    }
    return [isValid, isValid ? null : 'Invalid decoded address checksum'];
  }

  const BN_BE_OPTS = {
    isLe: false
  };
  const BN_LE_OPTS = {
    isLe: true
  };
  const BN_LE_16_OPTS = {
    bitLength: 16,
    isLe: true
  };
  const BN_BE_32_OPTS = {
    bitLength: 32,
    isLe: false
  };
  const BN_LE_32_OPTS = {
    bitLength: 32,
    isLe: true
  };
  const BN_BE_256_OPTS = {
    bitLength: 256,
    isLe: false
  };
  const BN_LE_256_OPTS = {
    bitLength: 256,
    isLe: true
  };
  const BN_LE_512_OPTS = {
    bitLength: 512,
    isLe: true
  };

  function addressToU8a(who) {
    return decodeAddress(who);
  }

  const PREFIX$1 = util.stringToU8a('modlpy/utilisuba');
  function createKeyMulti(who, threshold) {
    return blake2AsU8a(util.u8aConcat(PREFIX$1, util.compactToU8a(who.length), ...util.u8aSorted(who.map(addressToU8a)), util.bnToU8a(threshold, BN_LE_16_OPTS)));
  }

  const PREFIX = util.stringToU8a('modlpy/utilisuba');
  function createKeyDerived(who, index) {
    return blake2AsU8a(util.u8aConcat(PREFIX, decodeAddress(who), util.bnToU8a(index, BN_LE_16_OPTS)));
  }

  const RE_NUMBER = /^\d+$/;
  const JUNCTION_ID_LEN = 32;
  class DeriveJunction {
    #chainCode = new Uint8Array(32);
    #isHard = false;
    static from(value) {
      const result = new DeriveJunction();
      const [code, isHard] = value.startsWith('/') ? [value.substring(1), true] : [value, false];
      result.soft(RE_NUMBER.test(code) ? new util.BN(code, 10) : code);
      return isHard ? result.harden() : result;
    }
    get chainCode() {
      return this.#chainCode;
    }
    get isHard() {
      return this.#isHard;
    }
    get isSoft() {
      return !this.#isHard;
    }
    hard(value) {
      return this.soft(value).harden();
    }
    harden() {
      this.#isHard = true;
      return this;
    }
    soft(value) {
      if (util.isNumber(value) || util.isBn(value) || util.isBigInt(value)) {
        return this.soft(util.bnToU8a(value, BN_LE_256_OPTS));
      } else if (util.isHex(value)) {
        return this.soft(util.hexToU8a(value));
      } else if (util.isString(value)) {
        return this.soft(util.compactAddLength(util.stringToU8a(value)));
      } else if (value.length > JUNCTION_ID_LEN) {
        return this.soft(blake2AsU8a(value));
      }
      this.#chainCode.fill(0);
      this.#chainCode.set(value, 0);
      return this;
    }
    soften() {
      this.#isHard = false;
      return this;
    }
  }

  const RE_JUNCTION = /\/(\/?)([^/]+)/g;
  function keyExtractPath(derivePath) {
    const parts = derivePath.match(RE_JUNCTION);
    const path = [];
    let constructed = '';
    if (parts) {
      constructed = parts.join('');
      for (const p of parts) {
        path.push(DeriveJunction.from(p.substring(1)));
      }
    }
    if (constructed !== derivePath) {
      throw new Error(`Re-constructed path "${constructed}" does not match input`);
    }
    return {
      parts,
      path
    };
  }

  const RE_CAPTURE = /^(\w+( \w+)*)((\/\/?[^/]+)*)(\/\/\/(.*))?$/;
  function keyExtractSuri(suri) {
    const matches = suri.match(RE_CAPTURE);
    if (matches === null) {
      throw new Error('Unable to match provided value to a secret URI');
    }
    const [, phrase,, derivePath,,, password] = matches;
    const {
      path
    } = keyExtractPath(derivePath);
    return {
      derivePath,
      password,
      path,
      phrase
    };
  }

  const HDKD$1 = util.compactAddLength(util.stringToU8a('Secp256k1HDKD'));
  function secp256k1DeriveHard(seed, chainCode) {
    if (!util.isU8a(chainCode) || chainCode.length !== 32) {
      throw new Error('Invalid chainCode passed to derive');
    }
    return blake2AsU8a(util.u8aConcat(HDKD$1, seed, chainCode), 256);
  }

  function secp256k1PairFromSeed(seed, onlyJs) {
    if (seed.length !== 32) {
      throw new Error('Expected valid 32-byte private key as a seed');
    }
    if (!util.hasBigInt || !onlyJs && isReady()) {
      const full = secp256k1FromSeed(seed);
      const publicKey = full.slice(32);
      if (util.u8aEmpty(publicKey)) {
        throw new Error('Invalid publicKey generated from WASM interface');
      }
      return {
        publicKey,
        secretKey: full.slice(0, 32)
      };
    }
    return {
      publicKey: getPublicKey(seed, true),
      secretKey: seed
    };
  }

  function createSeedDeriveFn(fromSeed, derive) {
    return (keypair, {
      chainCode,
      isHard
    }) => {
      if (!isHard) {
        throw new Error('A soft key was found in the path and is not supported');
      }
      return fromSeed(derive(keypair.secretKey.subarray(0, 32), chainCode));
    };
  }

  const keyHdkdEcdsa = createSeedDeriveFn(secp256k1PairFromSeed, secp256k1DeriveHard);

  var ed2curve$1 = {exports: {}};

  function commonjsRequire(path) {
  	throw new Error('Could not dynamically require "' + path + '". Please configure the dynamicRequireTargets or/and ignoreDynamicRequires option of @rollup/plugin-commonjs appropriately for this require call to work.');
  }

  var naclFast = {exports: {}};

  const require$$0 = /*@__PURE__*/getAugmentedNamespace(crypto$2);

  (function (module) {
  	(function(nacl) {
  	var gf = function(init) {
  	  var i, r = new Float64Array(16);
  	  if (init) for (i = 0; i < init.length; i++) r[i] = init[i];
  	  return r;
  	};
  	var randombytes = function() { throw new Error('no PRNG'); };
  	var _0 = new Uint8Array(16);
  	var _9 = new Uint8Array(32); _9[0] = 9;
  	var gf0 = gf(),
  	    gf1 = gf([1]),
  	    _121665 = gf([0xdb41, 1]),
  	    D = gf([0x78a3, 0x1359, 0x4dca, 0x75eb, 0xd8ab, 0x4141, 0x0a4d, 0x0070, 0xe898, 0x7779, 0x4079, 0x8cc7, 0xfe73, 0x2b6f, 0x6cee, 0x5203]),
  	    D2 = gf([0xf159, 0x26b2, 0x9b94, 0xebd6, 0xb156, 0x8283, 0x149a, 0x00e0, 0xd130, 0xeef3, 0x80f2, 0x198e, 0xfce7, 0x56df, 0xd9dc, 0x2406]),
  	    X = gf([0xd51a, 0x8f25, 0x2d60, 0xc956, 0xa7b2, 0x9525, 0xc760, 0x692c, 0xdc5c, 0xfdd6, 0xe231, 0xc0a4, 0x53fe, 0xcd6e, 0x36d3, 0x2169]),
  	    Y = gf([0x6658, 0x6666, 0x6666, 0x6666, 0x6666, 0x6666, 0x6666, 0x6666, 0x6666, 0x6666, 0x6666, 0x6666, 0x6666, 0x6666, 0x6666, 0x6666]),
  	    I = gf([0xa0b0, 0x4a0e, 0x1b27, 0xc4ee, 0xe478, 0xad2f, 0x1806, 0x2f43, 0xd7a7, 0x3dfb, 0x0099, 0x2b4d, 0xdf0b, 0x4fc1, 0x2480, 0x2b83]);
  	function ts64(x, i, h, l) {
  	  x[i]   = (h >> 24) & 0xff;
  	  x[i+1] = (h >> 16) & 0xff;
  	  x[i+2] = (h >>  8) & 0xff;
  	  x[i+3] = h & 0xff;
  	  x[i+4] = (l >> 24)  & 0xff;
  	  x[i+5] = (l >> 16)  & 0xff;
  	  x[i+6] = (l >>  8)  & 0xff;
  	  x[i+7] = l & 0xff;
  	}
  	function vn(x, xi, y, yi, n) {
  	  var i,d = 0;
  	  for (i = 0; i < n; i++) d |= x[xi+i]^y[yi+i];
  	  return (1 & ((d - 1) >>> 8)) - 1;
  	}
  	function crypto_verify_16(x, xi, y, yi) {
  	  return vn(x,xi,y,yi,16);
  	}
  	function crypto_verify_32(x, xi, y, yi) {
  	  return vn(x,xi,y,yi,32);
  	}
  	function core_salsa20(o, p, k, c) {
  	  var j0  = c[ 0] & 0xff | (c[ 1] & 0xff)<<8 | (c[ 2] & 0xff)<<16 | (c[ 3] & 0xff)<<24,
  	      j1  = k[ 0] & 0xff | (k[ 1] & 0xff)<<8 | (k[ 2] & 0xff)<<16 | (k[ 3] & 0xff)<<24,
  	      j2  = k[ 4] & 0xff | (k[ 5] & 0xff)<<8 | (k[ 6] & 0xff)<<16 | (k[ 7] & 0xff)<<24,
  	      j3  = k[ 8] & 0xff | (k[ 9] & 0xff)<<8 | (k[10] & 0xff)<<16 | (k[11] & 0xff)<<24,
  	      j4  = k[12] & 0xff | (k[13] & 0xff)<<8 | (k[14] & 0xff)<<16 | (k[15] & 0xff)<<24,
  	      j5  = c[ 4] & 0xff | (c[ 5] & 0xff)<<8 | (c[ 6] & 0xff)<<16 | (c[ 7] & 0xff)<<24,
  	      j6  = p[ 0] & 0xff | (p[ 1] & 0xff)<<8 | (p[ 2] & 0xff)<<16 | (p[ 3] & 0xff)<<24,
  	      j7  = p[ 4] & 0xff | (p[ 5] & 0xff)<<8 | (p[ 6] & 0xff)<<16 | (p[ 7] & 0xff)<<24,
  	      j8  = p[ 8] & 0xff | (p[ 9] & 0xff)<<8 | (p[10] & 0xff)<<16 | (p[11] & 0xff)<<24,
  	      j9  = p[12] & 0xff | (p[13] & 0xff)<<8 | (p[14] & 0xff)<<16 | (p[15] & 0xff)<<24,
  	      j10 = c[ 8] & 0xff | (c[ 9] & 0xff)<<8 | (c[10] & 0xff)<<16 | (c[11] & 0xff)<<24,
  	      j11 = k[16] & 0xff | (k[17] & 0xff)<<8 | (k[18] & 0xff)<<16 | (k[19] & 0xff)<<24,
  	      j12 = k[20] & 0xff | (k[21] & 0xff)<<8 | (k[22] & 0xff)<<16 | (k[23] & 0xff)<<24,
  	      j13 = k[24] & 0xff | (k[25] & 0xff)<<8 | (k[26] & 0xff)<<16 | (k[27] & 0xff)<<24,
  	      j14 = k[28] & 0xff | (k[29] & 0xff)<<8 | (k[30] & 0xff)<<16 | (k[31] & 0xff)<<24,
  	      j15 = c[12] & 0xff | (c[13] & 0xff)<<8 | (c[14] & 0xff)<<16 | (c[15] & 0xff)<<24;
  	  var x0 = j0, x1 = j1, x2 = j2, x3 = j3, x4 = j4, x5 = j5, x6 = j6, x7 = j7,
  	      x8 = j8, x9 = j9, x10 = j10, x11 = j11, x12 = j12, x13 = j13, x14 = j14,
  	      x15 = j15, u;
  	  for (var i = 0; i < 20; i += 2) {
  	    u = x0 + x12 | 0;
  	    x4 ^= u<<7 | u>>>(32-7);
  	    u = x4 + x0 | 0;
  	    x8 ^= u<<9 | u>>>(32-9);
  	    u = x8 + x4 | 0;
  	    x12 ^= u<<13 | u>>>(32-13);
  	    u = x12 + x8 | 0;
  	    x0 ^= u<<18 | u>>>(32-18);
  	    u = x5 + x1 | 0;
  	    x9 ^= u<<7 | u>>>(32-7);
  	    u = x9 + x5 | 0;
  	    x13 ^= u<<9 | u>>>(32-9);
  	    u = x13 + x9 | 0;
  	    x1 ^= u<<13 | u>>>(32-13);
  	    u = x1 + x13 | 0;
  	    x5 ^= u<<18 | u>>>(32-18);
  	    u = x10 + x6 | 0;
  	    x14 ^= u<<7 | u>>>(32-7);
  	    u = x14 + x10 | 0;
  	    x2 ^= u<<9 | u>>>(32-9);
  	    u = x2 + x14 | 0;
  	    x6 ^= u<<13 | u>>>(32-13);
  	    u = x6 + x2 | 0;
  	    x10 ^= u<<18 | u>>>(32-18);
  	    u = x15 + x11 | 0;
  	    x3 ^= u<<7 | u>>>(32-7);
  	    u = x3 + x15 | 0;
  	    x7 ^= u<<9 | u>>>(32-9);
  	    u = x7 + x3 | 0;
  	    x11 ^= u<<13 | u>>>(32-13);
  	    u = x11 + x7 | 0;
  	    x15 ^= u<<18 | u>>>(32-18);
  	    u = x0 + x3 | 0;
  	    x1 ^= u<<7 | u>>>(32-7);
  	    u = x1 + x0 | 0;
  	    x2 ^= u<<9 | u>>>(32-9);
  	    u = x2 + x1 | 0;
  	    x3 ^= u<<13 | u>>>(32-13);
  	    u = x3 + x2 | 0;
  	    x0 ^= u<<18 | u>>>(32-18);
  	    u = x5 + x4 | 0;
  	    x6 ^= u<<7 | u>>>(32-7);
  	    u = x6 + x5 | 0;
  	    x7 ^= u<<9 | u>>>(32-9);
  	    u = x7 + x6 | 0;
  	    x4 ^= u<<13 | u>>>(32-13);
  	    u = x4 + x7 | 0;
  	    x5 ^= u<<18 | u>>>(32-18);
  	    u = x10 + x9 | 0;
  	    x11 ^= u<<7 | u>>>(32-7);
  	    u = x11 + x10 | 0;
  	    x8 ^= u<<9 | u>>>(32-9);
  	    u = x8 + x11 | 0;
  	    x9 ^= u<<13 | u>>>(32-13);
  	    u = x9 + x8 | 0;
  	    x10 ^= u<<18 | u>>>(32-18);
  	    u = x15 + x14 | 0;
  	    x12 ^= u<<7 | u>>>(32-7);
  	    u = x12 + x15 | 0;
  	    x13 ^= u<<9 | u>>>(32-9);
  	    u = x13 + x12 | 0;
  	    x14 ^= u<<13 | u>>>(32-13);
  	    u = x14 + x13 | 0;
  	    x15 ^= u<<18 | u>>>(32-18);
  	  }
  	   x0 =  x0 +  j0 | 0;
  	   x1 =  x1 +  j1 | 0;
  	   x2 =  x2 +  j2 | 0;
  	   x3 =  x3 +  j3 | 0;
  	   x4 =  x4 +  j4 | 0;
  	   x5 =  x5 +  j5 | 0;
  	   x6 =  x6 +  j6 | 0;
  	   x7 =  x7 +  j7 | 0;
  	   x8 =  x8 +  j8 | 0;
  	   x9 =  x9 +  j9 | 0;
  	  x10 = x10 + j10 | 0;
  	  x11 = x11 + j11 | 0;
  	  x12 = x12 + j12 | 0;
  	  x13 = x13 + j13 | 0;
  	  x14 = x14 + j14 | 0;
  	  x15 = x15 + j15 | 0;
  	  o[ 0] = x0 >>>  0 & 0xff;
  	  o[ 1] = x0 >>>  8 & 0xff;
  	  o[ 2] = x0 >>> 16 & 0xff;
  	  o[ 3] = x0 >>> 24 & 0xff;
  	  o[ 4] = x1 >>>  0 & 0xff;
  	  o[ 5] = x1 >>>  8 & 0xff;
  	  o[ 6] = x1 >>> 16 & 0xff;
  	  o[ 7] = x1 >>> 24 & 0xff;
  	  o[ 8] = x2 >>>  0 & 0xff;
  	  o[ 9] = x2 >>>  8 & 0xff;
  	  o[10] = x2 >>> 16 & 0xff;
  	  o[11] = x2 >>> 24 & 0xff;
  	  o[12] = x3 >>>  0 & 0xff;
  	  o[13] = x3 >>>  8 & 0xff;
  	  o[14] = x3 >>> 16 & 0xff;
  	  o[15] = x3 >>> 24 & 0xff;
  	  o[16] = x4 >>>  0 & 0xff;
  	  o[17] = x4 >>>  8 & 0xff;
  	  o[18] = x4 >>> 16 & 0xff;
  	  o[19] = x4 >>> 24 & 0xff;
  	  o[20] = x5 >>>  0 & 0xff;
  	  o[21] = x5 >>>  8 & 0xff;
  	  o[22] = x5 >>> 16 & 0xff;
  	  o[23] = x5 >>> 24 & 0xff;
  	  o[24] = x6 >>>  0 & 0xff;
  	  o[25] = x6 >>>  8 & 0xff;
  	  o[26] = x6 >>> 16 & 0xff;
  	  o[27] = x6 >>> 24 & 0xff;
  	  o[28] = x7 >>>  0 & 0xff;
  	  o[29] = x7 >>>  8 & 0xff;
  	  o[30] = x7 >>> 16 & 0xff;
  	  o[31] = x7 >>> 24 & 0xff;
  	  o[32] = x8 >>>  0 & 0xff;
  	  o[33] = x8 >>>  8 & 0xff;
  	  o[34] = x8 >>> 16 & 0xff;
  	  o[35] = x8 >>> 24 & 0xff;
  	  o[36] = x9 >>>  0 & 0xff;
  	  o[37] = x9 >>>  8 & 0xff;
  	  o[38] = x9 >>> 16 & 0xff;
  	  o[39] = x9 >>> 24 & 0xff;
  	  o[40] = x10 >>>  0 & 0xff;
  	  o[41] = x10 >>>  8 & 0xff;
  	  o[42] = x10 >>> 16 & 0xff;
  	  o[43] = x10 >>> 24 & 0xff;
  	  o[44] = x11 >>>  0 & 0xff;
  	  o[45] = x11 >>>  8 & 0xff;
  	  o[46] = x11 >>> 16 & 0xff;
  	  o[47] = x11 >>> 24 & 0xff;
  	  o[48] = x12 >>>  0 & 0xff;
  	  o[49] = x12 >>>  8 & 0xff;
  	  o[50] = x12 >>> 16 & 0xff;
  	  o[51] = x12 >>> 24 & 0xff;
  	  o[52] = x13 >>>  0 & 0xff;
  	  o[53] = x13 >>>  8 & 0xff;
  	  o[54] = x13 >>> 16 & 0xff;
  	  o[55] = x13 >>> 24 & 0xff;
  	  o[56] = x14 >>>  0 & 0xff;
  	  o[57] = x14 >>>  8 & 0xff;
  	  o[58] = x14 >>> 16 & 0xff;
  	  o[59] = x14 >>> 24 & 0xff;
  	  o[60] = x15 >>>  0 & 0xff;
  	  o[61] = x15 >>>  8 & 0xff;
  	  o[62] = x15 >>> 16 & 0xff;
  	  o[63] = x15 >>> 24 & 0xff;
  	}
  	function core_hsalsa20(o,p,k,c) {
  	  var j0  = c[ 0] & 0xff | (c[ 1] & 0xff)<<8 | (c[ 2] & 0xff)<<16 | (c[ 3] & 0xff)<<24,
  	      j1  = k[ 0] & 0xff | (k[ 1] & 0xff)<<8 | (k[ 2] & 0xff)<<16 | (k[ 3] & 0xff)<<24,
  	      j2  = k[ 4] & 0xff | (k[ 5] & 0xff)<<8 | (k[ 6] & 0xff)<<16 | (k[ 7] & 0xff)<<24,
  	      j3  = k[ 8] & 0xff | (k[ 9] & 0xff)<<8 | (k[10] & 0xff)<<16 | (k[11] & 0xff)<<24,
  	      j4  = k[12] & 0xff | (k[13] & 0xff)<<8 | (k[14] & 0xff)<<16 | (k[15] & 0xff)<<24,
  	      j5  = c[ 4] & 0xff | (c[ 5] & 0xff)<<8 | (c[ 6] & 0xff)<<16 | (c[ 7] & 0xff)<<24,
  	      j6  = p[ 0] & 0xff | (p[ 1] & 0xff)<<8 | (p[ 2] & 0xff)<<16 | (p[ 3] & 0xff)<<24,
  	      j7  = p[ 4] & 0xff | (p[ 5] & 0xff)<<8 | (p[ 6] & 0xff)<<16 | (p[ 7] & 0xff)<<24,
  	      j8  = p[ 8] & 0xff | (p[ 9] & 0xff)<<8 | (p[10] & 0xff)<<16 | (p[11] & 0xff)<<24,
  	      j9  = p[12] & 0xff | (p[13] & 0xff)<<8 | (p[14] & 0xff)<<16 | (p[15] & 0xff)<<24,
  	      j10 = c[ 8] & 0xff | (c[ 9] & 0xff)<<8 | (c[10] & 0xff)<<16 | (c[11] & 0xff)<<24,
  	      j11 = k[16] & 0xff | (k[17] & 0xff)<<8 | (k[18] & 0xff)<<16 | (k[19] & 0xff)<<24,
  	      j12 = k[20] & 0xff | (k[21] & 0xff)<<8 | (k[22] & 0xff)<<16 | (k[23] & 0xff)<<24,
  	      j13 = k[24] & 0xff | (k[25] & 0xff)<<8 | (k[26] & 0xff)<<16 | (k[27] & 0xff)<<24,
  	      j14 = k[28] & 0xff | (k[29] & 0xff)<<8 | (k[30] & 0xff)<<16 | (k[31] & 0xff)<<24,
  	      j15 = c[12] & 0xff | (c[13] & 0xff)<<8 | (c[14] & 0xff)<<16 | (c[15] & 0xff)<<24;
  	  var x0 = j0, x1 = j1, x2 = j2, x3 = j3, x4 = j4, x5 = j5, x6 = j6, x7 = j7,
  	      x8 = j8, x9 = j9, x10 = j10, x11 = j11, x12 = j12, x13 = j13, x14 = j14,
  	      x15 = j15, u;
  	  for (var i = 0; i < 20; i += 2) {
  	    u = x0 + x12 | 0;
  	    x4 ^= u<<7 | u>>>(32-7);
  	    u = x4 + x0 | 0;
  	    x8 ^= u<<9 | u>>>(32-9);
  	    u = x8 + x4 | 0;
  	    x12 ^= u<<13 | u>>>(32-13);
  	    u = x12 + x8 | 0;
  	    x0 ^= u<<18 | u>>>(32-18);
  	    u = x5 + x1 | 0;
  	    x9 ^= u<<7 | u>>>(32-7);
  	    u = x9 + x5 | 0;
  	    x13 ^= u<<9 | u>>>(32-9);
  	    u = x13 + x9 | 0;
  	    x1 ^= u<<13 | u>>>(32-13);
  	    u = x1 + x13 | 0;
  	    x5 ^= u<<18 | u>>>(32-18);
  	    u = x10 + x6 | 0;
  	    x14 ^= u<<7 | u>>>(32-7);
  	    u = x14 + x10 | 0;
  	    x2 ^= u<<9 | u>>>(32-9);
  	    u = x2 + x14 | 0;
  	    x6 ^= u<<13 | u>>>(32-13);
  	    u = x6 + x2 | 0;
  	    x10 ^= u<<18 | u>>>(32-18);
  	    u = x15 + x11 | 0;
  	    x3 ^= u<<7 | u>>>(32-7);
  	    u = x3 + x15 | 0;
  	    x7 ^= u<<9 | u>>>(32-9);
  	    u = x7 + x3 | 0;
  	    x11 ^= u<<13 | u>>>(32-13);
  	    u = x11 + x7 | 0;
  	    x15 ^= u<<18 | u>>>(32-18);
  	    u = x0 + x3 | 0;
  	    x1 ^= u<<7 | u>>>(32-7);
  	    u = x1 + x0 | 0;
  	    x2 ^= u<<9 | u>>>(32-9);
  	    u = x2 + x1 | 0;
  	    x3 ^= u<<13 | u>>>(32-13);
  	    u = x3 + x2 | 0;
  	    x0 ^= u<<18 | u>>>(32-18);
  	    u = x5 + x4 | 0;
  	    x6 ^= u<<7 | u>>>(32-7);
  	    u = x6 + x5 | 0;
  	    x7 ^= u<<9 | u>>>(32-9);
  	    u = x7 + x6 | 0;
  	    x4 ^= u<<13 | u>>>(32-13);
  	    u = x4 + x7 | 0;
  	    x5 ^= u<<18 | u>>>(32-18);
  	    u = x10 + x9 | 0;
  	    x11 ^= u<<7 | u>>>(32-7);
  	    u = x11 + x10 | 0;
  	    x8 ^= u<<9 | u>>>(32-9);
  	    u = x8 + x11 | 0;
  	    x9 ^= u<<13 | u>>>(32-13);
  	    u = x9 + x8 | 0;
  	    x10 ^= u<<18 | u>>>(32-18);
  	    u = x15 + x14 | 0;
  	    x12 ^= u<<7 | u>>>(32-7);
  	    u = x12 + x15 | 0;
  	    x13 ^= u<<9 | u>>>(32-9);
  	    u = x13 + x12 | 0;
  	    x14 ^= u<<13 | u>>>(32-13);
  	    u = x14 + x13 | 0;
  	    x15 ^= u<<18 | u>>>(32-18);
  	  }
  	  o[ 0] = x0 >>>  0 & 0xff;
  	  o[ 1] = x0 >>>  8 & 0xff;
  	  o[ 2] = x0 >>> 16 & 0xff;
  	  o[ 3] = x0 >>> 24 & 0xff;
  	  o[ 4] = x5 >>>  0 & 0xff;
  	  o[ 5] = x5 >>>  8 & 0xff;
  	  o[ 6] = x5 >>> 16 & 0xff;
  	  o[ 7] = x5 >>> 24 & 0xff;
  	  o[ 8] = x10 >>>  0 & 0xff;
  	  o[ 9] = x10 >>>  8 & 0xff;
  	  o[10] = x10 >>> 16 & 0xff;
  	  o[11] = x10 >>> 24 & 0xff;
  	  o[12] = x15 >>>  0 & 0xff;
  	  o[13] = x15 >>>  8 & 0xff;
  	  o[14] = x15 >>> 16 & 0xff;
  	  o[15] = x15 >>> 24 & 0xff;
  	  o[16] = x6 >>>  0 & 0xff;
  	  o[17] = x6 >>>  8 & 0xff;
  	  o[18] = x6 >>> 16 & 0xff;
  	  o[19] = x6 >>> 24 & 0xff;
  	  o[20] = x7 >>>  0 & 0xff;
  	  o[21] = x7 >>>  8 & 0xff;
  	  o[22] = x7 >>> 16 & 0xff;
  	  o[23] = x7 >>> 24 & 0xff;
  	  o[24] = x8 >>>  0 & 0xff;
  	  o[25] = x8 >>>  8 & 0xff;
  	  o[26] = x8 >>> 16 & 0xff;
  	  o[27] = x8 >>> 24 & 0xff;
  	  o[28] = x9 >>>  0 & 0xff;
  	  o[29] = x9 >>>  8 & 0xff;
  	  o[30] = x9 >>> 16 & 0xff;
  	  o[31] = x9 >>> 24 & 0xff;
  	}
  	function crypto_core_salsa20(out,inp,k,c) {
  	  core_salsa20(out,inp,k,c);
  	}
  	function crypto_core_hsalsa20(out,inp,k,c) {
  	  core_hsalsa20(out,inp,k,c);
  	}
  	var sigma = new Uint8Array([101, 120, 112, 97, 110, 100, 32, 51, 50, 45, 98, 121, 116, 101, 32, 107]);
  	function crypto_stream_salsa20_xor(c,cpos,m,mpos,b,n,k) {
  	  var z = new Uint8Array(16), x = new Uint8Array(64);
  	  var u, i;
  	  for (i = 0; i < 16; i++) z[i] = 0;
  	  for (i = 0; i < 8; i++) z[i] = n[i];
  	  while (b >= 64) {
  	    crypto_core_salsa20(x,z,k,sigma);
  	    for (i = 0; i < 64; i++) c[cpos+i] = m[mpos+i] ^ x[i];
  	    u = 1;
  	    for (i = 8; i < 16; i++) {
  	      u = u + (z[i] & 0xff) | 0;
  	      z[i] = u & 0xff;
  	      u >>>= 8;
  	    }
  	    b -= 64;
  	    cpos += 64;
  	    mpos += 64;
  	  }
  	  if (b > 0) {
  	    crypto_core_salsa20(x,z,k,sigma);
  	    for (i = 0; i < b; i++) c[cpos+i] = m[mpos+i] ^ x[i];
  	  }
  	  return 0;
  	}
  	function crypto_stream_salsa20(c,cpos,b,n,k) {
  	  var z = new Uint8Array(16), x = new Uint8Array(64);
  	  var u, i;
  	  for (i = 0; i < 16; i++) z[i] = 0;
  	  for (i = 0; i < 8; i++) z[i] = n[i];
  	  while (b >= 64) {
  	    crypto_core_salsa20(x,z,k,sigma);
  	    for (i = 0; i < 64; i++) c[cpos+i] = x[i];
  	    u = 1;
  	    for (i = 8; i < 16; i++) {
  	      u = u + (z[i] & 0xff) | 0;
  	      z[i] = u & 0xff;
  	      u >>>= 8;
  	    }
  	    b -= 64;
  	    cpos += 64;
  	  }
  	  if (b > 0) {
  	    crypto_core_salsa20(x,z,k,sigma);
  	    for (i = 0; i < b; i++) c[cpos+i] = x[i];
  	  }
  	  return 0;
  	}
  	function crypto_stream(c,cpos,d,n,k) {
  	  var s = new Uint8Array(32);
  	  crypto_core_hsalsa20(s,n,k,sigma);
  	  var sn = new Uint8Array(8);
  	  for (var i = 0; i < 8; i++) sn[i] = n[i+16];
  	  return crypto_stream_salsa20(c,cpos,d,sn,s);
  	}
  	function crypto_stream_xor(c,cpos,m,mpos,d,n,k) {
  	  var s = new Uint8Array(32);
  	  crypto_core_hsalsa20(s,n,k,sigma);
  	  var sn = new Uint8Array(8);
  	  for (var i = 0; i < 8; i++) sn[i] = n[i+16];
  	  return crypto_stream_salsa20_xor(c,cpos,m,mpos,d,sn,s);
  	}
  	var poly1305 = function(key) {
  	  this.buffer = new Uint8Array(16);
  	  this.r = new Uint16Array(10);
  	  this.h = new Uint16Array(10);
  	  this.pad = new Uint16Array(8);
  	  this.leftover = 0;
  	  this.fin = 0;
  	  var t0, t1, t2, t3, t4, t5, t6, t7;
  	  t0 = key[ 0] & 0xff | (key[ 1] & 0xff) << 8; this.r[0] = ( t0                     ) & 0x1fff;
  	  t1 = key[ 2] & 0xff | (key[ 3] & 0xff) << 8; this.r[1] = ((t0 >>> 13) | (t1 <<  3)) & 0x1fff;
  	  t2 = key[ 4] & 0xff | (key[ 5] & 0xff) << 8; this.r[2] = ((t1 >>> 10) | (t2 <<  6)) & 0x1f03;
  	  t3 = key[ 6] & 0xff | (key[ 7] & 0xff) << 8; this.r[3] = ((t2 >>>  7) | (t3 <<  9)) & 0x1fff;
  	  t4 = key[ 8] & 0xff | (key[ 9] & 0xff) << 8; this.r[4] = ((t3 >>>  4) | (t4 << 12)) & 0x00ff;
  	  this.r[5] = ((t4 >>>  1)) & 0x1ffe;
  	  t5 = key[10] & 0xff | (key[11] & 0xff) << 8; this.r[6] = ((t4 >>> 14) | (t5 <<  2)) & 0x1fff;
  	  t6 = key[12] & 0xff | (key[13] & 0xff) << 8; this.r[7] = ((t5 >>> 11) | (t6 <<  5)) & 0x1f81;
  	  t7 = key[14] & 0xff | (key[15] & 0xff) << 8; this.r[8] = ((t6 >>>  8) | (t7 <<  8)) & 0x1fff;
  	  this.r[9] = ((t7 >>>  5)) & 0x007f;
  	  this.pad[0] = key[16] & 0xff | (key[17] & 0xff) << 8;
  	  this.pad[1] = key[18] & 0xff | (key[19] & 0xff) << 8;
  	  this.pad[2] = key[20] & 0xff | (key[21] & 0xff) << 8;
  	  this.pad[3] = key[22] & 0xff | (key[23] & 0xff) << 8;
  	  this.pad[4] = key[24] & 0xff | (key[25] & 0xff) << 8;
  	  this.pad[5] = key[26] & 0xff | (key[27] & 0xff) << 8;
  	  this.pad[6] = key[28] & 0xff | (key[29] & 0xff) << 8;
  	  this.pad[7] = key[30] & 0xff | (key[31] & 0xff) << 8;
  	};
  	poly1305.prototype.blocks = function(m, mpos, bytes) {
  	  var hibit = this.fin ? 0 : (1 << 11);
  	  var t0, t1, t2, t3, t4, t5, t6, t7, c;
  	  var d0, d1, d2, d3, d4, d5, d6, d7, d8, d9;
  	  var h0 = this.h[0],
  	      h1 = this.h[1],
  	      h2 = this.h[2],
  	      h3 = this.h[3],
  	      h4 = this.h[4],
  	      h5 = this.h[5],
  	      h6 = this.h[6],
  	      h7 = this.h[7],
  	      h8 = this.h[8],
  	      h9 = this.h[9];
  	  var r0 = this.r[0],
  	      r1 = this.r[1],
  	      r2 = this.r[2],
  	      r3 = this.r[3],
  	      r4 = this.r[4],
  	      r5 = this.r[5],
  	      r6 = this.r[6],
  	      r7 = this.r[7],
  	      r8 = this.r[8],
  	      r9 = this.r[9];
  	  while (bytes >= 16) {
  	    t0 = m[mpos+ 0] & 0xff | (m[mpos+ 1] & 0xff) << 8; h0 += ( t0                     ) & 0x1fff;
  	    t1 = m[mpos+ 2] & 0xff | (m[mpos+ 3] & 0xff) << 8; h1 += ((t0 >>> 13) | (t1 <<  3)) & 0x1fff;
  	    t2 = m[mpos+ 4] & 0xff | (m[mpos+ 5] & 0xff) << 8; h2 += ((t1 >>> 10) | (t2 <<  6)) & 0x1fff;
  	    t3 = m[mpos+ 6] & 0xff | (m[mpos+ 7] & 0xff) << 8; h3 += ((t2 >>>  7) | (t3 <<  9)) & 0x1fff;
  	    t4 = m[mpos+ 8] & 0xff | (m[mpos+ 9] & 0xff) << 8; h4 += ((t3 >>>  4) | (t4 << 12)) & 0x1fff;
  	    h5 += ((t4 >>>  1)) & 0x1fff;
  	    t5 = m[mpos+10] & 0xff | (m[mpos+11] & 0xff) << 8; h6 += ((t4 >>> 14) | (t5 <<  2)) & 0x1fff;
  	    t6 = m[mpos+12] & 0xff | (m[mpos+13] & 0xff) << 8; h7 += ((t5 >>> 11) | (t6 <<  5)) & 0x1fff;
  	    t7 = m[mpos+14] & 0xff | (m[mpos+15] & 0xff) << 8; h8 += ((t6 >>>  8) | (t7 <<  8)) & 0x1fff;
  	    h9 += ((t7 >>> 5)) | hibit;
  	    c = 0;
  	    d0 = c;
  	    d0 += h0 * r0;
  	    d0 += h1 * (5 * r9);
  	    d0 += h2 * (5 * r8);
  	    d0 += h3 * (5 * r7);
  	    d0 += h4 * (5 * r6);
  	    c = (d0 >>> 13); d0 &= 0x1fff;
  	    d0 += h5 * (5 * r5);
  	    d0 += h6 * (5 * r4);
  	    d0 += h7 * (5 * r3);
  	    d0 += h8 * (5 * r2);
  	    d0 += h9 * (5 * r1);
  	    c += (d0 >>> 13); d0 &= 0x1fff;
  	    d1 = c;
  	    d1 += h0 * r1;
  	    d1 += h1 * r0;
  	    d1 += h2 * (5 * r9);
  	    d1 += h3 * (5 * r8);
  	    d1 += h4 * (5 * r7);
  	    c = (d1 >>> 13); d1 &= 0x1fff;
  	    d1 += h5 * (5 * r6);
  	    d1 += h6 * (5 * r5);
  	    d1 += h7 * (5 * r4);
  	    d1 += h8 * (5 * r3);
  	    d1 += h9 * (5 * r2);
  	    c += (d1 >>> 13); d1 &= 0x1fff;
  	    d2 = c;
  	    d2 += h0 * r2;
  	    d2 += h1 * r1;
  	    d2 += h2 * r0;
  	    d2 += h3 * (5 * r9);
  	    d2 += h4 * (5 * r8);
  	    c = (d2 >>> 13); d2 &= 0x1fff;
  	    d2 += h5 * (5 * r7);
  	    d2 += h6 * (5 * r6);
  	    d2 += h7 * (5 * r5);
  	    d2 += h8 * (5 * r4);
  	    d2 += h9 * (5 * r3);
  	    c += (d2 >>> 13); d2 &= 0x1fff;
  	    d3 = c;
  	    d3 += h0 * r3;
  	    d3 += h1 * r2;
  	    d3 += h2 * r1;
  	    d3 += h3 * r0;
  	    d3 += h4 * (5 * r9);
  	    c = (d3 >>> 13); d3 &= 0x1fff;
  	    d3 += h5 * (5 * r8);
  	    d3 += h6 * (5 * r7);
  	    d3 += h7 * (5 * r6);
  	    d3 += h8 * (5 * r5);
  	    d3 += h9 * (5 * r4);
  	    c += (d3 >>> 13); d3 &= 0x1fff;
  	    d4 = c;
  	    d4 += h0 * r4;
  	    d4 += h1 * r3;
  	    d4 += h2 * r2;
  	    d4 += h3 * r1;
  	    d4 += h4 * r0;
  	    c = (d4 >>> 13); d4 &= 0x1fff;
  	    d4 += h5 * (5 * r9);
  	    d4 += h6 * (5 * r8);
  	    d4 += h7 * (5 * r7);
  	    d4 += h8 * (5 * r6);
  	    d4 += h9 * (5 * r5);
  	    c += (d4 >>> 13); d4 &= 0x1fff;
  	    d5 = c;
  	    d5 += h0 * r5;
  	    d5 += h1 * r4;
  	    d5 += h2 * r3;
  	    d5 += h3 * r2;
  	    d5 += h4 * r1;
  	    c = (d5 >>> 13); d5 &= 0x1fff;
  	    d5 += h5 * r0;
  	    d5 += h6 * (5 * r9);
  	    d5 += h7 * (5 * r8);
  	    d5 += h8 * (5 * r7);
  	    d5 += h9 * (5 * r6);
  	    c += (d5 >>> 13); d5 &= 0x1fff;
  	    d6 = c;
  	    d6 += h0 * r6;
  	    d6 += h1 * r5;
  	    d6 += h2 * r4;
  	    d6 += h3 * r3;
  	    d6 += h4 * r2;
  	    c = (d6 >>> 13); d6 &= 0x1fff;
  	    d6 += h5 * r1;
  	    d6 += h6 * r0;
  	    d6 += h7 * (5 * r9);
  	    d6 += h8 * (5 * r8);
  	    d6 += h9 * (5 * r7);
  	    c += (d6 >>> 13); d6 &= 0x1fff;
  	    d7 = c;
  	    d7 += h0 * r7;
  	    d7 += h1 * r6;
  	    d7 += h2 * r5;
  	    d7 += h3 * r4;
  	    d7 += h4 * r3;
  	    c = (d7 >>> 13); d7 &= 0x1fff;
  	    d7 += h5 * r2;
  	    d7 += h6 * r1;
  	    d7 += h7 * r0;
  	    d7 += h8 * (5 * r9);
  	    d7 += h9 * (5 * r8);
  	    c += (d7 >>> 13); d7 &= 0x1fff;
  	    d8 = c;
  	    d8 += h0 * r8;
  	    d8 += h1 * r7;
  	    d8 += h2 * r6;
  	    d8 += h3 * r5;
  	    d8 += h4 * r4;
  	    c = (d8 >>> 13); d8 &= 0x1fff;
  	    d8 += h5 * r3;
  	    d8 += h6 * r2;
  	    d8 += h7 * r1;
  	    d8 += h8 * r0;
  	    d8 += h9 * (5 * r9);
  	    c += (d8 >>> 13); d8 &= 0x1fff;
  	    d9 = c;
  	    d9 += h0 * r9;
  	    d9 += h1 * r8;
  	    d9 += h2 * r7;
  	    d9 += h3 * r6;
  	    d9 += h4 * r5;
  	    c = (d9 >>> 13); d9 &= 0x1fff;
  	    d9 += h5 * r4;
  	    d9 += h6 * r3;
  	    d9 += h7 * r2;
  	    d9 += h8 * r1;
  	    d9 += h9 * r0;
  	    c += (d9 >>> 13); d9 &= 0x1fff;
  	    c = (((c << 2) + c)) | 0;
  	    c = (c + d0) | 0;
  	    d0 = c & 0x1fff;
  	    c = (c >>> 13);
  	    d1 += c;
  	    h0 = d0;
  	    h1 = d1;
  	    h2 = d2;
  	    h3 = d3;
  	    h4 = d4;
  	    h5 = d5;
  	    h6 = d6;
  	    h7 = d7;
  	    h8 = d8;
  	    h9 = d9;
  	    mpos += 16;
  	    bytes -= 16;
  	  }
  	  this.h[0] = h0;
  	  this.h[1] = h1;
  	  this.h[2] = h2;
  	  this.h[3] = h3;
  	  this.h[4] = h4;
  	  this.h[5] = h5;
  	  this.h[6] = h6;
  	  this.h[7] = h7;
  	  this.h[8] = h8;
  	  this.h[9] = h9;
  	};
  	poly1305.prototype.finish = function(mac, macpos) {
  	  var g = new Uint16Array(10);
  	  var c, mask, f, i;
  	  if (this.leftover) {
  	    i = this.leftover;
  	    this.buffer[i++] = 1;
  	    for (; i < 16; i++) this.buffer[i] = 0;
  	    this.fin = 1;
  	    this.blocks(this.buffer, 0, 16);
  	  }
  	  c = this.h[1] >>> 13;
  	  this.h[1] &= 0x1fff;
  	  for (i = 2; i < 10; i++) {
  	    this.h[i] += c;
  	    c = this.h[i] >>> 13;
  	    this.h[i] &= 0x1fff;
  	  }
  	  this.h[0] += (c * 5);
  	  c = this.h[0] >>> 13;
  	  this.h[0] &= 0x1fff;
  	  this.h[1] += c;
  	  c = this.h[1] >>> 13;
  	  this.h[1] &= 0x1fff;
  	  this.h[2] += c;
  	  g[0] = this.h[0] + 5;
  	  c = g[0] >>> 13;
  	  g[0] &= 0x1fff;
  	  for (i = 1; i < 10; i++) {
  	    g[i] = this.h[i] + c;
  	    c = g[i] >>> 13;
  	    g[i] &= 0x1fff;
  	  }
  	  g[9] -= (1 << 13);
  	  mask = (c ^ 1) - 1;
  	  for (i = 0; i < 10; i++) g[i] &= mask;
  	  mask = ~mask;
  	  for (i = 0; i < 10; i++) this.h[i] = (this.h[i] & mask) | g[i];
  	  this.h[0] = ((this.h[0]       ) | (this.h[1] << 13)                    ) & 0xffff;
  	  this.h[1] = ((this.h[1] >>>  3) | (this.h[2] << 10)                    ) & 0xffff;
  	  this.h[2] = ((this.h[2] >>>  6) | (this.h[3] <<  7)                    ) & 0xffff;
  	  this.h[3] = ((this.h[3] >>>  9) | (this.h[4] <<  4)                    ) & 0xffff;
  	  this.h[4] = ((this.h[4] >>> 12) | (this.h[5] <<  1) | (this.h[6] << 14)) & 0xffff;
  	  this.h[5] = ((this.h[6] >>>  2) | (this.h[7] << 11)                    ) & 0xffff;
  	  this.h[6] = ((this.h[7] >>>  5) | (this.h[8] <<  8)                    ) & 0xffff;
  	  this.h[7] = ((this.h[8] >>>  8) | (this.h[9] <<  5)                    ) & 0xffff;
  	  f = this.h[0] + this.pad[0];
  	  this.h[0] = f & 0xffff;
  	  for (i = 1; i < 8; i++) {
  	    f = (((this.h[i] + this.pad[i]) | 0) + (f >>> 16)) | 0;
  	    this.h[i] = f & 0xffff;
  	  }
  	  mac[macpos+ 0] = (this.h[0] >>> 0) & 0xff;
  	  mac[macpos+ 1] = (this.h[0] >>> 8) & 0xff;
  	  mac[macpos+ 2] = (this.h[1] >>> 0) & 0xff;
  	  mac[macpos+ 3] = (this.h[1] >>> 8) & 0xff;
  	  mac[macpos+ 4] = (this.h[2] >>> 0) & 0xff;
  	  mac[macpos+ 5] = (this.h[2] >>> 8) & 0xff;
  	  mac[macpos+ 6] = (this.h[3] >>> 0) & 0xff;
  	  mac[macpos+ 7] = (this.h[3] >>> 8) & 0xff;
  	  mac[macpos+ 8] = (this.h[4] >>> 0) & 0xff;
  	  mac[macpos+ 9] = (this.h[4] >>> 8) & 0xff;
  	  mac[macpos+10] = (this.h[5] >>> 0) & 0xff;
  	  mac[macpos+11] = (this.h[5] >>> 8) & 0xff;
  	  mac[macpos+12] = (this.h[6] >>> 0) & 0xff;
  	  mac[macpos+13] = (this.h[6] >>> 8) & 0xff;
  	  mac[macpos+14] = (this.h[7] >>> 0) & 0xff;
  	  mac[macpos+15] = (this.h[7] >>> 8) & 0xff;
  	};
  	poly1305.prototype.update = function(m, mpos, bytes) {
  	  var i, want;
  	  if (this.leftover) {
  	    want = (16 - this.leftover);
  	    if (want > bytes)
  	      want = bytes;
  	    for (i = 0; i < want; i++)
  	      this.buffer[this.leftover + i] = m[mpos+i];
  	    bytes -= want;
  	    mpos += want;
  	    this.leftover += want;
  	    if (this.leftover < 16)
  	      return;
  	    this.blocks(this.buffer, 0, 16);
  	    this.leftover = 0;
  	  }
  	  if (bytes >= 16) {
  	    want = bytes - (bytes % 16);
  	    this.blocks(m, mpos, want);
  	    mpos += want;
  	    bytes -= want;
  	  }
  	  if (bytes) {
  	    for (i = 0; i < bytes; i++)
  	      this.buffer[this.leftover + i] = m[mpos+i];
  	    this.leftover += bytes;
  	  }
  	};
  	function crypto_onetimeauth(out, outpos, m, mpos, n, k) {
  	  var s = new poly1305(k);
  	  s.update(m, mpos, n);
  	  s.finish(out, outpos);
  	  return 0;
  	}
  	function crypto_onetimeauth_verify(h, hpos, m, mpos, n, k) {
  	  var x = new Uint8Array(16);
  	  crypto_onetimeauth(x,0,m,mpos,n,k);
  	  return crypto_verify_16(h,hpos,x,0);
  	}
  	function crypto_secretbox(c,m,d,n,k) {
  	  var i;
  	  if (d < 32) return -1;
  	  crypto_stream_xor(c,0,m,0,d,n,k);
  	  crypto_onetimeauth(c, 16, c, 32, d - 32, c);
  	  for (i = 0; i < 16; i++) c[i] = 0;
  	  return 0;
  	}
  	function crypto_secretbox_open(m,c,d,n,k) {
  	  var i;
  	  var x = new Uint8Array(32);
  	  if (d < 32) return -1;
  	  crypto_stream(x,0,32,n,k);
  	  if (crypto_onetimeauth_verify(c, 16,c, 32,d - 32,x) !== 0) return -1;
  	  crypto_stream_xor(m,0,c,0,d,n,k);
  	  for (i = 0; i < 32; i++) m[i] = 0;
  	  return 0;
  	}
  	function set25519(r, a) {
  	  var i;
  	  for (i = 0; i < 16; i++) r[i] = a[i]|0;
  	}
  	function car25519(o) {
  	  var i, v, c = 1;
  	  for (i = 0; i < 16; i++) {
  	    v = o[i] + c + 65535;
  	    c = Math.floor(v / 65536);
  	    o[i] = v - c * 65536;
  	  }
  	  o[0] += c-1 + 37 * (c-1);
  	}
  	function sel25519(p, q, b) {
  	  var t, c = ~(b-1);
  	  for (var i = 0; i < 16; i++) {
  	    t = c & (p[i] ^ q[i]);
  	    p[i] ^= t;
  	    q[i] ^= t;
  	  }
  	}
  	function pack25519(o, n) {
  	  var i, j, b;
  	  var m = gf(), t = gf();
  	  for (i = 0; i < 16; i++) t[i] = n[i];
  	  car25519(t);
  	  car25519(t);
  	  car25519(t);
  	  for (j = 0; j < 2; j++) {
  	    m[0] = t[0] - 0xffed;
  	    for (i = 1; i < 15; i++) {
  	      m[i] = t[i] - 0xffff - ((m[i-1]>>16) & 1);
  	      m[i-1] &= 0xffff;
  	    }
  	    m[15] = t[15] - 0x7fff - ((m[14]>>16) & 1);
  	    b = (m[15]>>16) & 1;
  	    m[14] &= 0xffff;
  	    sel25519(t, m, 1-b);
  	  }
  	  for (i = 0; i < 16; i++) {
  	    o[2*i] = t[i] & 0xff;
  	    o[2*i+1] = t[i]>>8;
  	  }
  	}
  	function neq25519(a, b) {
  	  var c = new Uint8Array(32), d = new Uint8Array(32);
  	  pack25519(c, a);
  	  pack25519(d, b);
  	  return crypto_verify_32(c, 0, d, 0);
  	}
  	function par25519(a) {
  	  var d = new Uint8Array(32);
  	  pack25519(d, a);
  	  return d[0] & 1;
  	}
  	function unpack25519(o, n) {
  	  var i;
  	  for (i = 0; i < 16; i++) o[i] = n[2*i] + (n[2*i+1] << 8);
  	  o[15] &= 0x7fff;
  	}
  	function A(o, a, b) {
  	  for (var i = 0; i < 16; i++) o[i] = a[i] + b[i];
  	}
  	function Z(o, a, b) {
  	  for (var i = 0; i < 16; i++) o[i] = a[i] - b[i];
  	}
  	function M(o, a, b) {
  	  var v, c,
  	     t0 = 0,  t1 = 0,  t2 = 0,  t3 = 0,  t4 = 0,  t5 = 0,  t6 = 0,  t7 = 0,
  	     t8 = 0,  t9 = 0, t10 = 0, t11 = 0, t12 = 0, t13 = 0, t14 = 0, t15 = 0,
  	    t16 = 0, t17 = 0, t18 = 0, t19 = 0, t20 = 0, t21 = 0, t22 = 0, t23 = 0,
  	    t24 = 0, t25 = 0, t26 = 0, t27 = 0, t28 = 0, t29 = 0, t30 = 0,
  	    b0 = b[0],
  	    b1 = b[1],
  	    b2 = b[2],
  	    b3 = b[3],
  	    b4 = b[4],
  	    b5 = b[5],
  	    b6 = b[6],
  	    b7 = b[7],
  	    b8 = b[8],
  	    b9 = b[9],
  	    b10 = b[10],
  	    b11 = b[11],
  	    b12 = b[12],
  	    b13 = b[13],
  	    b14 = b[14],
  	    b15 = b[15];
  	  v = a[0];
  	  t0 += v * b0;
  	  t1 += v * b1;
  	  t2 += v * b2;
  	  t3 += v * b3;
  	  t4 += v * b4;
  	  t5 += v * b5;
  	  t6 += v * b6;
  	  t7 += v * b7;
  	  t8 += v * b8;
  	  t9 += v * b9;
  	  t10 += v * b10;
  	  t11 += v * b11;
  	  t12 += v * b12;
  	  t13 += v * b13;
  	  t14 += v * b14;
  	  t15 += v * b15;
  	  v = a[1];
  	  t1 += v * b0;
  	  t2 += v * b1;
  	  t3 += v * b2;
  	  t4 += v * b3;
  	  t5 += v * b4;
  	  t6 += v * b5;
  	  t7 += v * b6;
  	  t8 += v * b7;
  	  t9 += v * b8;
  	  t10 += v * b9;
  	  t11 += v * b10;
  	  t12 += v * b11;
  	  t13 += v * b12;
  	  t14 += v * b13;
  	  t15 += v * b14;
  	  t16 += v * b15;
  	  v = a[2];
  	  t2 += v * b0;
  	  t3 += v * b1;
  	  t4 += v * b2;
  	  t5 += v * b3;
  	  t6 += v * b4;
  	  t7 += v * b5;
  	  t8 += v * b6;
  	  t9 += v * b7;
  	  t10 += v * b8;
  	  t11 += v * b9;
  	  t12 += v * b10;
  	  t13 += v * b11;
  	  t14 += v * b12;
  	  t15 += v * b13;
  	  t16 += v * b14;
  	  t17 += v * b15;
  	  v = a[3];
  	  t3 += v * b0;
  	  t4 += v * b1;
  	  t5 += v * b2;
  	  t6 += v * b3;
  	  t7 += v * b4;
  	  t8 += v * b5;
  	  t9 += v * b6;
  	  t10 += v * b7;
  	  t11 += v * b8;
  	  t12 += v * b9;
  	  t13 += v * b10;
  	  t14 += v * b11;
  	  t15 += v * b12;
  	  t16 += v * b13;
  	  t17 += v * b14;
  	  t18 += v * b15;
  	  v = a[4];
  	  t4 += v * b0;
  	  t5 += v * b1;
  	  t6 += v * b2;
  	  t7 += v * b3;
  	  t8 += v * b4;
  	  t9 += v * b5;
  	  t10 += v * b6;
  	  t11 += v * b7;
  	  t12 += v * b8;
  	  t13 += v * b9;
  	  t14 += v * b10;
  	  t15 += v * b11;
  	  t16 += v * b12;
  	  t17 += v * b13;
  	  t18 += v * b14;
  	  t19 += v * b15;
  	  v = a[5];
  	  t5 += v * b0;
  	  t6 += v * b1;
  	  t7 += v * b2;
  	  t8 += v * b3;
  	  t9 += v * b4;
  	  t10 += v * b5;
  	  t11 += v * b6;
  	  t12 += v * b7;
  	  t13 += v * b8;
  	  t14 += v * b9;
  	  t15 += v * b10;
  	  t16 += v * b11;
  	  t17 += v * b12;
  	  t18 += v * b13;
  	  t19 += v * b14;
  	  t20 += v * b15;
  	  v = a[6];
  	  t6 += v * b0;
  	  t7 += v * b1;
  	  t8 += v * b2;
  	  t9 += v * b3;
  	  t10 += v * b4;
  	  t11 += v * b5;
  	  t12 += v * b6;
  	  t13 += v * b7;
  	  t14 += v * b8;
  	  t15 += v * b9;
  	  t16 += v * b10;
  	  t17 += v * b11;
  	  t18 += v * b12;
  	  t19 += v * b13;
  	  t20 += v * b14;
  	  t21 += v * b15;
  	  v = a[7];
  	  t7 += v * b0;
  	  t8 += v * b1;
  	  t9 += v * b2;
  	  t10 += v * b3;
  	  t11 += v * b4;
  	  t12 += v * b5;
  	  t13 += v * b6;
  	  t14 += v * b7;
  	  t15 += v * b8;
  	  t16 += v * b9;
  	  t17 += v * b10;
  	  t18 += v * b11;
  	  t19 += v * b12;
  	  t20 += v * b13;
  	  t21 += v * b14;
  	  t22 += v * b15;
  	  v = a[8];
  	  t8 += v * b0;
  	  t9 += v * b1;
  	  t10 += v * b2;
  	  t11 += v * b3;
  	  t12 += v * b4;
  	  t13 += v * b5;
  	  t14 += v * b6;
  	  t15 += v * b7;
  	  t16 += v * b8;
  	  t17 += v * b9;
  	  t18 += v * b10;
  	  t19 += v * b11;
  	  t20 += v * b12;
  	  t21 += v * b13;
  	  t22 += v * b14;
  	  t23 += v * b15;
  	  v = a[9];
  	  t9 += v * b0;
  	  t10 += v * b1;
  	  t11 += v * b2;
  	  t12 += v * b3;
  	  t13 += v * b4;
  	  t14 += v * b5;
  	  t15 += v * b6;
  	  t16 += v * b7;
  	  t17 += v * b8;
  	  t18 += v * b9;
  	  t19 += v * b10;
  	  t20 += v * b11;
  	  t21 += v * b12;
  	  t22 += v * b13;
  	  t23 += v * b14;
  	  t24 += v * b15;
  	  v = a[10];
  	  t10 += v * b0;
  	  t11 += v * b1;
  	  t12 += v * b2;
  	  t13 += v * b3;
  	  t14 += v * b4;
  	  t15 += v * b5;
  	  t16 += v * b6;
  	  t17 += v * b7;
  	  t18 += v * b8;
  	  t19 += v * b9;
  	  t20 += v * b10;
  	  t21 += v * b11;
  	  t22 += v * b12;
  	  t23 += v * b13;
  	  t24 += v * b14;
  	  t25 += v * b15;
  	  v = a[11];
  	  t11 += v * b0;
  	  t12 += v * b1;
  	  t13 += v * b2;
  	  t14 += v * b3;
  	  t15 += v * b4;
  	  t16 += v * b5;
  	  t17 += v * b6;
  	  t18 += v * b7;
  	  t19 += v * b8;
  	  t20 += v * b9;
  	  t21 += v * b10;
  	  t22 += v * b11;
  	  t23 += v * b12;
  	  t24 += v * b13;
  	  t25 += v * b14;
  	  t26 += v * b15;
  	  v = a[12];
  	  t12 += v * b0;
  	  t13 += v * b1;
  	  t14 += v * b2;
  	  t15 += v * b3;
  	  t16 += v * b4;
  	  t17 += v * b5;
  	  t18 += v * b6;
  	  t19 += v * b7;
  	  t20 += v * b8;
  	  t21 += v * b9;
  	  t22 += v * b10;
  	  t23 += v * b11;
  	  t24 += v * b12;
  	  t25 += v * b13;
  	  t26 += v * b14;
  	  t27 += v * b15;
  	  v = a[13];
  	  t13 += v * b0;
  	  t14 += v * b1;
  	  t15 += v * b2;
  	  t16 += v * b3;
  	  t17 += v * b4;
  	  t18 += v * b5;
  	  t19 += v * b6;
  	  t20 += v * b7;
  	  t21 += v * b8;
  	  t22 += v * b9;
  	  t23 += v * b10;
  	  t24 += v * b11;
  	  t25 += v * b12;
  	  t26 += v * b13;
  	  t27 += v * b14;
  	  t28 += v * b15;
  	  v = a[14];
  	  t14 += v * b0;
  	  t15 += v * b1;
  	  t16 += v * b2;
  	  t17 += v * b3;
  	  t18 += v * b4;
  	  t19 += v * b5;
  	  t20 += v * b6;
  	  t21 += v * b7;
  	  t22 += v * b8;
  	  t23 += v * b9;
  	  t24 += v * b10;
  	  t25 += v * b11;
  	  t26 += v * b12;
  	  t27 += v * b13;
  	  t28 += v * b14;
  	  t29 += v * b15;
  	  v = a[15];
  	  t15 += v * b0;
  	  t16 += v * b1;
  	  t17 += v * b2;
  	  t18 += v * b3;
  	  t19 += v * b4;
  	  t20 += v * b5;
  	  t21 += v * b6;
  	  t22 += v * b7;
  	  t23 += v * b8;
  	  t24 += v * b9;
  	  t25 += v * b10;
  	  t26 += v * b11;
  	  t27 += v * b12;
  	  t28 += v * b13;
  	  t29 += v * b14;
  	  t30 += v * b15;
  	  t0  += 38 * t16;
  	  t1  += 38 * t17;
  	  t2  += 38 * t18;
  	  t3  += 38 * t19;
  	  t4  += 38 * t20;
  	  t5  += 38 * t21;
  	  t6  += 38 * t22;
  	  t7  += 38 * t23;
  	  t8  += 38 * t24;
  	  t9  += 38 * t25;
  	  t10 += 38 * t26;
  	  t11 += 38 * t27;
  	  t12 += 38 * t28;
  	  t13 += 38 * t29;
  	  t14 += 38 * t30;
  	  c = 1;
  	  v =  t0 + c + 65535; c = Math.floor(v / 65536);  t0 = v - c * 65536;
  	  v =  t1 + c + 65535; c = Math.floor(v / 65536);  t1 = v - c * 65536;
  	  v =  t2 + c + 65535; c = Math.floor(v / 65536);  t2 = v - c * 65536;
  	  v =  t3 + c + 65535; c = Math.floor(v / 65536);  t3 = v - c * 65536;
  	  v =  t4 + c + 65535; c = Math.floor(v / 65536);  t4 = v - c * 65536;
  	  v =  t5 + c + 65535; c = Math.floor(v / 65536);  t5 = v - c * 65536;
  	  v =  t6 + c + 65535; c = Math.floor(v / 65536);  t6 = v - c * 65536;
  	  v =  t7 + c + 65535; c = Math.floor(v / 65536);  t7 = v - c * 65536;
  	  v =  t8 + c + 65535; c = Math.floor(v / 65536);  t8 = v - c * 65536;
  	  v =  t9 + c + 65535; c = Math.floor(v / 65536);  t9 = v - c * 65536;
  	  v = t10 + c + 65535; c = Math.floor(v / 65536); t10 = v - c * 65536;
  	  v = t11 + c + 65535; c = Math.floor(v / 65536); t11 = v - c * 65536;
  	  v = t12 + c + 65535; c = Math.floor(v / 65536); t12 = v - c * 65536;
  	  v = t13 + c + 65535; c = Math.floor(v / 65536); t13 = v - c * 65536;
  	  v = t14 + c + 65535; c = Math.floor(v / 65536); t14 = v - c * 65536;
  	  v = t15 + c + 65535; c = Math.floor(v / 65536); t15 = v - c * 65536;
  	  t0 += c-1 + 37 * (c-1);
  	  c = 1;
  	  v =  t0 + c + 65535; c = Math.floor(v / 65536);  t0 = v - c * 65536;
  	  v =  t1 + c + 65535; c = Math.floor(v / 65536);  t1 = v - c * 65536;
  	  v =  t2 + c + 65535; c = Math.floor(v / 65536);  t2 = v - c * 65536;
  	  v =  t3 + c + 65535; c = Math.floor(v / 65536);  t3 = v - c * 65536;
  	  v =  t4 + c + 65535; c = Math.floor(v / 65536);  t4 = v - c * 65536;
  	  v =  t5 + c + 65535; c = Math.floor(v / 65536);  t5 = v - c * 65536;
  	  v =  t6 + c + 65535; c = Math.floor(v / 65536);  t6 = v - c * 65536;
  	  v =  t7 + c + 65535; c = Math.floor(v / 65536);  t7 = v - c * 65536;
  	  v =  t8 + c + 65535; c = Math.floor(v / 65536);  t8 = v - c * 65536;
  	  v =  t9 + c + 65535; c = Math.floor(v / 65536);  t9 = v - c * 65536;
  	  v = t10 + c + 65535; c = Math.floor(v / 65536); t10 = v - c * 65536;
  	  v = t11 + c + 65535; c = Math.floor(v / 65536); t11 = v - c * 65536;
  	  v = t12 + c + 65535; c = Math.floor(v / 65536); t12 = v - c * 65536;
  	  v = t13 + c + 65535; c = Math.floor(v / 65536); t13 = v - c * 65536;
  	  v = t14 + c + 65535; c = Math.floor(v / 65536); t14 = v - c * 65536;
  	  v = t15 + c + 65535; c = Math.floor(v / 65536); t15 = v - c * 65536;
  	  t0 += c-1 + 37 * (c-1);
  	  o[ 0] = t0;
  	  o[ 1] = t1;
  	  o[ 2] = t2;
  	  o[ 3] = t3;
  	  o[ 4] = t4;
  	  o[ 5] = t5;
  	  o[ 6] = t6;
  	  o[ 7] = t7;
  	  o[ 8] = t8;
  	  o[ 9] = t9;
  	  o[10] = t10;
  	  o[11] = t11;
  	  o[12] = t12;
  	  o[13] = t13;
  	  o[14] = t14;
  	  o[15] = t15;
  	}
  	function S(o, a) {
  	  M(o, a, a);
  	}
  	function inv25519(o, i) {
  	  var c = gf();
  	  var a;
  	  for (a = 0; a < 16; a++) c[a] = i[a];
  	  for (a = 253; a >= 0; a--) {
  	    S(c, c);
  	    if(a !== 2 && a !== 4) M(c, c, i);
  	  }
  	  for (a = 0; a < 16; a++) o[a] = c[a];
  	}
  	function pow2523(o, i) {
  	  var c = gf();
  	  var a;
  	  for (a = 0; a < 16; a++) c[a] = i[a];
  	  for (a = 250; a >= 0; a--) {
  	      S(c, c);
  	      if(a !== 1) M(c, c, i);
  	  }
  	  for (a = 0; a < 16; a++) o[a] = c[a];
  	}
  	function crypto_scalarmult(q, n, p) {
  	  var z = new Uint8Array(32);
  	  var x = new Float64Array(80), r, i;
  	  var a = gf(), b = gf(), c = gf(),
  	      d = gf(), e = gf(), f = gf();
  	  for (i = 0; i < 31; i++) z[i] = n[i];
  	  z[31]=(n[31]&127)|64;
  	  z[0]&=248;
  	  unpack25519(x,p);
  	  for (i = 0; i < 16; i++) {
  	    b[i]=x[i];
  	    d[i]=a[i]=c[i]=0;
  	  }
  	  a[0]=d[0]=1;
  	  for (i=254; i>=0; --i) {
  	    r=(z[i>>>3]>>>(i&7))&1;
  	    sel25519(a,b,r);
  	    sel25519(c,d,r);
  	    A(e,a,c);
  	    Z(a,a,c);
  	    A(c,b,d);
  	    Z(b,b,d);
  	    S(d,e);
  	    S(f,a);
  	    M(a,c,a);
  	    M(c,b,e);
  	    A(e,a,c);
  	    Z(a,a,c);
  	    S(b,a);
  	    Z(c,d,f);
  	    M(a,c,_121665);
  	    A(a,a,d);
  	    M(c,c,a);
  	    M(a,d,f);
  	    M(d,b,x);
  	    S(b,e);
  	    sel25519(a,b,r);
  	    sel25519(c,d,r);
  	  }
  	  for (i = 0; i < 16; i++) {
  	    x[i+16]=a[i];
  	    x[i+32]=c[i];
  	    x[i+48]=b[i];
  	    x[i+64]=d[i];
  	  }
  	  var x32 = x.subarray(32);
  	  var x16 = x.subarray(16);
  	  inv25519(x32,x32);
  	  M(x16,x16,x32);
  	  pack25519(q,x16);
  	  return 0;
  	}
  	function crypto_scalarmult_base(q, n) {
  	  return crypto_scalarmult(q, n, _9);
  	}
  	function crypto_box_keypair(y, x) {
  	  randombytes(x, 32);
  	  return crypto_scalarmult_base(y, x);
  	}
  	function crypto_box_beforenm(k, y, x) {
  	  var s = new Uint8Array(32);
  	  crypto_scalarmult(s, x, y);
  	  return crypto_core_hsalsa20(k, _0, s, sigma);
  	}
  	var crypto_box_afternm = crypto_secretbox;
  	var crypto_box_open_afternm = crypto_secretbox_open;
  	function crypto_box(c, m, d, n, y, x) {
  	  var k = new Uint8Array(32);
  	  crypto_box_beforenm(k, y, x);
  	  return crypto_box_afternm(c, m, d, n, k);
  	}
  	function crypto_box_open(m, c, d, n, y, x) {
  	  var k = new Uint8Array(32);
  	  crypto_box_beforenm(k, y, x);
  	  return crypto_box_open_afternm(m, c, d, n, k);
  	}
  	var K = [
  	  0x428a2f98, 0xd728ae22, 0x71374491, 0x23ef65cd,
  	  0xb5c0fbcf, 0xec4d3b2f, 0xe9b5dba5, 0x8189dbbc,
  	  0x3956c25b, 0xf348b538, 0x59f111f1, 0xb605d019,
  	  0x923f82a4, 0xaf194f9b, 0xab1c5ed5, 0xda6d8118,
  	  0xd807aa98, 0xa3030242, 0x12835b01, 0x45706fbe,
  	  0x243185be, 0x4ee4b28c, 0x550c7dc3, 0xd5ffb4e2,
  	  0x72be5d74, 0xf27b896f, 0x80deb1fe, 0x3b1696b1,
  	  0x9bdc06a7, 0x25c71235, 0xc19bf174, 0xcf692694,
  	  0xe49b69c1, 0x9ef14ad2, 0xefbe4786, 0x384f25e3,
  	  0x0fc19dc6, 0x8b8cd5b5, 0x240ca1cc, 0x77ac9c65,
  	  0x2de92c6f, 0x592b0275, 0x4a7484aa, 0x6ea6e483,
  	  0x5cb0a9dc, 0xbd41fbd4, 0x76f988da, 0x831153b5,
  	  0x983e5152, 0xee66dfab, 0xa831c66d, 0x2db43210,
  	  0xb00327c8, 0x98fb213f, 0xbf597fc7, 0xbeef0ee4,
  	  0xc6e00bf3, 0x3da88fc2, 0xd5a79147, 0x930aa725,
  	  0x06ca6351, 0xe003826f, 0x14292967, 0x0a0e6e70,
  	  0x27b70a85, 0x46d22ffc, 0x2e1b2138, 0x5c26c926,
  	  0x4d2c6dfc, 0x5ac42aed, 0x53380d13, 0x9d95b3df,
  	  0x650a7354, 0x8baf63de, 0x766a0abb, 0x3c77b2a8,
  	  0x81c2c92e, 0x47edaee6, 0x92722c85, 0x1482353b,
  	  0xa2bfe8a1, 0x4cf10364, 0xa81a664b, 0xbc423001,
  	  0xc24b8b70, 0xd0f89791, 0xc76c51a3, 0x0654be30,
  	  0xd192e819, 0xd6ef5218, 0xd6990624, 0x5565a910,
  	  0xf40e3585, 0x5771202a, 0x106aa070, 0x32bbd1b8,
  	  0x19a4c116, 0xb8d2d0c8, 0x1e376c08, 0x5141ab53,
  	  0x2748774c, 0xdf8eeb99, 0x34b0bcb5, 0xe19b48a8,
  	  0x391c0cb3, 0xc5c95a63, 0x4ed8aa4a, 0xe3418acb,
  	  0x5b9cca4f, 0x7763e373, 0x682e6ff3, 0xd6b2b8a3,
  	  0x748f82ee, 0x5defb2fc, 0x78a5636f, 0x43172f60,
  	  0x84c87814, 0xa1f0ab72, 0x8cc70208, 0x1a6439ec,
  	  0x90befffa, 0x23631e28, 0xa4506ceb, 0xde82bde9,
  	  0xbef9a3f7, 0xb2c67915, 0xc67178f2, 0xe372532b,
  	  0xca273ece, 0xea26619c, 0xd186b8c7, 0x21c0c207,
  	  0xeada7dd6, 0xcde0eb1e, 0xf57d4f7f, 0xee6ed178,
  	  0x06f067aa, 0x72176fba, 0x0a637dc5, 0xa2c898a6,
  	  0x113f9804, 0xbef90dae, 0x1b710b35, 0x131c471b,
  	  0x28db77f5, 0x23047d84, 0x32caab7b, 0x40c72493,
  	  0x3c9ebe0a, 0x15c9bebc, 0x431d67c4, 0x9c100d4c,
  	  0x4cc5d4be, 0xcb3e42b6, 0x597f299c, 0xfc657e2a,
  	  0x5fcb6fab, 0x3ad6faec, 0x6c44198c, 0x4a475817
  	];
  	function crypto_hashblocks_hl(hh, hl, m, n) {
  	  var wh = new Int32Array(16), wl = new Int32Array(16),
  	      bh0, bh1, bh2, bh3, bh4, bh5, bh6, bh7,
  	      bl0, bl1, bl2, bl3, bl4, bl5, bl6, bl7,
  	      th, tl, i, j, h, l, a, b, c, d;
  	  var ah0 = hh[0],
  	      ah1 = hh[1],
  	      ah2 = hh[2],
  	      ah3 = hh[3],
  	      ah4 = hh[4],
  	      ah5 = hh[5],
  	      ah6 = hh[6],
  	      ah7 = hh[7],
  	      al0 = hl[0],
  	      al1 = hl[1],
  	      al2 = hl[2],
  	      al3 = hl[3],
  	      al4 = hl[4],
  	      al5 = hl[5],
  	      al6 = hl[6],
  	      al7 = hl[7];
  	  var pos = 0;
  	  while (n >= 128) {
  	    for (i = 0; i < 16; i++) {
  	      j = 8 * i + pos;
  	      wh[i] = (m[j+0] << 24) | (m[j+1] << 16) | (m[j+2] << 8) | m[j+3];
  	      wl[i] = (m[j+4] << 24) | (m[j+5] << 16) | (m[j+6] << 8) | m[j+7];
  	    }
  	    for (i = 0; i < 80; i++) {
  	      bh0 = ah0;
  	      bh1 = ah1;
  	      bh2 = ah2;
  	      bh3 = ah3;
  	      bh4 = ah4;
  	      bh5 = ah5;
  	      bh6 = ah6;
  	      bh7 = ah7;
  	      bl0 = al0;
  	      bl1 = al1;
  	      bl2 = al2;
  	      bl3 = al3;
  	      bl4 = al4;
  	      bl5 = al5;
  	      bl6 = al6;
  	      bl7 = al7;
  	      h = ah7;
  	      l = al7;
  	      a = l & 0xffff; b = l >>> 16;
  	      c = h & 0xffff; d = h >>> 16;
  	      h = ((ah4 >>> 14) | (al4 << (32-14))) ^ ((ah4 >>> 18) | (al4 << (32-18))) ^ ((al4 >>> (41-32)) | (ah4 << (32-(41-32))));
  	      l = ((al4 >>> 14) | (ah4 << (32-14))) ^ ((al4 >>> 18) | (ah4 << (32-18))) ^ ((ah4 >>> (41-32)) | (al4 << (32-(41-32))));
  	      a += l & 0xffff; b += l >>> 16;
  	      c += h & 0xffff; d += h >>> 16;
  	      h = (ah4 & ah5) ^ (~ah4 & ah6);
  	      l = (al4 & al5) ^ (~al4 & al6);
  	      a += l & 0xffff; b += l >>> 16;
  	      c += h & 0xffff; d += h >>> 16;
  	      h = K[i*2];
  	      l = K[i*2+1];
  	      a += l & 0xffff; b += l >>> 16;
  	      c += h & 0xffff; d += h >>> 16;
  	      h = wh[i%16];
  	      l = wl[i%16];
  	      a += l & 0xffff; b += l >>> 16;
  	      c += h & 0xffff; d += h >>> 16;
  	      b += a >>> 16;
  	      c += b >>> 16;
  	      d += c >>> 16;
  	      th = c & 0xffff | d << 16;
  	      tl = a & 0xffff | b << 16;
  	      h = th;
  	      l = tl;
  	      a = l & 0xffff; b = l >>> 16;
  	      c = h & 0xffff; d = h >>> 16;
  	      h = ((ah0 >>> 28) | (al0 << (32-28))) ^ ((al0 >>> (34-32)) | (ah0 << (32-(34-32)))) ^ ((al0 >>> (39-32)) | (ah0 << (32-(39-32))));
  	      l = ((al0 >>> 28) | (ah0 << (32-28))) ^ ((ah0 >>> (34-32)) | (al0 << (32-(34-32)))) ^ ((ah0 >>> (39-32)) | (al0 << (32-(39-32))));
  	      a += l & 0xffff; b += l >>> 16;
  	      c += h & 0xffff; d += h >>> 16;
  	      h = (ah0 & ah1) ^ (ah0 & ah2) ^ (ah1 & ah2);
  	      l = (al0 & al1) ^ (al0 & al2) ^ (al1 & al2);
  	      a += l & 0xffff; b += l >>> 16;
  	      c += h & 0xffff; d += h >>> 16;
  	      b += a >>> 16;
  	      c += b >>> 16;
  	      d += c >>> 16;
  	      bh7 = (c & 0xffff) | (d << 16);
  	      bl7 = (a & 0xffff) | (b << 16);
  	      h = bh3;
  	      l = bl3;
  	      a = l & 0xffff; b = l >>> 16;
  	      c = h & 0xffff; d = h >>> 16;
  	      h = th;
  	      l = tl;
  	      a += l & 0xffff; b += l >>> 16;
  	      c += h & 0xffff; d += h >>> 16;
  	      b += a >>> 16;
  	      c += b >>> 16;
  	      d += c >>> 16;
  	      bh3 = (c & 0xffff) | (d << 16);
  	      bl3 = (a & 0xffff) | (b << 16);
  	      ah1 = bh0;
  	      ah2 = bh1;
  	      ah3 = bh2;
  	      ah4 = bh3;
  	      ah5 = bh4;
  	      ah6 = bh5;
  	      ah7 = bh6;
  	      ah0 = bh7;
  	      al1 = bl0;
  	      al2 = bl1;
  	      al3 = bl2;
  	      al4 = bl3;
  	      al5 = bl4;
  	      al6 = bl5;
  	      al7 = bl6;
  	      al0 = bl7;
  	      if (i%16 === 15) {
  	        for (j = 0; j < 16; j++) {
  	          h = wh[j];
  	          l = wl[j];
  	          a = l & 0xffff; b = l >>> 16;
  	          c = h & 0xffff; d = h >>> 16;
  	          h = wh[(j+9)%16];
  	          l = wl[(j+9)%16];
  	          a += l & 0xffff; b += l >>> 16;
  	          c += h & 0xffff; d += h >>> 16;
  	          th = wh[(j+1)%16];
  	          tl = wl[(j+1)%16];
  	          h = ((th >>> 1) | (tl << (32-1))) ^ ((th >>> 8) | (tl << (32-8))) ^ (th >>> 7);
  	          l = ((tl >>> 1) | (th << (32-1))) ^ ((tl >>> 8) | (th << (32-8))) ^ ((tl >>> 7) | (th << (32-7)));
  	          a += l & 0xffff; b += l >>> 16;
  	          c += h & 0xffff; d += h >>> 16;
  	          th = wh[(j+14)%16];
  	          tl = wl[(j+14)%16];
  	          h = ((th >>> 19) | (tl << (32-19))) ^ ((tl >>> (61-32)) | (th << (32-(61-32)))) ^ (th >>> 6);
  	          l = ((tl >>> 19) | (th << (32-19))) ^ ((th >>> (61-32)) | (tl << (32-(61-32)))) ^ ((tl >>> 6) | (th << (32-6)));
  	          a += l & 0xffff; b += l >>> 16;
  	          c += h & 0xffff; d += h >>> 16;
  	          b += a >>> 16;
  	          c += b >>> 16;
  	          d += c >>> 16;
  	          wh[j] = (c & 0xffff) | (d << 16);
  	          wl[j] = (a & 0xffff) | (b << 16);
  	        }
  	      }
  	    }
  	    h = ah0;
  	    l = al0;
  	    a = l & 0xffff; b = l >>> 16;
  	    c = h & 0xffff; d = h >>> 16;
  	    h = hh[0];
  	    l = hl[0];
  	    a += l & 0xffff; b += l >>> 16;
  	    c += h & 0xffff; d += h >>> 16;
  	    b += a >>> 16;
  	    c += b >>> 16;
  	    d += c >>> 16;
  	    hh[0] = ah0 = (c & 0xffff) | (d << 16);
  	    hl[0] = al0 = (a & 0xffff) | (b << 16);
  	    h = ah1;
  	    l = al1;
  	    a = l & 0xffff; b = l >>> 16;
  	    c = h & 0xffff; d = h >>> 16;
  	    h = hh[1];
  	    l = hl[1];
  	    a += l & 0xffff; b += l >>> 16;
  	    c += h & 0xffff; d += h >>> 16;
  	    b += a >>> 16;
  	    c += b >>> 16;
  	    d += c >>> 16;
  	    hh[1] = ah1 = (c & 0xffff) | (d << 16);
  	    hl[1] = al1 = (a & 0xffff) | (b << 16);
  	    h = ah2;
  	    l = al2;
  	    a = l & 0xffff; b = l >>> 16;
  	    c = h & 0xffff; d = h >>> 16;
  	    h = hh[2];
  	    l = hl[2];
  	    a += l & 0xffff; b += l >>> 16;
  	    c += h & 0xffff; d += h >>> 16;
  	    b += a >>> 16;
  	    c += b >>> 16;
  	    d += c >>> 16;
  	    hh[2] = ah2 = (c & 0xffff) | (d << 16);
  	    hl[2] = al2 = (a & 0xffff) | (b << 16);
  	    h = ah3;
  	    l = al3;
  	    a = l & 0xffff; b = l >>> 16;
  	    c = h & 0xffff; d = h >>> 16;
  	    h = hh[3];
  	    l = hl[3];
  	    a += l & 0xffff; b += l >>> 16;
  	    c += h & 0xffff; d += h >>> 16;
  	    b += a >>> 16;
  	    c += b >>> 16;
  	    d += c >>> 16;
  	    hh[3] = ah3 = (c & 0xffff) | (d << 16);
  	    hl[3] = al3 = (a & 0xffff) | (b << 16);
  	    h = ah4;
  	    l = al4;
  	    a = l & 0xffff; b = l >>> 16;
  	    c = h & 0xffff; d = h >>> 16;
  	    h = hh[4];
  	    l = hl[4];
  	    a += l & 0xffff; b += l >>> 16;
  	    c += h & 0xffff; d += h >>> 16;
  	    b += a >>> 16;
  	    c += b >>> 16;
  	    d += c >>> 16;
  	    hh[4] = ah4 = (c & 0xffff) | (d << 16);
  	    hl[4] = al4 = (a & 0xffff) | (b << 16);
  	    h = ah5;
  	    l = al5;
  	    a = l & 0xffff; b = l >>> 16;
  	    c = h & 0xffff; d = h >>> 16;
  	    h = hh[5];
  	    l = hl[5];
  	    a += l & 0xffff; b += l >>> 16;
  	    c += h & 0xffff; d += h >>> 16;
  	    b += a >>> 16;
  	    c += b >>> 16;
  	    d += c >>> 16;
  	    hh[5] = ah5 = (c & 0xffff) | (d << 16);
  	    hl[5] = al5 = (a & 0xffff) | (b << 16);
  	    h = ah6;
  	    l = al6;
  	    a = l & 0xffff; b = l >>> 16;
  	    c = h & 0xffff; d = h >>> 16;
  	    h = hh[6];
  	    l = hl[6];
  	    a += l & 0xffff; b += l >>> 16;
  	    c += h & 0xffff; d += h >>> 16;
  	    b += a >>> 16;
  	    c += b >>> 16;
  	    d += c >>> 16;
  	    hh[6] = ah6 = (c & 0xffff) | (d << 16);
  	    hl[6] = al6 = (a & 0xffff) | (b << 16);
  	    h = ah7;
  	    l = al7;
  	    a = l & 0xffff; b = l >>> 16;
  	    c = h & 0xffff; d = h >>> 16;
  	    h = hh[7];
  	    l = hl[7];
  	    a += l & 0xffff; b += l >>> 16;
  	    c += h & 0xffff; d += h >>> 16;
  	    b += a >>> 16;
  	    c += b >>> 16;
  	    d += c >>> 16;
  	    hh[7] = ah7 = (c & 0xffff) | (d << 16);
  	    hl[7] = al7 = (a & 0xffff) | (b << 16);
  	    pos += 128;
  	    n -= 128;
  	  }
  	  return n;
  	}
  	function crypto_hash(out, m, n) {
  	  var hh = new Int32Array(8),
  	      hl = new Int32Array(8),
  	      x = new Uint8Array(256),
  	      i, b = n;
  	  hh[0] = 0x6a09e667;
  	  hh[1] = 0xbb67ae85;
  	  hh[2] = 0x3c6ef372;
  	  hh[3] = 0xa54ff53a;
  	  hh[4] = 0x510e527f;
  	  hh[5] = 0x9b05688c;
  	  hh[6] = 0x1f83d9ab;
  	  hh[7] = 0x5be0cd19;
  	  hl[0] = 0xf3bcc908;
  	  hl[1] = 0x84caa73b;
  	  hl[2] = 0xfe94f82b;
  	  hl[3] = 0x5f1d36f1;
  	  hl[4] = 0xade682d1;
  	  hl[5] = 0x2b3e6c1f;
  	  hl[6] = 0xfb41bd6b;
  	  hl[7] = 0x137e2179;
  	  crypto_hashblocks_hl(hh, hl, m, n);
  	  n %= 128;
  	  for (i = 0; i < n; i++) x[i] = m[b-n+i];
  	  x[n] = 128;
  	  n = 256-128*(n<112?1:0);
  	  x[n-9] = 0;
  	  ts64(x, n-8,  (b / 0x20000000) | 0, b << 3);
  	  crypto_hashblocks_hl(hh, hl, x, n);
  	  for (i = 0; i < 8; i++) ts64(out, 8*i, hh[i], hl[i]);
  	  return 0;
  	}
  	function add(p, q) {
  	  var a = gf(), b = gf(), c = gf(),
  	      d = gf(), e = gf(), f = gf(),
  	      g = gf(), h = gf(), t = gf();
  	  Z(a, p[1], p[0]);
  	  Z(t, q[1], q[0]);
  	  M(a, a, t);
  	  A(b, p[0], p[1]);
  	  A(t, q[0], q[1]);
  	  M(b, b, t);
  	  M(c, p[3], q[3]);
  	  M(c, c, D2);
  	  M(d, p[2], q[2]);
  	  A(d, d, d);
  	  Z(e, b, a);
  	  Z(f, d, c);
  	  A(g, d, c);
  	  A(h, b, a);
  	  M(p[0], e, f);
  	  M(p[1], h, g);
  	  M(p[2], g, f);
  	  M(p[3], e, h);
  	}
  	function cswap(p, q, b) {
  	  var i;
  	  for (i = 0; i < 4; i++) {
  	    sel25519(p[i], q[i], b);
  	  }
  	}
  	function pack(r, p) {
  	  var tx = gf(), ty = gf(), zi = gf();
  	  inv25519(zi, p[2]);
  	  M(tx, p[0], zi);
  	  M(ty, p[1], zi);
  	  pack25519(r, ty);
  	  r[31] ^= par25519(tx) << 7;
  	}
  	function scalarmult(p, q, s) {
  	  var b, i;
  	  set25519(p[0], gf0);
  	  set25519(p[1], gf1);
  	  set25519(p[2], gf1);
  	  set25519(p[3], gf0);
  	  for (i = 255; i >= 0; --i) {
  	    b = (s[(i/8)|0] >> (i&7)) & 1;
  	    cswap(p, q, b);
  	    add(q, p);
  	    add(p, p);
  	    cswap(p, q, b);
  	  }
  	}
  	function scalarbase(p, s) {
  	  var q = [gf(), gf(), gf(), gf()];
  	  set25519(q[0], X);
  	  set25519(q[1], Y);
  	  set25519(q[2], gf1);
  	  M(q[3], X, Y);
  	  scalarmult(p, q, s);
  	}
  	function crypto_sign_keypair(pk, sk, seeded) {
  	  var d = new Uint8Array(64);
  	  var p = [gf(), gf(), gf(), gf()];
  	  var i;
  	  if (!seeded) randombytes(sk, 32);
  	  crypto_hash(d, sk, 32);
  	  d[0] &= 248;
  	  d[31] &= 127;
  	  d[31] |= 64;
  	  scalarbase(p, d);
  	  pack(pk, p);
  	  for (i = 0; i < 32; i++) sk[i+32] = pk[i];
  	  return 0;
  	}
  	var L = new Float64Array([0xed, 0xd3, 0xf5, 0x5c, 0x1a, 0x63, 0x12, 0x58, 0xd6, 0x9c, 0xf7, 0xa2, 0xde, 0xf9, 0xde, 0x14, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x10]);
  	function modL(r, x) {
  	  var carry, i, j, k;
  	  for (i = 63; i >= 32; --i) {
  	    carry = 0;
  	    for (j = i - 32, k = i - 12; j < k; ++j) {
  	      x[j] += carry - 16 * x[i] * L[j - (i - 32)];
  	      carry = Math.floor((x[j] + 128) / 256);
  	      x[j] -= carry * 256;
  	    }
  	    x[j] += carry;
  	    x[i] = 0;
  	  }
  	  carry = 0;
  	  for (j = 0; j < 32; j++) {
  	    x[j] += carry - (x[31] >> 4) * L[j];
  	    carry = x[j] >> 8;
  	    x[j] &= 255;
  	  }
  	  for (j = 0; j < 32; j++) x[j] -= carry * L[j];
  	  for (i = 0; i < 32; i++) {
  	    x[i+1] += x[i] >> 8;
  	    r[i] = x[i] & 255;
  	  }
  	}
  	function reduce(r) {
  	  var x = new Float64Array(64), i;
  	  for (i = 0; i < 64; i++) x[i] = r[i];
  	  for (i = 0; i < 64; i++) r[i] = 0;
  	  modL(r, x);
  	}
  	function crypto_sign(sm, m, n, sk) {
  	  var d = new Uint8Array(64), h = new Uint8Array(64), r = new Uint8Array(64);
  	  var i, j, x = new Float64Array(64);
  	  var p = [gf(), gf(), gf(), gf()];
  	  crypto_hash(d, sk, 32);
  	  d[0] &= 248;
  	  d[31] &= 127;
  	  d[31] |= 64;
  	  var smlen = n + 64;
  	  for (i = 0; i < n; i++) sm[64 + i] = m[i];
  	  for (i = 0; i < 32; i++) sm[32 + i] = d[32 + i];
  	  crypto_hash(r, sm.subarray(32), n+32);
  	  reduce(r);
  	  scalarbase(p, r);
  	  pack(sm, p);
  	  for (i = 32; i < 64; i++) sm[i] = sk[i];
  	  crypto_hash(h, sm, n + 64);
  	  reduce(h);
  	  for (i = 0; i < 64; i++) x[i] = 0;
  	  for (i = 0; i < 32; i++) x[i] = r[i];
  	  for (i = 0; i < 32; i++) {
  	    for (j = 0; j < 32; j++) {
  	      x[i+j] += h[i] * d[j];
  	    }
  	  }
  	  modL(sm.subarray(32), x);
  	  return smlen;
  	}
  	function unpackneg(r, p) {
  	  var t = gf(), chk = gf(), num = gf(),
  	      den = gf(), den2 = gf(), den4 = gf(),
  	      den6 = gf();
  	  set25519(r[2], gf1);
  	  unpack25519(r[1], p);
  	  S(num, r[1]);
  	  M(den, num, D);
  	  Z(num, num, r[2]);
  	  A(den, r[2], den);
  	  S(den2, den);
  	  S(den4, den2);
  	  M(den6, den4, den2);
  	  M(t, den6, num);
  	  M(t, t, den);
  	  pow2523(t, t);
  	  M(t, t, num);
  	  M(t, t, den);
  	  M(t, t, den);
  	  M(r[0], t, den);
  	  S(chk, r[0]);
  	  M(chk, chk, den);
  	  if (neq25519(chk, num)) M(r[0], r[0], I);
  	  S(chk, r[0]);
  	  M(chk, chk, den);
  	  if (neq25519(chk, num)) return -1;
  	  if (par25519(r[0]) === (p[31]>>7)) Z(r[0], gf0, r[0]);
  	  M(r[3], r[0], r[1]);
  	  return 0;
  	}
  	function crypto_sign_open(m, sm, n, pk) {
  	  var i;
  	  var t = new Uint8Array(32), h = new Uint8Array(64);
  	  var p = [gf(), gf(), gf(), gf()],
  	      q = [gf(), gf(), gf(), gf()];
  	  if (n < 64) return -1;
  	  if (unpackneg(q, pk)) return -1;
  	  for (i = 0; i < n; i++) m[i] = sm[i];
  	  for (i = 0; i < 32; i++) m[i+32] = pk[i];
  	  crypto_hash(h, m, n);
  	  reduce(h);
  	  scalarmult(p, q, h);
  	  scalarbase(q, sm.subarray(32));
  	  add(p, q);
  	  pack(t, p);
  	  n -= 64;
  	  if (crypto_verify_32(sm, 0, t, 0)) {
  	    for (i = 0; i < n; i++) m[i] = 0;
  	    return -1;
  	  }
  	  for (i = 0; i < n; i++) m[i] = sm[i + 64];
  	  return n;
  	}
  	var crypto_secretbox_KEYBYTES = 32,
  	    crypto_secretbox_NONCEBYTES = 24,
  	    crypto_secretbox_ZEROBYTES = 32,
  	    crypto_secretbox_BOXZEROBYTES = 16,
  	    crypto_scalarmult_BYTES = 32,
  	    crypto_scalarmult_SCALARBYTES = 32,
  	    crypto_box_PUBLICKEYBYTES = 32,
  	    crypto_box_SECRETKEYBYTES = 32,
  	    crypto_box_BEFORENMBYTES = 32,
  	    crypto_box_NONCEBYTES = crypto_secretbox_NONCEBYTES,
  	    crypto_box_ZEROBYTES = crypto_secretbox_ZEROBYTES,
  	    crypto_box_BOXZEROBYTES = crypto_secretbox_BOXZEROBYTES,
  	    crypto_sign_BYTES = 64,
  	    crypto_sign_PUBLICKEYBYTES = 32,
  	    crypto_sign_SECRETKEYBYTES = 64,
  	    crypto_sign_SEEDBYTES = 32,
  	    crypto_hash_BYTES = 64;
  	nacl.lowlevel = {
  	  crypto_core_hsalsa20: crypto_core_hsalsa20,
  	  crypto_stream_xor: crypto_stream_xor,
  	  crypto_stream: crypto_stream,
  	  crypto_stream_salsa20_xor: crypto_stream_salsa20_xor,
  	  crypto_stream_salsa20: crypto_stream_salsa20,
  	  crypto_onetimeauth: crypto_onetimeauth,
  	  crypto_onetimeauth_verify: crypto_onetimeauth_verify,
  	  crypto_verify_16: crypto_verify_16,
  	  crypto_verify_32: crypto_verify_32,
  	  crypto_secretbox: crypto_secretbox,
  	  crypto_secretbox_open: crypto_secretbox_open,
  	  crypto_scalarmult: crypto_scalarmult,
  	  crypto_scalarmult_base: crypto_scalarmult_base,
  	  crypto_box_beforenm: crypto_box_beforenm,
  	  crypto_box_afternm: crypto_box_afternm,
  	  crypto_box: crypto_box,
  	  crypto_box_open: crypto_box_open,
  	  crypto_box_keypair: crypto_box_keypair,
  	  crypto_hash: crypto_hash,
  	  crypto_sign: crypto_sign,
  	  crypto_sign_keypair: crypto_sign_keypair,
  	  crypto_sign_open: crypto_sign_open,
  	  crypto_secretbox_KEYBYTES: crypto_secretbox_KEYBYTES,
  	  crypto_secretbox_NONCEBYTES: crypto_secretbox_NONCEBYTES,
  	  crypto_secretbox_ZEROBYTES: crypto_secretbox_ZEROBYTES,
  	  crypto_secretbox_BOXZEROBYTES: crypto_secretbox_BOXZEROBYTES,
  	  crypto_scalarmult_BYTES: crypto_scalarmult_BYTES,
  	  crypto_scalarmult_SCALARBYTES: crypto_scalarmult_SCALARBYTES,
  	  crypto_box_PUBLICKEYBYTES: crypto_box_PUBLICKEYBYTES,
  	  crypto_box_SECRETKEYBYTES: crypto_box_SECRETKEYBYTES,
  	  crypto_box_BEFORENMBYTES: crypto_box_BEFORENMBYTES,
  	  crypto_box_NONCEBYTES: crypto_box_NONCEBYTES,
  	  crypto_box_ZEROBYTES: crypto_box_ZEROBYTES,
  	  crypto_box_BOXZEROBYTES: crypto_box_BOXZEROBYTES,
  	  crypto_sign_BYTES: crypto_sign_BYTES,
  	  crypto_sign_PUBLICKEYBYTES: crypto_sign_PUBLICKEYBYTES,
  	  crypto_sign_SECRETKEYBYTES: crypto_sign_SECRETKEYBYTES,
  	  crypto_sign_SEEDBYTES: crypto_sign_SEEDBYTES,
  	  crypto_hash_BYTES: crypto_hash_BYTES,
  	  gf: gf,
  	  D: D,
  	  L: L,
  	  pack25519: pack25519,
  	  unpack25519: unpack25519,
  	  M: M,
  	  A: A,
  	  S: S,
  	  Z: Z,
  	  pow2523: pow2523,
  	  add: add,
  	  set25519: set25519,
  	  modL: modL,
  	  scalarmult: scalarmult,
  	  scalarbase: scalarbase,
  	};
  	function checkLengths(k, n) {
  	  if (k.length !== crypto_secretbox_KEYBYTES) throw new Error('bad key size');
  	  if (n.length !== crypto_secretbox_NONCEBYTES) throw new Error('bad nonce size');
  	}
  	function checkBoxLengths(pk, sk) {
  	  if (pk.length !== crypto_box_PUBLICKEYBYTES) throw new Error('bad public key size');
  	  if (sk.length !== crypto_box_SECRETKEYBYTES) throw new Error('bad secret key size');
  	}
  	function checkArrayTypes() {
  	  for (var i = 0; i < arguments.length; i++) {
  	    if (!(arguments[i] instanceof Uint8Array))
  	      throw new TypeError('unexpected type, use Uint8Array');
  	  }
  	}
  	function cleanup(arr) {
  	  for (var i = 0; i < arr.length; i++) arr[i] = 0;
  	}
  	nacl.randomBytes = function(n) {
  	  var b = new Uint8Array(n);
  	  randombytes(b, n);
  	  return b;
  	};
  	nacl.secretbox = function(msg, nonce, key) {
  	  checkArrayTypes(msg, nonce, key);
  	  checkLengths(key, nonce);
  	  var m = new Uint8Array(crypto_secretbox_ZEROBYTES + msg.length);
  	  var c = new Uint8Array(m.length);
  	  for (var i = 0; i < msg.length; i++) m[i+crypto_secretbox_ZEROBYTES] = msg[i];
  	  crypto_secretbox(c, m, m.length, nonce, key);
  	  return c.subarray(crypto_secretbox_BOXZEROBYTES);
  	};
  	nacl.secretbox.open = function(box, nonce, key) {
  	  checkArrayTypes(box, nonce, key);
  	  checkLengths(key, nonce);
  	  var c = new Uint8Array(crypto_secretbox_BOXZEROBYTES + box.length);
  	  var m = new Uint8Array(c.length);
  	  for (var i = 0; i < box.length; i++) c[i+crypto_secretbox_BOXZEROBYTES] = box[i];
  	  if (c.length < 32) return null;
  	  if (crypto_secretbox_open(m, c, c.length, nonce, key) !== 0) return null;
  	  return m.subarray(crypto_secretbox_ZEROBYTES);
  	};
  	nacl.secretbox.keyLength = crypto_secretbox_KEYBYTES;
  	nacl.secretbox.nonceLength = crypto_secretbox_NONCEBYTES;
  	nacl.secretbox.overheadLength = crypto_secretbox_BOXZEROBYTES;
  	nacl.scalarMult = function(n, p) {
  	  checkArrayTypes(n, p);
  	  if (n.length !== crypto_scalarmult_SCALARBYTES) throw new Error('bad n size');
  	  if (p.length !== crypto_scalarmult_BYTES) throw new Error('bad p size');
  	  var q = new Uint8Array(crypto_scalarmult_BYTES);
  	  crypto_scalarmult(q, n, p);
  	  return q;
  	};
  	nacl.scalarMult.base = function(n) {
  	  checkArrayTypes(n);
  	  if (n.length !== crypto_scalarmult_SCALARBYTES) throw new Error('bad n size');
  	  var q = new Uint8Array(crypto_scalarmult_BYTES);
  	  crypto_scalarmult_base(q, n);
  	  return q;
  	};
  	nacl.scalarMult.scalarLength = crypto_scalarmult_SCALARBYTES;
  	nacl.scalarMult.groupElementLength = crypto_scalarmult_BYTES;
  	nacl.box = function(msg, nonce, publicKey, secretKey) {
  	  var k = nacl.box.before(publicKey, secretKey);
  	  return nacl.secretbox(msg, nonce, k);
  	};
  	nacl.box.before = function(publicKey, secretKey) {
  	  checkArrayTypes(publicKey, secretKey);
  	  checkBoxLengths(publicKey, secretKey);
  	  var k = new Uint8Array(crypto_box_BEFORENMBYTES);
  	  crypto_box_beforenm(k, publicKey, secretKey);
  	  return k;
  	};
  	nacl.box.after = nacl.secretbox;
  	nacl.box.open = function(msg, nonce, publicKey, secretKey) {
  	  var k = nacl.box.before(publicKey, secretKey);
  	  return nacl.secretbox.open(msg, nonce, k);
  	};
  	nacl.box.open.after = nacl.secretbox.open;
  	nacl.box.keyPair = function() {
  	  var pk = new Uint8Array(crypto_box_PUBLICKEYBYTES);
  	  var sk = new Uint8Array(crypto_box_SECRETKEYBYTES);
  	  crypto_box_keypair(pk, sk);
  	  return {publicKey: pk, secretKey: sk};
  	};
  	nacl.box.keyPair.fromSecretKey = function(secretKey) {
  	  checkArrayTypes(secretKey);
  	  if (secretKey.length !== crypto_box_SECRETKEYBYTES)
  	    throw new Error('bad secret key size');
  	  var pk = new Uint8Array(crypto_box_PUBLICKEYBYTES);
  	  crypto_scalarmult_base(pk, secretKey);
  	  return {publicKey: pk, secretKey: new Uint8Array(secretKey)};
  	};
  	nacl.box.publicKeyLength = crypto_box_PUBLICKEYBYTES;
  	nacl.box.secretKeyLength = crypto_box_SECRETKEYBYTES;
  	nacl.box.sharedKeyLength = crypto_box_BEFORENMBYTES;
  	nacl.box.nonceLength = crypto_box_NONCEBYTES;
  	nacl.box.overheadLength = nacl.secretbox.overheadLength;
  	nacl.sign = function(msg, secretKey) {
  	  checkArrayTypes(msg, secretKey);
  	  if (secretKey.length !== crypto_sign_SECRETKEYBYTES)
  	    throw new Error('bad secret key size');
  	  var signedMsg = new Uint8Array(crypto_sign_BYTES+msg.length);
  	  crypto_sign(signedMsg, msg, msg.length, secretKey);
  	  return signedMsg;
  	};
  	nacl.sign.open = function(signedMsg, publicKey) {
  	  checkArrayTypes(signedMsg, publicKey);
  	  if (publicKey.length !== crypto_sign_PUBLICKEYBYTES)
  	    throw new Error('bad public key size');
  	  var tmp = new Uint8Array(signedMsg.length);
  	  var mlen = crypto_sign_open(tmp, signedMsg, signedMsg.length, publicKey);
  	  if (mlen < 0) return null;
  	  var m = new Uint8Array(mlen);
  	  for (var i = 0; i < m.length; i++) m[i] = tmp[i];
  	  return m;
  	};
  	nacl.sign.detached = function(msg, secretKey) {
  	  var signedMsg = nacl.sign(msg, secretKey);
  	  var sig = new Uint8Array(crypto_sign_BYTES);
  	  for (var i = 0; i < sig.length; i++) sig[i] = signedMsg[i];
  	  return sig;
  	};
  	nacl.sign.detached.verify = function(msg, sig, publicKey) {
  	  checkArrayTypes(msg, sig, publicKey);
  	  if (sig.length !== crypto_sign_BYTES)
  	    throw new Error('bad signature size');
  	  if (publicKey.length !== crypto_sign_PUBLICKEYBYTES)
  	    throw new Error('bad public key size');
  	  var sm = new Uint8Array(crypto_sign_BYTES + msg.length);
  	  var m = new Uint8Array(crypto_sign_BYTES + msg.length);
  	  var i;
  	  for (i = 0; i < crypto_sign_BYTES; i++) sm[i] = sig[i];
  	  for (i = 0; i < msg.length; i++) sm[i+crypto_sign_BYTES] = msg[i];
  	  return (crypto_sign_open(m, sm, sm.length, publicKey) >= 0);
  	};
  	nacl.sign.keyPair = function() {
  	  var pk = new Uint8Array(crypto_sign_PUBLICKEYBYTES);
  	  var sk = new Uint8Array(crypto_sign_SECRETKEYBYTES);
  	  crypto_sign_keypair(pk, sk);
  	  return {publicKey: pk, secretKey: sk};
  	};
  	nacl.sign.keyPair.fromSecretKey = function(secretKey) {
  	  checkArrayTypes(secretKey);
  	  if (secretKey.length !== crypto_sign_SECRETKEYBYTES)
  	    throw new Error('bad secret key size');
  	  var pk = new Uint8Array(crypto_sign_PUBLICKEYBYTES);
  	  for (var i = 0; i < pk.length; i++) pk[i] = secretKey[32+i];
  	  return {publicKey: pk, secretKey: new Uint8Array(secretKey)};
  	};
  	nacl.sign.keyPair.fromSeed = function(seed) {
  	  checkArrayTypes(seed);
  	  if (seed.length !== crypto_sign_SEEDBYTES)
  	    throw new Error('bad seed size');
  	  var pk = new Uint8Array(crypto_sign_PUBLICKEYBYTES);
  	  var sk = new Uint8Array(crypto_sign_SECRETKEYBYTES);
  	  for (var i = 0; i < 32; i++) sk[i] = seed[i];
  	  crypto_sign_keypair(pk, sk, true);
  	  return {publicKey: pk, secretKey: sk};
  	};
  	nacl.sign.publicKeyLength = crypto_sign_PUBLICKEYBYTES;
  	nacl.sign.secretKeyLength = crypto_sign_SECRETKEYBYTES;
  	nacl.sign.seedLength = crypto_sign_SEEDBYTES;
  	nacl.sign.signatureLength = crypto_sign_BYTES;
  	nacl.hash = function(msg) {
  	  checkArrayTypes(msg);
  	  var h = new Uint8Array(crypto_hash_BYTES);
  	  crypto_hash(h, msg, msg.length);
  	  return h;
  	};
  	nacl.hash.hashLength = crypto_hash_BYTES;
  	nacl.verify = function(x, y) {
  	  checkArrayTypes(x, y);
  	  if (x.length === 0 || y.length === 0) return false;
  	  if (x.length !== y.length) return false;
  	  return (vn(x, 0, y, 0, x.length) === 0) ? true : false;
  	};
  	nacl.setPRNG = function(fn) {
  	  randombytes = fn;
  	};
  	(function() {
  	  var crypto = typeof self !== 'undefined' ? (self.crypto || self.msCrypto) : null;
  	  if (crypto && crypto.getRandomValues) {
  	    var QUOTA = 65536;
  	    nacl.setPRNG(function(x, n) {
  	      var i, v = new Uint8Array(n);
  	      for (i = 0; i < n; i += QUOTA) {
  	        crypto.getRandomValues(v.subarray(i, i + Math.min(n - i, QUOTA)));
  	      }
  	      for (i = 0; i < n; i++) x[i] = v[i];
  	      cleanup(v);
  	    });
  	  } else if (typeof commonjsRequire !== 'undefined') {
  	    crypto = require$$0;
  	    if (crypto && crypto.randomBytes) {
  	      nacl.setPRNG(function(x, n) {
  	        var i, v = crypto.randomBytes(n);
  	        for (i = 0; i < n; i++) x[i] = v[i];
  	        cleanup(v);
  	      });
  	    }
  	  }
  	})();
  	})(module.exports ? module.exports : (self.nacl = self.nacl || {}));
  } (naclFast));
  const nacl = naclFast.exports;

  (function (module) {
  	(function(root, f) {
  	  if (module.exports) module.exports = f(naclFast.exports);
  	  else root.ed2curve = f(root.nacl);
  	}(commonjsGlobal, function(nacl) {
  	  if (!nacl) throw new Error('tweetnacl not loaded');
  	  var gf = function(init) {
  	    var i, r = new Float64Array(16);
  	    if (init) for (i = 0; i < init.length; i++) r[i] = init[i];
  	    return r;
  	  };
  	  var gf0 = gf(),
  	      gf1 = gf([1]),
  	      D = gf([0x78a3, 0x1359, 0x4dca, 0x75eb, 0xd8ab, 0x4141, 0x0a4d, 0x0070, 0xe898, 0x7779, 0x4079, 0x8cc7, 0xfe73, 0x2b6f, 0x6cee, 0x5203]),
  	      I = gf([0xa0b0, 0x4a0e, 0x1b27, 0xc4ee, 0xe478, 0xad2f, 0x1806, 0x2f43, 0xd7a7, 0x3dfb, 0x0099, 0x2b4d, 0xdf0b, 0x4fc1, 0x2480, 0x2b83]);
  	  function car25519(o) {
  	    var c;
  	    var i;
  	    for (i = 0; i < 16; i++) {
  	      o[i] += 65536;
  	      c = Math.floor(o[i] / 65536);
  	      o[(i+1)*(i<15?1:0)] += c - 1 + 37 * (c-1) * (i===15?1:0);
  	      o[i] -= (c * 65536);
  	    }
  	  }
  	  function sel25519(p, q, b) {
  	    var t, c = ~(b-1);
  	    for (var i = 0; i < 16; i++) {
  	      t = c & (p[i] ^ q[i]);
  	      p[i] ^= t;
  	      q[i] ^= t;
  	    }
  	  }
  	  function unpack25519(o, n) {
  	    var i;
  	    for (i = 0; i < 16; i++) o[i] = n[2*i] + (n[2*i+1] << 8);
  	    o[15] &= 0x7fff;
  	  }
  	  function A(o, a, b) {
  	    var i;
  	    for (i = 0; i < 16; i++) o[i] = (a[i] + b[i])|0;
  	  }
  	  function Z(o, a, b) {
  	    var i;
  	    for (i = 0; i < 16; i++) o[i] = (a[i] - b[i])|0;
  	  }
  	  function M(o, a, b) {
  	    var i, j, t = new Float64Array(31);
  	    for (i = 0; i < 31; i++) t[i] = 0;
  	    for (i = 0; i < 16; i++) {
  	      for (j = 0; j < 16; j++) {
  	        t[i+j] += a[i] * b[j];
  	      }
  	    }
  	    for (i = 0; i < 15; i++) {
  	      t[i] += 38 * t[i+16];
  	    }
  	    for (i = 0; i < 16; i++) o[i] = t[i];
  	    car25519(o);
  	    car25519(o);
  	  }
  	  function S(o, a) {
  	    M(o, a, a);
  	  }
  	  function inv25519(o, i) {
  	    var c = gf();
  	    var a;
  	    for (a = 0; a < 16; a++) c[a] = i[a];
  	    for (a = 253; a >= 0; a--) {
  	      S(c, c);
  	      if(a !== 2 && a !== 4) M(c, c, i);
  	    }
  	    for (a = 0; a < 16; a++) o[a] = c[a];
  	  }
  	  function pack25519(o, n) {
  	    var i, j, b;
  	    var m = gf(), t = gf();
  	    for (i = 0; i < 16; i++) t[i] = n[i];
  	    car25519(t);
  	    car25519(t);
  	    car25519(t);
  	    for (j = 0; j < 2; j++) {
  	      m[0] = t[0] - 0xffed;
  	      for (i = 1; i < 15; i++) {
  	        m[i] = t[i] - 0xffff - ((m[i-1]>>16) & 1);
  	        m[i-1] &= 0xffff;
  	      }
  	      m[15] = t[15] - 0x7fff - ((m[14]>>16) & 1);
  	      b = (m[15]>>16) & 1;
  	      m[14] &= 0xffff;
  	      sel25519(t, m, 1-b);
  	    }
  	    for (i = 0; i < 16; i++) {
  	      o[2*i] = t[i] & 0xff;
  	      o[2*i+1] = t[i] >> 8;
  	    }
  	  }
  	  function par25519(a) {
  	    var d = new Uint8Array(32);
  	    pack25519(d, a);
  	    return d[0] & 1;
  	  }
  	  function vn(x, xi, y, yi, n) {
  	    var i, d = 0;
  	    for (i = 0; i < n; i++) d |= x[xi + i] ^ y[yi + i];
  	    return (1 & ((d - 1) >>> 8)) - 1;
  	  }
  	  function crypto_verify_32(x, xi, y, yi) {
  	    return vn(x, xi, y, yi, 32);
  	  }
  	  function neq25519(a, b) {
  	    var c = new Uint8Array(32), d = new Uint8Array(32);
  	    pack25519(c, a);
  	    pack25519(d, b);
  	    return crypto_verify_32(c, 0, d, 0);
  	  }
  	  function pow2523(o, i) {
  	    var c = gf();
  	    var a;
  	    for (a = 0; a < 16; a++) c[a] = i[a];
  	    for (a = 250; a >= 0; a--) {
  	      S(c, c);
  	      if (a !== 1) M(c, c, i);
  	    }
  	    for (a = 0; a < 16; a++) o[a] = c[a];
  	  }
  	  function set25519(r, a) {
  	    var i;
  	    for (i = 0; i < 16; i++) r[i] = a[i] | 0;
  	  }
  	  function unpackneg(r, p) {
  	    var t = gf(), chk = gf(), num = gf(),
  	      den = gf(), den2 = gf(), den4 = gf(),
  	      den6 = gf();
  	    set25519(r[2], gf1);
  	    unpack25519(r[1], p);
  	    S(num, r[1]);
  	    M(den, num, D);
  	    Z(num, num, r[2]);
  	    A(den, r[2], den);
  	    S(den2, den);
  	    S(den4, den2);
  	    M(den6, den4, den2);
  	    M(t, den6, num);
  	    M(t, t, den);
  	    pow2523(t, t);
  	    M(t, t, num);
  	    M(t, t, den);
  	    M(t, t, den);
  	    M(r[0], t, den);
  	    S(chk, r[0]);
  	    M(chk, chk, den);
  	    if (neq25519(chk, num)) M(r[0], r[0], I);
  	    S(chk, r[0]);
  	    M(chk, chk, den);
  	    if (neq25519(chk, num)) return -1;
  	    if (par25519(r[0]) === (p[31] >> 7)) Z(r[0], gf0, r[0]);
  	    M(r[3], r[0], r[1]);
  	    return 0;
  	  }
  	  function convertPublicKey(pk) {
  	    var z = new Uint8Array(32),
  	      q = [gf(), gf(), gf(), gf()],
  	      a = gf(), b = gf();
  	    if (unpackneg(q, pk)) return null;
  	    var y = q[1];
  	    A(a, gf1, y);
  	    Z(b, gf1, y);
  	    inv25519(b, b);
  	    M(a, a, b);
  	    pack25519(z, a);
  	    return z;
  	  }
  	  function convertSecretKey(sk) {
  	    var d = new Uint8Array(64), o = new Uint8Array(32), i;
  	    nacl.lowlevel.crypto_hash(d, sk, 32);
  	    d[0] &= 248;
  	    d[31] &= 127;
  	    d[31] |= 64;
  	    for (i = 0; i < 32; i++) o[i] = d[i];
  	    for (i = 0; i < 64; i++) d[i] = 0;
  	    return o;
  	  }
  	  function convertKeyPair(edKeyPair) {
  	    var publicKey = convertPublicKey(edKeyPair.publicKey);
  	    if (!publicKey) return null;
  	    return {
  	      publicKey: publicKey,
  	      secretKey: convertSecretKey(edKeyPair.secretKey)
  	    };
  	  }
  	  return {
  	    convertPublicKey: convertPublicKey,
  	    convertSecretKey: convertSecretKey,
  	    convertKeyPair: convertKeyPair,
  	  };
  	}));
  } (ed2curve$1));
  const ed2curve = ed2curve$1.exports;

  function convertSecretKeyToCurve25519(secretKey) {
    return ed2curve.convertSecretKey(secretKey);
  }
  function convertPublicKeyToCurve25519(publicKey) {
    return util.assertReturn(ed2curve.convertPublicKey(publicKey), 'Unable to convert publicKey to ed25519');
  }

  const HDKD = util.compactAddLength(util.stringToU8a('Ed25519HDKD'));
  function ed25519DeriveHard(seed, chainCode) {
    if (!util.isU8a(chainCode) || chainCode.length !== 32) {
      throw new Error('Invalid chainCode passed to derive');
    }
    return blake2AsU8a(util.u8aConcat(HDKD, seed, chainCode));
  }

  function randomAsU8a(length = 32) {
    return browser.getRandomValues(new Uint8Array(length));
  }
  const randomAsHex = createAsHex(randomAsU8a);

  const BN_53 = new util.BN(0b11111111111111111111111111111111111111111111111111111);
  function randomAsNumber() {
    return util.hexToBn(randomAsHex(8)).and(BN_53).toNumber();
  }

  function ed25519PairFromSeed(seed, onlyJs) {
    if (!onlyJs && isReady()) {
      const full = ed25519KeypairFromSeed(seed);
      return {
        publicKey: full.slice(32),
        secretKey: full.slice(0, 64)
      };
    }
    return nacl.sign.keyPair.fromSeed(seed);
  }

  function ed25519PairFromRandom() {
    return ed25519PairFromSeed(randomAsU8a());
  }

  function ed25519PairFromSecret(secret) {
    return nacl.sign.keyPair.fromSecretKey(secret);
  }

  function ed25519PairFromString(value) {
    return ed25519PairFromSeed(blake2AsU8a(util.stringToU8a(value)));
  }

  function ed25519Sign(message, {
    publicKey,
    secretKey
  }, onlyJs) {
    if (!secretKey) {
      throw new Error('Expected a valid secretKey');
    }
    const messageU8a = util.u8aToU8a(message);
    return !onlyJs && isReady() ? ed25519Sign$1(publicKey, secretKey.subarray(0, 32), messageU8a) : nacl.sign.detached(messageU8a, secretKey);
  }

  function ed25519Verify(message, signature, publicKey, onlyJs) {
    const messageU8a = util.u8aToU8a(message);
    const publicKeyU8a = util.u8aToU8a(publicKey);
    const signatureU8a = util.u8aToU8a(signature);
    if (publicKeyU8a.length !== 32) {
      throw new Error(`Invalid publicKey, received ${publicKeyU8a.length}, expected 32`);
    } else if (signatureU8a.length !== 64) {
      throw new Error(`Invalid signature, received ${signatureU8a.length} bytes, expected 64`);
    }
    return !onlyJs && isReady() ? ed25519Verify$1(signatureU8a, messageU8a, publicKeyU8a) : nacl.sign.detached.verify(messageU8a, signatureU8a, publicKeyU8a);
  }

  const keyHdkdEd25519 = createSeedDeriveFn(ed25519PairFromSeed, ed25519DeriveHard);

  const SEC_LEN = 64;
  const PUB_LEN = 32;
  const TOT_LEN = SEC_LEN + PUB_LEN;
  function sr25519PairFromU8a(full) {
    const fullU8a = util.u8aToU8a(full);
    if (fullU8a.length !== TOT_LEN) {
      throw new Error(`Expected keypair with ${TOT_LEN} bytes, found ${fullU8a.length}`);
    }
    return {
      publicKey: fullU8a.slice(SEC_LEN, TOT_LEN),
      secretKey: fullU8a.slice(0, SEC_LEN)
    };
  }

  function sr25519KeypairToU8a({
    publicKey,
    secretKey
  }) {
    return util.u8aConcat(secretKey, publicKey).slice();
  }

  function createDeriveFn(derive) {
    return (keypair, chainCode) => {
      if (!util.isU8a(chainCode) || chainCode.length !== 32) {
        throw new Error('Invalid chainCode passed to derive');
      }
      return sr25519PairFromU8a(derive(sr25519KeypairToU8a(keypair), chainCode));
    };
  }

  const sr25519DeriveHard = createDeriveFn(sr25519DeriveKeypairHard);

  const sr25519DeriveSoft = createDeriveFn(sr25519DeriveKeypairSoft);

  function keyHdkdSr25519(keypair, {
    chainCode,
    isSoft
  }) {
    return isSoft ? sr25519DeriveSoft(keypair, chainCode) : sr25519DeriveHard(keypair, chainCode);
  }

  const generators = {
    ecdsa: keyHdkdEcdsa,
    ed25519: keyHdkdEd25519,
    ethereum: keyHdkdEcdsa,
    sr25519: keyHdkdSr25519
  };
  function keyFromPath(pair, path, type) {
    const keyHdkd = generators[type];
    let result = pair;
    for (const junction of path) {
      result = keyHdkd(result, junction);
    }
    return result;
  }

  function sr25519Agreement(secretKey, publicKey) {
    const secretKeyU8a = util.u8aToU8a(secretKey);
    const publicKeyU8a = util.u8aToU8a(publicKey);
    if (publicKeyU8a.length !== 32) {
      throw new Error(`Invalid publicKey, received ${publicKeyU8a.length} bytes, expected 32`);
    } else if (secretKeyU8a.length !== 64) {
      throw new Error(`Invalid secretKey, received ${secretKeyU8a.length} bytes, expected 64`);
    }
    return sr25519Agree(publicKeyU8a, secretKeyU8a);
  }

  function sr25519DerivePublic(publicKey, chainCode) {
    const publicKeyU8a = util.u8aToU8a(publicKey);
    if (!util.isU8a(chainCode) || chainCode.length !== 32) {
      throw new Error('Invalid chainCode passed to derive');
    } else if (publicKeyU8a.length !== 32) {
      throw new Error(`Invalid publicKey, received ${publicKeyU8a.length} bytes, expected 32`);
    }
    return sr25519DerivePublicSoft(publicKeyU8a, chainCode);
  }

  function sr25519PairFromSeed(seed) {
    const seedU8a = util.u8aToU8a(seed);
    if (seedU8a.length !== 32) {
      throw new Error(`Expected a seed matching 32 bytes, found ${seedU8a.length}`);
    }
    return sr25519PairFromU8a(sr25519KeypairFromSeed(seedU8a));
  }

  function sr25519Sign(message, {
    publicKey,
    secretKey
  }) {
    if ((publicKey == null ? void 0 : publicKey.length) !== 32) {
      throw new Error('Expected a valid publicKey, 32-bytes');
    } else if ((secretKey == null ? void 0 : secretKey.length) !== 64) {
      throw new Error('Expected a valid secretKey, 64-bytes');
    }
    return sr25519Sign$1(publicKey, secretKey, util.u8aToU8a(message));
  }

  function sr25519Verify(message, signature, publicKey) {
    const publicKeyU8a = util.u8aToU8a(publicKey);
    const signatureU8a = util.u8aToU8a(signature);
    if (publicKeyU8a.length !== 32) {
      throw new Error(`Invalid publicKey, received ${publicKeyU8a.length} bytes, expected 32`);
    } else if (signatureU8a.length !== 64) {
      throw new Error(`Invalid signature, received ${signatureU8a.length} bytes, expected 64`);
    }
    return sr25519Verify$1(signatureU8a, util.u8aToU8a(message), publicKeyU8a);
  }

  const EMPTY_U8A$1 = new Uint8Array();
  function sr25519VrfSign(message, {
    secretKey
  }, context = EMPTY_U8A$1, extra = EMPTY_U8A$1) {
    if ((secretKey == null ? void 0 : secretKey.length) !== 64) {
      throw new Error('Invalid secretKey, expected 64-bytes');
    }
    return vrfSign(secretKey, util.u8aToU8a(context), util.u8aToU8a(message), util.u8aToU8a(extra));
  }

  const EMPTY_U8A = new Uint8Array();
  function sr25519VrfVerify(message, signOutput, publicKey, context = EMPTY_U8A, extra = EMPTY_U8A) {
    const publicKeyU8a = util.u8aToU8a(publicKey);
    const proofU8a = util.u8aToU8a(signOutput);
    if (publicKeyU8a.length !== 32) {
      throw new Error('Invalid publicKey, expected 32-bytes');
    } else if (proofU8a.length !== 96) {
      throw new Error('Invalid vrfSign output, expected 96 bytes');
    }
    return vrfVerify(publicKeyU8a, util.u8aToU8a(context), util.u8aToU8a(message), util.u8aToU8a(extra), proofU8a);
  }

  function encodeAddress(key, ss58Format = defaults.prefix) {
    const u8a = decodeAddress(key);
    if (ss58Format < 0 || ss58Format > 16383 || [46, 47].includes(ss58Format)) {
      throw new Error('Out of range ss58Format specified');
    } else if (!defaults.allowedDecodedLengths.includes(u8a.length)) {
      throw new Error(`Expected a valid key to convert, with length ${defaults.allowedDecodedLengths.join(', ')}`);
    }
    const input = util.u8aConcat(ss58Format < 64 ? [ss58Format] : [(ss58Format & 0b0000000011111100) >> 2 | 0b01000000, ss58Format >> 8 | (ss58Format & 0b0000000000000011) << 6], u8a);
    return base58Encode(util.u8aConcat(input, sshash(input).subarray(0, [32, 33].includes(u8a.length) ? 2 : 1)));
  }

  function filterHard({
    isHard
  }) {
    return isHard;
  }
  function deriveAddress(who, suri, ss58Format) {
    const {
      path
    } = keyExtractPath(suri);
    if (!path.length || path.every(filterHard)) {
      throw new Error('Expected suri to contain a combination of non-hard paths');
    }
    let publicKey = decodeAddress(who);
    for (const {
      chainCode
    } of path) {
      publicKey = sr25519DerivePublic(publicKey, chainCode);
    }
    return encodeAddress(publicKey, ss58Format);
  }

  function encodeDerivedAddress(who, index, ss58Format) {
    return encodeAddress(createKeyDerived(decodeAddress(who), index), ss58Format);
  }

  function encodeMultiAddress(who, threshold, ss58Format) {
    return encodeAddress(createKeyMulti(who, threshold), ss58Format);
  }

  const [SHA3_PI, SHA3_ROTL, _SHA3_IOTA] = [[], [], []];
  const _0n = BigInt(0);
  const _1n = BigInt(1);
  const _2n = BigInt(2);
  const _7n$1 = BigInt(7);
  const _256n$1 = BigInt(256);
  const _0x71n = BigInt(0x71);
  for (let round = 0, R = _1n, x = 1, y = 0; round < 24; round++) {
      [x, y] = [y, (2 * x + 3 * y) % 5];
      SHA3_PI.push(2 * (5 * y + x));
      SHA3_ROTL.push((((round + 1) * (round + 2)) / 2) % 64);
      let t = _0n;
      for (let j = 0; j < 7; j++) {
          R = ((R << _1n) ^ ((R >> _7n$1) * _0x71n)) % _256n$1;
          if (R & _2n)
              t ^= _1n << ((_1n << BigInt(j)) - _1n);
      }
      _SHA3_IOTA.push(t);
  }
  const [SHA3_IOTA_H, SHA3_IOTA_L] = u64.split(_SHA3_IOTA, true);
  const rotlH = (h, l, s) => s > 32 ? u64.rotlBH(h, l, s) : u64.rotlSH(h, l, s);
  const rotlL = (h, l, s) => s > 32 ? u64.rotlBL(h, l, s) : u64.rotlSL(h, l, s);
  function keccakP(s, rounds = 24) {
      const B = new Uint32Array(5 * 2);
      for (let round = 24 - rounds; round < 24; round++) {
          for (let x = 0; x < 10; x++)
              B[x] = s[x] ^ s[x + 10] ^ s[x + 20] ^ s[x + 30] ^ s[x + 40];
          for (let x = 0; x < 10; x += 2) {
              const idx1 = (x + 8) % 10;
              const idx0 = (x + 2) % 10;
              const B0 = B[idx0];
              const B1 = B[idx0 + 1];
              const Th = rotlH(B0, B1, 1) ^ B[idx1];
              const Tl = rotlL(B0, B1, 1) ^ B[idx1 + 1];
              for (let y = 0; y < 50; y += 10) {
                  s[x + y] ^= Th;
                  s[x + y + 1] ^= Tl;
              }
          }
          let curH = s[2];
          let curL = s[3];
          for (let t = 0; t < 24; t++) {
              const shift = SHA3_ROTL[t];
              const Th = rotlH(curH, curL, shift);
              const Tl = rotlL(curH, curL, shift);
              const PI = SHA3_PI[t];
              curH = s[PI];
              curL = s[PI + 1];
              s[PI] = Th;
              s[PI + 1] = Tl;
          }
          for (let y = 0; y < 50; y += 10) {
              for (let x = 0; x < 10; x++)
                  B[x] = s[y + x];
              for (let x = 0; x < 10; x++)
                  s[y + x] ^= ~B[(x + 2) % 10] & B[(x + 4) % 10];
          }
          s[0] ^= SHA3_IOTA_H[round];
          s[1] ^= SHA3_IOTA_L[round];
      }
      B.fill(0);
  }
  class Keccak extends Hash {
      constructor(blockLen, suffix, outputLen, enableXOF = false, rounds = 24) {
          super();
          this.blockLen = blockLen;
          this.suffix = suffix;
          this.outputLen = outputLen;
          this.enableXOF = enableXOF;
          this.rounds = rounds;
          this.pos = 0;
          this.posOut = 0;
          this.finished = false;
          this.destroyed = false;
          assert.number(outputLen);
          if (0 >= this.blockLen || this.blockLen >= 200)
              throw new Error('Sha3 supports only keccak-f1600 function');
          this.state = new Uint8Array(200);
          this.state32 = u32(this.state);
      }
      keccak() {
          keccakP(this.state32, this.rounds);
          this.posOut = 0;
          this.pos = 0;
      }
      update(data) {
          assert.exists(this);
          const { blockLen, state } = this;
          data = toBytes(data);
          const len = data.length;
          for (let pos = 0; pos < len;) {
              const take = Math.min(blockLen - this.pos, len - pos);
              for (let i = 0; i < take; i++)
                  state[this.pos++] ^= data[pos++];
              if (this.pos === blockLen)
                  this.keccak();
          }
          return this;
      }
      finish() {
          if (this.finished)
              return;
          this.finished = true;
          const { state, suffix, pos, blockLen } = this;
          state[pos] ^= suffix;
          if ((suffix & 0x80) !== 0 && pos === blockLen - 1)
              this.keccak();
          state[blockLen - 1] ^= 0x80;
          this.keccak();
      }
      writeInto(out) {
          assert.exists(this, false);
          assert.bytes(out);
          this.finish();
          const bufferOut = this.state;
          const { blockLen } = this;
          for (let pos = 0, len = out.length; pos < len;) {
              if (this.posOut >= blockLen)
                  this.keccak();
              const take = Math.min(blockLen - this.posOut, len - pos);
              out.set(bufferOut.subarray(this.posOut, this.posOut + take), pos);
              this.posOut += take;
              pos += take;
          }
          return out;
      }
      xofInto(out) {
          if (!this.enableXOF)
              throw new Error('XOF is not possible for this instance');
          return this.writeInto(out);
      }
      xof(bytes) {
          assert.number(bytes);
          return this.xofInto(new Uint8Array(bytes));
      }
      digestInto(out) {
          assert.output(out, this);
          if (this.finished)
              throw new Error('digest() was already called');
          this.writeInto(out);
          this.destroy();
          return out;
      }
      digest() {
          return this.digestInto(new Uint8Array(this.outputLen));
      }
      destroy() {
          this.destroyed = true;
          this.state.fill(0);
      }
      _cloneInto(to) {
          const { blockLen, suffix, outputLen, rounds, enableXOF } = this;
          to || (to = new Keccak(blockLen, suffix, outputLen, enableXOF, rounds));
          to.state32.set(this.state32);
          to.pos = this.pos;
          to.posOut = this.posOut;
          to.finished = this.finished;
          to.rounds = rounds;
          to.suffix = suffix;
          to.outputLen = outputLen;
          to.enableXOF = enableXOF;
          to.destroyed = this.destroyed;
          return to;
      }
  }
  const gen = (suffix, blockLen, outputLen) => wrapConstructor(() => new Keccak(blockLen, suffix, outputLen));
  gen(0x06, 144, 224 / 8);
  gen(0x06, 136, 256 / 8);
  gen(0x06, 104, 384 / 8);
  gen(0x06, 72, 512 / 8);
  gen(0x01, 144, 224 / 8);
  const keccak_256 = gen(0x01, 136, 256 / 8);
  gen(0x01, 104, 384 / 8);
  const keccak_512 = gen(0x01, 72, 512 / 8);
  const genShake = (suffix, blockLen, outputLen) => wrapConstructorWithOpts((opts = {}) => new Keccak(blockLen, suffix, opts.dkLen === undefined ? outputLen : opts.dkLen, true));
  genShake(0x1f, 168, 128 / 8);
  genShake(0x1f, 136, 256 / 8);

  const keccakAsU8a = createDualHasher({
    256: keccak256,
    512: keccak512
  }, {
    256: keccak_256,
    512: keccak_512
  });
  const keccak256AsU8a = createBitHasher(256, keccakAsU8a);
  const keccak512AsU8a = createBitHasher(512, keccakAsU8a);
  const keccakAsHex = createAsHex(keccakAsU8a);

  function hasher(hashType, data, onlyJs) {
    return hashType === 'keccak' ? keccakAsU8a(data, undefined, onlyJs) : blake2AsU8a(data, undefined, undefined, onlyJs);
  }

  function evmToAddress(evmAddress, ss58Format, hashType = 'blake2') {
    const message = util.u8aConcat('evm:', evmAddress);
    if (message.length !== 24) {
      throw new Error(`Converting ${evmAddress}: Invalid evm address length`);
    }
    return encodeAddress(hasher(hashType, message), ss58Format);
  }

  function addressEq(a, b) {
    return util.u8aEq(decodeAddress(a), decodeAddress(b));
  }

  function validateAddress(encoded, ignoreChecksum, ss58Format) {
    return !!decodeAddress(encoded, ignoreChecksum, ss58Format);
  }

  function isAddress(address, ignoreChecksum, ss58Format) {
    try {
      return validateAddress(address, ignoreChecksum, ss58Format);
    } catch (error) {
      return false;
    }
  }

  const l = util.logger('setSS58Format');
  function setSS58Format(prefix) {
    l.warn('Global setting of the ss58Format is deprecated and not recommended. Set format on the keyring (if used) or as part of the address encode function');
    defaults.prefix = prefix;
  }

  function sortAddresses(addresses, ss58Format) {
    const u8aToAddress = u8a => encodeAddress(u8a, ss58Format);
    return util.u8aSorted(addresses.map(addressToU8a)).map(u8aToAddress);
  }

  const chars = 'abcdefghijklmnopqrstuvwxyz234567';
  const config$1 = {
    chars,
    coder: utils.chain(
    utils.radix2(5), utils.alphabet(chars), {
      decode: input => input.split(''),
      encode: input => input.join('')
    }),
    ipfs: 'b',
    type: 'base32'
  };
  const base32Validate = createValidate(config$1);
  const isBase32 = createIs(base32Validate);
  const base32Decode = createDecode(config$1, base32Validate);
  const base32Encode = createEncode(config$1);

  const config = {
    chars: 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/',
    coder: base64,
    type: 'base64'
  };
  const base64Validate = createValidate(config);
  const isBase64 = createIs(base64Validate);
  const base64Decode = createDecode(config, base64Validate);
  const base64Encode = createEncode(config);

  function base64Pad(value) {
    return value.padEnd(value.length + value.length % 4, '=');
  }

  function base64Trim(value) {
    while (value.length && value[value.length - 1] === '=') {
      value = value.slice(0, -1);
    }
    return value;
  }

  function secp256k1Compress(publicKey, onlyJs) {
    if (publicKey.length === 33) {
      return publicKey;
    }
    if (publicKey.length !== 65) {
      throw new Error('Invalid publicKey provided');
    }
    return !util.hasBigInt || !onlyJs && isReady() ? secp256k1Compress$1(publicKey) : Point.fromHex(publicKey).toRawBytes(true);
  }

  function secp256k1Expand(publicKey, onlyJs) {
    if (publicKey.length === 65) {
      return publicKey.subarray(1);
    }
    if (publicKey.length !== 33) {
      throw new Error('Invalid publicKey provided');
    }
    if (!util.hasBigInt || !onlyJs && isReady()) {
      return secp256k1Expand$1(publicKey).subarray(1);
    }
    const {
      x,
      y
    } = Point.fromHex(publicKey);
    return util.u8aConcat(util.bnToU8a(x, BN_BE_256_OPTS), util.bnToU8a(y, BN_BE_256_OPTS));
  }

  function secp256k1Recover(msgHash, signature, recovery, hashType = 'blake2', onlyJs) {
    const sig = util.u8aToU8a(signature).subarray(0, 64);
    const msg = util.u8aToU8a(msgHash);
    const publicKey = !util.hasBigInt || !onlyJs && isReady() ? secp256k1Recover$1(msg, sig, recovery) : recoverPublicKey(msg, Signature.fromCompact(sig).toRawBytes(), recovery);
    if (!publicKey) {
      throw new Error('Unable to recover publicKey from signature');
    }
    return hashType === 'keccak' ? secp256k1Expand(publicKey, onlyJs) : secp256k1Compress(publicKey, onlyJs);
  }

  function secp256k1Sign(message, {
    secretKey
  }, hashType = 'blake2', onlyJs) {
    if ((secretKey == null ? void 0 : secretKey.length) !== 32) {
      throw new Error('Expected valid secp256k1 secretKey, 32-bytes');
    }
    const data = hasher(hashType, message, onlyJs);
    if (!util.hasBigInt || !onlyJs && isReady()) {
      return secp256k1Sign$1(data, secretKey);
    }
    const [sigBytes, recoveryParam] = signSync(data, secretKey, {
      canonical: true,
      recovered: true
    });
    const {
      r,
      s
    } = Signature.fromHex(sigBytes);
    return util.u8aConcat(util.bnToU8a(r, BN_BE_256_OPTS), util.bnToU8a(s, BN_BE_256_OPTS), new Uint8Array([recoveryParam || 0]));
  }

  const N = 'ffffffff ffffffff ffffffff fffffffe baaedce6 af48a03b bfd25e8c d0364141'.replace(/ /g, '');
  const N_BI = BigInt$1(`0x${N}`);
  const N_BN = new util.BN(N, 'hex');
  function addBi(seckey, tweak) {
    let res = util.u8aToBigInt(tweak, BN_BE_OPTS);
    if (res >= N_BI) {
      throw new Error('Tweak parameter is out of range');
    }
    res += util.u8aToBigInt(seckey, BN_BE_OPTS);
    if (res >= N_BI) {
      res -= N_BI;
    }
    if (res === util._0n) {
      throw new Error('Invalid resulting private key');
    }
    return util.nToU8a(res, BN_BE_256_OPTS);
  }
  function addBn(seckey, tweak) {
    const res = new util.BN(tweak);
    if (res.cmp(N_BN) >= 0) {
      throw new Error('Tweak parameter is out of range');
    }
    res.iadd(new util.BN(seckey));
    if (res.cmp(N_BN) >= 0) {
      res.isub(N_BN);
    }
    if (res.isZero()) {
      throw new Error('Invalid resulting private key');
    }
    return util.bnToU8a(res, BN_BE_256_OPTS);
  }
  function secp256k1PrivateKeyTweakAdd(seckey, tweak, onlyBn) {
    if (!util.isU8a(seckey) || seckey.length !== 32) {
      throw new Error('Expected seckey to be an Uint8Array with length 32');
    } else if (!util.isU8a(tweak) || tweak.length !== 32) {
      throw new Error('Expected tweak to be an Uint8Array with length 32');
    }
    return !util.hasBigInt || onlyBn ? addBn(seckey, tweak) : addBi(seckey, tweak);
  }

  function secp256k1Verify(msgHash, signature, address, hashType = 'blake2', onlyJs) {
    const sig = util.u8aToU8a(signature);
    if (sig.length !== 65) {
      throw new Error(`Expected signature with 65 bytes, ${sig.length} found instead`);
    }
    const publicKey = secp256k1Recover(hasher(hashType, msgHash), sig, sig[64], hashType, onlyJs);
    const signerAddr = hasher(hashType, publicKey, onlyJs);
    const inputAddr = util.u8aToU8a(address);
    return util.u8aEq(publicKey, inputAddr) || (hashType === 'keccak' ? util.u8aEq(signerAddr.slice(-20), inputAddr.slice(-20)) : util.u8aEq(signerAddr, inputAddr));
  }

  function getH160(u8a) {
    if ([33, 65].includes(u8a.length)) {
      u8a = keccakAsU8a(secp256k1Expand(u8a));
    }
    return u8a.slice(-20);
  }
  function ethereumEncode(addressOrPublic) {
    if (!addressOrPublic) {
      return '0x';
    }
    const u8aAddress = util.u8aToU8a(addressOrPublic);
    if (![20, 32, 33, 65].includes(u8aAddress.length)) {
      throw new Error('Invalid address or publicKey passed');
    }
    const address = util.u8aToHex(getH160(u8aAddress), -1, false);
    const hash = util.u8aToHex(keccakAsU8a(address), -1, false);
    let result = '';
    for (let i = 0; i < 40; i++) {
      result = `${result}${parseInt(hash[i], 16) > 7 ? address[i].toUpperCase() : address[i]}`;
    }
    return `0x${result}`;
  }

  function isInvalidChar(char, byte) {
    return char !== (byte > 7 ? char.toUpperCase() : char.toLowerCase());
  }
  function isEthereumChecksum(_address) {
    const address = _address.replace('0x', '');
    const hash = util.u8aToHex(keccakAsU8a(address.toLowerCase()), -1, false);
    for (let i = 0; i < 40; i++) {
      if (isInvalidChar(address[i], parseInt(hash[i], 16))) {
        return false;
      }
    }
    return true;
  }

  function isEthereumAddress(address) {
    if (!address || address.length !== 42 || !util.isHex(address)) {
      return false;
    } else if (/^(0x)?[0-9a-f]{40}$/.test(address) || /^(0x)?[0-9A-F]{40}$/.test(address)) {
      return true;
    }
    return isEthereumChecksum(address);
  }

  const HARDENED = 0x80000000;
  function hdValidatePath(path) {
    if (!path.startsWith('m/')) {
      return false;
    }
    const parts = path.split('/').slice(1);
    for (const p of parts) {
      const n = /^\d+'?$/.test(p) ? parseInt(p.replace(/'$/, ''), 10) : Number.NaN;
      if (isNaN(n) || n >= HARDENED || n < 0) {
        return false;
      }
    }
    return true;
  }

  const MASTER_SECRET = util.stringToU8a('Bitcoin seed');
  function createCoded(secretKey, chainCode) {
    return {
      chainCode,
      publicKey: secp256k1PairFromSeed(secretKey).publicKey,
      secretKey
    };
  }
  function deriveChild(hd, index) {
    const indexBuffer = util.bnToU8a(index, BN_BE_32_OPTS);
    const data = index >= HARDENED ? util.u8aConcat(new Uint8Array(1), hd.secretKey, indexBuffer) : util.u8aConcat(hd.publicKey, indexBuffer);
    try {
      const I = hmacShaAsU8a(hd.chainCode, data, 512);
      return createCoded(secp256k1PrivateKeyTweakAdd(hd.secretKey, I.slice(0, 32)), I.slice(32));
    } catch (err) {
      return deriveChild(hd, index + 1);
    }
  }
  function hdEthereum(seed, path = '') {
    const I = hmacShaAsU8a(MASTER_SECRET, seed, 512);
    let hd = createCoded(I.slice(0, 32), I.slice(32));
    if (!path || path === 'm' || path === 'M' || path === "m'" || path === "M'") {
      return hd;
    }
    if (!hdValidatePath(path)) {
      throw new Error('Invalid derivation path');
    }
    const parts = path.split('/').slice(1);
    for (const p of parts) {
      hd = deriveChild(hd, parseInt(p, 10) + (p.length > 1 && p.endsWith("'") ? HARDENED : 0));
    }
    return hd;
  }

  function pbkdf2Init(hash, _password, _salt, _opts) {
      assert.hash(hash);
      const opts = checkOpts({ dkLen: 32, asyncTick: 10 }, _opts);
      const { c, dkLen, asyncTick } = opts;
      assert.number(c);
      assert.number(dkLen);
      assert.number(asyncTick);
      if (c < 1)
          throw new Error('PBKDF2: iterations (c) should be >= 1');
      const password = toBytes(_password);
      const salt = toBytes(_salt);
      const DK = new Uint8Array(dkLen);
      const PRF = hmac.create(hash, password);
      const PRFSalt = PRF._cloneInto().update(salt);
      return { c, dkLen, asyncTick, DK, PRF, PRFSalt };
  }
  function pbkdf2Output(PRF, PRFSalt, DK, prfW, u) {
      PRF.destroy();
      PRFSalt.destroy();
      if (prfW)
          prfW.destroy();
      u.fill(0);
      return DK;
  }
  function pbkdf2(hash, password, salt, opts) {
      const { c, dkLen, DK, PRF, PRFSalt } = pbkdf2Init(hash, password, salt, opts);
      let prfW;
      const arr = new Uint8Array(4);
      const view = createView(arr);
      const u = new Uint8Array(PRF.outputLen);
      for (let ti = 1, pos = 0; pos < dkLen; ti++, pos += PRF.outputLen) {
          const Ti = DK.subarray(pos, pos + PRF.outputLen);
          view.setInt32(0, ti, false);
          (prfW = PRFSalt._cloneInto(prfW)).update(arr).digestInto(u);
          Ti.set(u.subarray(0, Ti.length));
          for (let ui = 1; ui < c; ui++) {
              PRF._cloneInto(prfW).update(u).digestInto(u);
              for (let i = 0; i < Ti.length; i++)
                  Ti[i] ^= u[i];
          }
      }
      return pbkdf2Output(PRF, PRFSalt, DK, prfW, u);
  }

  function pbkdf2Encode(passphrase, salt = randomAsU8a(), rounds = 2048, onlyJs) {
    const u8aPass = util.u8aToU8a(passphrase);
    const u8aSalt = util.u8aToU8a(salt);
    return {
      password: !util.hasBigInt || !onlyJs && isReady() ? pbkdf2$1(u8aPass, u8aSalt, rounds) : pbkdf2(sha512, u8aPass, u8aSalt, {
        c: rounds,
        dkLen: 64
      }),
      rounds,
      salt
    };
  }

  const shaAsU8a = createDualHasher({
    256: sha256$1,
    512: sha512$1
  }, {
    256: sha256,
    512: sha512
  });
  const sha256AsU8a = createBitHasher(256, shaAsU8a);
  const sha512AsU8a = createBitHasher(512, shaAsU8a);

  const DEFAULT_WORDLIST = 'abandon|ability|able|about|above|absent|absorb|abstract|absurd|abuse|access|accident|account|accuse|achieve|acid|acoustic|acquire|across|act|action|actor|actress|actual|adapt|add|addict|address|adjust|admit|adult|advance|advice|aerobic|affair|afford|afraid|again|age|agent|agree|ahead|aim|air|airport|aisle|alarm|album|alcohol|alert|alien|all|alley|allow|almost|alone|alpha|already|also|alter|always|amateur|amazing|among|amount|amused|analyst|anchor|ancient|anger|angle|angry|animal|ankle|announce|annual|another|answer|antenna|antique|anxiety|any|apart|apology|appear|apple|approve|april|arch|arctic|area|arena|argue|arm|armed|armor|army|around|arrange|arrest|arrive|arrow|art|artefact|artist|artwork|ask|aspect|assault|asset|assist|assume|asthma|athlete|atom|attack|attend|attitude|attract|auction|audit|august|aunt|author|auto|autumn|average|avocado|avoid|awake|aware|away|awesome|awful|awkward|axis|baby|bachelor|bacon|badge|bag|balance|balcony|ball|bamboo|banana|banner|bar|barely|bargain|barrel|base|basic|basket|battle|beach|bean|beauty|because|become|beef|before|begin|behave|behind|believe|below|belt|bench|benefit|best|betray|better|between|beyond|bicycle|bid|bike|bind|biology|bird|birth|bitter|black|blade|blame|blanket|blast|bleak|bless|blind|blood|blossom|blouse|blue|blur|blush|board|boat|body|boil|bomb|bone|bonus|book|boost|border|boring|borrow|boss|bottom|bounce|box|boy|bracket|brain|brand|brass|brave|bread|breeze|brick|bridge|brief|bright|bring|brisk|broccoli|broken|bronze|broom|brother|brown|brush|bubble|buddy|budget|buffalo|build|bulb|bulk|bullet|bundle|bunker|burden|burger|burst|bus|business|busy|butter|buyer|buzz|cabbage|cabin|cable|cactus|cage|cake|call|calm|camera|camp|can|canal|cancel|candy|cannon|canoe|canvas|canyon|capable|capital|captain|car|carbon|card|cargo|carpet|carry|cart|case|cash|casino|castle|casual|cat|catalog|catch|category|cattle|caught|cause|caution|cave|ceiling|celery|cement|census|century|cereal|certain|chair|chalk|champion|change|chaos|chapter|charge|chase|chat|cheap|check|cheese|chef|cherry|chest|chicken|chief|child|chimney|choice|choose|chronic|chuckle|chunk|churn|cigar|cinnamon|circle|citizen|city|civil|claim|clap|clarify|claw|clay|clean|clerk|clever|click|client|cliff|climb|clinic|clip|clock|clog|close|cloth|cloud|clown|club|clump|cluster|clutch|coach|coast|coconut|code|coffee|coil|coin|collect|color|column|combine|come|comfort|comic|common|company|concert|conduct|confirm|congress|connect|consider|control|convince|cook|cool|copper|copy|coral|core|corn|correct|cost|cotton|couch|country|couple|course|cousin|cover|coyote|crack|cradle|craft|cram|crane|crash|crater|crawl|crazy|cream|credit|creek|crew|cricket|crime|crisp|critic|crop|cross|crouch|crowd|crucial|cruel|cruise|crumble|crunch|crush|cry|crystal|cube|culture|cup|cupboard|curious|current|curtain|curve|cushion|custom|cute|cycle|dad|damage|damp|dance|danger|daring|dash|daughter|dawn|day|deal|debate|debris|decade|december|decide|decline|decorate|decrease|deer|defense|define|defy|degree|delay|deliver|demand|demise|denial|dentist|deny|depart|depend|deposit|depth|deputy|derive|describe|desert|design|desk|despair|destroy|detail|detect|develop|device|devote|diagram|dial|diamond|diary|dice|diesel|diet|differ|digital|dignity|dilemma|dinner|dinosaur|direct|dirt|disagree|discover|disease|dish|dismiss|disorder|display|distance|divert|divide|divorce|dizzy|doctor|document|dog|doll|dolphin|domain|donate|donkey|donor|door|dose|double|dove|draft|dragon|drama|drastic|draw|dream|dress|drift|drill|drink|drip|drive|drop|drum|dry|duck|dumb|dune|during|dust|dutch|duty|dwarf|dynamic|eager|eagle|early|earn|earth|easily|east|easy|echo|ecology|economy|edge|edit|educate|effort|egg|eight|either|elbow|elder|electric|elegant|element|elephant|elevator|elite|else|embark|embody|embrace|emerge|emotion|employ|empower|empty|enable|enact|end|endless|endorse|enemy|energy|enforce|engage|engine|enhance|enjoy|enlist|enough|enrich|enroll|ensure|enter|entire|entry|envelope|episode|equal|equip|era|erase|erode|erosion|error|erupt|escape|essay|essence|estate|eternal|ethics|evidence|evil|evoke|evolve|exact|example|excess|exchange|excite|exclude|excuse|execute|exercise|exhaust|exhibit|exile|exist|exit|exotic|expand|expect|expire|explain|expose|express|extend|extra|eye|eyebrow|fabric|face|faculty|fade|faint|faith|fall|false|fame|family|famous|fan|fancy|fantasy|farm|fashion|fat|fatal|father|fatigue|fault|favorite|feature|february|federal|fee|feed|feel|female|fence|festival|fetch|fever|few|fiber|fiction|field|figure|file|film|filter|final|find|fine|finger|finish|fire|firm|first|fiscal|fish|fit|fitness|fix|flag|flame|flash|flat|flavor|flee|flight|flip|float|flock|floor|flower|fluid|flush|fly|foam|focus|fog|foil|fold|follow|food|foot|force|forest|forget|fork|fortune|forum|forward|fossil|foster|found|fox|fragile|frame|frequent|fresh|friend|fringe|frog|front|frost|frown|frozen|fruit|fuel|fun|funny|furnace|fury|future|gadget|gain|galaxy|gallery|game|gap|garage|garbage|garden|garlic|garment|gas|gasp|gate|gather|gauge|gaze|general|genius|genre|gentle|genuine|gesture|ghost|giant|gift|giggle|ginger|giraffe|girl|give|glad|glance|glare|glass|glide|glimpse|globe|gloom|glory|glove|glow|glue|goat|goddess|gold|good|goose|gorilla|gospel|gossip|govern|gown|grab|grace|grain|grant|grape|grass|gravity|great|green|grid|grief|grit|grocery|group|grow|grunt|guard|guess|guide|guilt|guitar|gun|gym|habit|hair|half|hammer|hamster|hand|happy|harbor|hard|harsh|harvest|hat|have|hawk|hazard|head|health|heart|heavy|hedgehog|height|hello|helmet|help|hen|hero|hidden|high|hill|hint|hip|hire|history|hobby|hockey|hold|hole|holiday|hollow|home|honey|hood|hope|horn|horror|horse|hospital|host|hotel|hour|hover|hub|huge|human|humble|humor|hundred|hungry|hunt|hurdle|hurry|hurt|husband|hybrid|ice|icon|idea|identify|idle|ignore|ill|illegal|illness|image|imitate|immense|immune|impact|impose|improve|impulse|inch|include|income|increase|index|indicate|indoor|industry|infant|inflict|inform|inhale|inherit|initial|inject|injury|inmate|inner|innocent|input|inquiry|insane|insect|inside|inspire|install|intact|interest|into|invest|invite|involve|iron|island|isolate|issue|item|ivory|jacket|jaguar|jar|jazz|jealous|jeans|jelly|jewel|job|join|joke|journey|joy|judge|juice|jump|jungle|junior|junk|just|kangaroo|keen|keep|ketchup|key|kick|kid|kidney|kind|kingdom|kiss|kit|kitchen|kite|kitten|kiwi|knee|knife|knock|know|lab|label|labor|ladder|lady|lake|lamp|language|laptop|large|later|latin|laugh|laundry|lava|law|lawn|lawsuit|layer|lazy|leader|leaf|learn|leave|lecture|left|leg|legal|legend|leisure|lemon|lend|length|lens|leopard|lesson|letter|level|liar|liberty|library|license|life|lift|light|like|limb|limit|link|lion|liquid|list|little|live|lizard|load|loan|lobster|local|lock|logic|lonely|long|loop|lottery|loud|lounge|love|loyal|lucky|luggage|lumber|lunar|lunch|luxury|lyrics|machine|mad|magic|magnet|maid|mail|main|major|make|mammal|man|manage|mandate|mango|mansion|manual|maple|marble|march|margin|marine|market|marriage|mask|mass|master|match|material|math|matrix|matter|maximum|maze|meadow|mean|measure|meat|mechanic|medal|media|melody|melt|member|memory|mention|menu|mercy|merge|merit|merry|mesh|message|metal|method|middle|midnight|milk|million|mimic|mind|minimum|minor|minute|miracle|mirror|misery|miss|mistake|mix|mixed|mixture|mobile|model|modify|mom|moment|monitor|monkey|monster|month|moon|moral|more|morning|mosquito|mother|motion|motor|mountain|mouse|move|movie|much|muffin|mule|multiply|muscle|museum|mushroom|music|must|mutual|myself|mystery|myth|naive|name|napkin|narrow|nasty|nation|nature|near|neck|need|negative|neglect|neither|nephew|nerve|nest|net|network|neutral|never|news|next|nice|night|noble|noise|nominee|noodle|normal|north|nose|notable|note|nothing|notice|novel|now|nuclear|number|nurse|nut|oak|obey|object|oblige|obscure|observe|obtain|obvious|occur|ocean|october|odor|off|offer|office|often|oil|okay|old|olive|olympic|omit|once|one|onion|online|only|open|opera|opinion|oppose|option|orange|orbit|orchard|order|ordinary|organ|orient|original|orphan|ostrich|other|outdoor|outer|output|outside|oval|oven|over|own|owner|oxygen|oyster|ozone|pact|paddle|page|pair|palace|palm|panda|panel|panic|panther|paper|parade|parent|park|parrot|party|pass|patch|path|patient|patrol|pattern|pause|pave|payment|peace|peanut|pear|peasant|pelican|pen|penalty|pencil|people|pepper|perfect|permit|person|pet|phone|photo|phrase|physical|piano|picnic|picture|piece|pig|pigeon|pill|pilot|pink|pioneer|pipe|pistol|pitch|pizza|place|planet|plastic|plate|play|please|pledge|pluck|plug|plunge|poem|poet|point|polar|pole|police|pond|pony|pool|popular|portion|position|possible|post|potato|pottery|poverty|powder|power|practice|praise|predict|prefer|prepare|present|pretty|prevent|price|pride|primary|print|priority|prison|private|prize|problem|process|produce|profit|program|project|promote|proof|property|prosper|protect|proud|provide|public|pudding|pull|pulp|pulse|pumpkin|punch|pupil|puppy|purchase|purity|purpose|purse|push|put|puzzle|pyramid|quality|quantum|quarter|question|quick|quit|quiz|quote|rabbit|raccoon|race|rack|radar|radio|rail|rain|raise|rally|ramp|ranch|random|range|rapid|rare|rate|rather|raven|raw|razor|ready|real|reason|rebel|rebuild|recall|receive|recipe|record|recycle|reduce|reflect|reform|refuse|region|regret|regular|reject|relax|release|relief|rely|remain|remember|remind|remove|render|renew|rent|reopen|repair|repeat|replace|report|require|rescue|resemble|resist|resource|response|result|retire|retreat|return|reunion|reveal|review|reward|rhythm|rib|ribbon|rice|rich|ride|ridge|rifle|right|rigid|ring|riot|ripple|risk|ritual|rival|river|road|roast|robot|robust|rocket|romance|roof|rookie|room|rose|rotate|rough|round|route|royal|rubber|rude|rug|rule|run|runway|rural|sad|saddle|sadness|safe|sail|salad|salmon|salon|salt|salute|same|sample|sand|satisfy|satoshi|sauce|sausage|save|say|scale|scan|scare|scatter|scene|scheme|school|science|scissors|scorpion|scout|scrap|screen|script|scrub|sea|search|season|seat|second|secret|section|security|seed|seek|segment|select|sell|seminar|senior|sense|sentence|series|service|session|settle|setup|seven|shadow|shaft|shallow|share|shed|shell|sheriff|shield|shift|shine|ship|shiver|shock|shoe|shoot|shop|short|shoulder|shove|shrimp|shrug|shuffle|shy|sibling|sick|side|siege|sight|sign|silent|silk|silly|silver|similar|simple|since|sing|siren|sister|situate|six|size|skate|sketch|ski|skill|skin|skirt|skull|slab|slam|sleep|slender|slice|slide|slight|slim|slogan|slot|slow|slush|small|smart|smile|smoke|smooth|snack|snake|snap|sniff|snow|soap|soccer|social|sock|soda|soft|solar|soldier|solid|solution|solve|someone|song|soon|sorry|sort|soul|sound|soup|source|south|space|spare|spatial|spawn|speak|special|speed|spell|spend|sphere|spice|spider|spike|spin|spirit|split|spoil|sponsor|spoon|sport|spot|spray|spread|spring|spy|square|squeeze|squirrel|stable|stadium|staff|stage|stairs|stamp|stand|start|state|stay|steak|steel|stem|step|stereo|stick|still|sting|stock|stomach|stone|stool|story|stove|strategy|street|strike|strong|struggle|student|stuff|stumble|style|subject|submit|subway|success|such|sudden|suffer|sugar|suggest|suit|summer|sun|sunny|sunset|super|supply|supreme|sure|surface|surge|surprise|surround|survey|suspect|sustain|swallow|swamp|swap|swarm|swear|sweet|swift|swim|swing|switch|sword|symbol|symptom|syrup|system|table|tackle|tag|tail|talent|talk|tank|tape|target|task|taste|tattoo|taxi|teach|team|tell|ten|tenant|tennis|tent|term|test|text|thank|that|theme|then|theory|there|they|thing|this|thought|three|thrive|throw|thumb|thunder|ticket|tide|tiger|tilt|timber|time|tiny|tip|tired|tissue|title|toast|tobacco|today|toddler|toe|together|toilet|token|tomato|tomorrow|tone|tongue|tonight|tool|tooth|top|topic|topple|torch|tornado|tortoise|toss|total|tourist|toward|tower|town|toy|track|trade|traffic|tragic|train|transfer|trap|trash|travel|tray|treat|tree|trend|trial|tribe|trick|trigger|trim|trip|trophy|trouble|truck|true|truly|trumpet|trust|truth|try|tube|tuition|tumble|tuna|tunnel|turkey|turn|turtle|twelve|twenty|twice|twin|twist|two|type|typical|ugly|umbrella|unable|unaware|uncle|uncover|under|undo|unfair|unfold|unhappy|uniform|unique|unit|universe|unknown|unlock|until|unusual|unveil|update|upgrade|uphold|upon|upper|upset|urban|urge|usage|use|used|useful|useless|usual|utility|vacant|vacuum|vague|valid|valley|valve|van|vanish|vapor|various|vast|vault|vehicle|velvet|vendor|venture|venue|verb|verify|version|very|vessel|veteran|viable|vibrant|vicious|victory|video|view|village|vintage|violin|virtual|virus|visa|visit|visual|vital|vivid|vocal|voice|void|volcano|volume|vote|voyage|wage|wagon|wait|walk|wall|walnut|want|warfare|warm|warrior|wash|wasp|waste|water|wave|way|wealth|weapon|wear|weasel|weather|web|wedding|weekend|weird|welcome|west|wet|whale|what|wheat|wheel|when|where|whip|whisper|wide|width|wife|wild|will|win|window|wine|wing|wink|winner|winter|wire|wisdom|wise|wish|witness|wolf|woman|wonder|wood|wool|word|work|world|worry|worth|wrap|wreck|wrestle|wrist|write|wrong|yard|year|yellow|you|young|youth|zebra|zero|zone|zoo'.split('|');

  const INVALID_MNEMONIC = 'Invalid mnemonic';
  const INVALID_ENTROPY = 'Invalid entropy';
  const INVALID_CHECKSUM = 'Invalid mnemonic checksum';
  function normalize(str) {
    return (str || '').normalize('NFKD');
  }
  function binaryToByte(bin) {
    return parseInt(bin, 2);
  }
  function bytesToBinary(bytes) {
    return bytes.map(x => x.toString(2).padStart(8, '0')).join('');
  }
  function deriveChecksumBits(entropyBuffer) {
    return bytesToBinary(Array.from(sha256AsU8a(entropyBuffer))).slice(0, entropyBuffer.length * 8 / 32);
  }
  function mnemonicToSeedSync(mnemonic, password) {
    return pbkdf2Encode(util.stringToU8a(normalize(mnemonic)), util.stringToU8a(`mnemonic${normalize(password)}`)).password;
  }
  function mnemonicToEntropy$1(mnemonic) {
    const words = normalize(mnemonic).split(' ');
    if (words.length % 3 !== 0) {
      throw new Error(INVALID_MNEMONIC);
    }
    const bits = words.map(word => {
      const index = DEFAULT_WORDLIST.indexOf(word);
      if (index === -1) {
        throw new Error(INVALID_MNEMONIC);
      }
      return index.toString(2).padStart(11, '0');
    }).join('');
    const dividerIndex = Math.floor(bits.length / 33) * 32;
    const entropyBits = bits.slice(0, dividerIndex);
    const checksumBits = bits.slice(dividerIndex);
    const matched = entropyBits.match(/(.{1,8})/g);
    const entropyBytes = matched && matched.map(binaryToByte);
    if (!entropyBytes || entropyBytes.length % 4 !== 0 || entropyBytes.length < 16 || entropyBytes.length > 32) {
      throw new Error(INVALID_ENTROPY);
    }
    const entropy = util.u8aToU8a(entropyBytes);
    if (deriveChecksumBits(entropy) !== checksumBits) {
      throw new Error(INVALID_CHECKSUM);
    }
    return entropy;
  }
  function entropyToMnemonic(entropy) {
    if (entropy.length % 4 !== 0 || entropy.length < 16 || entropy.length > 32) {
      throw new Error(INVALID_ENTROPY);
    }
    const matched = `${bytesToBinary(Array.from(entropy))}${deriveChecksumBits(entropy)}`.match(/(.{1,11})/g);
    const mapped = matched && matched.map(binary => DEFAULT_WORDLIST[binaryToByte(binary)]);
    if (!mapped || mapped.length < 12) {
      throw new Error('Unable to map entropy to mnemonic');
    }
    return mapped.join(' ');
  }
  function generateMnemonic(numWords) {
    return entropyToMnemonic(randomAsU8a(numWords / 3 * 4));
  }
  function validateMnemonic(mnemonic) {
    try {
      mnemonicToEntropy$1(mnemonic);
    } catch (e) {
      return false;
    }
    return true;
  }

  function mnemonicGenerate(numWords = 12, onlyJs) {
    return !util.hasBigInt || !onlyJs && isReady() ? bip39Generate(numWords) : generateMnemonic(numWords);
  }

  function mnemonicToEntropy(mnemonic, onlyJs) {
    return !util.hasBigInt || !onlyJs && isReady() ? bip39ToEntropy(mnemonic) : mnemonicToEntropy$1(mnemonic);
  }

  function mnemonicValidate(mnemonic, onlyJs) {
    return !util.hasBigInt || !onlyJs && isReady() ? bip39Validate(mnemonic) : validateMnemonic(mnemonic);
  }

  function mnemonicToLegacySeed(mnemonic, password = '', onlyJs, byteLength = 32) {
    if (!mnemonicValidate(mnemonic)) {
      throw new Error('Invalid bip39 mnemonic specified');
    } else if (![32, 64].includes(byteLength)) {
      throw new Error(`Invalid seed length ${byteLength}, expected 32 or 64`);
    }
    return byteLength === 32 ? !util.hasBigInt || !onlyJs && isReady() ? bip39ToSeed(mnemonic, password) : mnemonicToSeedSync(mnemonic, password).subarray(0, 32) : mnemonicToSeedSync(mnemonic, password);
  }

  function mnemonicToMiniSecret(mnemonic, password = '', onlyJs) {
    if (!mnemonicValidate(mnemonic)) {
      throw new Error('Invalid bip39 mnemonic specified');
    }
    if (!onlyJs && isReady()) {
      return bip39ToMiniSecret(mnemonic, password);
    }
    const entropy = mnemonicToEntropy(mnemonic);
    const salt = util.stringToU8a(`mnemonic${password}`);
    return pbkdf2Encode(entropy, salt).password.slice(0, 32);
  }

  function ledgerDerivePrivate(xprv, index) {
    const kl = xprv.subarray(0, 32);
    const kr = xprv.subarray(32, 64);
    const cc = xprv.subarray(64, 96);
    const data = util.u8aConcat([0], kl, kr, util.bnToU8a(index, BN_LE_32_OPTS));
    const z = hmacShaAsU8a(cc, data, 512);
    data[0] = 0x01;
    return util.u8aConcat(util.bnToU8a(util.u8aToBn(kl, BN_LE_OPTS).iadd(util.u8aToBn(z.subarray(0, 28), BN_LE_OPTS).imul(util.BN_EIGHT)), BN_LE_512_OPTS).subarray(0, 32), util.bnToU8a(util.u8aToBn(kr, BN_LE_OPTS).iadd(util.u8aToBn(z.subarray(32, 64), BN_LE_OPTS)), BN_LE_512_OPTS).subarray(0, 32), hmacShaAsU8a(cc, data, 512).subarray(32, 64));
  }

  const ED25519_CRYPTO = 'ed25519 seed';
  function ledgerMaster(mnemonic, password) {
    const seed = mnemonicToSeedSync(mnemonic, password);
    const chainCode = hmacShaAsU8a(ED25519_CRYPTO, new Uint8Array([1, ...seed]), 256);
    let priv;
    while (!priv || priv[31] & 0b00100000) {
      priv = hmacShaAsU8a(ED25519_CRYPTO, priv || seed, 512);
    }
    priv[0] &= 0b11111000;
    priv[31] &= 0b01111111;
    priv[31] |= 0b01000000;
    return util.u8aConcat(priv, chainCode);
  }

  function hdLedger(_mnemonic, path) {
    const words = _mnemonic.split(' ').map(s => s.trim()).filter(s => s);
    if (![12, 24, 25].includes(words.length)) {
      throw new Error('Expected a mnemonic with 24 words (or 25 including a password)');
    }
    const [mnemonic, password] = words.length === 25 ? [words.slice(0, 24).join(' '), words[24]] : [words.join(' '), ''];
    if (!mnemonicValidate(mnemonic)) {
      throw new Error('Invalid mnemonic passed to ledger derivation');
    } else if (!hdValidatePath(path)) {
      throw new Error('Invalid derivation path');
    }
    const parts = path.split('/').slice(1);
    let seed = ledgerMaster(mnemonic, password);
    for (const p of parts) {
      const n = parseInt(p.replace(/'$/, ''), 10);
      seed = ledgerDerivePrivate(seed, n < HARDENED ? n + HARDENED : n);
    }
    return ed25519PairFromSeed(seed.slice(0, 32));
  }

  function naclDecrypt(encrypted, nonce, secret) {
    return nacl.secretbox.open(encrypted, nonce, secret) || null;
  }

  function naclEncrypt(message, secret, nonce = randomAsU8a(24)) {
    return {
      encrypted: nacl.secretbox(message, nonce, secret),
      nonce
    };
  }

  function naclBoxPairFromSecret(secret) {
    return nacl.box.keyPair.fromSecretKey(secret.slice(0, 32));
  }

  function naclOpen(sealed, nonce, senderBoxPublic, receiverBoxSecret) {
    return nacl.box.open(sealed, nonce, senderBoxPublic, receiverBoxSecret) || null;
  }

  function naclSeal(message, senderBoxSecret, receiverBoxPublic, nonce = randomAsU8a(24)) {
    return {
      nonce,
      sealed: nacl.box(message, nonce, receiverBoxPublic, senderBoxSecret)
    };
  }

  const rotl$1 = (a, b) => (a << b) | (a >>> (32 - b));
  function XorAndSalsa(prev, pi, input, ii, out, oi) {
      let y00 = prev[pi++] ^ input[ii++], y01 = prev[pi++] ^ input[ii++];
      let y02 = prev[pi++] ^ input[ii++], y03 = prev[pi++] ^ input[ii++];
      let y04 = prev[pi++] ^ input[ii++], y05 = prev[pi++] ^ input[ii++];
      let y06 = prev[pi++] ^ input[ii++], y07 = prev[pi++] ^ input[ii++];
      let y08 = prev[pi++] ^ input[ii++], y09 = prev[pi++] ^ input[ii++];
      let y10 = prev[pi++] ^ input[ii++], y11 = prev[pi++] ^ input[ii++];
      let y12 = prev[pi++] ^ input[ii++], y13 = prev[pi++] ^ input[ii++];
      let y14 = prev[pi++] ^ input[ii++], y15 = prev[pi++] ^ input[ii++];
      let x00 = y00, x01 = y01, x02 = y02, x03 = y03, x04 = y04, x05 = y05, x06 = y06, x07 = y07, x08 = y08, x09 = y09, x10 = y10, x11 = y11, x12 = y12, x13 = y13, x14 = y14, x15 = y15;
      for (let i = 0; i < 8; i += 2) {
          x04 ^= rotl$1(x00 + x12 | 0, 7);
          x08 ^= rotl$1(x04 + x00 | 0, 9);
          x12 ^= rotl$1(x08 + x04 | 0, 13);
          x00 ^= rotl$1(x12 + x08 | 0, 18);
          x09 ^= rotl$1(x05 + x01 | 0, 7);
          x13 ^= rotl$1(x09 + x05 | 0, 9);
          x01 ^= rotl$1(x13 + x09 | 0, 13);
          x05 ^= rotl$1(x01 + x13 | 0, 18);
          x14 ^= rotl$1(x10 + x06 | 0, 7);
          x02 ^= rotl$1(x14 + x10 | 0, 9);
          x06 ^= rotl$1(x02 + x14 | 0, 13);
          x10 ^= rotl$1(x06 + x02 | 0, 18);
          x03 ^= rotl$1(x15 + x11 | 0, 7);
          x07 ^= rotl$1(x03 + x15 | 0, 9);
          x11 ^= rotl$1(x07 + x03 | 0, 13);
          x15 ^= rotl$1(x11 + x07 | 0, 18);
          x01 ^= rotl$1(x00 + x03 | 0, 7);
          x02 ^= rotl$1(x01 + x00 | 0, 9);
          x03 ^= rotl$1(x02 + x01 | 0, 13);
          x00 ^= rotl$1(x03 + x02 | 0, 18);
          x06 ^= rotl$1(x05 + x04 | 0, 7);
          x07 ^= rotl$1(x06 + x05 | 0, 9);
          x04 ^= rotl$1(x07 + x06 | 0, 13);
          x05 ^= rotl$1(x04 + x07 | 0, 18);
          x11 ^= rotl$1(x10 + x09 | 0, 7);
          x08 ^= rotl$1(x11 + x10 | 0, 9);
          x09 ^= rotl$1(x08 + x11 | 0, 13);
          x10 ^= rotl$1(x09 + x08 | 0, 18);
          x12 ^= rotl$1(x15 + x14 | 0, 7);
          x13 ^= rotl$1(x12 + x15 | 0, 9);
          x14 ^= rotl$1(x13 + x12 | 0, 13);
          x15 ^= rotl$1(x14 + x13 | 0, 18);
      }
      out[oi++] = (y00 + x00) | 0;
      out[oi++] = (y01 + x01) | 0;
      out[oi++] = (y02 + x02) | 0;
      out[oi++] = (y03 + x03) | 0;
      out[oi++] = (y04 + x04) | 0;
      out[oi++] = (y05 + x05) | 0;
      out[oi++] = (y06 + x06) | 0;
      out[oi++] = (y07 + x07) | 0;
      out[oi++] = (y08 + x08) | 0;
      out[oi++] = (y09 + x09) | 0;
      out[oi++] = (y10 + x10) | 0;
      out[oi++] = (y11 + x11) | 0;
      out[oi++] = (y12 + x12) | 0;
      out[oi++] = (y13 + x13) | 0;
      out[oi++] = (y14 + x14) | 0;
      out[oi++] = (y15 + x15) | 0;
  }
  function BlockMix(input, ii, out, oi, r) {
      let head = oi + 0;
      let tail = oi + 16 * r;
      for (let i = 0; i < 16; i++)
          out[tail + i] = input[ii + (2 * r - 1) * 16 + i];
      for (let i = 0; i < r; i++, head += 16, ii += 16) {
          XorAndSalsa(out, tail, input, ii, out, head);
          if (i > 0)
              tail += 16;
          XorAndSalsa(out, head, input, (ii += 16), out, tail);
      }
  }
  function scryptInit(password, salt, _opts) {
      const opts = checkOpts({
          dkLen: 32,
          asyncTick: 10,
          maxmem: 1024 ** 3 + 1024,
      }, _opts);
      const { N, r, p, dkLen, asyncTick, maxmem, onProgress } = opts;
      assert.number(N);
      assert.number(r);
      assert.number(p);
      assert.number(dkLen);
      assert.number(asyncTick);
      assert.number(maxmem);
      if (onProgress !== undefined && typeof onProgress !== 'function')
          throw new Error('progressCb should be function');
      const blockSize = 128 * r;
      const blockSize32 = blockSize / 4;
      if (N <= 1 || (N & (N - 1)) !== 0 || N >= 2 ** (blockSize / 8) || N > 2 ** 32) {
          throw new Error('Scrypt: N must be larger than 1, a power of 2, less than 2^(128 * r / 8) and less than 2^32');
      }
      if (p < 0 || p > ((2 ** 32 - 1) * 32) / blockSize) {
          throw new Error('Scrypt: p must be a positive integer less than or equal to ((2^32 - 1) * 32) / (128 * r)');
      }
      if (dkLen < 0 || dkLen > (2 ** 32 - 1) * 32) {
          throw new Error('Scrypt: dkLen should be positive integer less than or equal to (2^32 - 1) * 32');
      }
      const memUsed = blockSize * (N + p);
      if (memUsed > maxmem) {
          throw new Error(`Scrypt: parameters too large, ${memUsed} (128 * r * (N + p)) > ${maxmem} (maxmem)`);
      }
      const B = pbkdf2(sha256, password, salt, { c: 1, dkLen: blockSize * p });
      const B32 = u32(B);
      const V = u32(new Uint8Array(blockSize * N));
      const tmp = u32(new Uint8Array(blockSize));
      let blockMixCb = () => { };
      if (onProgress) {
          const totalBlockMix = 2 * N * p;
          const callbackPer = Math.max(Math.floor(totalBlockMix / 10000), 1);
          let blockMixCnt = 0;
          blockMixCb = () => {
              blockMixCnt++;
              if (onProgress && (!(blockMixCnt % callbackPer) || blockMixCnt === totalBlockMix))
                  onProgress(blockMixCnt / totalBlockMix);
          };
      }
      return { N, r, p, dkLen, blockSize32, V, B32, B, tmp, blockMixCb, asyncTick };
  }
  function scryptOutput(password, dkLen, B, V, tmp) {
      const res = pbkdf2(sha256, password, B, { c: 1, dkLen });
      B.fill(0);
      V.fill(0);
      tmp.fill(0);
      return res;
  }
  function scrypt(password, salt, opts) {
      const { N, r, p, dkLen, blockSize32, V, B32, B, tmp, blockMixCb } = scryptInit(password, salt, opts);
      for (let pi = 0; pi < p; pi++) {
          const Pi = blockSize32 * pi;
          for (let i = 0; i < blockSize32; i++)
              V[i] = B32[Pi + i];
          for (let i = 0, pos = 0; i < N - 1; i++) {
              BlockMix(V, pos, V, (pos += blockSize32), r);
              blockMixCb();
          }
          BlockMix(V, (N - 1) * blockSize32, B32, Pi, r);
          blockMixCb();
          for (let i = 0; i < N; i++) {
              const j = B32[Pi + blockSize32 - 16] % N;
              for (let k = 0; k < blockSize32; k++)
                  tmp[k] = B32[Pi + k] ^ V[j * blockSize32 + k];
              BlockMix(tmp, 0, B32, Pi, r);
              blockMixCb();
          }
      }
      return scryptOutput(password, dkLen, B, V, tmp);
  }

  const DEFAULT_PARAMS = {
    N: 1 << 15,
    p: 1,
    r: 8
  };

  function scryptEncode(passphrase, salt = randomAsU8a(), params = DEFAULT_PARAMS, onlyJs) {
    const u8a = util.u8aToU8a(passphrase);
    return {
      params,
      password: !util.hasBigInt || !onlyJs && isReady() ? scrypt$1(u8a, salt, Math.log2(params.N), params.r, params.p) : scrypt(u8a, salt, util.objectSpread({
        dkLen: 64
      }, params)),
      salt
    };
  }

  function scryptFromU8a(data) {
    const salt = data.subarray(0, 32);
    const N = util.u8aToBn(data.subarray(32 + 0, 32 + 4), BN_LE_OPTS).toNumber();
    const p = util.u8aToBn(data.subarray(32 + 4, 32 + 8), BN_LE_OPTS).toNumber();
    const r = util.u8aToBn(data.subarray(32 + 8, 32 + 12), BN_LE_OPTS).toNumber();
    if (N !== DEFAULT_PARAMS.N || p !== DEFAULT_PARAMS.p || r !== DEFAULT_PARAMS.r) {
      throw new Error('Invalid injected scrypt params found');
    }
    return {
      params: {
        N,
        p,
        r
      },
      salt
    };
  }

  function scryptToU8a(salt, {
    N,
    p,
    r
  }) {
    return util.u8aConcat(salt, util.bnToU8a(N, BN_LE_32_OPTS), util.bnToU8a(p, BN_LE_32_OPTS), util.bnToU8a(r, BN_LE_32_OPTS));
  }

  const ENCODING = ['scrypt', 'xsalsa20-poly1305'];
  const ENCODING_NONE = ['none'];
  const ENCODING_VERSION = '3';
  const NONCE_LENGTH = 24;
  const SCRYPT_LENGTH = 32 + 3 * 4;

  function jsonDecryptData(encrypted, passphrase, encType = ENCODING) {
    if (!encrypted) {
      throw new Error('No encrypted data available to decode');
    } else if (encType.includes('xsalsa20-poly1305') && !passphrase) {
      throw new Error('Password required to decode encrypted data');
    }
    let encoded = encrypted;
    if (passphrase) {
      let password;
      if (encType.includes('scrypt')) {
        const {
          params,
          salt
        } = scryptFromU8a(encrypted);
        password = scryptEncode(passphrase, salt, params).password;
        encrypted = encrypted.subarray(SCRYPT_LENGTH);
      } else {
        password = util.stringToU8a(passphrase);
      }
      encoded = naclDecrypt(encrypted.subarray(NONCE_LENGTH), encrypted.subarray(0, NONCE_LENGTH), util.u8aFixLength(password, 256, true));
    }
    if (!encoded) {
      throw new Error('Unable to decode using the supplied passphrase');
    }
    return encoded;
  }

  function jsonDecrypt({
    encoded,
    encoding
  }, passphrase) {
    if (!encoded) {
      throw new Error('No encrypted data available to decode');
    }
    return jsonDecryptData(util.isHex(encoded) ? util.hexToU8a(encoded) : base64Decode(encoded), passphrase, Array.isArray(encoding.type) ? encoding.type : [encoding.type]);
  }

  function jsonEncryptFormat(encoded, contentType, isEncrypted) {
    return {
      encoded: base64Encode(encoded),
      encoding: {
        content: contentType,
        type: isEncrypted ? ENCODING : ENCODING_NONE,
        version: ENCODING_VERSION
      }
    };
  }

  function jsonEncrypt(data, contentType, passphrase) {
    let isEncrypted = false;
    let encoded = data;
    if (passphrase) {
      const {
        params,
        password,
        salt
      } = scryptEncode(passphrase);
      const {
        encrypted,
        nonce
      } = naclEncrypt(encoded, password.subarray(0, 32));
      isEncrypted = true;
      encoded = util.u8aConcat(scryptToU8a(salt, params), nonce, encrypted);
    }
    return jsonEncryptFormat(encoded, contentType, isEncrypted);
  }

  const secp256k1VerifyHasher = hashType => (message, signature, publicKey) => secp256k1Verify(message, signature, publicKey, hashType);
  const VERIFIERS_ECDSA = [['ecdsa', secp256k1VerifyHasher('blake2')], ['ethereum', secp256k1VerifyHasher('keccak')]];
  const VERIFIERS = [['ed25519', ed25519Verify], ['sr25519', sr25519Verify], ...VERIFIERS_ECDSA];
  const CRYPTO_TYPES = ['ed25519', 'sr25519', 'ecdsa'];
  function verifyDetect(result, {
    message,
    publicKey,
    signature
  }, verifiers = VERIFIERS) {
    result.isValid = verifiers.some(([crypto, verify]) => {
      try {
        if (verify(message, signature, publicKey)) {
          result.crypto = crypto;
          return true;
        }
      } catch (error) {
      }
      return false;
    });
    return result;
  }
  function verifyMultisig(result, {
    message,
    publicKey,
    signature
  }) {
    if (![0, 1, 2].includes(signature[0])) {
      throw new Error(`Unknown crypto type, expected signature prefix [0..2], found ${signature[0]}`);
    }
    const type = CRYPTO_TYPES[signature[0]] || 'none';
    result.crypto = type;
    try {
      result.isValid = {
        ecdsa: () => verifyDetect(result, {
          message,
          publicKey,
          signature: signature.subarray(1)
        }, VERIFIERS_ECDSA).isValid,
        ed25519: () => ed25519Verify(message, signature.subarray(1), publicKey),
        none: () => {
          throw Error('no verify for `none` crypto type');
        },
        sr25519: () => sr25519Verify(message, signature.subarray(1), publicKey)
      }[type]();
    } catch (error) {
    }
    return result;
  }
  function getVerifyFn(signature) {
    return [0, 1, 2].includes(signature[0]) && [65, 66].includes(signature.length) ? verifyMultisig : verifyDetect;
  }
  function signatureVerify(message, signature, addressOrPublicKey) {
    const signatureU8a = util.u8aToU8a(signature);
    if (![64, 65, 66].includes(signatureU8a.length)) {
      throw new Error(`Invalid signature length, expected [64..66] bytes, found ${signatureU8a.length}`);
    }
    const publicKey = decodeAddress(addressOrPublicKey);
    const input = {
      message: util.u8aToU8a(message),
      publicKey,
      signature: signatureU8a
    };
    const result = {
      crypto: 'none',
      isValid: false,
      isWrapped: util.u8aIsWrapped(input.message, true),
      publicKey
    };
    const isWrappedBytes = util.u8aIsWrapped(input.message, false);
    const verifyFn = getVerifyFn(signatureU8a);
    verifyFn(result, input);
    if (result.crypto !== 'none' || result.isWrapped && !isWrappedBytes) {
      return result;
    }
    input.message = isWrappedBytes ? util.u8aUnwrapBytes(input.message) : util.u8aWrapBytes(input.message);
    return verifyFn(result, input);
  }

  const P64_1 = BigInt$1('11400714785074694791');
  const P64_2 = BigInt$1('14029467366897019727');
  const P64_3 = BigInt$1('1609587929392839161');
  const P64_4 = BigInt$1('9650029242287828579');
  const P64_5 = BigInt$1('2870177450012600261');
  const U64 = BigInt$1('0xffffffffffffffff');
  const _7n = BigInt$1(7);
  const _11n = BigInt$1(11);
  const _12n = BigInt$1(12);
  const _16n = BigInt$1(16);
  const _18n = BigInt$1(18);
  const _23n = BigInt$1(23);
  const _27n = BigInt$1(27);
  const _29n = BigInt$1(29);
  const _31n = BigInt$1(31);
  const _32n = BigInt$1(32);
  const _33n = BigInt$1(33);
  const _64n = BigInt$1(64);
  const _256n = BigInt$1(256);
  function rotl(a, b) {
    const c = a & U64;
    return (c << b | c >> _64n - b) & U64;
  }
  function fromU8a(u8a, p, count) {
    const bigints = new Array(count);
    let offset = 0;
    for (let i = 0; i < count; i++, offset += 2) {
      bigints[i] = BigInt$1(u8a[p + offset] | u8a[p + 1 + offset] << 8);
    }
    let result = util._0n;
    for (let i = count - 1; i >= 0; i--) {
      result = (result << _16n) + bigints[i];
    }
    return result;
  }
  function toU8a(h64) {
    const result = new Uint8Array(8);
    for (let i = 7; i >= 0; i--) {
      result[i] = Number(h64 % _256n);
      h64 = h64 / _256n;
    }
    return result;
  }
  function state(initSeed) {
    const seed = BigInt$1(initSeed);
    return {
      seed,
      u8a: new Uint8Array(32),
      u8asize: 0,
      v1: seed + P64_1 + P64_2,
      v2: seed + P64_2,
      v3: seed,
      v4: seed - P64_1
    };
  }
  function init(state, input) {
    if (input.length < 32) {
      state.u8a.set(input);
      state.u8asize = input.length;
      return state;
    }
    const limit = input.length - 32;
    let p = 0;
    if (limit >= 0) {
      const adjustV = v => P64_1 * rotl(v + P64_2 * fromU8a(input, p, 4), _31n);
      do {
        state.v1 = adjustV(state.v1);
        p += 8;
        state.v2 = adjustV(state.v2);
        p += 8;
        state.v3 = adjustV(state.v3);
        p += 8;
        state.v4 = adjustV(state.v4);
        p += 8;
      } while (p <= limit);
    }
    if (p < input.length) {
      state.u8a.set(input.subarray(p, input.length));
      state.u8asize = input.length - p;
    }
    return state;
  }
  function xxhash64(input, initSeed) {
    const {
      seed,
      u8a,
      u8asize,
      v1,
      v2,
      v3,
      v4
    } = init(state(initSeed), input);
    let p = 0;
    let h64 = U64 & BigInt$1(input.length) + (input.length >= 32 ? ((((rotl(v1, util._1n) + rotl(v2, _7n) + rotl(v3, _12n) + rotl(v4, _18n) ^ P64_1 * rotl(v1 * P64_2, _31n)) * P64_1 + P64_4 ^ P64_1 * rotl(v2 * P64_2, _31n)) * P64_1 + P64_4 ^ P64_1 * rotl(v3 * P64_2, _31n)) * P64_1 + P64_4 ^ P64_1 * rotl(v4 * P64_2, _31n)) * P64_1 + P64_4 : seed + P64_5);
    while (p <= u8asize - 8) {
      h64 = U64 & P64_4 + P64_1 * rotl(h64 ^ P64_1 * rotl(P64_2 * fromU8a(u8a, p, 4), _31n), _27n);
      p += 8;
    }
    if (p + 4 <= u8asize) {
      h64 = U64 & P64_3 + P64_2 * rotl(h64 ^ P64_1 * fromU8a(u8a, p, 2), _23n);
      p += 4;
    }
    while (p < u8asize) {
      h64 = U64 & P64_1 * rotl(h64 ^ P64_5 * BigInt$1(u8a[p++]), _11n);
    }
    h64 = U64 & P64_2 * (h64 ^ h64 >> _33n);
    h64 = U64 & P64_3 * (h64 ^ h64 >> _29n);
    return toU8a(U64 & (h64 ^ h64 >> _32n));
  }

  function xxhashAsU8a(data, bitLength = 64, onlyJs) {
    const rounds = Math.ceil(bitLength / 64);
    const u8a = util.u8aToU8a(data);
    if (!util.hasBigInt || !onlyJs && isReady()) {
      return twox(u8a, rounds);
    }
    const result = new Uint8Array(rounds * 8);
    for (let seed = 0; seed < rounds; seed++) {
      result.set(xxhash64(u8a, seed).reverse(), seed * 8);
    }
    return result;
  }
  const xxhashAsHex = createAsHex(xxhashAsU8a);

  exports.addressEq = addressEq;
  exports.addressToEvm = addressToEvm;
  exports.allNetworks = allNetworks;
  exports.availableNetworks = availableNetworks;
  exports.base32Decode = base32Decode;
  exports.base32Encode = base32Encode;
  exports.base32Validate = base32Validate;
  exports.base58Decode = base58Decode;
  exports.base58Encode = base58Encode;
  exports.base58Validate = base58Validate;
  exports.base64Decode = base64Decode;
  exports.base64Encode = base64Encode;
  exports.base64Pad = base64Pad;
  exports.base64Trim = base64Trim;
  exports.base64Validate = base64Validate;
  exports.blake2AsHex = blake2AsHex;
  exports.blake2AsU8a = blake2AsU8a;
  exports.checkAddress = checkAddress;
  exports.checkAddressChecksum = checkAddressChecksum;
  exports.convertPublicKeyToCurve25519 = convertPublicKeyToCurve25519;
  exports.convertSecretKeyToCurve25519 = convertSecretKeyToCurve25519;
  exports.createKeyDerived = createKeyDerived;
  exports.createKeyMulti = createKeyMulti;
  exports.cryptoIsReady = cryptoIsReady;
  exports.cryptoWaitReady = cryptoWaitReady;
  exports.decodeAddress = decodeAddress;
  exports.deriveAddress = deriveAddress;
  exports.ed25519DeriveHard = ed25519DeriveHard;
  exports.ed25519PairFromRandom = ed25519PairFromRandom;
  exports.ed25519PairFromSecret = ed25519PairFromSecret;
  exports.ed25519PairFromSeed = ed25519PairFromSeed;
  exports.ed25519PairFromString = ed25519PairFromString;
  exports.ed25519Sign = ed25519Sign;
  exports.ed25519Verify = ed25519Verify;
  exports.encodeAddress = encodeAddress;
  exports.encodeDerivedAddress = encodeDerivedAddress;
  exports.encodeMultiAddress = encodeMultiAddress;
  exports.ethereumEncode = ethereumEncode;
  exports.evmToAddress = evmToAddress;
  exports.hdEthereum = hdEthereum;
  exports.hdLedger = hdLedger;
  exports.hdValidatePath = hdValidatePath;
  exports.hmacSha256AsU8a = hmacSha256AsU8a;
  exports.hmacSha512AsU8a = hmacSha512AsU8a;
  exports.hmacShaAsU8a = hmacShaAsU8a;
  exports.isAddress = isAddress;
  exports.isBase32 = isBase32;
  exports.isBase58 = isBase58;
  exports.isBase64 = isBase64;
  exports.isEthereumAddress = isEthereumAddress;
  exports.isEthereumChecksum = isEthereumChecksum;
  exports.jsonDecrypt = jsonDecrypt;
  exports.jsonDecryptData = jsonDecryptData;
  exports.jsonEncrypt = jsonEncrypt;
  exports.jsonEncryptFormat = jsonEncryptFormat;
  exports.keccak256AsU8a = keccak256AsU8a;
  exports.keccak512AsU8a = keccak512AsU8a;
  exports.keccakAsHex = keccakAsHex;
  exports.keccakAsU8a = keccakAsU8a;
  exports.keyExtractPath = keyExtractPath;
  exports.keyExtractSuri = keyExtractSuri;
  exports.keyFromPath = keyFromPath;
  exports.keyHdkdEcdsa = keyHdkdEcdsa;
  exports.keyHdkdEd25519 = keyHdkdEd25519;
  exports.keyHdkdSr25519 = keyHdkdSr25519;
  exports.mnemonicGenerate = mnemonicGenerate;
  exports.mnemonicToEntropy = mnemonicToEntropy;
  exports.mnemonicToLegacySeed = mnemonicToLegacySeed;
  exports.mnemonicToMiniSecret = mnemonicToMiniSecret;
  exports.mnemonicValidate = mnemonicValidate;
  exports.naclBoxPairFromSecret = naclBoxPairFromSecret;
  exports.naclDecrypt = naclDecrypt;
  exports.naclEncrypt = naclEncrypt;
  exports.naclOpen = naclOpen;
  exports.naclSeal = naclSeal;
  exports.packageInfo = packageInfo;
  exports.pbkdf2Encode = pbkdf2Encode;
  exports.randomAsHex = randomAsHex;
  exports.randomAsNumber = randomAsNumber;
  exports.randomAsU8a = randomAsU8a;
  exports.scryptEncode = scryptEncode;
  exports.scryptFromU8a = scryptFromU8a;
  exports.scryptToU8a = scryptToU8a;
  exports.secp256k1Compress = secp256k1Compress;
  exports.secp256k1Expand = secp256k1Expand;
  exports.secp256k1PairFromSeed = secp256k1PairFromSeed;
  exports.secp256k1PrivateKeyTweakAdd = secp256k1PrivateKeyTweakAdd;
  exports.secp256k1Recover = secp256k1Recover;
  exports.secp256k1Sign = secp256k1Sign;
  exports.secp256k1Verify = secp256k1Verify;
  exports.selectableNetworks = selectableNetworks;
  exports.setSS58Format = setSS58Format;
  exports.sha256AsU8a = sha256AsU8a;
  exports.sha512AsU8a = sha512AsU8a;
  exports.shaAsU8a = shaAsU8a;
  exports.signatureVerify = signatureVerify;
  exports.sortAddresses = sortAddresses;
  exports.sr25519Agreement = sr25519Agreement;
  exports.sr25519DeriveHard = sr25519DeriveHard;
  exports.sr25519DerivePublic = sr25519DerivePublic;
  exports.sr25519DeriveSoft = sr25519DeriveSoft;
  exports.sr25519PairFromSeed = sr25519PairFromSeed;
  exports.sr25519Sign = sr25519Sign;
  exports.sr25519Verify = sr25519Verify;
  exports.sr25519VrfSign = sr25519VrfSign;
  exports.sr25519VrfVerify = sr25519VrfVerify;
  exports.validateAddress = validateAddress;
  exports.xxhashAsHex = xxhashAsHex;
  exports.xxhashAsU8a = xxhashAsU8a;

}));

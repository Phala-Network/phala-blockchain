const fs = require('fs');
const BN = require("bn.js");

function normalizeHex(str) {
    return str.startsWith('0x') ? str : '0x' + str;
}

function praseBn(str) {
    let s = str.replace(/,/g, '');
    return new BN(s);
}

function loadJson(path) {
    const data = fs.readFileSync(path, {'encoding': 'utf-8'});
    return JSON.parse(data);
}

module.exports = { normalizeHex, praseBn, loadJson };

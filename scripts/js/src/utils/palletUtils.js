const BN = require('bn.js');
const { Decimal } = require('decimal.js');

const { hexToU8a, stringToU8a, u8aToHex, u8aConcat } = require('@polkadot/util');
const { blake2AsU8a } = require('@polkadot/util-crypto');

const { FixedPointConverter } = require('./fixedUtils');
const fp = new FixedPointConverter();

const bn1e12 = new BN(10).pow(new BN(12));
const dec1 = new Decimal(1);
const dec1e12 = new Decimal(bn1e12.toString());

function balanceDecToBn(dec) {
    return new BN(dec.mul(dec1e12).round().toFixed());
}
function balanceBnToDec(bn) {
    return new Decimal(bn.toString()).div(dec1e12);
}

function poolSubAccount(api, pid, worker) {
    let preimage = api.createType('u64', pid).toU8a();
    preimage = u8aConcat(preimage, hexToU8a(worker));

    const hash = blake2AsU8a(preimage);
    const data = u8aConcat(stringToU8a("spm/"), hash);

    return api.createType('AccountId', u8aToHex(data));
}

function calculateReturnSlash(minerInfo, origStake) {
    const ve = fp.fromBits(minerInfo.ve);
    const v = fp.fromBits(minerInfo.v);

    const origStakeFixed = balanceBnToDec(origStake);

    const returnRate = Decimal.min(v.div(ve), dec1);
    const returned = returnRate.mul(origStakeFixed);
    const slashed = origStakeFixed.sub(returned);

    return [
        balanceDecToBn(returned),
        balanceDecToBn(slashed),
    ];
}

module.exports = {
    poolSubAccount,
    calculateReturnSlash,
    balanceDecToBn,
    balanceBnToDec,
};

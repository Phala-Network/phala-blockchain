const { FixedPointConverter } = require("./fixedUtils");
const { Decimal } = require('decimal.js');
const BN = require('bn.js');

const fpc = new FixedPointConverter();
function fp(bits) {
    return fpc.fromBits(bits).toFixed();
}
function bits(d) {
    return fpc.toBits(new Decimal(d));
}

function toHuman(data) {
    const convertedParams = {
        'phaRate': fp(data.phaRate),
        'rho': fp(data.rho),
        'budgetPerBlock': fp(data.budgetPerBlock),
        'vMax': fp(data.vMax),
        'costK': fp(data.costK),
        'costB': fp(data.costB),
        'slashRate': fp(data.slashRate),
        'treasuryRatio': fp(data.treasuryRatio),
        'heartbeatWindow': data.heartbeatWindow.toString(),
        'rigK': fp(data.rigK),
        'rigB': fp(data.rigB),
        're': fp(data.re),
        'k': fp(data.k),
        'kappa': fp(data.kappa),
    };
    return convertedParams;
}

function humanToTyped(api, data) {
    const convertedParams = {
        'phaRate': bits(data.phaRate),
        'rho': bits(data.rho),
        'budgetPerBlock': bits(data.budgetPerBlock),
        'vMax': bits(data.vMax),
        'costK': bits(data.costK),
        'costB': bits(data.costB),
        'slashRate': bits(data.slashRate),
        'treasuryRatio': bits(data.treasuryRatio),
        'heartbeatWindow': new BN(data.heartbeatWindow),
        'rigK': bits(data.rigK),
        'rigB': bits(data.rigB),
        're': bits(data.re),
        'k': bits(data.k),
        'kappa': bits(data.kappa),
    }
    return api.createType('TokenomicParams', convertedParams);
}

function createUpdateCall(api, data) {
    return api.tx.phalaMining.updateTokenomic(data)
}

async function readFromChain(api, hash) {
    const params = await (hash
        ? api.query.phalaMining.tokenomicParameters.at(hash)
        : api.query.phalaMining.tokenomicParameters()
    );
    return toHuman(params.unwrap());
}

module.exports = { toHuman, humanToTyped, createUpdateCall, readFromChain };

require('dotenv').config();

const { ApiPromise, WsProvider } = require('@polkadot/api');
const BN = require('bn.js');

const bn1e12 = new BN(10).pow(new BN(12));
const bn1e6 = new BN(10).pow(new BN(6));

const targets = [
    {user: '44y4iARLQwVnziYGHfv5TjUGcmiBgppskuuK2tQZvPwAsdd5', pid: 573, total: 998.7998, amount: 49.93999},
    {user: '3zuCPzHJKwjjrvAnaoMcQU5Wb8nzv1VAEbeeDbPhHvuCLyDJ', pid: 131, total: 3000, amount: 150},
    {user: '3zuCPzHJKwjjrvAnaoMcQU5Wb8nzv1VAEbeeDbPhHvuCLyDJ', pid: 131, total: 3000, amount: 150},
    {user: '3zuCPzHJKwjjrvAnaoMcQU5Wb8nzv1VAEbeeDbPhHvuCLyDJ', pid: 131, total: 13100, amount: 655},
    {user: '3zuCPzHJKwjjrvAnaoMcQU5Wb8nzv1VAEbeeDbPhHvuCLyDJ', pid: 131, total: 13100, amount: 655},
    {user: '46LN656z5cMrYpogSVmu5VZxjF6M62K9f1QEyBCwJzqTWyhn', pid: 260, total: 1030, amount: 51.5},
    {user: '45PRz5125928Uwmv2j6gBBv7QTH5e5p3vkkEgQsNmHmqTXQo', pid: 105, total: 3000, amount: 150},
    {user: '444nAQx3MpLB4R5mETyb4YzHvFLGH9YmTxvGGqqTaVgm4s6v', pid: 246, total: 3000, amount: 150},
    {user: '41KHJeNpHYXjNYtEkGghG243MAFHgpahvXN7iLeueebpGeCH', pid: 405, total: 2100, amount: 105},
    {user: '44XTrMhSJa7tPiTR3XSES1rfv5rgMrqdCsodYLonRcRtaWU5', pid: 241, total: 3243.9526, amount: 162.19763},
    {user: '42MYAwfFyS4RNAkLtSqVurgaTHda4EVcDVepvpByzMaDELob', pid: 241, total: 0.6999, amount: 0.034995},
    {user: '43keT8ekXi8PvTMmTuVcKBWJH35mzdMdfWwaHedaQTmRQr1f', pid: 241, total: 31, amount: 1.55},
    {user: '41GKTLcGwZUjTTCJjXywiXG2JxuysBd8gtarieibKE6ra83n', pid: 241, total: 1385.3475, amount: 69.267375},
    {user: '41GKTLcGwZUjTTCJjXywiXG2JxuysBd8gtarieibKE6ra83n', pid: 241, total: 4691.9999, amount: 234.599995},
    {user: '3zfoQB7RKnQ2TPPsacMCsVJGKjJCwKmkhujq8kAsqnxGyYwk', pid: 268, total: 3000, amount: 150},
    {user: '3zfoQB7RKnQ2TPPsacMCsVJGKjJCwKmkhujq8kAsqnxGyYwk', pid: 268, total: 3000, amount: 150},
    {user: '46EmLYZ8eZsErKxLngbUyjXVP8KrjwEwPHmsgTKLTjqtVS4r', pid: 267, total: 6222.5856, amount: 311.12928},
    {user: '43iLkuFsa5kZL2M71HcL2NAUGvv75HUZHAx6DNVFeQAR1Bsa', pid: 307, total: 999.9810018, amount: 49.9990501},
    {user: '44uyiTRJ2gEpHYcBzn3TjXhkFqdy3Bbbt742Gyx85mt2KfDg', pid: 307, total: 1939.602301, amount: 96.980115},
    {user: '44uyiTRJ2gEpHYcBzn3TjXhkFqdy3Bbbt742Gyx85mt2KfDg', pid: 307, total: 1060.340705, amount: 53.0170352},
    {user: '41tn9UvQzhSYgbLvuhH1GADX8bedjiaq2KP2U4jT7fBeCKdX', pid: 307, total: 2939.844147, amount: 146.992207},
    {user: '41ofhHnutERPGfcCKb3KkDLyYZn4ZDJUodU8hDJgnaB1iRR8', pid: 307, total: 9729.815148, amount: 486.490757},
    {user: '44WfV6XtUH4TAEs3rdRjfkWc7F8apCYRrse5kQq4A7UtsRgX', pid: 555, total: 0.5135289731, amount: 0.02567645},
    {user: '44WfV6XtUH4TAEs3rdRjfkWc7F8apCYRrse5kQq4A7UtsRgX', pid: 555, total: 1401.974971, amount: 70.0987486},
    {user: '44RUe6Nzgoz2Z8nriSVoRHjZCJWg5XecvcbhQMKZFbJpB9Cd', pid: 555, total: 229.4159673, amount: 11.4707984},
    {user: '42dQjecvLXqsve3sUWru4LoGp2Q8yXiqUxCTKonLSP4hkru6', pid: 627, total: 2000, amount: 100},
    {user: '42G54erGDgXLzTD2vRFn6ha67V2igkBiTjX2WR7CYAReptyH', pid: 155, total: 1489.7741, amount: 74.488705},
    {user: '41fN1jj674bfLsjmBNR2aYsspBy9g5cbd5hBp31k4XxAFbB2', pid: 155, total: 1301.293688, amount: 65.0646844},
    {user: '42VAzCCoCQi8RLJMxvYDAkPcdr4LYgbkKWWuTijERcK9Kt4o', pid: 87, total: 3200, amount: 160},
    {user: '42G54erGDgXLzTD2vRFn6ha67V2igkBiTjX2WR7CYAReptyH', pid: 153, total: 2842.8823, amount: 142.144115},
    {user: '45Gu17eihkUYJrAexZiVWhCESXAcKgNqfWaTLSpT2Avbg2dF', pid: 628, total: 1198.8998, amount: 59.94499},
    {user: '44h296FDYC6znCQkrzNd6cvMKZZyziP6ufcKvfjoFaU375DJ', pid: 628, total: 4000.1002, amount: 200.00501},
    {user: '45H6EGgC1V4hJrsGyNjNuipiPwiPhATBCmiXabAmZndsU8Ma', pid: 373, total: 3480, amount: 174},
    {user: '43yUpyFauPzNeXWstZ5ZDU77H8UcZZtG75u3W46ZYQ3PwGZ3', pid: 818, total: 1209.4488, amount: 60.47244},
    {user: '42kcaqTmNDQWV872LsP7Rk41BxERxBzvfPD3yGsGz1rVx4P2', pid: 818, total: 599, amount: 29.95},
    {user: '42qwxtUimLgKTKm5ZMTCCVrNyfPbwThZGrXBrtESGbcYPqrw', pid: 818, total: 241.5512, amount: 12.07756},
    {user: '42qwxtUimLgKTKm5ZMTCCVrNyfPbwThZGrXBrtESGbcYPqrw', pid: 818, total: 1808.4488, amount: 90.42244},
    {user: '43cHyUhveFABo6KVNh3jEf6dgWd5yvk3ostjgMhf1iLNeboZ', pid: 818, total: 741.551, amount: 37.07755},
    {user: '46BXiqakSyjjFLpnQ6yVY5akRzrVJnwn9gBunaCwoipa4s9U', pid: 824, total: 2050, amount: 102.5},
    {user: '42VKVU26hKSbts9d19mw4vsiZQTxgb6crMsAwWCVJJNepeXB', pid: 824, total: 2060, amount: 103},
    {user: '42VKVU26hKSbts9d19mw4vsiZQTxgb6crMsAwWCVJJNepeXB', pid: 824, total: 1480, amount: 74},
    {user: '44LxYowxgRRmJ6DzPe4Liv6dVF2UfCxYFQz9ZkuR3JVCkojh', pid: 824, total: 6460, amount: 323},
    {user: '44LxYowxgRRmJ6DzPe4Liv6dVF2UfCxYFQz9ZkuR3JVCkojh', pid: 824, total: 3540, amount: 177},
    {user: '41omLTbNMwxUCbC3QZAt1c5VoTgjoGqZG9qW2Mpf2VzFqUvu', pid: 824, total: 500, amount: 25},
];

// modlphala/pp
const PHALAPP = '436H4jat7TobTbNYLdSJ3cmNy9K4frmE4Yuc4R2nNnaf56DL';

function balanceFromReal(x) {
    return new BN(Math.round(x * 1e6)).mul(bn1e6)
}

async function main() {
    const wsProvider = new WsProvider(process.env.ENDPOINT);
    const api = await ApiPromise.create({ provider: wsProvider });

    const merged = {};
    for (const {user, amount} of targets) {
        if (!merged[user]) {
            merged[user] = 0;
        }
        merged[user] += amount;
    }

    const tx = api.tx.utility.batchAll(
        Object.entries(merged).map(([account, amount]) =>
            api.tx.balances.forceTransfer(PHALAPP, account, balanceFromReal(amount))
        )
    );

    // get raw encoded proposal for notePreimage()
    console.log(tx.method.toHex());
}

main().catch(console.error).finally(() => process.exit());

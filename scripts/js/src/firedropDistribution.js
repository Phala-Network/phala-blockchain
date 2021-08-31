require('dotenv').config();

const { ApiPromise, Keyring, WsProvider } = require('@polkadot/api');
const BN = require('bn.js');

const typedefs = require('@phala/typedefs').phalaDev;
const bn1e12 = new BN(10).pow(new BN(12));

const poc3Prize = [{
    // Assume amounts are integer
    data: [
        ['44AHCQtUiEzf1aE9QDQKP6wLAERhDpYdLTLsDNxnqUC4gK4G', 8499],
        ['41dfUfy5JexS1L9CAf1gHmKws8GHbXBDXeJvpfdpVxRvA894', 98546],
        ['44Uxw91hz88zLytWwWwTbH1c1SvTset4cvAwxree2oVdAyQk', 27692],
        ['45FKTTyMTVqGFysvv2JtDRodWMNuYWbwARnyjkD6bZXTuJn5', 51688],
        ['46EzvipMtzQ7z6YscCXcZjhobaLLxjuicookgf78keSrBfMY', 15851],
        ['42t19Y26GGZbZyp4FPcJKEdKmyZt7kYdfoovf6DFjgB1BVKi', 15716],
        ['41UoKhSWFt2ShPYvbphs9RaGynGU2Nxbij2kgAc673B4p2X8', 107010],
        ['43qMqcraek34k1Ass6xTJhySBSuD76LxHHwUQLDAdumLdunZ', 53371],
        ['444reqUqZst1dPxJP8e3ACNaXR41CdViiWWtEkzu8eUk4wPx', 26061],
        ['43t4PmV9GqCcG72ns5A76bLU4524w4uxiNUaV6oveKUWvJbG', 186892],
        ['44jhFBS1H9xmYW7qPKZMVREa8KnQqPey5N5Ec9PmKTHRKGo7', 12802],
        ['43UKqmgoGJzJctvAhNbnUaf5J8D7CzrgXB9oxdTFrfUKJqLm', 72876],
        ['44khVN5Gs4VgRh33189seEqwQoMvA74CiG1Vt6qAEYKM7jdd', 10196],
        ['45RaHh6wCR1uU1DN2P2LK76hYASuMY9HKvsKsW4NhM58thPm', 32800],
    ],
    context: {
        "blockNum": 110277,
        "blockHash": "0x50294f31df05190e696241e786c21476e91772847de485b58644fcfefdea880e"
    }
}, {
    // Assume amounts are integer
    data: [
        ["42wm1sm39gs3ngTGxapLaMUvFQHtjERo7JD6CkZ2VL6KQrpK",38129],
        ["42qurrwZ53YaP3vtpSZNCrJADDjaUWBf8MiGcA9jQQuUsmcn",12928],
        ["44i4Dpm79kLPyErbLhy1bhnudBNmQ2brVug6xkjythBYkv1c",20909],
        ["43k8j5AX9sKmEL195mrCkqb4oPfbpX5VixF9M4MpbCAkUN97",33634],
        ["44HhYTW3AJF6ZqNG8zgYsF4z8HTUKdxZQ59VxBb68q23q6kk",1263],
        ["44TtxmTstgf1ns1hBAUBne6uUDueKFY27RJd44A9Kd8G1UE7",45626],
        ["45mukZ3RhgwL9ZANmawBu6izEa2nnDfLhSsSjXiRsqcBB3ow",4330],
        ["44FAD7fevqwf8YUx97fuCKGPbuACZS4xr2icZz1E8CxyBStN",7300],
        ["441bRVk56V1u5vBG56BkQFx1nwJRCeh3GxoRQoGYuEgNp2wy",25373],
        ["43jnxA4HZPNeU1yRRBucGyNzJgwCkmTNLSeh1Cep5zAwQjXa",21482],
        ["41AbqJUke2zgtGE6ftY6NfV6jzEEv9PxVu2G9MS8GpTuxvP4",40671],
        ["43iF6zUPjHRghpfNXNdq4iwUuBP8NL5VdLHFVdhnBstFEvdB",34519],
        ["41SxLZymSn5tUaRMPbFDrDuYFads13Y4VkAhXxJyat3PL1Hg",9251],
        ["45qPYPFLytVanq5ThhSEJTY5tPrA7FqfNdtr3YCV7mXkVtSH",10953],
        ["459BZcn6oTQJfK4pBPFynmrnJRv2rt4V5XGKemeGbjeG5ppS",13589],
        ["46KSjF38thvjd1VLt2io3LS4EicReyX52vmLkcgkB9WccNH7",18856],
        ["459C4KLMzm8ZyCjxRynSgVNEoyR8UyNGg5WKjjyc7gPJxSTs",16109],
        ["43P1V2MGomDrGbpR7q7gUguqdkT9wxWmKRSM7kK42Gv54Xvh",12932],
        ["3zsAjyMhtLzsonWNdCBgq7bLtZeinDw8VDQJrJBNqWQhdYAb",16719],
        ["42FkcGa4aANzBPAG3Hz8NpNGqjVxuDiV72JHpqi5jeYqGKv3",15573],
        ["45C6u5LDk21EG1fhQ2HxtadmUsRdux7fy9G9VokrqE2CofA2",3366],
        ["3zdR8kg51TRS9LezkvV1dAUYs6yS9CwGRMvkJ45ZJEaHhMw2",12955],
        ["45UdKNgExq9FH1Nwq3JZerwfhhM655Yra8Z48taEJnW7cUop",15842],
        ["44qTDccE9oN1ek6LTr6Td8ZHLtstLaBgGw5g9w99wmPUxpNq",409],
        ["45xJ52kthVuAFTGrDbAySrm3iagnxZaDAktF8xHPMrqMCNhV",7690],
        ["46MsBwFUD46wLgP8Si1TjS4kruDoQyFhFfTawQwCwNEbCXYF",20972],
        ["45uquvkE2ij8ERA83HwNoxqh793mJcvXN2kqbaYHv1UpZAuh",80778],
        ["44Ew2kE76R2E7B4e5QMGhE4FZ7FVtSkTx222UKXThteqMWpS",41101],
        ["41ofhHnutERPGfcCKb3KkDLyYZn4ZDJUodU8hDJgnaB1iRR8",56162],
        ["41oHFx9DQ2nJzEAZtaqGEjffnUktPzgsGsrnsKV6CLu7x1ZD",13220],
        ["41mJagM7FgBhMk8vrojEbC8hEk9Ngoy1zwANMVG29Xs2rLEG",25713],
        ["45RgbbUak7NbJwPaZ71xzAXoteBrE32MdwdfaeQP5UNrMa9u",28408],
        ["4449QTQYK9BzuaDWYFte4LnMTZjtg6H7dcrReteXzhToX2nd",13238],
    ],
    context: {
        "blockNum": 252600,
        "blockHash": "0x7d98d0a541d522125967ed88828899ec6a10fd936be016ed1292866c6926cabd"
    }
}];

async function getPayoutAddress (api, hash, stash) {
    const stashInfo = await api.query.phala.stashState.at(hash, stash);
    return stashInfo.payoutPrefs.target;
}

async function main () {
    const wsProvider = new WsProvider(process.env.ENDPOINT);
    const api = await ApiPromise.create({ provider: wsProvider, types: typedefs });

    const keyring = new Keyring({ type: 'sr25519' });
    const root = keyring.addFromUri(process.env.PRIVKEY);

    const {data, context} = poc3Prize[poc3Prize.length - 1];

    const argAddrs = [];
    const argAmounts = [];
    for (let [addr, amount] of data) {
        const payoutAddr = await getPayoutAddress(api, context.blockHash, addr);
        argAddrs.push(payoutAddr);
        argAmounts.push(new BN(amount).mul(bn1e12));
    }

    console.log('payouts', argAddrs.map((addr, i) => ({addr: addr.toHuman(), amount: data[i][1]})));

    await new Promise(async resolve => {
        const unsub = await api.tx.sudo.sudo(api.tx.phala.forceAddFire(argAddrs, argAmounts))
            .signAndSend(root, (result) => {
                console.log(`Current status is ${result.status}`);
                if (result.status.isInBlock) {
                    console.log(`Transaction included at blockHash ${result.status.asInBlock}`);
                } else if (result.status.isFinalized) {
                    console.log(`Transaction finalized at blockHash ${result.status.asFinalized}`);
                    unsub();
                    resolve();
                }
            });
    });
}

main().catch(console.error).finally(() => process.exit());

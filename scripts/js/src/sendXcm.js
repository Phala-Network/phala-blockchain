require('dotenv').config();

const { ApiPromise, Keyring, WsProvider } = require('@polkadot/api');
const BN = require('bn.js');

const typedefs = require('@phala/typedefs').phalaDev;
const bn1e12 = new BN(10).pow(new BN(12));

function createTransferXcm(
    api, fromLocation='PHA30', toParaId=100, amount=100,
    toAddr='45R2pfjQUW2s9PQRHU48HQKLKHVMaDja7N3wpBtmF28UYDs2'
) {
    const bnAmount = bn1e12.muln(amount);
    return api.createType('Xcm', {
        WithdrawAsset: api.createType('WithdrawAsset', {
            assets: [
                api.createType('MultiAsset', {
                    ConcreteFungible: api.createType('ConcreteFungible', {
                        id: api.createType('MultiLocation', {
                            X1: api.createType('Junction', {
                                'GeneralKey': fromLocation
                            })
                        }),
                        amount: api.createType('Compact<U128>', bnAmount),
                    })
                }),
            ],
            effects: [
                api.createType('Order', {
                    DepositReserveAsset: api.createType('DepositReserveAsset', {
                        assets: [api.createType('MultiAsset', 'All')],
                        dest: api.createType('MultiLocation', {
                            X2: [
                                api.createType('Junction', 'Parent'),
                                api.createType('Junction', {
                                    Parachain: api.createType('Compact<U32>', toParaId),
                                }),
                            ]
                        }),
                        effects: [
                            api.createType('Order', {
                                DepositAsset: api.createType('DepositAsset', {
                                    assets: [
                                        api.createType('MultiAsset', 'All'),
                                    ],
                                    dest: api.createType('MultiLocation', {
                                        X1: api.createType('Junction', {
                                            AccountId32: api.createType('AccountId32Junction', {
                                                network: api.createType('NetworkId', 'Any'),
                                                id: toAddr,
                                            })
                                        })
                                    })
                                })
                            }),
                        ]
                    })
                })
            ],
        })
    });
}

async function main () {
    const wsProvider = new WsProvider(process.env.ENDPOINT);
    const api = await ApiPromise.create({ provider: wsProvider, types: typedefs });

    const keyring = new Keyring({ type: 'sr25519' });
    const sender = keyring.addFromUri('//Alice');

    await new Promise(async resolve => {
        const unsub = await api.tx.xcmHandler.executeXcm(createTransferXcm(api))
        .signAndSend(sender, (result) => {
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

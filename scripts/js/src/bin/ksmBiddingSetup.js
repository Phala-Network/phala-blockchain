const { ApiPromise, WsProvider } = require('@polkadot/api');
const {
    createKeyMulti,
    encodeAddress,
    // sortAddresses
} = require('@polkadot/util-crypto');

const { program } = require('commander');

program
    // .option('--set-alice-as-root', 'Set //Alice as the root account.', false)
    // .argument('<orig-spec>', 'The original chain spec to modify.')
    .action(main)
    .parse(process.argv);


async function main(origSpecPath) {
    const opts = program.opts();

    const phala = 'GFLdqBZKfPfbpbVB8rAc8tqqWSKpKHskkGHPGAgQ4atRkJ7';
    const alistiar = 'H4jWXq8yoQB78e9uCUdKuao217s4YCmkB41yStq667r2Uet';
    const judge = 'H74B8TPXg3eMiFv25h4HAh3ofopnYyQhHQL6YnAYvDmc4t5';

    const wsProvider = new WsProvider('wss://kusama-rpc.polkadot.io');
    const api = await ApiPromise.create({ provider: wsProvider });

    const newProxy = api.tx.proxy.anonymous(
        'Any',    // proxyType
        0,        // delay
        0,        // index
    ).toHex()

    const funding = 'HXYacFPwGfZn2n41sGQ6v9tbxx2NXL1vF5YoRJwVLL1aEPy'

    const multisig = encodeAddress(
        createKeyMulti(
            [phala, alistiar, judge],  // accounts
            2,                        // threshold
        ),
        2,    // Kusama's SS58 Prefix
    );

    console.log({multisig});

    const setup = api.tx.proxy.proxy(
        funding,    // realAccount
        null,       // forceProxyType
        api.tx.utility.batch([
            api.tx.proxy.addProxy(
                multisig,    // delegate
                'Any',       // proxyType
                0,           // delay
            ),
            api.tx.proxy.addProxy(
                phala,       // delegate
                'Auction',   // proxyType
                0,           // delay
            ),
            // api.tx.proxy.removeProxy(
            //     phala,       // delegate
            //     'Any',       // proxyType
            //     0,           // delay
            // ),
        ])
    ).toHex()
    console.log({setup})

    const newPara = 2111;
    const proxiedTransferCall = api.tx.proxy.proxy(
        funding,
        null,
        api.tx.auctions.bid(
            newPara, // para
            27,     // auctionIndex
            20,      // firstSlot
            37,      // lastSlot
        )
    );
    proxiedTransferCall.call.toHex()
}

require('dotenv').config();

const { ApiPromise, WsProvider } = require('@polkadot/api');
const BN = require('bn.js');

const typedefs = require('@phala/typedefs').phalaDev;

const bn64b = new BN(2).pow(new BN(64));
const bn1e12 = new BN(10).pow(new BN(12));

const src = '5H1cbKvXJNjxCwU2czeNgVgSKCmFAeAi9Tg1fwtW2mFqGgLP';
const batch1 = [
    ['44jXiW8Ci4EteNyAsy7XfVdGgM49prBjdQXYpeATgicfrDX7', 100],
    ['13bKVPpEhCh6EymusGvwh1Qvfo5Nfnso8xnznrGfLTRqW3w8', 100],
    ['41ASbiCHTsg36ZmrYwNNFRz8LQQyesML3oyyFGJPwxGwcGtP', 100],
    ['41fnZ5ee8E9JgVDi8hGzSSWF7CyPGKGrJJAog2RcNZYL24hA', 100],
    ['44pyz2jFQhn2WEBi199rVvL2Ddums7VzeJKhUFYwKxeQkVeD', 100],
    ['1ncqoz7X1FzdsuoYRT8qabUoYCRM8CMJp88pFC4tvjweSm8', 100],
    ['5FW1h5RbVNA76Xew4g77uyybDoMBZszQPiWFd8rVTzEALXTF', 100],
    ['142zWpyz338ZV912tzMZay3PuAb8QWJ3c1kXbNz4AKWpUUgr', 100],
    ['5EL3n2eQh5BHeVpxSefq8PRR8LecKFKzKkxBjqQFV3TR5ZUd', 100],
    ['43iLhVJsR6kUqFQgujMnmKmEbY835WmhVst22mR2BDpXtsPp', 100],
    ['EwPaaFJcPjsdVtGYV96oM7jMH8jQaWAszsuZCg8aJTp7grZ', 100],
    ['43iLkuFsa5kZL2M71HcL2NAUGvv75HUZHAx6DNVFeQAR1Bsa', 100],
    ['16XLPwp8VXJAso85Uf5NLAKjfv12U32F8qbEw868MJ3rLPnS', 100],
];

const batch2 = [
    ['44jXiW8Ci4EteNyAsy7XfVdGgM49prBjdQXYpeATgicfrDX7', 937400],
    ['13bKVPpEhCh6EymusGvwh1Qvfo5Nfnso8xnznrGfLTRqW3w8', 1562400],
    ['41ASbiCHTsg36ZmrYwNNFRz8LQQyesML3oyyFGJPwxGwcGtP', 937400],
    ['41fnZ5ee8E9JgVDi8hGzSSWF7CyPGKGrJJAog2RcNZYL24hA', 312400],
    ['44pyz2jFQhn2WEBi199rVvL2Ddums7VzeJKhUFYwKxeQkVeD', 1785614],
    ['1ncqoz7X1FzdsuoYRT8qabUoYCRM8CMJp88pFC4tvjweSm8', 312400],
    ['5FW1h5RbVNA76Xew4g77uyybDoMBZszQPiWFd8rVTzEALXTF', 312400],
    ['5EL3n2eQh5BHeVpxSefq8PRR8LecKFKzKkxBjqQFV3TR5ZUd', 20312400],
    ['43iLhVJsR6kUqFQgujMnmKmEbY835WmhVst22mR2BDpXtsPp', 749900],
    ['EwPaaFJcPjsdVtGYV96oM7jMH8jQaWAszsuZCg8aJTp7grZ', 87496],
    ['43iLkuFsa5kZL2M71HcL2NAUGvv75HUZHAx6DNVFeQAR1Bsa', 272122],
    ['16XLPwp8VXJAso85Uf5NLAKjfv12U32F8qbEw868MJ3rLPnS', 2104368],
]

async function main() {
    const wsProvider = new WsProvider(process.env.ENDPOINT);
    const api = await ApiPromise.create({ provider: wsProvider, types: typedefs });

    const tx = api.tx.sudo.sudo(
        api.tx.utility.batchAll(
            batch2.map(([addr, amount]) =>
                api.tx.balances.forceTransfer(src, addr, new BN(amount).mul(bn1e12))
            )
        )
    );

    console.log(tx.toHex());
}

main().catch(console.error).finally(() => process.exit());

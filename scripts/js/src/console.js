require('dotenv').config();

const { program } = require('commander');
const axios = require('axios').default;
const { Decimal } = require('decimal.js');
const { ApiPromise, Keyring, WsProvider } = require('@polkadot/api');
const { cryptoWaitReady } = require('@polkadot/util-crypto');
const phalaTypes = require('@phala/typedefs').khalaDev;

const { FixedPointConverter } = require('./utils/fixedUtils');
const tokenomic  = require('./utils/tokenomic');
const { normalizeHex, praseBn, loadJson } = require('./utils/common');
const { pruntime_rpc } = require('./proto/pruntime_rpc');

function run(afn) {
    function runner(...args) {
        afn(...args)
            .catch(console.error)
            .then(process.exit)
            .finally(() => process.exit(-1));
    };
    return runner;
}

class PRuntimeApi {
    constructor(endpoint) {
        this.api = axios.create({
            baseURL: endpoint,
            headers: {
                'Content-Type': 'application/octet-stream',
            },
            baseURL: endpoint,
            responseType: 'arraybuffer',
        });
    }
    async req(method, data = undefined) {
        const r = await this.api.post('/prpc/' + method, data);
        return pruntime_rpc.PhactoryInfo.decode(r.data);
    }
    async query(_contractId, _request) {
        throw new Error('Unimplemented');
    }

    async getInfo() {
        return await this.req('PhactoryAPI.GetInfo');
    }
}

function parseXUS(assets) {
    const m = assets.match(/(\d+(\.\d*)?) XUS/);
    if (!m) {
        throw new Error(`Couldn't parse asset ${assets}`);
    }
    return (parseFloat(m[1]) * 1e6) | 0;
}

function pruntimeApi() {
    const { pruntimeEndpoint } = program.opts();
    return new PRuntimeApi(pruntimeEndpoint);
}

async function substrateApi() {
    const { substrateWsEndpoint } = program.opts();
    const wsProvider = new WsProvider(substrateWsEndpoint);
    const api = await ApiPromise.create({ provider: wsProvider, types: phalaTypes });
    return api;
}

function printObject(obj, depth=3, getter=true) {
    if (program.opts().json) {
        console.log(JSON.stringify(obj, undefined, 2));
    } else {
        console.dir(obj, {depth, getter});
    }
}

const CONTRACT_PDIEM = 5;

program
    .option('--pruntime-endpoint <url>', 'pRuntime API endpoint', process.env.PRUNTIME_ENDPOINT || 'http://localhost:8000')
    .option('--substrate-ws-endpoint <url>', 'Substrate WS endpoint', process.env.ENDPOINT || 'ws://localhost:9944')
    .option('--json', 'output regular json', false);

// Blockchain operations
const chain = program
    .command('chain')
    .description('blockchain actions');

chain
    .command('push-command')
    .description('push a unencrypted command to a confidential contract')
    .argument('<contract-id>', 'confidential contract id (number)')
    .argument('<plain-command>', 'the plain command payload (string or json, depending on the definition)')
    .option('-s, --suri <suri>', 'specify sender\'s privkey', process.env.PRIVKEY || '//Alice')
    .action(run(async (contractId, plainCommand, options) => {
        const api = await substrateApi();
        const cid = parseInt(contractId);
        const command = JSON.parse(plainCommand);
        const keyring = new Keyring({ type: 'sr25519' });
        const pair = keyring.addFromUri(options.suri);
        const r = await api.tx.phala.pushCommand(
            cid,
            JSON.stringify({
                Plain: JSON.stringify(command)
            })
        ).signAndSend(pair);
        console.log(r.toHuman());
    }));

chain
    .command('sync-state')
    .description('show the chain status; returns 0 if it\'s in sync')
    .action(run(async () => {
        const api = await substrateApi();
        const hash = await api.rpc.chain.getBlockHash();
        const header = await api.rpc.chain.getHeader(hash);
        const syncState = await api.rpc.system.syncState();
        const tsObj = await api.query.timestamp.now.at(hash);
        const blockTs = tsObj.toNumber();
        const now = Date.now();

        const timestampDelta = now - blockTs;

        printObject({
            hash: hash.toJSON(),
            blockTs,
            timestampDelta,
            syncState: syncState.toJSON(),
            header: header.toJSON(),
        });

        // Return -1 if it's not in sync (delta > 5mins)
        return timestampDelta <= 50 * 60 * 1000 ? 0 : -1;
    }));

chain
    .command('free-balance')
    .description('get the firee blance of an account')
    .argument('<account>', 'the account to lookup')
    .action(run (async (account) => {
        const api = await substrateApi();
        const accountData = await api.query.system.account(account);
        const freeBalance = accountData.data.free.toString();
        console.log(freeBalance);
        return 0;
    }));

chain
    .command('inspect-worker')
    .description('get the mining related info with the worker public key')
    .argument('<worker-key>', 'the worker public key in hex')
    .action(run (async (workerKey) => {
        workerKey = normalizeHex(workerKey);

        const api = await substrateApi();
        let [workerInfo, miner, pid] = await Promise.all([
            api.query.phalaRegistry.workers(workerKey),
            api.query.phalaMining.workerBindings(workerKey),
            api.query.phalaStakePool.workerAssignments(workerKey),
        ]);
        workerInfo = workerInfo.unwrapOr();
        miner = miner.unwrapOr();
        pid = pid.unwrapOr();

        const minerInfo = miner ? await api.query.phalaMining.miners(miner) : undefined;
        const poolInfo = pid ? await api.query.phalaStakePool.stakePools(pid) : undefined;

        const toObj = x => x ? (x.unwrapOr ? x.unwrapOr(undefined) : x).toJSON() : undefined;
        printObject({
            workerInfo: toObj(workerInfo),
            miner: toObj(miner),
            pid: toObj(pid),
            minerInfo: toObj(minerInfo),
            poolInfo: toObj(poolInfo),
        });
    }));

chain
    .command('get-tokenomic')
    .description('read the tokenomic parameters from the blockchain')
    .action(run(async () => {
        const api = await substrateApi();
        const p = await tokenomic.readFromChain(api);
        printObject(p);
    }));

chain
    .command('update-tokenomic')
    .argument('<json>', 'tokenomic parameter json file path')
    .description('create a call to update tokenomic parameters and optionally send a sudo tx')
    .option('-s, --suri <suri>', 'specify sender\'s privkey', process.env.PRIVKEY || '//Alice')
    .action(run(async (path, options) => {
        const p = loadJson(path);
        const api = await substrateApi();
        const typedP = tokenomic.humanToTyped(api, p);
        const call = tokenomic.createUpdateCall(api, typedP);
        console.log(call.toHex());

        if (options.suri) {
            const keyring = new Keyring({ type: 'sr25519' });
            const pair = keyring.addFromUri(options.suri);
            let result = await api.tx.sudo.sudo(call).signAndSend(pair, {nonce: -1});
            console.log('Transaction result', result.toJSON());
        }
    }));

// pRuntime operations
const pruntime = program
    .command('pruntime')
    .description('pRuntime commands');

pruntime
    .command('get-info')
    .description('get the running status')
    .action(run(async () => {
        const pr = pruntimeApi();
        printObject(await pr.getInfo());
    }));

pruntime
    .command('req')
    .description('send a request to pruntime')
    .argument('<method>', 'the method name')
    .option('--body', 'a json request body', '')
    .action(run(async (method, opt) => {
        console.warn('This feature could be broken.');
        const pr = pruntimeApi();
        let body;
        if (opt.body) {
            body = JSON.parse(opt.body);
        }
        printObject(await pr.req(method, body ? body : {}));
    }))

pruntime
    .command('query')
    .description('send a query to a confidential contract via pRuntime directly (anonymously)')
    .argument('<contract-id>', 'confidential contract id (number)')
    .argument('<plain-query>', 'the plain query payload (string or json, depending on the definition)')
    .action(run(async (contractId, plainQuery) => {
        console.warn('This feature could be broken.');
        const pr = pruntimeApi();
        const cid = parseInt(contractId);
        const plainQueryObj = JSON.parse(plainQuery);
        const r = await pr.query(cid, plainQueryObj);
        console.dir(r, {depth: 3});
    }))

// pDiem related
const pdiem = program
    .command('pdiem')
    .description('pDiem commands');

pdiem
    .command('balances')
    .description('get a list of the account info and balances')
    .action(run(async () => {
        const pr = pruntimeApi();
        console.dir(await pr.query(CONTRACT_PDIEM, 'AccountData'), {depth: 3});
    }));

pdiem
    .command('tx')
    .description('get a list of the verified transactions')
    .action(run(async () => {
        const pr = pruntimeApi();
        console.dir(await pr.query(CONTRACT_PDIEM, 'VerifiedTransactions'), {depth: 3});
    }));

pdiem
    .command('new-account')
    .description('create a new diem subaccount for deposit')
    .argument('<seq>', 'the sequence id of the VASP account')
    .argument('<suri>', 'the SURI of the sender Substrate account (sr25519)')
    .action(run(async (seq, suri) => {
        const api = await substrateApi();
        const seqNumber = parseInt(seq);
        const keyring = new Keyring({ type: 'sr25519' });
        const pair = keyring.addFromUri(suri);
        const r = await api.tx.phala.pushCommand(
            CONTRACT_PDIEM,
            JSON.stringify({
                Plain: JSON.stringify({
                    NewAccount: {
                        seq_number: seqNumber
                    }
                })
            })

        ).signAndSend(pair);
        console.log(r.toHuman());
    }));

pdiem
    .command('withdraw')
    .description('create a new diem subaccount for deposit')
    .argument('<dest>', 'the withdrawal destination Diem account')
    .argument('<amount>', 'the sequence id of the VASP account')
    .argument('<suri>', 'the SURI of the sender Substrate account (sr25519)')
    .action(run(async (dest, amount, suri) => {
        if (dest.toLowerCase().startsWith('0x')) {
            throw new Error('<dest> must not start with "0x"');
        }
        const xusAmount = parseXUS(amount);
        const api = await substrateApi();
        const keyring = new Keyring({ type: 'sr25519' });
        const pair = keyring.addFromUri(suri);

        const r = await api.tx.phala.pushCommand(
            CONTRACT_PDIEM,
            JSON.stringify({
                Plain: JSON.stringify({
                    TransferXUS: {
                        to: dest,
                        amount: xusAmount,
                    }
                })
            })
        ).signAndSend(pair);
        console.log(r.toHuman());
    }));

// Utilities

const utils = program
    .command('utils')
    .description('utility functions');


utils
    .command('verify')
    .description('verify some inputs (ss58 address or suri). (return 0 if it\'s valid or else -1)')
    .argument('<input>', 'the raw input data')
    .action(run(async (input) => {
        input = input.trim();
        const keyring = new Keyring({ type: 'sr25519', ss58Format: 30 });
        try {
            const decoded = keyring.decodeAddress(input);
            if (decoded) {
                const address = keyring.encodeAddress(decoded);
                console.log(address);
                return 0;
            }
        } catch {}
        try {
            await cryptoWaitReady();
            const pair = keyring.addFromUri(input);
            if (pair) {
                console.log(pair.address);
                return 0;
            }
        } catch {}
        console.log('Cannot decode the input');
        return -1;
    }));

utils
    .command('fp-encode')
    .description('encode a (U64F64) fixed point to bits')
    .argument('<fp>', 'fixed point to encode')
    .action(fp => {
        const decFp = new Decimal(fp);
        const fpc = new FixedPointConverter();
        console.log(fpc.toBits(decFp).toString());
    });

utils
    .command('fp-decode')
    .description('decode a (U64F64) fixed point bits to the number')
    .argument('<bits>', 'bits to decode')
    .action(bits => {
        const bnBits = praseBn(bits);
        const fpc = new FixedPointConverter();
        console.log(fpc.fromBits(bnBits).toString());
    });

program.parse(process.argv);

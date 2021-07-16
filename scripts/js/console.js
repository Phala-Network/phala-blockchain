require('dotenv').config();

const { program } = require('commander');
const axios = require('axios').default;
const { ApiPromise, Keyring, WsProvider } = require('@polkadot/api');
const { cryptoWaitReady } = require('@polkadot/util-crypto');
const phalaTypes = require('@phala/typedefs').latest;

function run(afn) {
    function runner(...args) {
        afn(...args)
            .catch(console.error)
            .then(process.exit)
            .finally(() => process.exit(-1));
    };
    return runner;
}

function rand() {
    return (Math.random() * 65536) | 0;
}

class PRuntimeApi {
    constructor(endpoint) {
        this.api = axios.create({
            baseURL: endpoint,
            headers: {
                'Content-Type': 'application/json',
            }
        });
    }
    async req(method, data={}) {
        const r = await this.api.post('/' + method, {
            input: data,
            nonce: { id: rand() }
        });
        if (r.data.status === 'ok') {
            return JSON.parse(r.data.payload);
        } else {
            throw new Error(`Got error response: ${r.data}`);
        }
    }
    async query(contractId, request) {
        const bodyJson = JSON.stringify({
            contract_id: contractId,
            nonce: rand(),
            request
        });
        const payloadJson = JSON.stringify({Plain: bodyJson});
        const queryData = {query_payload: payloadJson};
        const response = await this.req('query', queryData);
        const plainResp = JSON.parse(response.Plain);
        return plainResp;
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

const CONTRACT_PDIEM = 5;

program
    .option('--pruntime-endpoint <url>', 'pRuntime API endpoint', process.env.PRUNTIME_ENDPOINT || 'http://localhost:8000')
    .option('--substrate-ws-endpoint <url>', 'Substrate WS endpoint', process.env.ENDPOINT || 'ws://localhost:9944');

// Blockchain operations

program
    .command('push-command <contract-id> <plain-command>')
    .description('push a unencrypted command to a confidential contract', {
        'contract-id': 'confidential contract id (number)',
        'plain-command': 'the plain command payload (string or json, depending on the definition)',
    })
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

// pRuntime operations

program
    .command('get-info')
    .description('get the running status')
    .action(run(async () => {
        const pr = pruntimeApi();
        console.log(await pr.req('get_info'));
    }));

program
    .command('query <contract-id> <plain-query>')
    .description('send a query to a confidential contract via pRuntime directly (anonymously)', {
        'contract-id': 'confidential contract id (number)',
        'plain-command': 'the plain query payload (string or json, depending on the definition)',
    })
    .action(run(async (contractId, plainQuery) => {
        const pr = pruntimeApi();
        const cid = parseInt(contractId);
        const plainQueryObj = JSON.parse(plainQuery);
        const r = await pr.query(cid, plainQueryObj);
        console.dir(r, {depth: 3});
    }))

// pDiem related

program
    .command('pdiem-balances')
    .description('get a list of the account info and balances')
    .action(run(async () => {
        const pr = pruntimeApi();
        console.dir(await pr.query(CONTRACT_PDIEM, 'AccountData'), {depth: 3});
    }));

program
    .command('pdiem-tx')
    .description('get a list of the verified transactions')
    .action(run(async () => {
        const pr = pruntimeApi();
        console.dir(await pr.query(CONTRACT_PDIEM, 'VerifiedTransactions'), {depth: 3});
    }));

program
    .command('pdiem-new-account <seq> <suri>')
    .description('create a new diem subaccount for deposit', {
        seq: 'the sequence id of the VASP account',
        suri: 'the SURI of the sender Substrate account (sr25519)'
    })
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

program
    .command('pdiem-withdraw <dest> <amount> <suri>')
    .description('create a new diem subaccount for deposit', {
        dest: 'the withdrawal destination Diem account',
        amount: 'the sequence id of the VASP account',
        suri: 'the SURI of the sender Substrate account (sr25519)'
    })
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

program
    .command('verify <input>')
    .description('verify some inputs (ss58 address or suri). (return 0 if it\'s valid or else -1)', {
        input: 'the raw input data'
    })
    .action(run(async (input) => {
        input = input.trim();
        const keyring = new Keyring({ type: 'sr25519' });
        try {
            if (keyring.decodeAddress(input)) {
                console.log('Valid address');
                return 0;
            }
        } catch {}
        try {
            await cryptoWaitReady();
            if (keyring.addFromUri(input)) {
                console.log('Valid private key');
                return 0;
            }
        } catch {}
        console.log('Cannot decode the input');
        return -1;
    }))


program.parse(process.argv);

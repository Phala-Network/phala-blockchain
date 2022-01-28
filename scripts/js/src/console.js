require('dotenv').config();

const fs = require('fs');
const { program } = require('commander');
const axios = require('axios').default;
const { Decimal } = require('decimal.js');
const { ApiPromise, Keyring, WsProvider } = require('@polkadot/api');
const { cryptoWaitReady, blake2AsHex } = require('@polkadot/util-crypto');

const { FixedPointConverter } = require('./utils/fixedUtils');
const tokenomic  = require('./utils/tokenomic');
const { normalizeHex, praseBn, loadJson } = require('./utils/common');
const { poolSubAccount } = require('./utils/palletUtils');
const { decorateStakePool } = require('./utils/displayUtils');
const BN = require('bn.js');

function run(afn) {
    function runner(...args) {
        afn(...args)
            .then(process.exit)
            .catch(console.error)
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

function usePruntimeApi() {
    const { pruntimeEndpoint } = program.opts();
    return new PRuntimeApi(pruntimeEndpoint);
}

async function useApi() {
    const { substrateWsEndpoint, substrateNoRetry, at } = program.opts();
    const wsProvider = new WsProvider(substrateWsEndpoint);
    const api = await ApiPromise.create({
        provider: wsProvider,
        throwOnConnect: !substrateNoRetry,
    });
    if (at) {
        return await api.at(at);
    }
    return api;
}

// Gets the default key pair and the keyring
async function useKey() {
    await cryptoWaitReady();
    const { keyType, keySuri } = program.opts();
    const keyring = new Keyring({ type: keyType });
    const pair = keyring.addFromUri(keySuri);
    return { keyring, pair };
}

// Prints the tx or send it to the blockchain based on user's config
async function printTxOrSend(call) {
    if (program.opts().send) {
        const { pair } = await useKey();
        // const r = await call.signAndSend(pair, );
        // How to specify {nonce: -1}?
        const r = await new Promise(async (resolve, reject) => {
            const unsub = await call.signAndSend(pair, (result) => {
                if (result.status.isInBlock) {
                    let error;
                    for (const e of result.events) {
                        const { event: { data, method, section } } = e;
                        if (section === 'system' && method === 'ExtrinsicFailed') {
                            error = data[0];
                        }
                    }
                    unsub();
                    if (error) {
                        reject(error);
                    } else {
                        resolve({
                            hash: result.status.asInBlock.toHuman(),
                            events: result.toHuman().events,
                        });
                    }
                } else if (result.status.isInvalid) {
                    unsub();
                    resolve();
                    reject('Invalid transaction');
                }
            });
        });
        printObject(r, 4);
    } else {
        console.log(call.toHex());
    }
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
    .option('--substrate-no-retry', false)
    .option('--at <hash>', 'access the state at a certain block', null)
    .option('--key-type <type>', 'key type', 'sr25519')
    .option('-s, --key-suri <suri>', 'key suri', process.env.PRIVKEY || '//Alice')
    .option('--send', 'send the transaction instead of print the hex')
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
    .action(run(async (contractId, plainCommand, options) => {
        const api = await useApi();
        const cid = parseInt(contractId);
        const command = JSON.parse(plainCommand);
        const call = api.tx.phala.pushCommand(
            cid,
            JSON.stringify({
                Plain: JSON.stringify(command)
            })
        );
        await printTxOrSend(call);
    }));

chain
    .command('sync-state')
    .description('show the chain status; returns 0 if it\'s in sync')
    .action(run(async () => {
        const api = await useApi();
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
    .command('get-info')
    .description('show the node information')
    .action(run(async () => {
        const api = await useApi();
        const [localPeerId, localListenAddresses, health] = await Promise.all([
            api.rpc.system.localPeerId(),
            api.rpc.system.localListenAddresses(),
            api.rpc.system.health(),
        ]);
        printObject({
            localPeerId: localPeerId.toJSON(),
            localListenAddresses: localListenAddresses.toJSON(),
            health: health.toJSON(),
        });
    }));

chain
    .command('free-balance')
    .description('get the firee blance of an account')
    .argument('<account>', 'the account to lookup')
    .action(run (async (account) => {
        const api = await useApi();
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

        const api = await useApi();
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
        const api = await useApi();
        const p = await tokenomic.readFromChain(api);
        printObject(p);
    }));

chain
    .command('update-tokenomic')
    .argument('<json>', 'tokenomic parameter json file path')
    .description('create a call to update tokenomic parameters and print the raw call')
    .action(run(async (path) => {
        const p = loadJson(path);
        const api = await useApi();
        const typedP = tokenomic.humanToTyped(api, p);
        const call = tokenomic.createUpdateCall(api, typedP);
        console.log('Raw Call:', call.method.toHex());
    }));

chain
    .command('stake-pool-subaccount')
    .argument('<pid>', 'pid')
    .argument('<worker-pubkey>', 'the worker public key')
    .description('generate the stake pool subaccount by pid and worker pubkey')
    .action(run(async (pid, workerPubkey) => {
        const api = await useApi();
        const subAccount = poolSubAccount(api, pid, workerPubkey);
        console.log(subAccount.toHuman());
    }));

chain
    .command('stake-pool')
    .argument('<pid>', 'pid')
    .description('get the stake pool info')
    .action(run(async (pid) => {
        const api = await useApi();
        const pool = await api.query.phalaStakePool.stakePools(pid);
        printObject(decorateStakePool(pool.unwrap()));
    }));

chain
    .command('grab-gk-egress')
    .description('get the stake pool info')
    .option('--from <start_block>', 'Start block', '0')
    .option('--to <end_block>', 'End block', null)
    .action(run(async (opt) => {
        const api = await useApi();
        var blockNumber = parseInt(opt.from);

        while (true) {
            const hash = await api.rpc.chain.getBlockHash(blockNumber);
            const singedBlock = await api.rpc.chain.getBlock(hash);
            singedBlock.block.extrinsics.forEach(({method: { args, method, section }}) => {
                if (method === 'syncOffchainMessage' && section === 'phalaMq') {
                    const message = args[0].message;
                    const sender = message.sender.toString();
                    if (sender == "Gatekeeper") {
                        const destination = message.destination.toHuman();
                        const sequence = args[0].sequence;
                        const payloadHash = blake2AsHex(message.payload);
                        console.log(`block=${blockNumber}, seq=${sequence}, to=${destination}, payload_hash=${payloadHash}`);
                    }
                }
            });
            blockNumber += 1;
            if (opt.to && blockNumber > opt.to) {
                break;
            }
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
        const pr = usePruntimeApi();
        printObject(await pr.req('get_info'));
    }));

pruntime
    .command('req')
    .description('send a request to pruntime')
    .argument('<method>', 'the method name')
    .option('--body', 'a json request body', '')
    .action(run(async (method, opt) => {
        const pr = usePruntimeApi();
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
        const pr = usePruntimeApi();
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
        const pr = usePruntimeApi();
        console.dir(await pr.query(CONTRACT_PDIEM, 'AccountData'), {depth: 3});
    }));

pdiem
    .command('tx')
    .description('get a list of the verified transactions')
    .action(run(async () => {
        const pr = usePruntimeApi();
        console.dir(await pr.query(CONTRACT_PDIEM, 'VerifiedTransactions'), {depth: 3});
    }));

pdiem
    .command('new-account')
    .description('create a new diem subaccount for deposit')
    .argument('<seq>', 'the sequence id of the VASP account')
    .action(run(async (seq) => {
        const api = await useApi();
        const seqNumber = parseInt(seq);
        const call = api.tx.phala.pushCommand(
            CONTRACT_PDIEM,
            JSON.stringify({
                Plain: JSON.stringify({
                    NewAccount: {
                        seq_number: seqNumber
                    }
                })
            })
        );
        await printTxOrSend(call);
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
        const api = await useApi();
        const call = api.tx.phala.pushCommand(
            CONTRACT_PDIEM,
            JSON.stringify({
                Plain: JSON.stringify({
                    TransferXUS: {
                        to: dest,
                        amount: xusAmount,
                    }
                })
            })
        );
        await printTxOrSend(call);
    }));

const xcmp = program
    .command('xcmp')
    .description('XCMP tools');

xcmp
    .command('transact')
    .description('send a transact xcm to the relay chain on behalf of the parachain\'s sovereign account')
    .option('--max-fee <fee>', 'the max fee to pay for the transaction (in pico)', '1000000000000')
    .argument('<data>', 'the raw transac data in hex')
    .action(run(async (data, opt) => {
        const api = await useApi();
        const paraId = await api.query.parachainInfo.parachainId();
        const maxFee = opt.maxFee;
        console.log(`Max fee: ${maxFee}`);
        console.log(`Our ParaId: ${paraId.toNumber()}`);

        const dest = api.createType('XcmVersionedMultiLocation', {
            V1: { parents: 1, interior: 'Here' }
        });

        const message = api.createType('XcmVersionedXcm', {
            V2: [
                // Withdraw 1 KSM to buy execution
                {
                    WithdrawAsset: [{
                        id: { Concrete: { parents: 0, interior: 'Here' } },
                        fun: { Fungible: new BN(maxFee) },
                    }],
                },
                {
                    BuyExecution: {
                        fees: {
                            id: { Concrete: { parents: 0, interior: 'Here' } },
                            fun: { Fungible: new BN(maxFee) },
                        },
                        weightLimit: 'Unlimited',
                    }
                },
                // Transact on behalf of the parachain's sovereign account
                {
                    Transact: {
                        originType: 'Native',
                        requiredWeightAtMost: 0,
                        call: { encoded: data },
                    }
                },
                // Pay back the remaining fee
                {
                    DepositAsset: {
                        assets: { Wild: 'All' },
                        maxAssets: 1,
                        beneficiary: {
                            parents: 0,
                            interior: { X1: { Parachain: paraId } },
                        }
                    }
                },
            ]
        });
        console.dir(message.toHuman(), {depth: 6});
        const call = api.tx.polkadotXcm.send(dest, message);
        console.log(call.method.toHex());
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
            // We don't call useKey because we just want to validate the input
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

const contract = program
    .command('contract')
    .description('fat contract utilities');

contract
    .command('upload-code')
    .description('upload an ink wasm contract to the blockchain and get the hex')
    .argument('<wasm>', 'path to the wasm code')
    .action(run(async (wasmPath) => {
        const api = await useApi();
        // TODO: move to phalaContract later
        const code = fs.readFileSync(wasmPath);
        const codeHex = '0x' + code.toString('hex');
        const call = api.tx.phalaRegistry.uploadCode(codeHex);
        await printTxOrSend(call);
    }));

contract
    .command('instantiate')
    .description('instantiate a fat contract')
    .argument('<code-hash>', 'the hash of the code, in hex')
    .argument('<call-data>', 'the encoded arguments in hex')
    .argument('<worker>', 'the targeted worker to deploy the contract')
    .action(run(async (codeHash, callData, worker) => {
        const api = await useApi();
        const call = api.tx.phalaRegistry.instantiateContract(
            { 'WasmCode': codeHash },
            callData,
            '0x',
            worker
        );
        await printTxOrSend(call);
    }));

program.parse(process.argv);

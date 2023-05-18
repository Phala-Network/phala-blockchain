require('dotenv').config();

const fs = require('fs');
const { program } = require('commander');
const { Decimal } = require('decimal.js');
const { ApiPromise, Keyring, WsProvider } = require('@polkadot/api');
const { cryptoWaitReady, blake2AsHex } = require('@polkadot/util-crypto');
const { hexToU8a, u8aConcat, u8aToHex } = require('@polkadot/util');

const { FixedPointConverter } = require('./utils/fixedUtils');
const tokenomic  = require('./utils/tokenomic');
const { normalizeHex, praseBn, loadJson } = require('./utils/common');
const { poolSubAccount, createMotion } = require('./utils/palletUtils');
const { decorateStakePool } = require('./utils/displayUtils');
const { createPRuntimeApi } = require('./utils/pruntime');
const { resolveDefaultEndpoint } = require('./utils/builtinEndpoints');
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

function usePruntimeApi() {
    const { pruntimeEndpoint } = program.opts();
    return createPRuntimeApi(pruntimeEndpoint);
}

async function useApi() {
    let { substrateWsEndpoint, substrateNoRetry, at } = program.opts();
    const endpoint = resolveDefaultEndpoint(substrateWsEndpoint);
    const wsProvider = new WsProvider(endpoint);
    const api = await ApiPromise.create({
        provider: wsProvider,
        throwOnConnect: !substrateNoRetry,
        noInitWarn: true,
    });
    if (at) {
        if (!at.startsWith('0x') && !isNaN(at)) {
            // Get the block hash at some height
            at = (await api.rpc.chain.getBlockHash(at)).toString();
        }
        console.debug('Accessing the data at:', at);
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

function hexOrFile(arg) {
    if (arg.startsWith('0x')) {
        return arg;
    } else {
        const data = fs.readFileSync(arg, { encoding: 'utf-8' }).trim();
        if (!data.startsWith('0x')) {
            console.error('File must be hex encoded.', arg);
            process.exit(-1);
        }
        return data;
    }
}

async function estimateFeeOn(endpoint, callHex) {
    // Create another API because we are not querying Phala
    endpoint = resolveDefaultEndpoint(endpoint);
    const wsProvider = new WsProvider(endpoint);
    const api = await ApiPromise.create({
        provider: wsProvider,
        throwOnConnect: true,
    });
    const { pair } = await useKey();
    // Walk-around to decode a call to a unsigned extrinsics
    const call = api.createType('Call', callHex);
    const { method, section } = api.registry.findMetaCall(call.callIndex);
    const extrinsicFn = api.tx[section][method];
    const extrinsic = extrinsicFn(...call.args);
    // Estimate weight
    const info = await extrinsic.paymentInfo(pair);
    return info;
}

program
    .option('--pruntime-endpoint <url>', 'pRuntime API endpoint', process.env.PRUNTIME_ENDPOINT || 'http://localhost:8000')
    .option('--substrate-ws-endpoint <url>', 'Substrate WS endpoint. Supported builtin endpoints: phala, khala.', process.env.ENDPOINT || 'ws://localhost:9944')
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
            api.query.phalaComputation.workerBindings(workerKey),
            api.query.phalaStakePool.workerAssignments(workerKey),
        ]);
        workerInfo = workerInfo.unwrapOr();
        miner = miner.unwrapOr();
        pid = pid.unwrapOr();

        const minerInfo = miner ? await api.query.phalaComputation.miners(miner) : undefined;
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
        console.log('Params:', typedP.toHex());
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
            function processExtrinsic(extrinsic) {
                const { args, method, section } = extrinsic;
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
                if (method === 'forceBatch' && section === 'utility') {
                    args[0].forEach((method) => {
                        processExtrinsic(method);
                    });
                }
            }
            singedBlock.block.extrinsics.forEach(({ method }) => {
                processExtrinsic(method);
            });
            blockNumber += 1;
            if (opt.to && blockNumber > opt.to) {
                break;
            }
        }
    }));

chain
    .command('motion')
    .description('generate a call to propose a motion')
    .argument('<threshold>', 'the threshold')
    .argument('<call>', 'the raw call in hex')
    .action(run(async (thresholdStr, callHex) => {
        const api = await useApi();
        const threshold = parseInt(thresholdStr);
        callHex = hexOrFile(callHex);
        const call = createMotion(api, threshold, callHex);
        console.log(call.toHex());
    }))

chain
    .command('external')
    .description('generate a call to propose a majority external proposal')
    .argument('<call>', 'the raw call in hex')
    .action(run(async (callHex) => {
        const api = await useApi();
        // Examine
        callHex = hexOrFile(callHex);
        console.log('Building majority external proposal for:');
        console.dir(api.createType('Call', callHex).toHuman(), {depth: 10})
        // Calculate the threshold
        const members = await api.query.council.members();
        const totalMembers = members.length;
        const threshold = Math.ceil(totalMembers * 0.75);
        // Build motion
        const externalCall = api.tx.democracy.externalProposeMajority({ Inline: callHex });
        const motionCall = createMotion(api, threshold, externalCall);
        console.log(motionCall.toHex(false));
    }))

// pRuntime operations
const pruntime = program
    .command('pruntime')
    .description('pRuntime commands');

pruntime
    .command('get-info')
    .description('get the running status')
    .action(run(async () => {
        const pr = usePruntimeApi();
        printObject(await pr.getInfo({}));
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
        printObject(await pr[method](body ? body : {}));
    }));

pruntime
    .command('query-raw')
    .description('query a raw contract')
    .argument('<contract-id>', 'the contract id')
    .argument('<selector>', 'the hex method selector')
    .argument('<data>', 'the hex call data')
    .action(run(async (method, opt) => {
        throw Error('TODO');
        // const pr = usePruntimeApi();
        // // @codec scale crate::crypto::EncryptedData
        // printObject(await pr.contractQuery({
        //     encodedEncryptedData:
        // }))
    }));

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

const xcmp = program
    .command('xcmp')
    .description('XCMP tools');

xcmp
    .command('transact')
    .description('send a transact xcm to the relay chain on behalf of the parachain\'s sovereign account')
    .option('--set-safe-xcm-version <version>', 'if specified, set the SafeXcmVersion first')
    .option('--max-fee <fee>', 'the max fee to pay for the transaction (in pico)', '100000000000')
    .option('--transact-weight <weight>', 'set RequireWeightAtMost in transact', '2000000000')
    .option(
        '--estimate-on-relay <ws-url>',
        'specify the relay chain ws endpoint to enable transact call weight estimation. supported builtin endpoints: polkadot, kusama.'
    )
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

        const paraIdNumber = paraId.toNumber();
        const sovereignAccount = u8aToHex(u8aConcat(
            // sovereign account prefix
            hexToU8a('0x70617261'),
            // encoded para id
            new Uint8Array([
                paraIdNumber & 0xff,
                (paraIdNumber & 0xff00) >> 8,
            ]),
            // postfix
            hexToU8a('0x0000000000000000000000000000000000000000000000000000')
        ));

        // Estimate weight on relay chain
        if (opt.estimateOnRelay) {
            const info = await estimateFeeOn(opt.estimateOnRelay, data);
            const estWeight = info.weight;
            const weight = new BN(opt.transactWeight);
            if (weight.lt(estWeight)) {
                console.error(`No enough weight (${opt.transactWeight} < ${estWeight.toString()})`);
                return;
            }
            console.log(`Estimated weight: ${estWeight.toString()}`);
        }

        const message = api.createType('XcmVersionedXcm', {
            V2: [
                // Withdraw 0.1 KSM to buy execution by default, please take care of DOT which decimal is 10
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
                        requireWeightAtMost: opt.transactWeight,
                        call: { encoded: data },
                    }
                },
                // Pay back the remaining fee
                'RefundSurplus',
                {
                    DepositAsset: {
                        assets: {
                            Wild: {
                                AllOf: {
                                    id: { Concrete: { parents: 0, interior: 'Here' } },
                                    fun: 'Fungible',
                                }
                            }
                        },
                        maxAssets: 1,
                        beneficiary: {
                            parents: 0,
                            interior: {
                                X1: {
                                    AccountId32: {
                                        network: 'Any',
                                        id: sovereignAccount,
                                    }
                                }
                            },
                        }
                    }
                },
            ]
        });
        console.dir(message.toHuman(), {depth: 7});
        let call = api.tx.polkadotXcm.send(dest, message);
        if (opt.setSafeXcmVersion) {
            call = api.tx.utility.batch([
                api.tx.polkadotXcm.forceDefaultXcmVersion(opt.setSafeXcmVersion),
                call,
            ])
        };

        // Send transaction as root
        if (program.opts().send) {
            let sudoCall = api.tx.sudo.sudo(call);
            await printTxOrSend(sudoCall);
        }
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
    .description('phat contract utilities');

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
    .description('instantiate a phat contract')
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


const debug = program
    .command('debug')
    .description('debugging utilities');

debug
    .command('fsck')
    .description('check blockchain database integrity')
    .argument('<from>', 'the first block to check (number)')
    .argument('<to>', 'the last block to check (number)')
    .option('--early-stop', 'stop when encounting any error', false)
    .option('--progress', 'show progress per 1000 blocks', false)
    .action(run(async (from, to, opt) => {
        const api = await useApi();
        const { progress, earlyStop } = opt;

        let prevHash = null;
        for (let i = from; i <= to; i++) {
            // Progress report
            if (progress && i % 1000 == 0) {
                console.log(`Scanning block ${i}`);
            }

            const hash = u8aToHex(await api.rpc.chain.getBlockHash(i));
            const header = await api.rpc.chain.getHeader(hash);

            let hasErr = false;
            // Check hash(blockHeader) == hashKeyInDb
            let blockHash = blake2AsHex(header.toU8a());
            if (blockHash != hash) {
                console.error(`[Block ${i}] hash of the block doesn't match the db key (actual hash = ${blockHash}, hash in db = ${hash})`);
                hasErr = true;
            }
            // Check block.parentHash == parentHash
            let hashPointer = u8aToHex(header.parentHash);
            if (prevHash && prevHash != hashPointer) {
                console.error(`[Block ${i}] prev_hash mismatch previous block! (actual block = ${prevHash}, field = ${hashPointer})`)
                hasErr = true;
            }
            // Early stop
            if (hasErr && earlyStop) {
                break;
            }
            prevHash = hash;
        }
    }));


program.parse(process.argv);

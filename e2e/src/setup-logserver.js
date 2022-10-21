const { ApiPromise, WsProvider, Keyring } = require('@polkadot/api');
const { ContractPromise } = require('@polkadot/api-contract');
const Phala = require('@phala/sdk');
const fs = require('fs');
const crypto = require('crypto');
const { PRuntimeApi } = require('./utils/pruntime');

function loadContractFile(contractFile) {
    const metadata = JSON.parse(fs.readFileSync(contractFile));
    const constructor = metadata.V3.spec.constructors.find(c => c.label == 'default').selector;
    const name = metadata.contract.name;
    const wasm = metadata.source.wasm;
    return { wasm, metadata, constructor, name };
}

async function deployDriverContract(api, txqueue, system, pair, cert, contract, clusterId, name, salt) {
    console.log(`Contracts: uploading ${contract.name}`);
    // upload the contract 
    const { events: deployEvents } = await txqueue.submit(
        api.tx.utility.batchAll(
            [
                api.tx.phalaFatContracts.clusterUploadResource(clusterId, 'InkCode', contract.wasm),
                api.tx.phalaFatContracts.instantiateContract(
                    { WasmCode: contract.metadata.source.hash },
                    contract.constructor,
                    salt ? salt : hex(crypto.randomBytes(4)),
                    clusterId,
                )
            ]
        ),
        pair
    );
    const contractIds = deployEvents
        .filter(ev => ev.event.section == 'phalaFatContracts' && ev.event.method == 'Instantiating')
        .map(ev => ev.event.data[0].toString());
    const numContracts = 1;
    console.assert(contractIds.length == numContracts, 'Incorrect length:', `${contractIds.length} vs ${numContracts}`);
    contract.address = contractIds[0];
    await checkUntilEq(
        async () => (await api.query.phalaFatContracts.clusterContracts(clusterId))
            .filter(c => contractIds.includes(c.toString()))
            .length,
        numContracts,
        4 * 6000
    );
    console.log('Contracts: uploaded');
    await checkUntil(
        async () => (await api.query.phalaRegistry.contractKeys(contract.address)).isSome,
        4 * 6000
    );
    await txqueue.submit(
        system.tx["system::setDriver"]({}, name, contract.address),
        pair
    );
    await txqueue.submit(
        system.tx["system::grantAdmin"]({}, contract.address),
        pair
    );
    await checkUntil(
        async () => {
            const { output } = await system.query["system::getDriver"](cert, {}, name);
            return output.isSome && output.unwrap().eq(contract.address);
        },
        4 * 6000
    );
    console.log(`Contracts: ${contract.name} deployed`);
    return contract.address;
}

async function uploadSystemCode(api, txqueue, pair, wasm) {
    console.log(`Uploading system code`);
    await txqueue.submit(
        api.tx.sudo.sudo(api.tx.phalaFatContracts.setPinkSystemCode(hex(wasm))),
        pair
    );
    console.log(`Uploaded system code`);
}

class TxQueue {
    constructor(api) {
        this.nonceTracker = {};
        this.api = api;
    }
    async nextNonce(address) {
        const byCache = this.nonceTracker[address] || 0;
        const byRpc = (await this.api.rpc.system.accountNextIndex(address)).toNumber();
        return Math.max(byCache, byRpc);
    }
    markNonceFailed(address, nonce) {
        if (!this.nonceTracker[address]) {
            return;
        }
        if (nonce < this.nonceTracker[address]) {
            this.nonceTracker[address] = nonce;
        }
    }
    async submit(txBuilder, signer, waitForFinalization = false) {
        const address = signer.address;
        const nonce = await this.nextNonce(address);
        this.nonceTracker[address] = nonce + 1;
        let hash;
        return new Promise(async (resolve, reject) => {
            const unsub = await txBuilder.signAndSend(signer, { nonce }, (result) => {
                if (result.status.isInBlock) {
                    for (const e of result.events) {
                        const { event: { data, method, section } } = e;
                        if (section === 'system' && method === 'ExtrinsicFailed') {
                            unsub();
                            reject(data[0].toHuman())
                        }
                    }
                    if (!waitForFinalization) {
                        unsub();
                        resolve({
                            hash: result.status.asInBlock,
                            events: result.events,
                        });
                    } else {
                        hash = result.status.asInBlock;
                    }
                } else if (result.status.isFinalized) {
                    resolve({
                        hash,
                        events: result.events,
                    })
                } else if (result.status.isInvalid) {
                    unsub();
                    this.markNonceFailed(address, nonce);
                    reject('Invalid transaction');
                }
            });
        });
    }
}

async function sleep(t) {
    await new Promise(resolve => {
        setTimeout(resolve, t);
    });
}

async function checkUntil(async_fn, timeout) {
    const t0 = new Date().getTime();
    while (true) {
        if (await async_fn()) {
            return;
        }
        const t = new Date().getTime();
        if (t - t0 >= timeout) {
            throw new Error('timeout');
        }
        await sleep(100);
    }
}

async function checkUntilEq(async_fn, expected, timeout, verbose = true) {
    const t0 = new Date().getTime();
    let lastActual = undefined;
    while (true) {
        const actual = await async_fn();
        if (actual == expected) {
            return;
        }
        if (actual != lastActual && verbose) {
            console.log(`Waiting... (current = ${actual}, expected = ${expected})`)
            lastActual = actual;
        }
        const t = new Date().getTime();
        if (t - t0 >= timeout) {
            throw new Error('timeout');
        }
        await sleep(100);
    }
}

function hex(b) {
    if (typeof b != "string") {
        b = Buffer.from(b).toString('hex');
    }
    if (!b.startsWith('0x')) {
        return '0x' + b;
    } else {
        return b;
    }
}

async function getWorkerPubkey(api) {
    const workers = await api.query.phalaRegistry.workers.entries();
    const worker = workers[0][0].args[0].toString();
    return worker;
}

async function setupGatekeeper(api, txpool, pair, worker) {
    if ((await api.query.phalaRegistry.gatekeeper()).length > 0) {
        return;
    }
    console.log('Gatekeeper: registering');
    await txpool.submit(
        api.tx.sudo.sudo(
            api.tx.phalaRegistry.registerGatekeeper(worker)
        ),
        pair,
    );
    await checkUntil(
        async () => (await api.query.phalaRegistry.gatekeeper()).length == 1,
        4 * 6000
    );
    console.log('Gatekeeper: added');
    await checkUntil(
        async () => (await api.query.phalaRegistry.gatekeeperMasterPubkey()).isSome,
        4 * 6000
    );
    console.log('Gatekeeper: master key ready');
}

async function deployCluster(api, txqueue, sudoer, owner, worker, defaultCluster = '0x0000000000000000000000000000000000000000000000000000000000000000') {
    const clusterInfo = await api.query.phalaFatContracts.clusters(defaultCluster);
    if (clusterInfo.isSome) {
        return { clusterId: defaultCluster, systemContract: clusterInfo.unwrap().systemContract.toHex() };
    }
    console.log('Cluster: creating');
    // crete contract cluster and wait for the setup
    const { events } = await txqueue.submit(
        api.tx.sudo.sudo(api.tx.phalaFatContracts.addCluster(
            owner,
            'Public', // can be {'OnlyOwner': accountId}
            [worker]
        )),
        sudoer
    );
    const ev = events[1].event;
    console.assert(ev.section == 'phalaFatContracts' && ev.method == 'ClusterCreated');
    const clusterId = ev.data[0].toString();
    const systemContract = ev.data[1].toString();
    console.log('Cluster: created', clusterId)
    await checkUntil(
        async () => (await api.query.phalaRegistry.clusterKeys(clusterId)).isSome,
        4 * 6000
    );
    await checkUntil(
        async () => (await api.query.phalaRegistry.contractKeys(systemContract)).isSome,
        4 * 6000
    );
    return { clusterId, systemContract };
}

async function contractApi(api, pruntimeURL, contract) {
    const newApi = await api.clone().isReady;
    const phala = await Phala.create({ api: newApi, baseURL: pruntimeURL, contractId: contract.address });
    const contractApi = new ContractPromise(
        phala.api,
        contract.metadata,
        contract.address,
    );
    contractApi.sidevmQuery = phala.sidevmQuery;
    return contractApi;
}

function toBytes(s) {
    let utf8Encode = new TextEncoder();
    return utf8Encode.encode(s)
}

async function main() {
    const contractSystem = loadContractFile('./res/system.contract');
    const contractSidevmop = loadContractFile('./res/sidevm_deployer.contract');
    const contractLogServer = loadContractFile('./res/log_server.contract');
    const logServerSidevmWasm = fs.readFileSync('./res/log_server.sidevm.wasm', 'hex');
    const nodeURL = 'ws://localhost:19944';
    const pruntimeURL = 'http://localhost:8000';

    // Connect to the chain
    const wsProvider = new WsProvider(nodeURL);
    const api = await ApiPromise.create({
        provider: wsProvider,
        types: {
            ...Phala.types,
            'GistQuote': {
                username: 'String',
                accountId: 'AccountId',
            },
        }
    });
    const txqueue = new TxQueue(api);

    // Prepare accounts
    const keyring = new Keyring({ type: 'sr25519' })
    const alice = keyring.addFromUri('//Alice')
    const certAlice = await Phala.signCertificate({ api, pair: alice });

    // Connect to pruntime
    const prpc = new PRuntimeApi(pruntimeURL);
    const worker = await getWorkerPubkey(api);
    const connectedWorker = hex((await prpc.getInfo({})).publicKey);
    console.log('Worker:', worker);
    console.log('Connected worker:', connectedWorker);

    // Basic phala network setup
    await setupGatekeeper(api, txqueue, alice, worker);

    // Upload the pink-system wasm to the chain. It is required to create a cluster.
    await uploadSystemCode(api, txqueue, alice, contractSystem.wasm);

    const { clusterId, systemContract } = await deployCluster(api, txqueue, alice, alice.address, worker);
    contractSystem.address = systemContract;

    const system = await contractApi(api, pruntimeURL, contractSystem);

    // Deploy driver: Sidevm deployer
    await deployDriverContract(api, txqueue, system, alice, certAlice, contractSidevmop, clusterId, "SidevmOperation");

    const sidevmDeployer = await contractApi(api, pruntimeURL, contractSidevmop);

    // Allow the logger to deploy sidevm
    const salt = hex(crypto.randomBytes(4));
    const { id: loggerId } = await prpc.calculateContractId({
        deployer: hex(alice.publicKey),
        clusterId,
        codeHash: contractLogServer.metadata.source.hash,
        salt,
    });
    console.log(`calculated loggerId = ${loggerId}`);

    await txqueue.submit(
        sidevmDeployer.tx.allow({}, loggerId),
        alice
    );

    // Upload the logger's sidevm wasm code
    await txqueue.submit(
        api.tx.phalaFatContracts.clusterUploadResource(clusterId, 'SidevmCode', hex(logServerSidevmWasm)),
        alice);

    // Deploy the logger contract
    await deployDriverContract(api, txqueue, system, alice, certAlice, contractLogServer, clusterId, "PinkLogger", salt);

    await sleep(2000);
    const logger = await contractApi(api, pruntimeURL, contractLogServer);
    // Trigger some contract logs
    for (var i = 0; i < 100; i++) {
        await logger.query.logTest(certAlice, {}, "hello " + i);
    }
    // Query input: a JSON doc with three optinal fields:
    const condition = {
        // What to do. Only `GetLog` is supported currently
        action: 'GetLog',
        // The target contract to query. Default to all contracts
        contract: contractLogServer.address,
        // The sequence number start from. Default to 0.
        from: 20,
        // Max number of items should returned. Default to not limited.
        count: 2,
    };
    const data = hex(toBytes(JSON.stringify(condition)));
    const hexlog = await logger.sidevmQuery(data, certAlice);
    const bytes = api.createType("Vec<u8>", hexlog);
    const decoder = new TextDecoder();
    const jsonText = decoder.decode(bytes);
    console.log('log:', jsonText);
    // Sample query response:
    const _ = {
        "next": 3, // Sequence number for the next query. For pagination.
        "records": [
            {
                "blockNumber": 0,
                "contract": "0x0101010101010101010101010101010101010101010101010101010101010101",
                "inQuery": true,
                "level": 0,
                "message": "hello", // Log content
                "sequence": 0,
                "timestamp": 1,
                "type": "Log" // Type of the records. could be one of ['Log', 'Event', 'MessageOutput']
            },
            {
                "blockNumber": 1,
                "contract": "0x0101010101010101010101010101010101010101010101010101010101010101",
                "payload": "0x01020304",
                "sequence": 1,
                "topics": [
                    "0x0202020202020202020202020202020202020202020202020202020202020202",
                    "0x0303030303030303030303030303030303030303030303030303030303030303"
                ],
                "type": "Event"
            },
            {
                "blockNumber": 2,
                "contract": "0x0202020202020202020202020202020202020202020202020202020202020202",
                "nonce": "0x0102030405",
                "origin": "0x0101010101010101010101010101010101010101010101010101010101010101",
                "output": "0x0504030201",
                "sequence": 2,
                "type": "MessageOutput"
            }
        ]
    };
}

main().then(process.exit).catch(err => console.error('Crashed', err)).finally(() => process.exit(-1));

require('dotenv').config();
const axios = require('axios');
const { assert } = require('chai');
const path = require('path');
const portfinder = require('portfinder');
const fs = require('fs');
const { ApiPromise, WsProvider, Keyring } = require('@polkadot/api');
const { cryptoWaitReady, mnemonicGenerate } = require('@polkadot/util-crypto');
const { ContractPromise } = require('@polkadot/api-contract');
const Phala = require('@phala/sdk');
const { typeDefinitions } = require('@polkadot/types/bundle');
const BN = require('bn.js');

const { types, typeAlias, typeOverrides } = require('./utils/typeoverride');
// TODO: fixit
// const types = require('@phala/typedefs').phalaDev;

const { Process, TempDir } = require('./utils/pm');
const { PRuntimeApi } = require('./utils/pruntime');
const { checkUntil, skipSlowTest, sleep } = require('./utils');

const pathNode = path.resolve('../target/release/phala-node');
const pathRelayer = path.resolve('../target/release/pherry');

const pRuntimeBin = "pruntime";
const pRuntimeDir = path.resolve(`../standalone/pruntime/bin`);
const pathPRuntime = path.resolve(`${pRuntimeDir}/${pRuntimeBin}`);
const inSgx = process.env.E2E_IN_SGX == '1';
const sgxLoader = "gramine-sgx";

const CENTS = 10_000_000_000;

console.log(`Testing in ${inSgx ? "SGX Hardware" : "Software"} mode`);

// TODO: Switch to [instant-seal-consensus](https://substrate.dev/recipes/kitchen-node.html) for faster test

describe('A full stack', function () {
    this.timeout(160000);

    let cluster;
    let api, keyring, alice, bob;
    let pruntime;
    let pherry;
    const tmpDir = new TempDir();
    const tmpPath = tmpDir.dir;

    before(async () => {
        // Check binary files
        [pathNode, pathRelayer, pathPRuntime].map(fs.accessSync);
        // Bring up a cluster
        cluster = new Cluster(5, pathNode, pathRelayer, pathPRuntime, tmpPath);
        await cluster.start();
        // APIs
        api = await cluster.api;
        pruntime = cluster.workers.map(w => w.api);
        pherry = cluster.workers.map(w => w.processRelayer);
        // Create polkadot api and keyring
        await cryptoWaitReady();
        keyring = new Keyring({ type: 'sr25519', ss58Format: 30 });
        alice = keyring.addFromUri('//Alice');
        bob = keyring.addFromUri('//Bob');
    });

    after(async function () {
        // TODO: consider handle the signals and process.on('exit') event:
        //   https://stackoverflow.com/questions/14031763/doing-a-cleanup-action-just-before-node-js-exits
        if (api) await api.disconnect();
        await cluster.kill();
        if (process.env.KEEP_TEST_FILES != '1') {
            tmpDir.cleanup();
        } else {
            console.log(`The test datadir is kept at ${cluster.tmpPath}`);
        }
    });

    it('should be up and running', async function () {
        assert.isFalse(cluster.processNode.stopped);
        for (const w of cluster.workers) {
            assert.isFalse(w.processRelayer.stopped);
            assert.isFalse(w.processPRuntime.stopped);
        }
    });

    let workerKey;
    describe('pRuntime', () => {
        it('is initialized', async function () {
            let info;
            assert.isTrue(await checkUntil(async () => {
                info = await pruntime[0].getInfo();
                return info.initialized;
            }, 1000), 'not initialized in time');
            // A bit guly. Any better way?
            workerKey = Uint8Array.from(Buffer.from(info.system.publicKey, 'hex'));
        });

        it('can sync block', async function () {
            assert.isTrue(await checkUntil(async () => {
                const info = await pruntime[0].getInfo();
                return info.blocknum > 0;
            }, 7000), 'stuck at block 0');
        });

        it('is registered', async function () {
            if (skipSlowTest()) {
                this.skip();
            }
            // Finalization takes 2-3 blocks. So we wait for 3 blocks here.
            assert.isTrue(await checkUntil(async () => {
                const info = await pruntime[0].getInfo();
                return info.system?.registered;
            }, 4 * 6000), 'not registered in time');
        });

        it('finishes the benchmark', async function () {
            if (skipSlowTest()) {
                this.skip();
            }
            assert.isTrue(await checkUntil(async () => {
                const workerInfo = await api.query.phalaRegistry.workers(workerKey);
                return workerInfo.unwrap().initialScore.isSome;
            }, 3 * 6000), 'benchmark timeout');
        });
    });

    describe('Gatekeeper', () => {
        it('pre-mines blocks', async function () {
            assert.isTrue(await checkUntil(async () => {
                const info = await pruntime[0].getInfo();
                return info.blocknum > 10;
            }, 10 * 6000), 'not enough blocks mined');
        });

        it('can be registered as first gatekeeper', async function () {
            // Register worker1 as Gatekeeper
            const info = await pruntime[0].getInfo();

            await assert.txAccepted(
                api.tx.sudo.sudo(
                    api.tx.phalaRegistry.registerGatekeeper(hex(info.system.publicKey))
                ),
                alice,
            );
            // Finalization takes 2-3 blocks. So we wait for 3 blocks here.
            assert.isTrue(await checkUntil(async () => {
                const info = await pruntime[0].getInfo();
                return info.system?.registered;
            }, 4 * 6000), 'not registered in time');

            // Check if the on-chain role is Gatekeeper
            assert.isTrue(await checkUntil(async () => {
                const info = await pruntime[0].getInfo();
                const gatekeepers = await api.query.phalaRegistry.gatekeeper();
                // console.log(`Gatekeepers after registeration: ${gatekeepers}`);
                return gatekeepers.toHuman().includes(hex(info.system.publicKey));
            }, 4 * 6000), 'not registered as gatekeeper');
        });

        it('finishes master pubkey upload', async function () {
            assert.isTrue(await checkUntil(async () => {
                const masterPubkey = await api.query.phalaRegistry.gatekeeperMasterPubkey();
                return masterPubkey.isSome;
            }, 4 * 6000), 'master pubkey not uploaded');
            const launchedAt = await api.query.phalaRegistry.gatekeeperLaunchedAt();
            assert.isTrue(launchedAt.isSome);
        });
    });

    describe('Gatekeeper2', () => {
        it('can be registered', async function () {
            // Register worker1 as Gatekeeper
            const info = await pruntime[1].getInfo();

            await assert.txAccepted(
                api.tx.sudo.sudo(
                    api.tx.phalaRegistry.registerGatekeeper(hex(info.system.publicKey))
                ),
                alice,
            );
            // Finalization takes 2-3 blocks. So we wait for 3 blocks here.
            assert.isTrue(await checkUntil(async () => {
                const info = await pruntime[1].getInfo();
                return info.system?.registered;
            }, 4 * 6000), 'not registered in time');

            // Check if the on-chain role is Gatekeeper
            assert.isTrue(await checkUntil(async () => {
                const info = await pruntime[1].getInfo();
                const gatekeepers = await api.query.phalaRegistry.gatekeeper();
                // console.log(`Gatekeepers after registeration: ${gatekeepers}`);
                return gatekeepers.toHuman().includes(hex(info.system.publicKey));
            }, 4 * 6000), 'not registered as gatekeeper');
        });

        it('can receive master key', async function () {
            // Wait for the successful dispatch of master key
            // pRuntime[1] should be down
            assert.isTrue(
                await cluster.waitWorkerExitAndRestart(1, 10 * 6000),
                'worker1 restart timeout'
            );
            const dataDir = "data";
            assert.isTrue(
                fs.existsSync(`${tmpPath}/pruntime1/${dataDir}/protected_files/master_key.seal`),
                'master key not received'
            );
        });

        it('becomes active', async function () {
            assert.isTrue(await checkUntil(async () => {
                const info = await pruntime[1].getInfo();
                return info.system?.gatekeeper.role == 2;  // 2: GatekeeperRole.Active in protobuf
            }, 1000))

            // Step 3: wait a few more blocks and ensure there are no conflicts in gatekeepers' shared mq
        });

        it('post-mines blocks', async function () {
            const gatekeeper = api.createType('MessageOrigin', 'Gatekeeper');
            let seqStart = await api.query.phalaMq.offchainIngress(gatekeeper);
            seqStart = seqStart.unwrap().toNumber();
            assert.isTrue(await checkUntil(async () => {
                let seq = await api.query.phalaMq.offchainIngress(gatekeeper);
                seq = seq.unwrap().toNumber();
                return seq >= seqStart + 1;
            }, 500 * 6000), 'ingress stale');
        });

        it('can be unregistered', async function () {
            const info = await pruntime[1].getInfo();
            await assert.txAccepted(
                api.tx.sudo.sudo(
                    api.tx.phalaRegistry.unregisterGatekeeper(hex(info.system.publicKey))
                ),
                alice,
            );
            // Check if the role is no longer Gatekeeper
            assert.isTrue(await checkUntil(async () => {
                const info = await pruntime[1].getInfo();
                const gatekeepers = await api.query.phalaRegistry.gatekeeper();
                // console.log(`Gatekeepers after unregisteration: ${gatekeepers}`);
                return info.system?.gatekeeper.role != 2 && !gatekeepers.includes(hex(info.system.publicKey));
            }, 4 * 6000), 'not unregistered');
        });

        it('can be re-registered before rotation', async function () {
            const info = await pruntime[1].getInfo();
            await assert.txAccepted(
                api.tx.sudo.sudo(
                    api.tx.phalaRegistry.registerGatekeeper(hex(info.system.publicKey))
                ),
                alice,
            );

            // Check if the on-chain role is Gatekeeper
            assert.isTrue(await checkUntil(async () => {
                const info = await pruntime[1].getInfo();
                const gatekeepers = await api.query.phalaRegistry.gatekeeper();
                // console.log(`Gatekeepers after registeration: ${gatekeepers}`);
                return gatekeepers.toHuman().includes(hex(info.system.publicKey));
            }, 4 * 6000), 'not registered as gatekeeper');
            // the GK should resume without restart
            assert.isTrue(await checkUntil(async () => {
                const info = await pruntime[1].getInfo();
                return info.system?.gatekeeper.role == 2;  // 2: GatekeeperRole.Active in protobuf
            }, 4 * 6000), 'gatekeeper role not changed')
        });
    });

    describe('Master Key Rotation', () => {
        it('can register and un-reg gatekeeper4', async function () {
            // Register worker4 as Gatekeeper
            const info = await pruntime[3].getInfo();
            await assert.txAccepted(
                api.tx.sudo.sudo(
                    api.tx.phalaRegistry.registerGatekeeper(hex(info.system.publicKey))
                ),
                alice,
            );
            // Finalization takes 2-3 blocks. So we wait for 3 blocks here.
            assert.isTrue(await checkUntil(async () => {
                const info = await pruntime[3].getInfo();
                return info.system?.registered;
            }, 4 * 6000), 'not registered in time');

            // Check if the on-chain role is Gatekeeper
            assert.isTrue(await checkUntil(async () => {
                const info = await pruntime[3].getInfo();
                const gatekeepers = await api.query.phalaRegistry.gatekeeper();
                // console.log(`Gatekeepers after registeration: ${gatekeepers}`);
                return gatekeepers.toHuman().includes(hex(info.system.publicKey));
            }, 4 * 6000), 'not registered as gatekeeper');

            assert.isTrue(
                await cluster.waitWorkerExitAndRestart(3, 10 * 6000),
                'worker4 restart timeout'
            );

            // Ensure it is up-to-date
            const gatekeeper = api.createType('MessageOrigin', 'Gatekeeper');
            let seqStart = await api.query.phalaMq.offchainIngress(gatekeeper);
            seqStart = seqStart.unwrap().toNumber();
            assert.isTrue(await checkUntil(async () => {
                let seq = await api.query.phalaMq.offchainIngress(gatekeeper);
                seq = seq.unwrap().toNumber();
                return seq >= seqStart + 1;
            }, 500 * 6000), 'ingress stale');

            // Unregister
            await assert.txAccepted(
                api.tx.sudo.sudo(
                    api.tx.phalaRegistry.unregisterGatekeeper(hex(info.system.publicKey))
                ),
                alice,
            );
            // Check if the role is no longer Gatekeeper
            assert.isTrue(await checkUntil(async () => {
                const info = await pruntime[3].getInfo();
                const gatekeepers = await api.query.phalaRegistry.gatekeeper();
                // console.log(`Gatekeepers after unregisteration: ${gatekeepers}`);
                return info.system?.gatekeeper.role != 2 && !gatekeepers.toHuman().includes(hex(info.system.publicKey));
            }, 4 * 6000), 'not unregistered');
        });

        it('can rotate master key', async function () {
            const info = await pruntime[0].getInfo();
            const old_master_pubkey = hex(info.gatekeeper.masterPublicKey);
            await assert.txAccepted(
                api.tx.sudo.sudo(
                    api.tx.phalaRegistry.rotateMasterKey()
                ),
                alice,
            );
            // only one rotation is allowed one time
            await assert.txFailed(
                api.tx.sudo.sudo(
                    api.tx.phalaRegistry.rotateMasterKey()
                ),
                alice,
            )

            assert.isTrue(await checkUntil(async () => {
                const info1 = await pruntime[0].getInfo();
                const info2 = await pruntime[1].getInfo();
                return hex(info1.gatekeeper.masterPublicKey) != old_master_pubkey
                    && hex(info1.gatekeeper.masterPublicKey) == hex(info2.gatekeeper.masterPublicKey);
            }, 4 * 6000), 'local master key not rotated');

            assert.isTrue(await checkUntil(async () => {
                const info = await pruntime[0].getInfo();
                const masterPubkey = await api.query.phalaRegistry.gatekeeperMasterPubkey();
                // console.log(`Master PubKey: ${masterPubkey}`);
                return masterPubkey == hex(info.gatekeeper.masterPublicKey);
            }, 4 * 6000), 'on-chain master key not updated');
        });

        it('will kill unregistered gatekeeper', async function () {
            assert.isTrue(await checkUntil(async () => {
                const info = await pruntime[3].getInfo();
                return info.system?.gatekeeper.role == 0;
            }, 4 * 6000), 'outdated gatekeeper not remove');
        });

        it('can re-register the gatekeeper4 after rotation', async function () {
            // Register worker4 as Gatekeeper
            const info = await pruntime[3].getInfo();
            await assert.txAccepted(
                api.tx.sudo.sudo(
                    api.tx.phalaRegistry.registerGatekeeper(hex(info.system.publicKey))
                ),
                alice,
            );
            // Finalization takes 2-3 blocks. So we wait for 3 blocks here.
            assert.isTrue(await checkUntil(async () => {
                const info = await pruntime[3].getInfo();
                return info.system?.registered;
            }, 4 * 6000), 'not registered in time');

            // Check if the on-chain role is Gatekeeper
            assert.isTrue(await checkUntil(async () => {
                const info = await pruntime[3].getInfo();
                const gatekeepers = await api.query.phalaRegistry.gatekeeper();
                // console.log(`Gatekeepers after registeration: ${gatekeepers}`);
                return gatekeepers.toHuman().includes(hex(info.system.publicKey));
            }, 4 * 6000), 'not registered as gatekeeper');

            assert.isTrue(
                await cluster.waitWorkerExitAndRestart(3, 10 * 6000),
                'worker4 restart timeout'
            );

            // Ensure it is up-to-date
            const gatekeeper = api.createType('MessageOrigin', 'Gatekeeper');
            let seqStart = await api.query.phalaMq.offchainIngress(gatekeeper);
            seqStart = seqStart.unwrap().toNumber();
            assert.isTrue(await checkUntil(async () => {
                let seq = await api.query.phalaMq.offchainIngress(gatekeeper);
                seq = seq.unwrap().toNumber();
                return seq >= seqStart + 1;
            }, 500 * 6000), 'ingress stale');
        });
    });

    describe('Gatekeeper3 after rotation', () => {
        it('can be registered after rotation', async function () {
            // Register worker3 as Gatekeeper
            const info = await pruntime[2].getInfo();
            await assert.txAccepted(
                api.tx.sudo.sudo(
                    api.tx.phalaRegistry.registerGatekeeper(hex(info.system.publicKey))
                ),
                alice,
            );
            // Finalization takes 2-3 blocks. So we wait for 3 blocks here.
            assert.isTrue(await checkUntil(async () => {
                const info = await pruntime[2].getInfo();
                return info.system?.registered;
            }, 4 * 6000), 'not registered in time');

            // Check if the on-chain role is Gatekeeper
            assert.isTrue(await checkUntil(async () => {
                const info = await pruntime[2].getInfo();
                const gatekeepers = await api.query.phalaRegistry.gatekeeper();
                // console.log(`Gatekeepers after registeration: ${gatekeepers}`);
                return gatekeepers.toHuman().includes(hex(info.system.publicKey));
            }, 4 * 6000), 'not registered as gatekeeper');
        });

        it('can receive master key', async function () {
            // Wait for the successful dispatch of master key
            // pRuntime[2] should be down
            assert.isTrue(
                await cluster.waitWorkerExitAndRestart(2, 10 * 6000),
                'worker1 restart timeout'
            );
            const dataDir = "data";
            assert.isTrue(
                fs.existsSync(`${tmpPath}/pruntime2/${dataDir}/protected_files/master_key.seal`),
                'master key not received'
            );
        });

        it('becomes active', async function () {
            assert.isTrue(await checkUntil(async () => {
                const info = await pruntime[2].getInfo();
                return info.system?.gatekeeper.role == 2;  // 2: GatekeeperRole.Active in protobuf
            }, 1000))

            // Step 3: wait a few more blocks and ensure there are no conflicts in gatekeepers' shared mq
        });

        it('post-mines blocks', async function () {
            const gatekeeper = api.createType('MessageOrigin', 'Gatekeeper');
            let seqStart = await api.query.phalaMq.offchainIngress(gatekeeper);
            seqStart = seqStart.unwrap().toNumber();
            assert.isTrue(await checkUntil(async () => {
                let seq = await api.query.phalaMq.offchainIngress(gatekeeper);
                seq = seq.unwrap().toNumber();
                return seq >= seqStart + 1;
            }, 500 * 6000), 'ingress stale');
        });

        it('syncs latest master key', async function () {
            assert.isTrue(await checkUntil(async () => {
                const info = await pruntime[2].getInfo();
                const masterPubkey = await api.query.phalaRegistry.gatekeeperMasterPubkey();
                // console.log(`Master PubKey: ${masterPubkey}`);
                return masterPubkey == hex(info.gatekeeper.masterPublicKey);
            }, 1000));
        });
    });

    describe('Cluster & Contract', () => {
        const systemMetadata = JSON.parse(fs.readFileSync('./res/system.contract'));
        const system2Metadata = JSON.parse(fs.readFileSync('./res/prebuilt/system-v0xffff.contract'));
        const checkerMetadata = JSON.parse(fs.readFileSync('./res/check_system.contract'));
        const indeterminMetadata = JSON.parse(fs.readFileSync('./res/indeterministic_functions.contract'));
        const quickjsMetadata = JSON.parse(fs.readFileSync('./res/prebuilt/qjs.contract'));
        const sidevmCode = fs.readFileSync('./res/check_system.sidevm.wasm');
        const contract = checkerMetadata.source;
        const codeHash = hex(contract.hash);
        const initSelector = hex('0xed4b9d1b'); // for default() function
        const txConfig = { gasLimit: "10000000000000", storageDepositLimit: null };

        let certAlice;
        let ContractSystemChecker;
        let ContractSystem;
        let certBob;

        let clusterId;
        let registry;

        before(async () => {
            certAlice = await Phala.signCertificate({ api, pair: alice });
            certBob = await Phala.signCertificate({ api, pair: bob });
        });

        after(async () => {
            for (const contract of [ContractSystem, ContractSystemChecker]) {
                if (contract) {
                    await contract.api.disconnect();
                }
            }
        });

        it('can upload system code', async function () {
            const systemCode = systemMetadata.source.wasm;
            await assert.txAccepted(
                api.tx.sudo.sudo(api.tx.phalaPhatContracts.setPinkRuntimeVersion([1, 2])),
                alice,
            );
            await assert.txAccepted(
                api.tx.sudo.sudo(api.tx.phalaPhatContracts.setPinkSystemCode(systemCode)),
                alice,
            );
            assert.isTrue(await checkUntil(async () => {
                let code = await api.query.phalaPhatContracts.pinkSystemCode();
                return code[1] == systemCode;
            }, 4 * 6000), 'upload system code failed');
        });

        it('can create cluster', async function () {
            const perm = api.createType('ClusterPermission', { 'OnlyOwner': alice.address });
            const runtime0 = await pruntime[0].getInfo();
            const runtime1 = await pruntime[1].getInfo();
            await assert.txAccepted(
                api.tx.sudo.sudo(
                    api.tx.phalaPhatContracts.addCluster(
                        alice.address,
                        perm,
                        [hex(runtime0.system.publicKey), hex(runtime1.system.publicKey)],
                        CENTS * 100, 1, 1, 1, alice.address
                    )),
                alice,
            );

            assert.isTrue(await checkUntil(async () => {
                const clusters = await api.query.phalaPhatContracts.clusters.entries();
                clusterId = clusters[0][0].args[0].toString();
                return clusters.length == 1;
            }, 4 * 6000), 'cluster creation failed');

            assert.isTrue(await checkUntil(async () => {
                let info = await pruntime[0].getInfo();
                return info.system.numberOfClusters == 1;
            }, 4 * 6000), 'cluster creation in pruntime failed');

            const clusterInfo = await api.query.phalaPhatContracts.clusters(clusterId);
            const { systemContract } = clusterInfo.unwrap();
            assert.isTrue(await checkUntil(async () => {
                const contractInfo = await api.query.phalaPhatContracts.contracts(systemContract.toHex());
                return contractInfo.isSome;
            }, 4 * 6000), 'system contract instantiation failed');
            // ContractSystem = await createContractApi(api, pruntime[0].uri, systemContract, systemMetadata);
            const systemContractId = systemContract
            registry = await Phala.OnChainRegistry.create(api, { clusterId, pruntimeURL: pruntime[0].uri, systemContractId, workerId: runtime0.system.publicKey, skipCheck: true })
            registry.clusterInfo = { ...clusterInfo.toJSON(), gasPrice: new BN(1) }
            const contractKey = await registry.getContractKeyOrFail(systemContractId)
            ContractSystem = new Phala.PinkContractPromise(api, registry, systemMetadata, systemContractId, contractKey)
        });

        it('can generate cluster key', async function () {
            assert.isTrue(await checkUntil(async () => {
                const clusterKey = await api.query.phalaRegistry.clusterKeys(clusterId);
                return clusterKey.isSome;
            }, 4 * 6000), 'cluster pubkey not uploaded');
        });

        it('can deploy cluster to multiple workers', async function () {
            assert.isTrue(await checkUntil(async () => {
                const clusterWorkers = await api.query.phalaPhatContracts.clusterWorkers(clusterId);
                return clusterWorkers.length == 2;
            }, 4 * 6000), 'cluster not deployed');
        });

        it('can upload code with access control', async function () {
            const code = hex(contract.wasm);
            const codeHash = hex(contract.hash);

            await assert.txAccepted(
                api.tx.phalaPhatContracts.clusterUploadResource(clusterId, 'InkCode', hex(code)),
                alice,
            );

            assert.isTrue(await checkUntil(async () => {
                const { output } = await ContractSystem.query['system::codeExists'](alice.address, { cert: certAlice }, codeHash, 'Ink');
                return output?.eq({ Ok: true })
            }, 4 * 6000), 'Upload system checker code failed');

            await assert.txFailed(
                api.tx.phalaPhatContracts.clusterUploadResource(clusterId, 'InkCode', hex(code)),
                bob,
            )
        });

        it('can instantiate contract with access control', async function () {
            const codeIndex = api.createType('CodeIndex', { 'WasmCode': codeHash });
            const { events } = await assert.txAccepted(
                api.tx.phalaPhatContracts.instantiateContract(codeIndex, initSelector, 0, clusterId, 0, "10000000000000", null, 0),
                alice,
            );
            assertEvents(events, [
                ['balances', 'Withdraw'],
                ['phalaPhatContracts', 'Instantiating']
            ]);

            const { event } = events[1];
            let contractId = hex(event.toJSON().data[0]);
            let contractInfo = await api.query.phalaPhatContracts.contracts(contractId);
            assert.isTrue(contractInfo.isSome, 'no contract info');

            assert.isTrue(await checkUntil(async () => {
                let key = await api.query.phalaRegistry.contractKeys(contractId);
                return key.isSome;
            }, 4 * 6000), 'contract key generation failed');

            assert.isTrue(await checkUntil(async () => {
                let clusterContracts = await api.query.phalaPhatContracts.clusterContracts(clusterId);
                // A system contract and the user deployed one.
                return clusterContracts.length == 2;
            }, 4 * 6000), 'instantiation failed');

            // ContractSystemChecker = await createContractApi(api, pruntime[0].uri, contractId, checkerMetadata);
            const contractKey = await registry.getContractKeyOrFail(contractId)
            ContractSystemChecker = new Phala.PinkContractPromise(api, registry, checkerMetadata, contractId, contractKey)
        });

        it('can upgrade runtime', async function () {
            const info = await pruntime[0].getInfo();
            const maxVersion = info.maxSupportedPinkRuntimeVersion.split('.').map(Number);
            await assert.txAccepted(
                ContractSystem.tx['system::upgradeRuntime'](txConfig, maxVersion),
                alice,
            );
            assert.isTrue(await checkUntil(async () => {
                const { output } = await ContractSystemChecker.query['runtimeVersion'](alice.address, { cert: certAlice });
                return output?.eq({ Ok: maxVersion })
            }, 4 * 6000), 'Upgrade runtime failed');
        });

        it('can not set hook without admin permission', async function () {
            // Give some money to the ContractSystemChecker to run the on_block_end
            await assert.txAccepted(
                api.tx.phalaPhatContracts.transferToCluster(CENTS * 1000, clusterId, ContractSystemChecker.address),
                alice,
            );
            await assert.txAccepted(
                ContractSystemChecker.tx.setHook(txConfig, "1000000000000"),
                alice,
            );
            assert.isFalse(await checkUntil(async () => {
                const { output } = await ContractSystemChecker.query.onBlockEndCalled(alice.address, { cert: certAlice });
                return output.asOk.valueOf();
            }, 6000 * 2), 'Set hook should not success without granting admin first');
        });

        it('can set hook with admin permission', async function () {
            await assert.txAccepted(
                ContractSystem.tx['system::grantAdmin'](txConfig, ContractSystemChecker.address),
                alice,
            );
            await assert.txAccepted(
                ContractSystemChecker.tx.setHook(txConfig, "1000000000000"),
                alice,
            );
            assert.isTrue(await checkUntil(async () => {
                const { output } = await ContractSystemChecker.query.onBlockEndCalled(bob.address, { cert: certBob });
                return output.asOk.valueOf();
            }, 2 * 6000), 'Set hook should success after granted admin');
        });

        it('can eval JavaScript in contract delegate call', async function () {
            const code = quickjsMetadata.source.wasm;
            const codeHash = quickjsMetadata.source.hash;
            await assert.txAccepted(
                api.tx.phalaPhatContracts.clusterUploadResource(clusterId, 'IndeterministicInkCode', hex(code)),
                alice,
            );
            assert.isTrue(await checkUntil(async () => {
                const { output } = await ContractSystem.query['system::codeExists'](alice.address, { cert: certAlice }, codeHash, 'Ink');
                return output?.eq({ Ok: true })
            }, 4 * 6000), 'Upload qjs code failed');
            {
                // args works
                const jsCode = `
                (function(){
                    console.log("Hello, World!");
                    return scriptArgs[0];
                })()
                `;
                const arg0 = "Powered by QuickJS in ink!";
                const { output } = await ContractSystemChecker.query.evalJs(alice.address, { cert: certAlice }, codeHash, jsCode, [arg0]);
                assert.isTrue(output?.eq({ Ok: { Ok: { String: arg0 } } }));
            }

            {
                // Can return bytes
                const jsCode = `
                (function(){
                    return new Uint8Array([1, 2, 3]);
                })()
                `;
                const { output } = await ContractSystemChecker.query.evalJs(alice.address, { cert: certAlice }, codeHash, jsCode, []);
                assert.isTrue(output?.eq({ Ok: { Ok: { Bytes: "0x010203" } } }), "Failed to return bytes");
            }
        });
        it('can parse json in contract delegate call', async function () {
            const code = indeterminMetadata.source.wasm;
            const codeHash = indeterminMetadata.source.hash;
            await assert.txAccepted(
                api.tx.phalaPhatContracts.clusterUploadResource(clusterId, 'IndeterministicInkCode', hex(code)),
                alice,
            );
            assert.isTrue(await checkUntil(async () => {
                const { output } = await ContractSystem.query['system::codeExists'](alice.address, { cert: certAlice }, codeHash, 'Ink');
                return output?.eq({ Ok: true })
            }, 4 * 6000), 'Upload test contract code failed');
            const { output } = await ContractSystemChecker.query.parseUsd(alice.address, { cert: certAlice }, codeHash, '{"usd":1.23}');
            assert.isTrue(output?.asOk.unwrap()?.usd.eq(123));
        });

        it('tokenomic driver works', async function () {
            {
                // Should be unable to use local cache before staking
                const { output } = await ContractSystemChecker.query.cacheSet(alice.address, { cert: certAlice }, "0xdead", "0xbeef");
                assert.isFalse(output.asOk.valueOf());
            }
            await assert.txAccepted(
                ContractSystem.tx['system::setDriver'](txConfig, "ContractDeposit", ContractSystemChecker.address),
                alice,
            );
            const weight = 10;
            await assert.txAccepted(
                api.tx.phalaPhatTokenomic.adjustStake(ContractSystemChecker.address, weight * CENTS),
                alice,
            );
            assert.isTrue(await checkUntil(async () => {
                const info = await pruntime[0].getContractInfo(ContractSystemChecker.address.toHex());
                return info?.weight == weight;
            }, 4 * 6000), 'Failed to apply deposit to contract weight');
            {
                // Should be able to use local cache after staking
                {
                    const { output } = await ContractSystemChecker.query.cacheSet(alice.address, { cert: certAlice }, "0xdead", "0xbeef");
                    assert.isTrue(output.asOk.valueOf());
                }
                {
                    const { output } = await ContractSystemChecker.query.cacheGet(alice.address, { cert: certAlice }, "0xdead");
                    assert.isTrue(output.asOk.isSome);
                    assert.equal(output.asOk.unwrap(), "0xbeef");
                }
            }
        });

        it('can set the sidevm as pending state without code uploaded', async function () {
            const { output } = await ContractSystemChecker.query.startSidevm(alice.address, { cert: certAlice });
            assert.isTrue(output.asOk.valueOf());
            assert.isTrue(await checkUntil(async () => {
                const info = await pruntime[0].getContractInfo(ContractSystemChecker.address.toHex());
                return info?.sidevm?.state == 'stopped';
            }, 1000), "The sidevm instance wasn't created");
        });

        it('can upload sidevm code via pRPC', async function () {
            await pruntime[0].uploadSidevmCode(ContractSystemChecker.address, sidevmCode);
            const info = await pruntime[0].getContractInfo(ContractSystemChecker.address.toHex());
            assert.equal(info?.sidevm?.state, 'running');
        });

        it('can send batch http request', async function () {
            const info = await pruntime[0].getInfo();
            const url = `${pruntime[0].uri}/info`;
            const urls = [url, url];
            const { output } = await ContractSystemChecker.query.batchHttpGet(alice.address, { cert: certAlice }, urls, 1000);
            const responses = output.asOk.valueOf();
            assert.equal(responses.length, urls.length);
            responses.forEach(([code, body]) => {
                assert.equal(code, 200);
                assert.equal(JSON.parse(body).system.public_key, info.system.publicKey);
            });
        });

        it('cannot dup-instantiate', async function () {
            const codeIndex = api.createType('CodeIndex', { 'WasmCode': codeHash });
            await assert.txFailed(
                api.tx.phalaPhatContracts.instantiateContract(codeIndex, initSelector, 0, clusterId, 0, "10000000000000", null, 0),
                alice,
            );
        });

        it('can upload the second system contract', async function () {
            const systemCode = system2Metadata.source.wasm;
            await assert.txAccepted(
                api.tx.sudo.sudo(api.tx.phalaPhatContracts.setPinkSystemCode(systemCode)),
                alice,
            );
            assert.isTrue(await checkUntil(async () => {
                let code = await api.query.phalaPhatContracts.pinkSystemCode();
                return code[1] == systemCode;
            }, 4 * 6000), 'upload system code failed');
        });

        it('can upgrade system contract', async function () {
            await assert.txAccepted(
                ContractSystem.tx['system::upgradeSystemContract'](txConfig),
                alice,
            );
            assert.isTrue(await checkUntil(async () => {
                const { output } = await ContractSystem.query['system::version'](alice.address, { cert: certAlice });
                return output?.eq({ Ok: [0xffff, 0, 0] })
            }, 4 * 6000), 'Upgrade system failed');
        });


        it('can add worker to cluster', async function () {
            const info = await pruntime[4].getInfo();
            const { events } = await assert.txAccepted(
                api.tx.phalaPhatContracts.addWorkerToCluster(hex(info.system?.publicKey), hex(clusterId)),
                alice,
            );
            assertEvents(events, [
                ['balances', 'Withdraw'],
                ['phalaPhatContracts', 'WorkerAddedToCluster']
            ]);
            assert.isTrue(await checkUntil(async () => {
                const workerCluster = await api.query.phalaPhatContracts.clusterByWorkers(hex(info.system?.publicKey));
                return workerCluster.eq(clusterId);
            }, 4 * 6000), 'Failed to add worker to cluster');
            pherry[4].kill();
            const req = await pruntime[4].rpc.generateClusterStateRequest({});
            await sleep(5000);
            console.log(`Saving cluster state`);
            let state;
            for (let i = 0; i < 5; i++) {
                try {
                    state = await pruntime[0].rpc.saveClusterState(req);
                    break;
                } catch (e) {
                    if (e.message?.includes('RCU in progress')) {
                        console.log(`RCU in progress, retrying...`);
                        await sleep(500);
                    } else {
                        console.error(e);
                        break;
                    }
                }
            }
            assert.isTrue(state.filename.startsWith('cluster-'));
            const url = `${pruntime[0].uri}/download/${state.filename}`;
            const destDir = cluster.workers[4].dirs.storageDir;
            console.log(`Downloading ${url} to ${destDir}`);
            await downloadFile(url, destDir, state.filename);
            console.log(`Loading cluster state`);
            await pruntime[4].rpc.loadClusterState(state);
            console.log(`Restarting pherry`);
            pherry[4].start();
            await sleep(1000);
            const clusterInfo = await pruntime[0].rpc.getClusterInfo({});
            assert.equal(clusterInfo?.info?.id, clusterId);
            assert.isTrue(clusterInfo?.info?.numberOfContracts > 0);
        });

        it('can destory cluster', async function () {
            await assert.txAccepted(
                api.tx.sudo.sudo(api.tx.phalaPhatContracts.clusterDestroy(clusterId)),
                alice,
            );
            assert.isTrue(await checkUntil(async () => {
                return cluster.workers[0].processPRuntime.stopped;
            }, 4 * 6000), 'destroy cluster failed');
        });
    });

    describe('Workerkey handover', () => {
        it('can handover worker key', async function () {
            await cluster.launchKeyHandoverAndWait();

            const workers = cluster.key_handover_cluster.workers.map(w => w.api);
            const server_info = await workers[0].getInfo();

            assert.isTrue(await checkUntil(async () => {
                const dataDir = "data";
                return fs.existsSync(`${tmpPath}/pruntime_key_client/${dataDir}/protected_files/runtime-data.seal`);
            }, 6000), 'key not received');

            await cluster.restartKeyHandoverClient();
            assert.isTrue(await checkUntil(async () => {
                // info.publicKey can be null since client worker will be initiated after the finish of server worker syncing
                let info = await workers[1].getInfo();
                return info.system?.publicKey != null
                    && hex(info.system.publicKey) == hex(server_info.system?.publicKey);
            }, 6000), 'key handover failed');
        });
    });

    describe.skip('Solo mining workflow', () => {
        let miner;
        before(function () {
            miner = keyring.addFromUri(mnemonicGenerate());
        })

        // worker registered

        it.skip('can bind the worker', async function () {
            const workerInfo = await api.query.phalaRegistry.worker(workerKey);
            const operator = workerInfo.unwrap().operator.unwrap();
            assert.equal(operator.toHuman(), alice.address, 'bad operator');

            await assert.txAccepted(api.tx.phalaMining.bind(workerKey), alice);

            let actualWorker = await api.query.phalaMining.minerBinding(alice.address);
            let actualMiner = await api.query.phalaMining.workerBinding(workerKey);

            actualWorker = Uint8Array.from(actualWorker.unwrap());
            actualMiner = actualMiner.unwrap();
            assert.deepEqual(actualWorker, workerKey, 'wrong bounded worker');
            assert.equal(actualMiner.toHuman(), alice.address, 'wrong bounded miner');

            let minerInfo = await api.query.phalaMining.miners(alice.address);
            assert.isTrue(minerInfo.isSome, 'miner entity should exist');

            minerInfo = minerInfo.unwrap();
            assert.isTrue(
                minerInfo.state.isReady,
                'miner bounded to worker must be in Ready state'
            );
        })

        it('can deposite some stake')

        it.skip('can start mining', async function () {
            await assert.txAccepted(api.tx.phalaMining.startMining(), alice);
            let minerInfo = await api.query.phalaMining.miners(alice.address);
            minerInfo = minerInfo.unwrap();
            assert.isTrue(minerInfo.state.isIdle, 'miner state should be MiningIdle');
        })

        it('triggers a heartbeat and wait for payout')
        it('goes to MiningUnresponsive state if offline')
        it('can stop mining')
        it('settles after the cooling down period')
    })

    testPruntimeManagement(tmpPath);
});

function testPruntimeManagement(workDir) {
    describe("PRuntime management", function () {
        this.timeout(120000);

        let cluster;
        let api, keyring, alice;
        let worker;
        let tmpPath;
        let test_case = 0;

        before(async () => {
            // Create polkadot api and keyring
            await cryptoWaitReady();
            keyring = new Keyring({ type: 'sr25519', ss58Format: 30 });
            root = alice = keyring.addFromUri('//Alice');
            bob = keyring.addFromUri('//Bob');
        });

        beforeEach(async () => {
            tmpPath = path.resolve(`${workDir}/pruntime_life_case${test_case}`);
            fs.mkdirSync(tmpPath);
            // Check binary files
            [pathNode, pathRelayer, pathPRuntime].map(fs.accessSync);
            // Bring up a cluster
            cluster = new Cluster(1, pathNode, pathRelayer, pathPRuntime, tmpPath);
            await cluster.start();
            // APIs
            api = await cluster.api;
            worker = cluster.workers[0];

            assert.isFalse(cluster.processNode.stopped);
            for (const w of cluster.workers) {
                assert.isFalse(w.processRelayer.stopped);
                assert.isFalse(w.processPRuntime.stopped);
            }

            let info;
            assert.isTrue(await checkUntil(async () => {
                info = await worker.api.getInfo();
                return info.initialized;
            }, 1000), 'not initialized in time');
            assert.isTrue(await checkUntil(async () => {
                const info = await worker.api.getInfo();
                return info.blocknum > 0;
            }, 7000), 'stuck at block 0');

            info = await worker.api.getInfo();
        });

        afterEach(async () => {
            test_case += 1;
            if (api) await api.disconnect();
            await cluster.kill();
        });

        it("can retire old pruntimes", async () => {
            await assert.txAccepted(
                api.tx.sudo.sudo(
                    api.tx.phalaRegistry.setMinimumPruntimeVersion(
                        999,
                        0,
                        0,
                    )
                ),
                alice,
            );

            assert.equal(
                true,
                await checkUntil(
                    async () => {
                        return worker.processPRuntime.stopped;
                    },
                    6000
                ),
                "Failed to retire old version pruntime"
            );
        });

        it("can not set consensus version over max supported", async () => {
            const { maxSupportedConsensusVersion } = (await worker.api.getInfo()).system;
            await assert.txAccepted(
                api.tx.sudo.sudo(
                    api.tx.phalaRegistry.setPruntimeConsensusVersion(
                        maxSupportedConsensusVersion + 1
                    )
                ),
                alice,
            );
            assert.isTrue(
                await checkUntil(
                    async () => {
                        return worker.processPRuntime.stopped;
                    },
                    6000
                ),
                'Failed to wait pruntime to exit'
            );
        });
    });
}

async function assertSubmission(txBuilder, signer, shouldSucceed = true) {
    return await new Promise(async (resolve, _reject) => {
        const unsub = await txBuilder.signAndSend(signer, { nonce: -1 }, (result) => {
            if (result.status.isInBlock) {
                let error;
                for (const e of result.events) {
                    const { event: { data, method, section } } = e;
                    if (section === 'system' && method === 'ExtrinsicFailed') {
                        if (shouldSucceed) {
                            error = data[0];
                        } else {
                            unsub();
                            resolve(error);
                        }
                    }
                }
                if (error) {
                    assert.fail(`Extrinsic failed with error: ${error}`);
                }
                unsub();
                resolve({
                    hash: result.status.asInBlock,
                    events: result.events,
                });
            } else if (result.status.isInvalid) {
                assert.fail('Invalid transaction');
                unsub();
                resolve();
            }
        });
    });
}
assert.txAccepted = assertSubmission;
assert.txFailed = (txBuilder, signer) => assertSubmission(txBuilder, signer, false);

function fillPartialArray(obj, pattern) {
    for (const [idx, v] of obj.entries()) {
        if (pattern[idx] === undefined) {
            pattern[idx] = v;
        } else if (v instanceof Array && pattern[idx] instanceof Array) {
            fillPartialArray(v, pattern[idx]);
        }
    }
}

function assertEvents(actualEvents, expectedEvents, partial = true) {
    const simpleEvents = simplifyEvents(actualEvents, true);
    if (partial) {
        fillPartialArray(simpleEvents, expectedEvents);
    }
    assert.deepEqual(simpleEvents, expectedEvents, 'Events not equal');
}

function simplifyEvents(events, keepData = false) {
    const simpleEvents = [];
    for (const e of events) {
        const { event } = e;
        const { method, section } = event;
        if (keepData) {
            simpleEvents.push([section, method, event.toJSON().data]);
        } else {
            simpleEvents.push([section, method]);
        }
    }
    return simpleEvents;
}

class Cluster {
    constructor(numWorkers, pathNode, pathRelayer, pathPRuntime, tmpPath) {
        this.numWorkers = numWorkers;
        this.pathNode = pathNode;
        this.pathRelayer = pathRelayer;
        this.pathPRuntime = pathPRuntime;
        this.tmpPath = tmpPath;
        [pathNode, pathRelayer, pathPRuntime].map(fs.accessSync);
        // Prepare empty workers
        const workers = [];
        for (let i = 0; i < this.numWorkers; i++) {
            workers.push({});
        }
        this.workers = workers;
        this.key_handover_cluster = {
            workers: [{}, {}],
            relayer: {},
        };
    }

    async start() {
        await this._reservePorts();
        this._createProcesses();
        await this._launchAndWait();
        await this._createApi();
    }

    async kill() {
        await Promise.all([
            this.processNode.kill(),
            ...this.workers.map(w => [
                w.processPRuntime.kill('SIGKILL'),
                w.processRelayer.kill()
            ]).flat(),
        ]);

        if (this.key_handover_cluster.relayer.processRelayer != undefined) {
            await Promise.all([
                this.key_handover_cluster.relayer.processRelayer.kill(),
                ...this.key_handover_cluster.workers.map(w => w.processPRuntime.kill('SIGKILL')).flat(),
            ]);
        }
    }

    // Returns false if waiting is timeout; otherwise it restart the specified worker
    async waitWorkerExitAndRestart(i, timeout) {
        const w = this.workers[i];
        const succeed = await checkUntil(async () => {
            return w.processPRuntime.stopped && w.processRelayer.stopped
        }, timeout);
        if (!succeed) {
            return false;
        }
        this._createWorkerProcess(i);
        await waitPRuntimeOutput(w.processPRuntime);
        await waitRelayerOutput(w.processRelayer);
        return true;
    }

    async _reservePorts() {
        const [wsPort, ...workerPorts] = await Promise.all([
            portfinder.getPortPromise({ host: "127.0.0.1", port: 9944 }),
            ...this.workers.map((w, i) => portfinder.getPortPromise({ host: "127.0.0.1", port: 8100 + i * 10 }))
        ]);
        this.wsPort = wsPort;
        this.workers.forEach((w, i) => w.port = workerPorts[i]);
    }

    _createProcesses() {
        this.processNode = newNode(this.wsPort, this.tmpPath, 'node');
        this.workers.forEach((_, i) => {
            this._createWorkerProcess(i);
        })
        this.processes = [
            this.processNode,
            ...this.workers.map(w => [w.processRelayer, w.processPRuntime]).flat()
        ];
    }

    _createWorkerProcess(i) {
        const AVAILABLE_ACCOUNTS = [
            '//Bob',
            '//Charlie',
            '//Dave',
            '//Eve',
            '//Ferdie',
        ];
        const w = this.workers[i];
        const gasAccountKey = AVAILABLE_ACCOUNTS[i];
        const key = '0'.repeat(63) + (i + 1).toString();
        w.processRelayer = newRelayer(this.wsPort, w.port, this.tmpPath, gasAccountKey, key, `relayer${i}`);
        w.processPRuntime = newPRuntime(w.port, this.tmpPath, `pruntime${i}`);
        w.dirs = pRuntimeDirs(this.tmpPath, `pruntime${i}`);
    }

    async launchKeyHandoverAndWait() {
        const cluster = this.key_handover_cluster;

        const [...workerPorts] = await Promise.all([
            ...cluster.workers.map((w, i) => portfinder.getPortPromise({ host: "127.0.0.1", port: 8200 + i * 10 }))
        ]);
        cluster.workers.forEach((w, i) => w.port = workerPorts[i]);

        const server = cluster.workers[0];
        const client = cluster.workers[1];
        server.processPRuntime = newPRuntime(server.port, this.tmpPath, `pruntime_key_server`);
        client.processPRuntime = newPRuntime(client.port, this.tmpPath, `pruntime_key_client`);

        const gasAccountKey = '//Ferdie';
        const key = '0'.repeat(62) + '10';
        cluster.relayer.processRelayer = newRelayer(this.wsPort, server.port, this.tmpPath, gasAccountKey, key, `pruntime_key_relayer`, client.port);

        await Promise.all([
            ...cluster.workers.map(w => waitPRuntimeOutput(w.processPRuntime)),
        ]);
        await waitRelayerOutput(cluster.relayer.processRelayer);

        cluster.workers.forEach(w => {
            w.api = new PRuntimeApi(`http://localhost:${w.port}`);
        })
    }

    async restartKeyHandoverClient() {
        const cluster = this.key_handover_cluster;
        await checkUntil(async () => {
            return cluster.relayer.processRelayer.stopped
        }, 6000);

        const client = cluster.workers[1];
        const gasAccountKey = '//Ferdie';
        cluster.relayer.processRelayer = newRelayer(this.wsPort, client.port, this.tmpPath, gasAccountKey, '', `pruntime_key_relayer`);

        await waitRelayerOutput(cluster.relayer.processRelayer);
    }

    // Returns false if waiting is timeout; otherwise it restarts the pherry and the key handover client
    async waitKeyHandoverClientExitAndRestart(timeout) {
        const w = this.cluster.workers[1];
        const succeed = await checkUntil(async () => {
            return w.processPRuntime.stopped && w.processRelayer.stopped
        }, timeout);
        if (!succeed) {
            return false;
        }
        const client = cluster.workers[1];
        client.processPRuntime = newPRuntime(client.port, this.tmpPath, `pruntime_key_client`);
        // connect the pherry to the new pRuntime and inject no key
        cluster.relayer.processRelayer = newRelayer(this.wsPort, client.port, this.tmpPath, gasAccountKey, '', `pruntime_key_relayer`);
        await waitPRuntimeOutput(client.processPRuntime);
        await waitRelayerOutput(cluster.relayer.processRelayer);
        return true;
    }

    async _launchAndWait() {
        // Launch nodes & pruntime
        await Promise.all([
            waitNodeOutput(this.processNode),
            ...this.workers.map(w => waitPRuntimeOutput(w.processPRuntime)),
        ]);
        // Launch relayers
        await Promise.all(this.workers.map(w => waitRelayerOutput(w.processRelayer)));
    }

    async _createApi() {
        this.api = await ApiPromise.create({
            provider: new WsProvider(`ws://localhost:${this.wsPort}`),
            types: { ...types, ...typeDefinitions, ...Phala.types, ...typeOverrides },
            typeAlias
        });
        this.workers.forEach(w => {
            w.api = new PRuntimeApi(`http://localhost:${w.port}`);
        })
    }

}

function waitPRuntimeOutput(p) {
    return p.startAndWaitForOutput(/Rocket has launched from/);
}
function waitRelayerOutput(p) {
    return p.startAndWaitForOutput(/runtime_info: InitRuntimeResp/);
}
function waitNodeOutput(p) {
    return p.startAndWaitForOutput(/Imported #1/);
}


function newNode(rpcPort, tmpPath, name = 'node') {
    const cli = [
        pathNode, [
            '--dev',
            '--block-millisecs=1000',
            '--base-path=' + path.resolve(tmpPath, 'phala-node'),
            `--rpc-port=${rpcPort}`,
            '--rpc-methods=Unsafe',
            '--pruning=archive',
        ]
    ];
    const cmd = cli.flat().join(' ');
    fs.writeFileSync(`${tmpPath}/start-${name}.sh`, `#!/bin/bash\n${cmd}\n`, { encoding: 'utf-8' });
    return new Process(cli, { logPath: `${tmpPath}/${name}.log` });
}

function pRuntimeDirs(tmpPath, name) {
    const workDir = path.resolve(`${tmpPath}/${name}`);
    const sealDir = path.resolve(`${workDir}/data`);
    const protectedDataDir = path.resolve(`${sealDir}/protected_files/`);
    const storageDir = path.resolve(`${sealDir}/storage_files/`);
    return { workDir, sealDir, protectedDataDir, storageDir };
}

function newPRuntime(teePort, tmpPath, name = 'app') {
    const { workDir, protectedDataDir, storageDir } = pRuntimeDirs(tmpPath, name);
    if (!fs.existsSync(workDir)) {
        fs.cpSync(pRuntimeDir, workDir, { recursive: true })
        if (inSgx) {
            fs.mkdirSync(protectedDataDir, { recursive: true });
            fs.mkdirSync(storageDir, { recursive: true });
        }
    }
    const args = [
        '--cores=0',  // Disable benchmark
        '--checkpoint-interval=5',
        '--port', teePort.toString(),
    ];
    let bin = pRuntimeBin;
    if (inSgx) {
        bin = sgxLoader;
        args.splice(0, 0, pRuntimeBin);
    }
    return new Process([
        `${workDir}/${bin}`, args, {
            cwd: workDir,
            env: {
                ...process.env,
                ROCKET_PORT: teePort.toString(),
                RUST_LOG: 'debug'
            }
        }
    ], { logPath: `${tmpPath}/${name}.log` });
}

function newRelayer(wsPort, teePort, tmpPath, gasAccountKey, key = '', name = 'relayer', keyClientPort = '') {
    const args = [
        '--no-wait',
        `--mnemonic=${gasAccountKey}`,
        `--substrate-ws-endpoint=ws://localhost:${wsPort}`,
        `--pruntime-endpoint=http://localhost:${teePort}`,
        '--dev-wait-block-ms=1000',
        '--attestation-provider', 'none',
    ];

    if (key) {
        args.push(`--inject-key=${key}`);
    }
    if (keyClientPort) {
        args.push(`--next-pruntime-endpoint=http://localhost:${keyClientPort}`);
    }

    return new Process([
        pathRelayer, args
    ], { logPath: `${tmpPath}/${name}.log` });
}

function hex(b) {
    if (!b.startsWith('0x')) {
        return '0x' + b;
    } else {
        return b;
    }
}

async function createContractApi(api, pruntimeURL, contractId, metadata) {
    const newApi = await api.clone().isReady;
    const phala = await Phala.create({ api: newApi, baseURL: pruntimeURL, contractId, autoDeposit: true });
    return new ContractPromise(
        phala.api,
        metadata,
        contractId,
    );
}

async function downloadFile(url, directory, filename) {
    const response = await axios({
        method: 'get',
        url: url,
        responseType: 'stream'
    });

    const filePath = path.join(directory, filename);

    const writer = fs.createWriteStream(filePath);
    response.data.pipe(writer);

    return new Promise((resolve, reject) => {
        writer.on('finish', resolve);
        writer.on('error', reject);
    });
}

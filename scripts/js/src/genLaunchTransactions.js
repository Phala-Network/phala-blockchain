require('dotenv').config();

const { ApiPromise, WsProvider } = require('@polkadot/api');

const tokenomic  = require('./utils/tokenomic');

const khala = {
    // The allowed relay starting hashes for pRuntime to start syncing
    // - 0: The Kusama Genesis
    // - 8325311: The Kusama block right before the first parachain parent block.
    relayGenesisHashes: [
        [0, '0xb0a8d493285c2df73290dfb7e61f870f17b41801197a149ca93654499ea3dafe'],
        [8325311, '0xff93a4a903207ad45af110a3e15f8b66c903a0045f886c528c23fe7064532b08'],
    ],
    // The pRuntime hashes, signed by Phala's Intel SGX IAS key.
    pruntimeHashes: [
        '0x2099244f418ee770f71173f99956c23873f25f65c874082040010dff0a027a8300000000815f42f11cf64430c30bab7816ba596a1da0130c3b028b673133a66cf9a3e0e6',
    ],
    // The worker pubkey of the team-created GK in US
    genesisGatekeeper: '0x60067697c486c809737e50d30a67480c5f0cede44be181b96f7d59bc2116a850',
    // The worker pubkey of the team-created GK in EU
    secondGatekeeper: '0xaa37d91141bb1c0d77467ac66066e3927ece5708eded765e9090a7e1dcef5b2f',
    // The Khala tokenomic parameters. The notable change is to extend the heartbeatWindow from 10 to 20.
    tokenomic: {
        "phaRate": "0.83",
        "rho": "1.000000666600231",
        "budgetPerBlock": "8.3333333333333333333",
        "vMax": "30000",
        "costK": "0.000000019054528616676159186",
        "costB": "0.00004061623205149357237",
        "slashRate": "0",
        "treasuryRatio": "0.2",
        "heartbeatWindow": "20",
        "rigK": "0.36144578313253012048",
        "rigB": "0",
        "re": "1.5",
        "k": "50",
        "kappa": "1"
    },
    enableStakePoolAt: 414189,
};

// Create a council motion proposal with a threshold of 3 (3 of 5).
function propose(api, call) {
    const lenthBound = call.toU8a().length + 10;
    return api.tx.council.propose(3, call, lenthBound);
}

async function main() {
    const wsProvider = new WsProvider(process.env.ENDPOINT);
    const api = await ApiPromise.create({ provider: wsProvider });

    // We use the Khala parameter now.
    const params = khala;

    // Motion 2:

    const motion2 = api.tx.utility.batchAll([
        // 1. Register the whitelisted pRuntime hashes
        ... params.pruntimeHashes.map(h => api.tx.phalaRegistry.addPruntime(h)),
        // 2. Register the whitelisted relayc chain genesis hashes
        ... params.relayGenesisHashes.map(([_n, blockHash]) =>
            api.tx.phalaRegistry.addRelaychainGenesisBlockHash(blockHash)
        ),
    ]);

    // Motion 3:

    const typedP = tokenomic.humanToTyped(api, params.tokenomic);
    const updateTokenomicCall = tokenomic.createUpdateCall(api, typedP);

    const motion3 = api.tx.utility.batchAll([
        // 1. Register the Genesis Gatekeeper (with its registered worker public key)
        api.tx.phalaRegistry.registerGatekeeper(params.genesisGatekeeper),
        // 2. Update the tokenomic
        updateTokenomicCall,
        // 3. Schedule the StakePool pallet to enable mining at the specific block
        api.tx.scheduler.schedule(
            params.enableStakePoolAt, null, 0,
            api.tx.phalaStakePool.setMiningEnable(true)
        ),
    ]);

    // Motion 4:
    // - Register the second Gatekeeper

    const motion4 = api.tx.phalaRegistry.registerGatekeeper(params.secondGatekeeper);

    const proposal2 = propose(api, motion2);
    const proposal3 = propose(api, motion3);
    const proposal4 = propose(api, motion4);

    console.log({
        motion2: proposal2.toHex(),
        motion3: proposal3.toHex(),
        motion4: proposal4.toHex(),
    });

    // Print the decoded proposal
    console.dir({
        call2: proposal2.toHuman(),
        call3: proposal3.toHuman(),
        call4: proposal4.toHuman(),
    }, {depth: 9})
}

main().catch(console.error).finally(() => process.exit());

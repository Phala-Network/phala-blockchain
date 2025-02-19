import { Gauge, Registry } from "https://deno.land/x/ts_prometheus@v0.3.0/mod.ts"

import * as Phala from "npm:@phala/sdk@0.5.1";

import { ApiPromise, WsProvider } from 'npm:@polkadot/api@10.9.1';
import { typeDefinitions } from "npm:@polkadot/types@10.9.1";

import BN from "npm:bn.js@5.2.1";

const phalaAccounts = {
  '436H4jat7TobTbNYLdSJ3cmNy9K4frmE4Yuc4R2nNnaf56DL': 'Reward Account',

  // '42ha5pLkXGLg4M1aFkvBsbLWbmMYvgdCRetgUjSAMZ7swG53', // QC-0
  // '42sydUvocBuEorweEPqxY5vZae1VaTtWoJFiKMrPbRamy2BL', // QC-1
  // '457agnQ9yVdbdP5J57HHaUsBmwWogFeK7JCVSKd8bs5FxxKv', // QC-2
  '44vn1X95id7xt7xnbjvXgG9E6R4hmgKzXSa4QLBpkziyXz1x': 'Doyle Home',
  // '42u5qY3Qm2vFZ13AagKpKqT3QQZnVv4yNGhy6JnK2FJP1ybX': 'Suge',
  '43cPs4dWRNwfrqzMPKFQjewvkx6djemjbzwa1ZzYZcwe2TdM': 'HE - Fremont-L',
  '44tHNY4vuTxc1PBsS3eAxFUY4avJEmx6AMcwBswr7sDH7XMC': 'HE - Fremont-R',
  // '459LKVLV2U6fdtQLQ9ZXhojEQm9ncfqN76YRbR3CiJpCL3yb', // Zhe
  //'41gCjE9gLdQ1a8WEdVnRTQ3UENndsmpMcdgW85bHJgRXziVV': 'Crosswire - GKCL',
  '3zoj1DEpKUvZdmFZtYri5WTLMeufLZCNEP8iY6VYkbUoG2i2': 'OVH - PHAT-4 - GK',

  // '45EbDsTXpy7dRwoYfA5rLT5BJ6Z3hMxU2AKQB2LYNLWoMNj6', // PRB QC
  // '43XEkxJ3VZUTQNvsXSDKsWpLQ6Y34ZWncL12SSq3cCK6Bi8h', // PRB NC
  '432CHQ6ed9fJfkrxZDr4ocZotq8gNRkefvXayJsZAPx7fKAE': 'OVH - PHAT - PRB',
};

const khalaAccounts = {
  '436H4jat7TobTbNYLdSJ3cmNy9K4frmE4Yuc4R2nNnaf56DL': 'Reward Account',

  //'44nahXbDbTT2n7dPFiwhh3VXJTVU6QeLfufZuDin8gnAmpsj': 'Crosswire - 1 Rotterdam (NL)',
  //'437f5a2y6FFT9NCKp166zSSMFxgPSq1mSWQy3xhfWwUo9Bz4': 'Crosswire - 2 Amsterdam (NL)',
  //'44Ts6ZBCu6yCnncJyFpsQAtPvUC9wgrUpwWJDPmMjEpceZYE': 'Crosswire - 3 Dusseldorf (DE)',
  //'41rFAFhxf3nUAAhZXyQh2diFb6LZvxMQnrDnw3uGuTS6GKmm': 'Crosswire - 4 Dusseldorf (DE)',
  '43cPs4dWRNwfrqzMPKFQjewvkx6djemjbzwa1ZzYZcwe2TdM': 'HE - Fremont-L',
  '44tHNY4vuTxc1PBsS3eAxFUY4avJEmx6AMcwBswr7sDH7XMC': 'HE - Fremont-R',
  // '459LKVLV2U6fdtQLQ9ZXhojEQm9ncfqN76YRbR3CiJpCL3yb', // Zhe
  '44vn1X95id7xt7xnbjvXgG9E6R4hmgKzXSa4QLBpkziyXz1x': 'Doyle Home',
  // '42ha5pLkXGLg4M1aFkvBsbLWbmMYvgdCRetgUjSAMZ7swG53', // QC-0
  // '42sydUvocBuEorweEPqxY5vZae1VaTtWoJFiKMrPbRamy2BL', // QC-1
  // '457agnQ9yVdbdP5J57HHaUsBmwWogFeK7JCVSKd8bs5FxxKv', // QC-2
  '3zoj1DEpKUvZdmFZtYri5WTLMeufLZCNEP8iY6VYkbUoG2i2': 'OVH - PHAT-4 - GK',
};

const PHALA_RPC_ENDPOINT = Deno.env.get('PHALA_RPC_ENDPOINT') ?? 'wss://priv-api.phala.network/phala/ws';
const KHALA_RPC_ENDPOINT = Deno.env.get('KHALA_RPC_ENDPOINT') ?? 'wss://priv-api.phala.network/khala/ws';

const sleep = (t) => new Promise(r => setTimeout(r, t))

export const registry = Registry.default;

const getInfoLoop = async () => {
  const api = await ApiPromise.create({
    provider: new WsProvider(PHALA_RPC_ENDPOINT),
    types: {
        ...Phala.types,
        ...typeDefinitions,
        CheckMqSequence: null,
    },
    noInitWarn: true,
  });
  const khalaApi = await ApiPromise.create({
    provider: new WsProvider(KHALA_RPC_ENDPOINT),
    types: {
        ...Phala.types,
        ...typeDefinitions,
        CheckMqSequence: null,
    },
    noInitWarn: true,
  });

  const workers = await api.query.phalaPhatContracts.clusterWorkers('0x0000000000000000000000000000000000000000000000000000000000000001');
  const accountIds = await Promise.all(workers.map(worker => api.query.phalaComputation.workerBindings(worker)));

  const workerLastChallengeTime = Gauge.with({
    name: 'worker_last_challenge_time',
    help: 'The last challenge timestamp of worker',
    labels: ['chain', 'public_key', 'state']
  });

  const freeBalance = Gauge.with({
    name: 'free_balance_in_milli',
    help: 'Free Balance',
    labels: ['chain', 'account', 'name']
  });

  const circulationGauge = Gauge.with({
    name: 'circulation',
    help: 'total circulation',
    labels: ['chain']
  });

  const tokenomicUpdatedAtGauge = Gauge.with({
    name: 'tokenomic_updated_at',
    help: 'Tokenomic Updated At',
    labels: ['chain']
  });

  while (true) {
    const sessions = await Promise.all(accountIds.map(accountId => api.query.phalaComputation.sessions(accountId.unwrap())));
    for (let i = 0; i < workers.length; i++) {
      const session = sessions[i].unwrap();
      const publicKey = workers[i].toString();
      workerLastChallengeTime.labels({
        chain: 'phala',
        public_key: publicKey,
        state: session.state,
      })
      .set(session.benchmark.challengeTimeLast);
    }

    for (const accountId in phalaAccounts) {
      const name = phalaAccounts[accountId];
      const { data: { free } } = await api.query.system.account(accountId);
      const pha = free.div(new BN('1000000000'));
      freeBalance.labels({
        chain: 'phala',
        account: accountId,
        name: name,
      })
      .set(pha.toNumber());
    }

    for (const accountId in khalaAccounts) {
      const name = khalaAccounts[accountId];
      const { data: { free } } = await khalaApi.query.system.account(accountId);
      const pha = free.div(new BN('1000000000'));
      freeBalance.labels({
        chain: 'khala',
        account: accountId,
        name: name,
      })
      .set(pha.toNumber());
    }

    await fetch_circulation("https://pha-circulation-server.vercel.app/api/all", circulationGauge);

    const phalaTokenomicUpdatedTime = await fetchTokenomicUpdatedTime('phala');
    tokenomicUpdatedAtGauge.labels({chain: 'phala'}).set(phalaTokenomicUpdatedTime);
    const khalaTokenomicUpdatedTime = await fetchTokenomicUpdatedTime('khala')
    tokenomicUpdatedAtGauge.labels({chain: 'khala'}).set(khalaTokenomicUpdatedTime);

    console.log('A round of fetch completed. Sleep 15 seconds.');
    await sleep(60_000)
  }
}

export const getInfoLoopPromise = getInfoLoop()

async function fetch_circulation(url, circulationGauge) {
  try {
    const res = await fetch(url, { method: 'GET' });
    if (res.ok) {
      const cir = await res.json();
      circulationGauge.labels({chain: 'phala'}).set(cir.phala.circulation);
      circulationGauge.labels({chain: 'khala'}).set(cir.khala.circulation);
      circulationGauge.labels({chain: 'ethereum'}).set(cir.ethereum.circulation);
      circulationGauge.labels({}).set(cir.totalCirculation);
    } else {
      console.error(`HTTP error! Status: ${res.status}`)
    }
  } catch (error) {
    console.error(error)
  }
}

async function fetchTokenomicUpdatedTime(chain) {
  const res = await fetch(
    `https://subsquid.phala.network/${chain}-computation/graphql`,
    {
      method: 'POST',
      headers: {'content-type': 'application/json'},
      body: JSON.stringify({
        query:
          'query TokenomicUpdatedTime { globalStateById(id: "0") { tokenomicUpdatedTime }}',
        operationName: 'TokenomicUpdatedTime',
      }),
    },
  )
  const json = await res.json()

  return Date.parse(json.data.globalStateById.tokenomicUpdatedTime)
}

/*
async function main() {
  const endpoint = "wss://api.phala.network/ws";
  const api = await ApiPromise.create({
    provider: new WsProvider(endpoint),
    types: {
        ...Phala.types,
        ...typeDefinitions,
        CheckMqSequence: null,
    },
    noInitWarn: true,
  })

  //const keyring = new Keyring({ type: 'sr25519' });
  //const alice = keyring.addFromUri('//Alice');

  //let { data: { free } } = await api.query.system.account('');

  //console.log(`A has ${free.div(new BN('1000000000000'))}`);

  /*
  let finalizedHead = await api.rpc.chain.getFinalizedHead();
  let block = await api.rpc.chain.getBlock(finalizedHead);
  console.log(JSON.stringify(block));

  let session = await api.query.phalaComputation.sessions('43iKVMcmJSJSHerfEh7QaVzaMwxw1i5X8gfPA3wSEYDQBgc2');
  console.log(JSON.stringify(session));

  let clusterInfo = await api.query.phalaPhatContracts.clusters('0x0000000000000000000000000000000000000000000000000000000000000001');
  let workers = clusterInfo.unwrap().workers;
  let accountIds = await Promise.all(workers.map(worker => api.query.phalaComputation.workerBindings(worker)));
  let sessions = await Promise.all(accountIds.map(accountId => api.query.phalaComputation.sessions(accountId.unwrap())));
  console.log(workers.length);
  console.log(sessions.length);
}

main().then(process.exit).catch(err => console.error('Crashed', err)).finally(() => process.exit(-1));
*/

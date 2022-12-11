require('dotenv').config();

const fs = require('fs');

const { ApiPromise, WsProvider } = require('@polkadot/api');
const { Console } = require('console');

const typedefs = require('@phala/typedefs').khalaDev;

async function main() {
    const wsProvider = new WsProvider('ws://127.0.0.1:49944');
    const api = await ApiPromise.create({
      provider: wsProvider, types: {
          ...typedefs, 
          NftAttr: {
              shares: "Balance",
          },
      }
  })


  const since = parseInt(process.env.SINCE || '500`');
  const until = parseInt(process.env.UNTIL || '501');

  for (let height = since; height < until; height++) {
    const hash = await api.rpc.chain.getBlockHash(height);
    const apiAt = await api.at(hash);

    const pooldata = await apiAt.query.phalaBasePool.pools.entries();
    const nfts = await apiAt.query.rmrkCore.nfts.entries();
    const poolitems = pooldata.map(([k, v]) => [k.args[0], v.unwrap()]);
    const nftitems = nfts.map(([k, v]) => [[k.args[0], k.args[1]], v.unwrap()]);

    let shares = new Map();
    for (let i = 0; i < nftitems.length; i++) {
      if (nftitems[i][0][0].toNumber() <= 3) {
        continue;
      }
      test = (await api.query.rmrkCore.properties(nftitems[i][0][0].toNumber(), nftitems[i][0][1].toNumber(), "stake-info")).unwrap().toHex();
      let foo = api.createType('NftAttr', test);
      let share = Number(foo.toJSON().shares);

      if (shares[nftitems[i][0][0].toNumber()] == undefined) {
        shares[nftitems[i][0][0].toNumber()] = share.toString();
      } else {
        shares[nftitems[i][0][0].toNumber()] = (Number(shares[nftitems[i][0][0].toNumber()]) + share).toString();
      }
    }

    for (let i = 0; i < poolitems.length; i++) {
      if (poolitems[i][1].toJSON()["stakePool"] == undefined) {
        continue;
      }
      let total_shares = Number(poolitems[i][1].toJSON()["stakePool"]["basepool"]["totalShares"]).toString();
      let cid = poolitems[i][1].toJSON()["stakePool"]["basepool"]["cid"];
      let sum_share = shares[cid];
      if (total_shares != sum_share && (total_shares != 0 && sum_share != undefined)) {
        console.log(total_shares + " != " + sum_share + " at pid:" + poolitems[i][0] + " for total share");
      }
    }
    let assets = await apiAt.query.assets.account.entries();
    const assetsitems = assets.map(([k, v]) => [[k.args[0], k.args[1]], v.unwrap()]);
    let assetsmap = new Map();
    for (let i = 0; i < assetsitems.length; i++) {
      if (assetsitems[i][0][0].toNumber() == 1) {
          assetsmap[assetsitems[i][0][1]] = BigInt(assetsitems[i][1].toJSON()["balance"]);
        }
      }
      
    for (let i = 0; i < poolitems.length; i++) {
      if (poolitems[i][1].toJSON()["stakePool"] == undefined) {
        continue;
      }
      let pool_account_id = poolitems[i][1].toJSON()["stakePool"]["basepool"]["pool_account_id"];
      let lock_account_id = poolitems[i][1].toJSON()["stakePool"]["lock_account"];
      let free = 0;
      if (assetsmap[pool_account_id] != undefined) {
        free = assetsmap[pool_account_id];
      }
      let lock = 0;
      if (assetsmap[lock_account_id] != undefined) {
        lock = assetsmap[lock_account_id];
      }
      let pool_total = poolitems[i][1].toJSON()["stakePool"]["basepool"]["total_value"];
      if (poolitems[i][1].toJSON()["stakePool"]["basepool"]["total_value"] == undefined) {
        pool_total = "0";
      }
      let total = (lock + free).toString();
      if (pool_total != total) {
        console.log(pool_total + " != " + total + " at pid:" + poolitems[i][0] + " for total value");
      }
    }

    const oldpooldata = await apiAt.query.phalaStakePool.stakePools.entries();
    const oldpoolitems = oldpooldata.map(([k, v]) => [k.args[0], v.unwrap()]);
    let oldpoolmap = new Map();
    for (i = 0; i < oldpoolitems.length; i++) {
      oldpoolmap[oldpoolitems[i][0]] = BigInt(oldpoolitems[i][1]["releasingStake"]);
    }
    const workerbinding = await apiAt.query.phalaComputation.workerBindings.entries();
    const workerbinditems = workerbinding.map(([k, v]) => [k.args[0], v.unwrap()]);
    let bindmap = new Map();
    for (i = 0; i < workerbinditems.length; i++) {
      bindmap[workerbinditems[i][0]] = workerbinditems[i][1].toJSON();
    }
    const stakes = await apiAt.query.phalaComputation.stakes.entries();
    const stakeitems = stakes.map(([k, v]) => [k.args[0], v.unwrap()]);
    let stakemap = new Map();
    for (i = 0; i < stakeitems.length; i++) {
      stakemap[stakeitems[i][0]] = BigInt(stakeitems[i][1]);
    }
    for (let i = 0; i < poolitems.length; i++) {
      if (poolitems[i][1].toJSON()["stakePool"] == undefined) {
        continue;
      }
      let cd_workers = poolitems[i][1].toJSON()["stakePool"]["cdWorkers"];
      let sum_releasing = BigInt(0);
      for (let j = 0; j < cd_workers.length; j++) {
        let session_id = bindmap[cd_workers[j]];
        let stake = stakemap[session_id];
        sum_releasing += stake;
      }
      let pid = poolitems[i][0];
      let releasing = oldpoolmap[pid];

      if (releasing != sum_releasing) {
        console.log(releasing + " != " + sum_releasing + " at pid:" + poolitems[i][0] + " for releasing stakes");
      }
    }

  }
}

main().catch(console.error).finally(() => process.exit());

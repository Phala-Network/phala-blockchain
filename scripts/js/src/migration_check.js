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


  const since = parseInt(process.env.SINCE || '137');
  const until = parseInt(process.env.UNTIL || '138');

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
        console.log(total_shares + " != " + sum_share + " at pid:" + poolitems[i][0]);
      }
    }
  }
    
}

main().catch(console.error).finally(() => process.exit());

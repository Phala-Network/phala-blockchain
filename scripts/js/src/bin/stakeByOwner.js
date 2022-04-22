const fetch = require('node-fetch')
const Papa = require('papaparse');

// console.log("{\"query\":\"\\n    query StakePools($where: StakePoolsWhereInput, $orderBy: [StakePoolsOrderByWithRelationInput!], $take: Int, $skip: Int, $withStakePoolStakers: Boolean = false, $stakePoolStakersWhere: StakePoolStakersWhereInput, $withStakePoolWithdrawals: Boolean = false, $stakePoolWithdrawalsWhere: StakePoolWithdrawalsWhereInput, $withMiners: Boolean = false, $minersWhere: MinersWhereInput) {\\n  findManyStakePools(where: $where, orderBy: $orderBy, take: $take, skip: $skip) {\\n    pid\\n    ownerAddress\\n    commission\\n    ownerReward\\n    cap\\n    rewardAcc\\n    totalShares\\n    totalStake\\n    freeStake\\n    releasingStake\\n    usedStake\\n    remainingStake\\n    stakersCount\\n    minersCount\\n    theoreticalApr\\n    stakePoolStakers(where: $stakePoolStakersWhere) @include(if: $withStakePoolStakers) {\\n      address\\n      shares\\n      locked\\n      availableReward\\n      rewardDebt\\n      stake\\n      pendingReward\\n      stakeReward\\n      claimableReward\\n      instantClaimableReward\\n      isOwner\\n    }\\n    accounts {\\n      identity\\n      identityVerified\\n    }\\n    stakePoolWithdrawals(where: $stakePoolWithdrawalsWhere) @include(if: $withStakePoolWithdrawals) {\\n      shares\\n      startTime\\n      estimatesEndTime\\n      stake\\n      userAddress\\n    }\\n    miners(where: $minersWhere) @include(if: $withMiners) {\\n      estimatesReclaimableAt\\n      workerPublicKey\\n      stakes\\n    }\\n    withdrawalsCount\\n  }\\n  aggregateStakePools(where: $where) {\\n    _count {\\n      _all\\n    }\\n  }\\n}\\n    \",\"variables\":{\"take\":20,\"withStakePoolStakers\":true,\"withStakePoolWithdrawals\":true,\"withMiners\":true,\"skip\":20,\"orderBy\":{\"theoreticalApr\":\"asc\"},\"where\":{\"AND\":[{\"stakePoolStakers\":{\"some\":{\"address\":{\"equals\":\"46CAVopcw76MV3Vu2Na6JNg1jrjmMctJHy7kk8kyyumhTnVj\"},\"OR\":[{\"claimableReward\":{\"gt\":\"0\"}},{\"shares\":{\"gt\":\"0\"}}]}}}]},\"stakePoolStakersWhere\":{\"address\":{\"equals\":\"46CAVopcw76MV3Vu2Na6JNg1jrjmMctJHy7kk8kyyumhTnVj\"}},\"stakePoolWithdrawalsWhere\":{\"userAddress\":{\"equals\":\"46CAVopcw76MV3Vu2Na6JNg1jrjmMctJHy7kk8kyyumhTnVj\"}},\"minersWhere\":{\"estimatesReclaimableAt\":{\"lte\":\"2022-04-14T16:17:00.000Z\"}}}}")

// const addr = process.argv[1];

const addr = "46CAVopcw76MV3Vu2Na6JNg1jrjmMctJHy7kk8kyyumhTnVj";
const body = {
    "query" : `
        query StakePools(
            $where: StakePoolsWhereInput,
            $orderBy: [StakePoolsOrderByWithRelationInput!],
            $take: Int,
            $skip: Int,
            $withStakePoolStakers: Boolean = false,
            $stakePoolStakersWhere: StakePoolStakersWhereInput,
            $withStakePoolWithdrawals: Boolean = false,
            $stakePoolWithdrawalsWhere: StakePoolWithdrawalsWhereInput,
            $withMiners: Boolean = false,
            $minersWhere: MinersWhereInput
        ) {
            findManyStakePools(where: $where, orderBy: $orderBy, take: $take, skip: $skip) {
                pid
                ownerAddress
                commission
                ownerReward
                cap
                rewardAcc
                totalShares
                totalStake
                freeStake
                releasingStake
                usedStake
                remainingStake
                stakersCount
                minersCount
                theoreticalApr
                stakePoolStakers(where: $stakePoolStakersWhere) @include(if: $withStakePoolStakers) {
                    address
                    shares
                    locked
                    availableReward
                    rewardDebt
                    stake
                    pendingReward
                    stakeReward
                    claimableReward
                    instantClaimableReward
                    isOwner
                }
                accounts {
                    identity
                    identityVerified
                }
                stakePoolWithdrawals(where: $stakePoolWithdrawalsWhere) @include(if: $withStakePoolWithdrawals) {
                    shares
                    startTime
                    estimatesEndTime
                    stake
                    userAddress
                }
                miners(where: $minersWhere) @include(if: $withMiners) {
                    estimatesReclaimableAt
                    workerPublicKey
                    stakes
                }
                withdrawalsCount
            }
            aggregateStakePools(where: $where) {
                _count {
                    _all
                }
            }
        }
    `,
    "variables" : {
       "minersWhere" : {
          "estimatesReclaimableAt" : {
             "lte" : "2022-04-14T16:17:00.000Z"
          }
       },
       "orderBy" : {
          "theoreticalApr" : "asc"
       },
       "take" : 200,
       "skip" : 0,
       "stakePoolStakersWhere" : {
          "address" : {
             "equals" : addr
          }
       },
       "stakePoolWithdrawalsWhere" : {
          "userAddress" : {
             "equals" : addr
          }
       },
       "where" : {
          "AND" : [
             {
                "stakePoolStakers" : {
                   "some" : {
                      "OR" : [
                         {
                            "claimableReward" : {
                               "gt" : "0"
                            }
                         },
                         {
                            "shares" : {
                               "gt" : "0"
                            }
                         }
                      ],
                      "address" : {
                         "equals" : addr
                      }
                   }
                }
             }
          ]
       },
       "withMiners" : true,
       "withStakePoolStakers" : true,
       "withStakePoolWithdrawals" : true
    }
 };

fetch("https://app-api.phala.network/", {
  "headers": {
    "accept": "*/*",
    "accept-language": "en-US,en;q=0.9,zh-CN;q=0.8,zh;q=0.7",
    "content-type": "application/json",
    "sec-ch-ua": "\" Not A;Brand\";v=\"99\", \"Chromium\";v=\"99\", \"Google Chrome\";v=\"99\"",
    "sec-ch-ua-mobile": "?0",
    "sec-ch-ua-platform": "\"macOS\"",
    "sec-fetch-dest": "empty",
    "sec-fetch-mode": "cors",
    "sec-fetch-site": "same-site"
  },
  "referrerPolicy": "same-origin",
  "body": JSON.stringify(body),
  "method": "POST"
})
.then(resp => resp.json())
.then(json => {
    const data = json.data.findManyStakePools.map(p => ({
        pid: p.pid,
        commission: p.commission,
        apr: p.commission,
        stake: p.stakePoolStakers[0].stake,
        claimableReward: p.stakePoolStakers[0].claimableReward,
    }));
    const csv = Papa.unparse(data);
    console.log(csv);
});

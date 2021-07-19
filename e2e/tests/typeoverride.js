const types = {
	"Address": "MultiAddress",
	"LookupSource": "MultiAddress",
	"ChainId": "u8",
	"ResourceId": "[u8; 32]",
	"TokenId": "u256",
	"DepositNonce": "u64",
	"RawSolution": "RawSolutionWith24",
	"EcdsaPublicKey": "[u8; 33]",
	"WorkerPublicKey": "EcdsaPublicKey",
	"ContractPublicKey": "EcdsaPublicKey",
	"EcdhPublicKey": "[u8; 32]",
	"MessageOrigin": {
		"_enum": {
			"Pallet": "Vec<u8>",
			"Contract": "H256",
			"Worker": "EcdsaPublicKey",
			"AccountId": "H256",
			"MultiLocation": "Vec<u8>"
		}
	},
	"Attestation": {
		"_enum": {
			"SgxIas": "AttestationSgxIas"
		}
	},
	"AttestationSgxIas": {
		"raReport": "Vec<u8>",
		"signature": "Vec<u8>"
	},
	"SenderId": "MessageOrigin",
	"Path": "Vec<u8>",
	"Topic": "Path",
	"Message": {
		"sender": "SenderId",
		"destination": "Topic",
		"payload": "Vec<u8>"
	},
	"SignedMessage": {
		"message": "Message",
		"sequence": "u64",
		"signature": "Vec<u8>"
	},
	"MachineId": "[u8; 16]",
	"PRuntimeInfo": {
		"version": "u32",
		"machineId": "MachineId",
		"pubkey": "WorkerPublicKey",
		"ecdhPubkey": "EcdhPublicKey",
		"features": "Vec<u32>",
		"operator": "Option<AccountId>"
	},
	"PoolState": {
		"_enum": {
			"Ready": null,
			"Mining": null
		}
	},
	"PoolInfo": {
		"pid": "u64",
		"owner": "AccountId",
		"state": "PoolState",
		"payoutCommission": "u16",
		"ownerReward": "Balance",
		"cap": "Option<Balance>",
		"poolAcc": "Balance",
		"totalStake": "Balance",
		"freeStake": "Balance",
		"workers": "Vec<WorkerPublicKey>",
		"withdrawQueue": "Vec<WithdrawInfo>"
	},
	"WithdrawInfo": {
		"user": "AccountId",
		"amount": "Balance",
		"startTime": "u64"
	},
	"ProposalStatus": {
		"_enum": {
			"Initiated": null,
			"Approved": null,
			"Rejected": null
		}
	},
	"ProposalVotes": {
		"votes_for": "Vec<AccountId>",
		"votes_against": "Vec<AccountId>",
		"status": "ProposalStatus",
		"expiry": "BlockNumber"
	},
	"Kitty": {
		"id": "Hash",
		"dna": "Hash",
		"price": "Balance",
		"gen": "u64"
	},
	"WorkerStateEnum": {
		"_enum": {
			"Empty": null,
			"Free": null,
			"Gatekeeper": null,
			"MiningPending": null,
			"Mining": "BlockNumber",
			"MiningStopping": null
		}
	},
	"WorkerInfo": {
		"pubkey": "WorkerPublicKey",
		"ecdhPubkey": "EcdhPublicKey",
		"runtimeVersion": "u32",
		"lastUpdated": "u64",
		"operator": "Option<AccountId>",
		"confidenceLevel": "u8",
		"intialScore": "Option<u32>",
		"features": "Vec<u32>"
	},
	"MinerInfo": {
		"state": "MinerState",
		"ve": "u64",
		"v": "u64",
		"vUpdatedAt": "u64",
		"pInstant": "u64",
		"benchmark": "Benchmark",
		"coolingDownStart": "u64"
	},
	"Benchmark": {
		"iterations": "u64",
		"miningStartTime": "u64"
	},
	"MinerState": {
		"_enum": {
			"Ready": null,
			"MiningIdle": null,
			"MiningActive": null,
			"MiningUnresponsive": null,
			"MiningCollingDown": null
		}
	},
	"WorkerStat": {
		"totalReward": "Balance"
	},
	"_deprecated_WorkerInfo": {
		"machineId": "Vec<u8>",
		"pubkey": "Vec<u8>",
		"lastUpdated": "u64",
		"state": "WorkerStateEnum",
		"score": "Option<Score>",
		"confidenceLevel": "u8",
		"runtimeVersion": "u32"
	},
	"Score": {
		"overallScore": "u32",
		"features": "Vec<u32>"
	},
	"StashInfo": {
		"controller": "AccountId",
		"payoutPrefs": "PayoutPrefs"
	},
	"PayoutPrefs": {
		"commission": "u32",
		"target": "AccountId"
	},
	"HeartbeatChallenge": {
		"seed": "U256",
		"onlineTarget": "U256",
	},
	"RoundInfo": {
		"round": "u32",
		"startBlock": "BlockNumber"
	},
	"RoundStats": {
		"round": "u32",
		"onlineWorkers": "u32",
		"computeWorkers": "u32",
		"fracTargetOnlineReward": "u32",
		"totalPower": "u32",
		"fracTargetComputeReward": "u32"
	},
	"StashWorkerStats": {
		"slash": "Balance",
		"computeReceived": "Balance",
		"onlineReceived": "Balance"
	},
	"MinerStatsDelta": {
		"numWorker": "i32",
		"numPower": "i32"
	},
	"PayoutReason": {
		"_enum": {
			"OnlineReward": null,
			"ComputeReward": null
		}
	},
	"GatekeeperEvent": {
		"_enum": {
			"Registered": "NewGatekeeperEvent",
			"DispatchMasterKey": "DispatchMasterKeyEvent",
			"NewRandomNumber": "RandomNumberEvent",
			"UpdateTokenomic": "TokenomicParameters"
		}
	},
    "NewGatekeeperEvent": {
        "pubkey": "WorkerPublicKey",
        "ecdhPubkey": "EcdhPublicKey",
        "gatekeeperCount": "u32"
    },
    "DispatchMasterKeyEvent": {
        "dest": "WorkerPublicKey",
        "ecdhPubkey": "EcdhPublicKey",
        "encryptedMasterKey": "Vec<u8>",
        "iv": "[u8; 12]"
    },
    "RandomNumberEvent": {
        "blockNumber": "u32",
        "randomNumber": "[u8; 32]",
        "lastRandomNumber": "[u8; 32]"
    },
    "TokenomicParameters": {
        "phaRate": "U64F64Bits",
        "rho": "U64F64Bits",
        "budgetPerSec": "U64F64Bits",
        "vMax": "U64F64Bits",
        "costK": "U64F64Bits",
        "costB": "U64F64Bits",
        "slashRate": "U64F64Bits",
        "heartbeatWindow": "u32"
    },
	"TokenomicParams": "TokenomicParameters",
	"U64F64Bits": "u128",
	"UserStakeInfo": {
		"user": "AccountId",
		"amount": "Balance",
		"availableRewards": "Balance",
		"userDebt": "Balance",
	}
};

const typeAlias = {
	"phala": {
		"WorkerInfo": "_deprecated_WorkerInfo"
	}
};

module.exports = {types, typeAlias};

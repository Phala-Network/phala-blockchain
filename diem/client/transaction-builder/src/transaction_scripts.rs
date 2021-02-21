// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Rust representation of a Move transaction script that can be executed on the Libra blockchain.
//! Libra does not allow arbitrary transaction scripts; only scripts whose hashes are present in
//! the on-chain script allowlist. The genesis allowlist is derived from this file, and the
//! `Stdlib` script enum will be modified to reflect changes in the on-chain allowlist as time goes
//! on.

use anyhow::{anyhow, Error, Result};
use diem_crypto::HashValue;
use diem_types::transaction::{ScriptABI, SCRIPT_HASH_LENGTH};
use std::{convert::TryFrom, fmt};
use std::{string::{String, ToString}, vec::Vec};

const CHILD_ABI: &str = r#"196372656174655f6368696c645f766173705f6163636f756e74b20920437265617465206120604368696c645641535060206163636f756e7420666f722073656e6465722060706172656e745f766173706020617420606368696c645f6164647265737360207769746820612062616c616e6365206f660a20606368696c645f696e697469616c5f62616c616e63656020696e2060436f696e547970656020616e6420616e20696e697469616c2061757468656e7469636174696f6e5f6b65790a2060617574685f6b65795f707265666978207c206368696c645f61646472657373602e0a20496620606164645f616c6c5f63757272656e636965736020697320747275652c20746865206368696c6420616464726573732077696c6c20686176652061207a65726f2062616c616e636520696e20616c6c20617661696c61626c650a2063757272656e6369657320696e207468652073797374656d2e0a2054686973206163636f756e742077696c6c2061206368696c64206f6620746865207472616e73616374696f6e2073656e6465722c207768696368206d757374206265206120506172656e74564153502e0a0a2023232041626f7274730a20546865207472616e73616374696f6e2077696c6c2061626f72743a0a0a202a2049662060706172656e745f7661737060206973206e6f74206120706172656e7420766173702077697468206572726f723a2060526f6c65733a3a45494e56414c49445f504152454e545f524f4c45600a202a20496620606368696c645f616464726573736020616c7265616479206578697374732077697468206572726f723a2060526f6c65733a3a45524f4c455f414c52454144595f41535349474e4544600a202a2049662060706172656e745f766173706020616c72656164792068617320323536206368696c64206163636f756e74732077697468206572726f723a2060564153503a3a45544f4f5f4d414e595f4348494c4452454e600a202a2049662060436f696e5479706560206973206e6f74206120726567697374657265642063757272656e63792077697468206572726f723a20604c696272614163636f756e743a3a454e4f545f415f43555252454e4359600a202a2049662060706172656e745f76617370602773207769746864726177616c206361706162696c69747920686173206265656e206578747261637465642077697468206572726f723a2020604c696272614163636f756e743a3a455749544844524157414c5f4341504142494c4954595f414c52454144595f455854524143544544600a202a2049662060706172656e745f766173706020646f65736e277420686f6c642060436f696e547970656020616e6420606368696c645f696e697469616c5f62616c616e6365203e2030602077697468206572726f723a20604c696272614163636f756e743a3a4550415945525f444f45534e545f484f4c445f43555252454e4359600a202a2049662060706172656e745f766173706020646f65736e2774206174206c6561737420606368696c645f696e697469616c5f62616c616e636560206f662060436f696e547970656020696e20697473206163636f756e742062616c616e63652077697468206572726f723a20604c696272614163636f756e743a3a45494e53554646494349454e545f42414c414e434560b002a11ceb0b0100000008010002020204030616041c0405202307437b08be011006ce0104000000010100000200010101000302030000040401010100050301000006020604060c050a02010001060c0108000506080005030a020a0205060c050a0201030109000c4c696272614163636f756e741257697468647261774361706162696c697479196372656174655f6368696c645f766173705f6163636f756e741b657874726163745f77697468647261775f6361706162696c697479087061795f66726f6d1b726573746f72655f77697468647261775f6361706162696c697479000000000000000000000000000000010a02010001010503190a000a010b020a0338000a0406000000000000000024030a05160b0011010c050e050a010a040700070038010b05110305180b0001020109636f696e5f74797065040d6368696c645f61646472657373040f617574685f6b65795f7072656669780601126164645f616c6c5f63757272656e6369657300156368696c645f696e697469616c5f62616c616e636502"#;
const TRANSFER_ABI: &str = r#"1a706565725f746f5f706565725f776974685f6d65746164617461a413205472616e736665722060616d6f756e746020636f696e73206f662074797065206043757272656e6379602066726f6d206070617965726020746f2060706179656560207769746820286f7074696f6e616c29206173736f6369617465640a20606d657461646174616020616e6420616e20286f7074696f6e616c2920606d657461646174615f7369676e617475726560206f6e20746865206d6573736167650a20606d6574616461746160207c20605369676e65723a3a616464726573735f6f662870617965722960207c2060616d6f756e7460207c20604475616c4174746573746174696f6e3a3a444f4d41494e5f534550415241544f52602e0a2054686520606d657461646174616020616e6420606d657461646174615f7369676e61747572656020706172616d657465727320617265206f6e6c792072657175697265642069662060616d6f756e7460203e3d0a20604475616c4174746573746174696f6e3a3a6765745f6375725f6d6963726f6c696272615f6c696d697460204c425220616e64206070617965726020616e642060706179656560206172652064697374696e63742056415350732e0a20486f77657665722c2061207472616e73616374696f6e2073656e6465722063616e206f707420696e20746f206475616c206174746573746174696f6e206576656e207768656e206974206973206e6f742072657175697265642028652e672e2c20612044657369676e617465644465616c6572202d3e2056415350207061796d656e74292062792070726f766964696e672061206e6f6e2d656d70747920606d657461646174615f7369676e6174757265602e0a205374616e64617264697a656420606d6574616461746160204c435320666f726d61742063616e20626520666f756e6420696e20606c696272615f74797065733a3a7472616e73616374696f6e3a3a6d657461646174613a3a4d65746164617461602e0a0a202323204576656e74730a205768656e20746869732073637269707420657865637574657320776974686f75742061626f7274696e672c20697420656d6974732074776f206576656e74733a0a206053656e745061796d656e744576656e74207b20616d6f756e742c2063757272656e63795f636f6465203d2043757272656e63792c2070617965652c206d65746164617461207d600a206f6e2060706179657260277320604c696272614163636f756e743a3a73656e745f6576656e7473602068616e646c652c20616e640a20206052656365697665645061796d656e744576656e74207b20616d6f756e742c2063757272656e63795f636f6465203d2043757272656e63792c2070617965722c206d65746164617461207d600a206f6e2060706179656560277320604c696272614163636f756e743a3a72656365697665645f6576656e7473602068616e646c652e0a0a20232320436f6d6d6f6e2041626f7274730a2054686573652061626f7274732063616e20696e206f6363757220696e20616e79207061796d656e742e0a202a2041626f727473207769746820604c696272614163636f756e743a3a45494e53554646494349454e545f42414c414e4345602069662060616d6f756e74602069732067726561746572207468616e206070617965726027732062616c616e636520696e206043757272656e6379602e0a202a2041626f727473207769746820604c696272614163636f756e743a3a45434f494e5f4445504f5349545f49535f5a45524f602069662060616d6f756e7460206973207a65726f2e0a202a2041626f727473207769746820604c696272614163636f756e743a3a4550415945455f444f45535f4e4f545f455849535460206966206e6f206163636f756e742065786973747320617420746865206164647265737320607061796565602e0a202a2041626f727473207769746820604c696272614163636f756e743a3a4550415945455f43414e545f4143434550545f43555252454e43595f545950456020696620616e206163636f756e742065786973747320617420607061796565602c2062757420697420646f6573206e6f7420616363657074207061796d656e747320696e206043757272656e6379602e0a0a202323204475616c204174746573746174696f6e2041626f7274730a2054686573652061626f7274732063616e206f6363757220696e20616e79207061796d656e74207375626a65637420746f206475616c206174746573746174696f6e2e0a202a2041626f727473207769746820604475616c4174746573746174696f6e3a3a454d414c464f524d45445f4d455441444154415f5349474e41545552456020696620606d657461646174615f7369676e6174757265602773206973206e6f742036342062797465732e0a202a2041626f727473207769746820604475616c4174746573746174696f6e3a45494e56414c49445f4d455441444154415f5349474e41545552456020696620606d657461646174615f7369676e61747572656020646f6573206e6f7420766572696679206f6e20746865206d65737361676520606d6574616461746160207c2060706179657260207c206076616c756560207c2060444f4d41494e5f534550415241544f5260207573696e67207468652060636f6d706c69616e63655f7075626c69635f6b657960207075626c697368656420696e207468652060706179656560277320604475616c4174746573746174696f6e3a3a43726564656e7469616c60207265736f757263652e0a0a202323204f746865722041626f7274730a2054686573652061626f7274732073686f756c64206f6e6c792068617070656e207768656e2060706179657260206f7220607061796565602068617665206163636f756e74206c696d6974207265737472696374696f6e73206f720a2068617665206265656e2066726f7a656e206279204c696272612061646d696e6973747261746f72732e0a202a2041626f727473207769746820604c696272614163636f756e743a3a455749544844524157414c5f455843454544535f4c494d49545360206966206070617965726020686173206578636565646564207468656972206461696c790a207769746864726177616c206c696d6974732e0a202a2041626f727473207769746820604c696272614163636f756e743a3a454445504f5349545f455843454544535f4c494d49545360206966206070617965656020686173206578636565646564207468656972206461696c79206465706f736974206c696d6974732e0a202a2041626f727473207769746820604c696272614163636f756e743a3a454143434f554e545f46524f5a454e6020696620607061796572602773206163636f756e742069732066726f7a656e2ee101a11ceb0b010000000701000202020403061004160205181d0735610896011000000001010000020001000003020301010004010300010501060c0108000506080005030a020a020005060c05030a020a020109000c4c696272614163636f756e741257697468647261774361706162696c6974791b657874726163745f77697468647261775f6361706162696c697479087061795f66726f6d1b726573746f72655f77697468647261775f6361706162696c69747900000000000000000000000000000001010104010c0b0011000c050e050a010a020b030b0438000b05110202010863757272656e6379040570617965650406616d6f756e7402086d657461646174610601126d657461646174615f7369676e61747572650601"#;

/// All of the Move transaction scripts that can be executed on the Libra blockchain
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum StdlibScript {
    AddCurrencyToAccount,
    AddRecoveryRotationCapability,
    AddScriptAllowList,
    AddValidatorAndReconfigure,
    Burn,
    BurnTxnFees,
    CancelBurn,
    CreateChildVaspAccount,
    CreateDesignatedDealer,
    CreateParentVaspAccount,
    CreateRecoveryAddress,
    CreateValidatorAccount,
    CreateValidatorOperatorAccount,
    FreezeAccount,
    MintLbr,
    PeerToPeerWithMetadata,
    Preburn,
    PublishSharedEd2551PublicKey,
    RegisterValidatorConfig,
    RemoveValidatorAndReconfigure,
    RotateAuthenticationKey,
    RotateAuthenticationKeyWithNonce,
    RotateAuthenticationKeyWithNonceAdmin,
    RotateAuthenticationKeyWithRecoveryAddress,
    RotateDualAttestationInfo,
    RotateSharedEd2551PublicKey,
    SetValidatorConfigAndReconfigure,
    SetValidatorOperator,
    SetValidatorOperatorWithNonceAdmin,
    TieredMint,
    UnfreezeAccount,
    UnmintLbr,
    UpdateExchangeRate,
    UpdateLibraVersion,
    UpdateMintingAbility,
    UpdateDualAttestationLimit,
    // ...add new scripts here
}

impl StdlibScript {
    /// Return a vector containing all of the standard library scripts (i.e., all inhabitants of the
    /// StdlibScript enum)
    pub fn all() -> Vec<Self> {
        use StdlibScript::*;
        vec![
            AddCurrencyToAccount,
            AddRecoveryRotationCapability,
            AddScriptAllowList,
            AddValidatorAndReconfigure,
            Burn,
            BurnTxnFees,
            CancelBurn,
            CreateChildVaspAccount,
            CreateDesignatedDealer,
            CreateParentVaspAccount,
            CreateRecoveryAddress,
            CreateValidatorAccount,
            CreateValidatorOperatorAccount,
            FreezeAccount,
            MintLbr,
            PeerToPeerWithMetadata,
            Preburn,
            PublishSharedEd2551PublicKey,
            RegisterValidatorConfig,
            RemoveValidatorAndReconfigure,
            RotateAuthenticationKey,
            RotateAuthenticationKeyWithNonce,
            RotateAuthenticationKeyWithNonceAdmin,
            RotateAuthenticationKeyWithRecoveryAddress,
            RotateDualAttestationInfo,
            RotateSharedEd2551PublicKey,
            SetValidatorConfigAndReconfigure,
            SetValidatorOperator,
            SetValidatorOperatorWithNonceAdmin,
            TieredMint,
            UnfreezeAccount,
            UnmintLbr,
            UpdateExchangeRate,
            UpdateLibraVersion,
            UpdateMintingAbility,
            UpdateDualAttestationLimit,
            // ...add new scripts here
        ]
    }

    /// Construct the allowlist of script hashes used to determine whether a transaction script can
    /// be executed on the Libra blockchain
    pub fn allowlist() -> Vec<[u8; SCRIPT_HASH_LENGTH]> {
        StdlibScript::all()
            .iter()
            .map(|script| *script.compiled_bytes().hash().as_ref())
            .collect()
    }

    /// Return a lowercase-underscore style name for this script
    pub fn name(self) -> String {
        self.to_string()
    }

    /// Return true if `code_bytes` is the bytecode of one of the standard library scripts
    pub fn is(code_bytes: &[u8]) -> bool {
        Self::try_from(code_bytes).is_ok()
    }

    /// Return the Move bytecode that was produced by compiling this script.
    pub fn compiled_bytes(self) -> CompiledBytes {
        CompiledBytes(self.abi().code().to_vec())
    }

    /// Return the ABI of the script (including the bytecode).
    pub fn abi(self) -> ScriptABI {
        if self.name() == "create_child_vasp_account" {
            let content = hex::decode(CHILD_ABI).unwrap();
            bcs::from_bytes(&content)
                .unwrap_or_else(|err| panic!("Failed to deserialize ABI : {}", err))
        } else if self.name() == "peer_to_peer_with_metadata" {
            let content = hex::decode(TRANSFER_ABI).unwrap();
            bcs::from_bytes(&content)
                .unwrap_or_else(|err| panic!("Failed to deserialize ABI : {}", err))
        } else {
            let content = "Abi File does not exist".as_bytes();
            bcs::from_bytes(content)
                .unwrap_or_else(|err| panic!("Failed to deserialize ABI file : {}", err))
        }
    }

    /// Return the sha3-256 hash of the compiled script bytes.
    pub fn hash(self) -> HashValue {
        self.compiled_bytes().hash()
    }
}

/// Bytes produced by compiling a Move source language script into Move bytecode
#[derive(Clone)]
pub struct CompiledBytes(Vec<u8>);

impl CompiledBytes {
    /// Return the sha3-256 hash of the script bytes
    pub fn hash(&self) -> HashValue {
        Self::hash_bytes(&self.0)
    }

    /// Return the sha3-256 hash of the script bytes
    fn hash_bytes(bytes: &[u8]) -> HashValue {
        HashValue::sha3_256_of(bytes)
    }

    /// Convert this newtype wrapper into a vector of bytes
    pub fn into_vec(self) -> Vec<u8> {
        self.0
    }
}

impl TryFrom<&[u8]> for StdlibScript {
    type Error = Error;

    /// Return `Some(<script_name>)` if  `code_bytes` is the bytecode of one of the standard library
    /// scripts, None otherwise.
    fn try_from(code_bytes: &[u8]) -> Result<Self> {
        let hash = CompiledBytes::hash_bytes(code_bytes);
        Self::all()
            .iter()
            .find(|script| script.hash() == hash)
            .cloned()
            .ok_or_else(|| anyhow!("Could not create standard library script from bytes"))
    }
}

impl fmt::Display for StdlibScript {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use StdlibScript::*;
        write!(
            f,
            "{}",
            match self {
                AddValidatorAndReconfigure => "add_validator_and_reconfigure",
                AddCurrencyToAccount => "add_currency_to_account",
                AddRecoveryRotationCapability => "add_recovery_rotation_capability",
                AddScriptAllowList => "add_to_script_allow_list",
                Burn => "burn",
                BurnTxnFees => "burn_txn_fees",
                CancelBurn => "cancel_burn",
                CreateChildVaspAccount => "create_child_vasp_account",
                CreateDesignatedDealer => "create_designated_dealer",
                CreateParentVaspAccount => "create_parent_vasp_account",
                CreateRecoveryAddress => "create_recovery_address",
                CreateValidatorAccount => "create_validator_account",
                CreateValidatorOperatorAccount => "create_validator_operator_account",
                FreezeAccount => "freeze_account",
                MintLbr => "mint_lbr",
                PeerToPeerWithMetadata => "peer_to_peer_with_metadata",
                Preburn => "preburn",
                PublishSharedEd2551PublicKey => "publish_shared_ed25519_public_key",
                RegisterValidatorConfig => "register_validator_config",
                RemoveValidatorAndReconfigure => "remove_validator_and_reconfigure",
                RotateAuthenticationKey => "rotate_authentication_key",
                RotateAuthenticationKeyWithNonce => "rotate_authentication_key_with_nonce",
                RotateAuthenticationKeyWithNonceAdmin =>
                    "rotate_authentication_key_with_nonce_admin",
                RotateAuthenticationKeyWithRecoveryAddress =>
                    "rotate_authentication_key_with_recovery_address",
                RotateDualAttestationInfo => "rotate_dual_attestation_info",
                RotateSharedEd2551PublicKey => "rotate_shared_ed25519_public_key",
                SetValidatorConfigAndReconfigure => "set_validator_config_and_reconfigure",
                SetValidatorOperator => "set_validator_operator",
                SetValidatorOperatorWithNonceAdmin => "set_validator_operator_with_nonce_admin",
                TieredMint => "tiered_mint",
                UpdateDualAttestationLimit => "update_dual_attestation_limit",
                UnfreezeAccount => "unfreeze_account",
                UnmintLbr => "unmint_lbr",
                UpdateLibraVersion => "update_libra_version",
                UpdateExchangeRate => "update_exchange_rate",
                UpdateMintingAbility => "update_minting_ability",
            }
        )
    }
}

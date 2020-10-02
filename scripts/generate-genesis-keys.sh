#!/bin/bash

network=phat

read -s -p "Enter mnemonic: " secret
echo "[Generated code]"

function get_pubkey {
    tmp=tmp.key
    subkey "$@" > "$tmp"
    pk_with0x=$(awk '/Public key \(hex\): +(\w+?)/{print $4}' "$tmp")
    pk=${pk_with0x#0x}
    echo $pk
    rm tmp.key
}

printf """    let initial_authorities: Vec<(AccountId, AccountId, GrandpaId, BabeId, ImOnlineId, AuthorityDiscoveryId)> = vec![
    """

for i in 1 2 3 4 ; do
    stash=$(get_pubkey -s inspect "$secret"/"$network"/stash/$i)
    controller=$(get_pubkey -s inspect "$secret"/"$network"/controller/$i)
    session_gran=$(get_pubkey -e inspect "$secret"//"$network"//session//$i)
    session_babe=$(get_pubkey -s inspect "$secret"//"$network"//session//$i)

    printf """(
        // Stash
        hex![\"${stash}\"].into(),
        // Controller
        hex![\"${controller}\"].into(),
        // Session key ed25519
        hex![\"${session_gran}\"].unchecked_into(),
        // Session key sr25519
        hex![\"${session_babe}\"].unchecked_into(),
        hex![\"${session_babe}\"].unchecked_into(),
        hex![\"${session_babe}\"].unchecked_into()
    ),"""
done

echo """];

    // generated with secret: subkey inspect \"\$secret\"/"$network"
    let root_key: AccountId = hex![
        \"$(get_pubkey -s inspect "$secret"/"$network")\"
    ].into();
"""


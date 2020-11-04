#!/bin/bash

network=phat3

read -s -p "Enter mnemonic: " secret
echo "[Generated code]"

function get_pubkey {
    tmp=tmp.key
    ./phala-node key "$@" > "$tmp"
    pk_with0x=$(awk '/Public key \(hex\): +(\w+?)/{print $4}' "$tmp")
    pk=${pk_with0x#0x}
    echo $pk
    rm tmp.key
}

printf """    let initial_authorities: Vec<(AccountId, AccountId, GrandpaId, BabeId, ImOnlineId, AuthorityDiscoveryId)> = vec![
    """

for i in 1 2 3 4 ; do
    stash=$(get_pubkey inspect-key -n phala --scheme Sr25519 "$secret"/"$network"/stash/$i)
    controller=$(get_pubkey inspect-key -n phala --scheme Sr25519 "$secret"/"$network"/controller/$i)
    session_gran=$(get_pubkey inspect-key -n phala --scheme Ed25519 "$secret"//"$network"//session//$i)
    session_babe=$(get_pubkey inspect-key -n phala --scheme Sr25519 "$secret"//"$network"//session//$i)

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

    // generated with secret: phala-node inspect-key -n phala --scheme Sr25519 \"\$secret\"/"$network"
    let root_key: AccountId = hex![
        \"$(get_pubkey inspect-key -n phala --scheme Sr25519 "$secret"/"$network")\"
    ].into();
"""


use pnode_runtime;
use substrate_rpc_primitives::number::NumberOrHex;
use codec::{Encode, Decode};
use sr_primitives::{
    generic::SignedBlock,
    OpaqueExtrinsic
};

type Runtime = pnode_runtime::Runtime;
type Header = <Runtime as subxt::system::System>::Header;
type OpaqueBlock = sr_primitives::generic::Block<Header, OpaqueExtrinsic>;
type OpaqueSignedBlock = SignedBlock<OpaqueBlock>;

fn create_client() -> (tokio::runtime::Runtime, subxt::Client<Runtime>) {
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    let client_future = subxt::ClientBuilder::<Runtime>::new().build();
    let client = rt.block_on(client_future).unwrap();
    (rt, client)
}

fn deopaque_block(opaque_block: OpaqueBlock) -> pnode_runtime::Block {
    pnode_runtime::Block {
        header: opaque_block.header,
        extrinsics:
            opaque_block.extrinsics
                .iter().map(|x| {
                    // v = UncheckedExtrinsic.encode(obj)
                    // vv = Vec.decode(v)
                    let v: &Vec<u8> = &x.0;
                    let vv = Encode::encode(v);
                    pnode_runtime::UncheckedExtrinsic::decode(&mut vv.as_slice())
                        .expect("Block decode failed")
                })
                .collect()
    }
}

fn deopaque_signedblock(opaque_block: OpaqueSignedBlock) -> pnode_runtime::SignedBlock {
    pnode_runtime::SignedBlock {
        block: deopaque_block(opaque_block.block),
        justification: opaque_block.justification,
    }
}

 fn print_jutification(justification: &Vec<u8>) {
     let grandpa_j = GrandpaJustification::<Runtime::Block>::decode(&mut jutification.as_slice());
     println!("Justification: {:?}", grandpa_j);
 }

fn get_block_at(rt: &mut tokio::runtime::Runtime, client: &subxt::Client<Runtime>, h: Option<u32>) -> Option<pnode_runtime::SignedBlock> {
    let pos = match h {
        Some(h) => Some(NumberOrHex::Number(h)),
        None => None
    };
    let hash = match rt.block_on(client.block_hash(pos)).unwrap() {
        Some(hash) => hash,
        None => { eprintln!("Block hash not found!"); return None }
    };
    println!("hash: {:?}", hash);

    let opaque_block = match rt.block_on(client.block(Some(hash))).unwrap() {
        Some(block) => block,
        None => { eprintln!("Block not found"); return None },
    };

    let block = deopaque_signedblock(opaque_block);
    println!("block: {:?}", block);

    Some(block)
}

fn print_metadata(rt: &mut tokio::runtime::Runtime, client: &subxt::Client<Runtime>) {
    let metadata = client.metadata();
    println!("Metadata: {:?}", metadata);
}

fn main() {
    let (mut rt, client) = create_client();

    // print_metadata(&mut rt, &client);

    let signed_tip = get_block_at(&mut rt, &client, None).unwrap();

    for i in 1..3 {
        println!("--");
        let h = signed_tip.block.header.number - i;
        get_block_at(&mut rt, &client, Some(h));
    }
}

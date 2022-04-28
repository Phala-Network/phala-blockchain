// checkpoint related 

use pkvdb::raw::RawDB;
use phactory_api::proto_generated::InitRuntimeResponse;


const CHECKPOINT_VERSION: u32 = 2;
const CHECKPOINT_PREFIX: &str = "phala_runtime_checkpoint_";


#[derive(Default, Debug)]
pub struct Checkpoint {
    // meta
    pub version: u32, // always CHECKPOINT_VERSION const 

    // benchmarks related
    pub benchmarks_counter: u64,
    pub benchmarks_score: u64,
    pub benchmarks_paused: bool,

    // phactory related 
    pub phactory_skip_ra: bool,
    pub phactory_dev_mode: bool,
    pub phactory_machine_id: Vec<u8>,


    // phactory runtime state 

    // system state 


}

impl Checkpoint {

    pub fn save(&self, db: impl RawDB) {
        unimplemented!()
    }
}
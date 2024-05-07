use std::sync::Arc;

use chrono::{DateTime, Utc};
use phactory_api::blocks::{BlockHeader, BlockHeaderWithChanges, HeaderToSync, StorageProof};
use phactory_api::prpc::{ChainState, InitRuntimeRequest, PhactoryInfo};

use crate::processor::{PRuntimeRequest, PRuntimeResponse};

#[derive(Clone, Eq, PartialEq, Hash)]
pub enum RepoRequest {
    ChainState(u32),
    Headers(u32),
    ParaHeaders((u32 /* next */, u32 /* current relaychain */)),
    Blocks((u32, u32)),
}

pub struct MainlineContext {
    phactory_info: PhactoryInfo,

    headernum: u32,
    para_headernum: u32,
    blocknum: u32,

    pending_chain_state: Option<(u32, Arc<Vec<(Vec<u8>, Vec<u8>)>>)>,
    pending_headers: Arc<Vec<HeaderToSync>>,
    pending_para_headers: Arc<Vec<BlockHeader>>,
    pending_parahead_proof: Option<Arc<StorageProof>>,
    pending_blocks: Vec<Arc<BlockHeaderWithChanges>>,
}

impl MainlineContext {
    pub fn create(info: &PhactoryInfo) -> Self {
        Self {
            phactory_info: info.clone(),

            headernum: info.headernum,
            para_headernum: info.para_headernum,
            blocknum: info.blocknum,

            pending_chain_state: None,
            pending_headers: Arc::new(vec![]),
            pending_para_headers: Arc::new(vec![]),
            pending_parahead_proof: None,
            pending_blocks: vec![],
        }
    }

    pub fn update_phactory_info(
        &mut self,
        info: PhactoryInfo,
    ) {
        self.phactory_info = info;
    }

    pub fn update_headers_synced_to(
        &mut self,
        synced_to: u32,
    ) {
        self.headernum = synced_to + 1;
        self.pending_headers = Arc::new(vec![]);
    }

    pub fn update_para_headers_synced_to(
        &mut self,
        synced_to: u32,
    ) {
        self.para_headernum = synced_to + 1;
        self.pending_parahead_proof = None;
        self.pending_para_headers = Arc::new(vec![]);
    }

    pub fn update_blocks_synced_to(
        &mut self,
        synced_to: u32,
    ) {
        self.blocknum = synced_to + 1;
        self.pending_blocks.retain(|b| b.block_header.number > synced_to);
    }

    pub fn phactory_info(&self) -> PhactoryInfo {
        let mut result = self.phactory_info.clone();
        result.headernum = self.headernum;
        result.para_headernum = self.para_headernum;
        result.blocknum = self.blocknum;
        result
    }

    fn next_request_headernum(&self) -> u32 {
        match self.pending_headers.last() {
            Some(h) => h.header.number + 1,
            None => self.headernum,
        }
    }

    fn next_request_para_headernum(&self) -> u32 {
        match self.pending_para_headers.last() {
            Some(h) => h.number + 1,
            None => self.para_headernum,
        }
    }

    fn next_request_blocknum(&self) -> u32 {
        match self.pending_blocks.last() {
            Some(block) => block.block_header.number + 1,
            None => self.blocknum,
        }
    }

    pub fn next_repo_request(&self) -> Option<RepoRequest> {
        if !self.phactory_info.initialized {
            return None;
        }

        if self.phactory_info.can_load_chain_state {
            return Some(RepoRequest::ChainState(0));
        }

        let next_para_headernum = self.next_request_para_headernum();

        if self.blocknum < next_para_headernum {
            let next_blocknum = self.next_request_blocknum();
            return if next_blocknum < self.para_headernum {
                Some(RepoRequest::Blocks((next_blocknum, self.para_headernum - 1)))
            } else  {
                None
            }
        }

        if self.pending_parahead_proof.is_none() {
            return Some(RepoRequest::ParaHeaders((next_para_headernum, self.headernum - 1)));
        }
        
        if self.pending_headers.last().map(|h| h.header.number).unwrap_or(0) == 0 {
            Some(RepoRequest::Headers(self.headernum))
        } else {
            None
        }
    }

    pub fn buffer_chain_state(&mut self, number: u32, state: Arc<Vec<(Vec<u8>, Vec<u8>)>>) {
        self.pending_chain_state = Some((number, state));
    }

    pub fn buffer_headers(&mut self, headers: Arc<Vec<HeaderToSync>>) {
        self.pending_headers = headers;
        self.pending_parahead_proof = None;
    }

    pub fn buffer_para_headers(&mut self, para_headers: Arc<Vec<BlockHeader>>, proof: Arc<StorageProof>) {
        self.pending_para_headers = para_headers;
        self.pending_parahead_proof = Some(proof);
    }

    pub fn buffer_blocks(&mut self, mut blocks: Vec<Arc<BlockHeaderWithChanges>>) {
        if let Some(last_saved_block) = self.pending_blocks.last() {
            if let Some(first_new_block) = blocks.first() {
                if last_saved_block.block_header.number + 1 == first_new_block.block_header.number {
                    self.pending_blocks.append(&mut blocks);
                }
            }
        } else {
            self.pending_blocks = blocks;
        }
    }

    pub fn is_initialized(&self) -> bool {
        self.phactory_info.initialized
    }

    pub fn is_registered(&self) -> bool {
        self.phactory_info.registered
    }

    pub fn is_reached_chaintip(&self, para_chaintip: u32) -> bool {
        self.phactory_info.initialized && self.blocknum == para_chaintip + 1
    }

    pub fn next_pruntime_request(
        &mut self,
    ) -> Option<PRuntimeRequest> {
        if !self.phactory_info.initialized {
            return None;
        }

        if let Some((number, state)) = self.pending_chain_state.take() {
            if self.phactory_info.can_load_chain_state {
                return Some(PRuntimeRequest::LoadChainState(
                    ChainState::new(number, state.to_vec())
                ));
            }
        }

        if let Some(request) = self.next_pruntime_request_blocks() {
            return Some(request);
        }

        if let Some(request) = self.next_pruntime_request_para_headers() {
            return Some(request);
        }

        if let Some(request) = self.next_pruntime_request_headers() {
            return Some(request);
        }

        None
    }

    // fn next_pruntime_init(
    //     &self,
    //     init_runtime_request_ias: &InitRuntimeRequest,
    //     init_runtime_request_dcap: &InitRuntimeRequest,
    // ) -> Option<PRuntimeRequest> {
    //     if self.phactory_info.initialized {
    //         return None
    //     }

    //     let supported = &self.phactory_info.supported_attestation_methods;
    //     let request = if supported.is_empty() || supported.contains(&"epid".into()) {
    //         init_runtime_request_ias.clone()
    //     } else if supported.contains(&"dcap".into()) {
    //         init_runtime_request_dcap.clone()
    //     } else {
    //         error!("Supported attestation methods does not include epid or dcap.", self.work);
    //         return None;
    //     };

    //     Some(PRuntimeRequest::InitRuntime(request))
    // }

    fn next_pruntime_request_headers(&mut self) -> Option<PRuntimeRequest> {
        if let Some(header) = self.pending_headers.first() {
            if header.header.number == self.headernum {
                let headers = (*self.pending_headers).clone();
                let request = phactory_api::prpc::HeadersToSync::new(headers, None);
                return Some(PRuntimeRequest::SyncHeaders(request));
            } else {
                self.pending_headers = Arc::new(vec![]);
            }
        }
        None
    }

    fn next_pruntime_request_para_headers(&mut self) -> Option<PRuntimeRequest> {
        if let Some(header) = self.pending_para_headers.first() {
            if self.pending_para_headers.last().unwrap().number == 0 {
                return None;
            }
            if header.number == self.para_headernum && self.pending_parahead_proof.is_some() {
                let headers = (*self.pending_para_headers).clone();
                let proof = (**self.pending_parahead_proof.as_ref().unwrap()).clone();
                let request = phactory_api::prpc::ParaHeadersToSync::new(headers, proof);
                return Some(PRuntimeRequest::SyncParaHeaders(request));
            } else {
                self.pending_para_headers = Arc::new(vec![]);
            }
        }
        None
    }

    fn next_pruntime_request_blocks(&mut self) -> Option<PRuntimeRequest> {
        if let Some(block) = self.pending_blocks.first() {
            if block.block_header.number == self.blocknum {
                let blocks = (&self.pending_blocks)
                    .into_iter()
                    .take(std::cmp::min(4, self.pending_blocks.len()))
                    .map(|b| (**b).clone())
                    .collect::<Vec<_>>();
                let request = phactory_api::prpc::Blocks::new(blocks);
                return Some(PRuntimeRequest::SyncBlocks(request));
            } else {
                self.pending_blocks.clear();
            }
        }
        None
    }
}
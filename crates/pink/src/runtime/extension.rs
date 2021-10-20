use frame_support::log::{error, trace};
use pallet_contracts::chain_extension::{
    ChainExtension, Environment, Ext, InitState, RetVal, SysConfig, UncheckedFrom,
};
use sp_runtime::DispatchError;

use std::cell::RefCell;

pub type EcdhPublicKey = [u8; 32];

#[derive(Debug)]
pub struct Message {
    pub payload: Vec<u8>,
    pub topic: Vec<u8>,
}

#[derive(Debug)]
pub struct OspMessage {
    pub payload: Vec<u8>,
    pub topic: Vec<u8>,
    pub remote_pubkey: Option<EcdhPublicKey>,
}

#[derive(Default, Debug)]
pub struct PinkEgressMessages {
    pub messages: Vec<Message>,
    pub osp_messages: Vec<OspMessage>,
}

impl PinkEgressMessages {
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty() && self.osp_messages.is_empty()
    }
}

thread_local!(static EGRESS: RefCell<PinkEgressMessages> = Default::default());

pub fn take_mq_egress() -> PinkEgressMessages {
    EGRESS.with(move |cell| std::mem::replace(&mut *cell.borrow_mut(), Default::default()))
}

/// Contract extension for `pink contracts`
pub struct PinkExtension;

impl ChainExtension<super::PinkRuntime> for PinkExtension {
    fn call<E: Ext>(func_id: u32, env: Environment<E, InitState>) -> Result<RetVal, DispatchError>
    where
        <E::T as SysConfig>::AccountId: UncheckedFrom<<E::T as SysConfig>::Hash> + AsRef<[u8]>,
    {
        trace!(
            target: "pink",
            "[ChainExtension]|call|func_id:{:}",
            func_id
        );
        match func_id {
            // push_message
            1 => {
                let mut env = env.buf_in_buf_out();
                let (payload, topic): (Vec<u8>, Vec<u8>) = env.read_as_unbounded(env.in_len())?;
                EGRESS.with(move |f| f.borrow_mut().messages.push(Message { payload, topic }));
            }
            // push_osp_message
            2 => {
                let mut env = env.buf_in_buf_out();
                let (payload, topic, remote_pubkey): (Vec<u8>, Vec<u8>, Option<EcdhPublicKey>) =
                    env.read_as_unbounded(env.in_len())?;
                EGRESS.with(move |f| {
                    f.borrow_mut().osp_messages.push(OspMessage {
                        payload,
                        topic,
                        remote_pubkey,
                    })
                });
            }
            _ => {
                error!(target: "pink", "Called an unregistered `func_id`: {:}", func_id);
                return Err(DispatchError::Other("Unimplemented func_id"));
            }
        }
        Ok(RetVal::Converging(0))
    }
}

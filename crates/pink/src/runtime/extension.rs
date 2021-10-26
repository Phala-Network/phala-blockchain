use frame_support::log::error;
use pallet_contracts::chain_extension::{
    ChainExtension, Environment, Ext, InitState, RetVal, SysConfig, UncheckedFrom,
};
use pink_extension::{Message, OspMessage};
use scale::Decode;
use sp_runtime::DispatchError;

use crate::types::AccountId;

#[derive(Debug)]
pub enum EgressMessage {
    Message(Message),
    OspMessage(OspMessage),
}

#[derive(Default, Debug)]
pub struct ExecSideEffects {
    pub messages: Vec<(AccountId, EgressMessage)>,
    pub instantiated: Vec<(AccountId, AccountId)>,
}

pub fn get_side_effects() -> ExecSideEffects {
    let mut result = ExecSideEffects::default();
    for event in super::System::events() {
        if let super::Event::Contracts(ink_event) = event.event {
            use pallet_contracts::Event as ContractEvent;
            match ink_event {
                ContractEvent::Instantiated(deployer, address) => {
                    result.instantiated.push((deployer, address))
                }
                ContractEvent::ContractEmitted(address, data) => {
                    if event.topics.len() != 1 {
                        continue;
                    }
                    if event.topics[0].0 == pink_extension::Message::event_topic() {
                        match pink_extension::Message::decode(&mut &data[..]) {
                            Ok(msg) => {
                                result.messages.push((address, EgressMessage::Message(msg)));
                            }
                            Err(_) => {
                                error!("Contract emitted invalid message");
                            }
                        }
                    } else if event.topics[0].0 == pink_extension::OspMessage::event_topic() {
                        match pink_extension::OspMessage::decode(&mut &data[..]) {
                            Ok(msg) => {
                                result.messages.push((address, EgressMessage::OspMessage(msg)));
                            }
                            Err(_) => {
                                error!("Contract emitted invalid osp message");
                            }
                        }
                    }
                }
                _ => (),
            }
        }
    }
    result
}

/// Contract extension for `pink contracts`
pub struct PinkExtension;

impl ChainExtension<super::PinkRuntime> for PinkExtension {
    fn call<E: Ext>(func_id: u32, _env: Environment<E, InitState>) -> Result<RetVal, DispatchError>
    where
        <E::T as SysConfig>::AccountId: UncheckedFrom<<E::T as SysConfig>::Hash> + AsRef<[u8]>,
    {
        error!(target: "pink", "Called an unregistered `func_id`: {:}", func_id);
        return Err(DispatchError::Other("Unimplemented func_id"));
    }
}

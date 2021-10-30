use super::pallet_mq;
use codec::Decode;
use frame_support::dispatch::{DispatchError, DispatchResult};
use phala_types::messaging::{BindTopic, DecodedMessage, Message};

pub struct MessageRouteConfig;

fn try_dispatch<Msg, Func>(func: Func, message: &Message) -> DispatchResult
where
    Msg: Decode + BindTopic,
    Func: Fn(DecodedMessage<Msg>) -> DispatchResult,
{
    if message.destination.path() == &Msg::topic() {
        let msg: DecodedMessage<Msg> = message
            .decode()
            .ok_or(DispatchError::Other("MessageCodecError"))?;
        return (func)(msg);
    }
    Ok(())
}

impl pallet_mq::QueueNotifyConfig for MessageRouteConfig {
    /// Handles an incoming message
    fn on_message_received(message: &Message) -> DispatchResult {
        use super::*;
        macro_rules! route_handlers {
            ($($handler: path,)+) => {
                $(try_dispatch($handler, message)?;)+
            }
        }

        route_handlers! {
            PhalaRegistry::on_message_received,
            PhalaMining::on_gk_message_received,
            PhalaMining::on_mining_message_received,
        };
        Ok(())
    }
}

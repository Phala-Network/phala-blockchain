use core::marker::PhantomData;

use super::{pallet_phala, pallet_mq};
use phala_types::messaging::{Message, Lottery, BindTopic};
use phala_pallets::phala_legacy::OnMessageReceived;

pub struct MessageRouteConfig<T>(PhantomData<T>);

impl<T> pallet_mq::QueueNotifyConfig for MessageRouteConfig<T> where
    T: pallet_phala::Config,
{
    /// Handles an incoming message
    fn on_message_received(message: &Message) {

        let result = match &message.destination.path()[..] {
            Lottery::TOPIC => T::OnLotteryMessage::on_message_received(message),
            _ => Ok(()),
        };

        if result.is_err() {
            // TODO.kevin. What can we do here?
        }
    }
}

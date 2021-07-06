use super::pallet_mq;
use phala_pallets::phala_legacy::OnMessageReceived;
use phala_types::messaging::{self, BindTopic, Lottery, Message, MiningReportEvent};
use super::pallet_registry::RegistryEvent;

type BalanceTransfer = messaging::BalanceTransfer<super::AccountId, super::Balance>;
type KittyTransfer = messaging::KittyTransfer<super::AccountId>;

pub struct MessageRouteConfig;

impl pallet_mq::QueueNotifyConfig for MessageRouteConfig {
    /// Handles an incoming message
    fn on_message_received(message: &Message) {
        let result = match &message.destination.path()[..] {
            Lottery::TOPIC => super::BridgeTransfer::on_message_received(message),
            BalanceTransfer::TOPIC => super::Phala::on_transfer_message_received(message),
            KittyTransfer::TOPIC => super::KittyStorage::on_message_received(message),
            MiningReportEvent::TOPIC => super::PhalaMining::on_message_received(message),
            RegistryEvent::TOPIC => super::PhalaRegistry::on_message_received(message),
            _ => Ok(()),
        };

        if result.is_err() {
            // TODO.kevin. What can we do here?
        }
    }
}

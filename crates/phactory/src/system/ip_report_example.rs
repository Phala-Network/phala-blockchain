use chain::BlockNumber;
use phala_mq::traits::{MessagePreparing as _, MessageChannel as _};
use phala_mq::MessageChannel;

use crate::side_task::SideTaskManager;

pub fn process_block(
    block_number: BlockNumber,
    egress: &MessageChannel,
    side_task_man: &mut SideTaskManager,
) {
    if block_number % 10 == 0 {
        let egress = egress.clone();
        let seq = match egress.make_appointment() {
            Ok(seq) => seq,
            Err(err) => {
                log::error!("Failed to make appointment: {:?}", err);
                return;
            }
        };
        let default_messages = [(seq, egress.prepare_message_to(&"Timeout", "a/mq/topic"))];
        side_task_man.add_async_task(
            block_number,
            10,
            default_messages,
            async move {
                log::info!("Side task start to get ip");
                let mut response = surf::get("https://ip.kvin.wang").send().await.unwrap();
                let ip = response.body_string().await.unwrap();
                log::info!("Side task got ip: {}", ip);
                Ok([(seq, egress.prepare_message_to(&ip, "a/mq/topic"))])
            },
        );
    }
}

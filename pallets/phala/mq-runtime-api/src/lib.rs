#![cfg_attr(not(feature = "std"), no_std)]

sp_api::decl_runtime_apis! {
	pub trait MqApi {
		fn sender_sequence(sender: &phala_mq::MessageOrigin) -> Option<u64>;
	}
}

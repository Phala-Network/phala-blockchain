use phactory_api::blocks;
use scale_info::{IntoPortable, PortableRegistry, TypeInfo};

fn travel_types<T: TypeInfo>() -> String {
    let mut registry = Default::default();
    let _ = T::type_info().into_portable(&mut registry);
    serde_json::to_string_pretty(PortableRegistry::from(registry).types()).unwrap()
}

#[test]
fn test_sync_blocks_abi_should_not_change() {
    insta::assert_display_snapshot!(travel_types::<blocks::SyncHeaderReq>());
    insta::assert_display_snapshot!(travel_types::<blocks::SyncParachainHeaderReq>());
    insta::assert_display_snapshot!(travel_types::<blocks::SyncCombinedHeadersReq>());
    insta::assert_display_snapshot!(travel_types::<blocks::DispatchBlockReq>());
}

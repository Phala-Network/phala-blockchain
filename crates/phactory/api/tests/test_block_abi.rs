#![allow(non_snake_case)]

use phactory_api::blocks;
use type_info_stringify::type_info_stringify;

#[test]
fn typeof_SyncHeaderReq_should_not_change() {
    insta::assert_display_snapshot!(type_info_stringify::<blocks::SyncHeaderReq>());
}

#[test]
fn typeof_SyncParachainHeaderReq_should_not_change() {
    insta::assert_display_snapshot!(type_info_stringify::<blocks::SyncParachainHeaderReq>());
}

#[test]
fn typeof_SyncCombinedHeadersReq_should_not_change() {
    insta::assert_display_snapshot!(type_info_stringify::<blocks::SyncCombinedHeadersReq>());
}

#[test]
fn typeof_DispatchBlockReq_should_not_change() {
    insta::assert_display_snapshot!(type_info_stringify::<blocks::DispatchBlockReq>());
}

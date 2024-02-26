#![no_std]

pub fn git_revision() -> &'static str {
    env!("PHALA_GIT_REVISION")
}

pub fn git_commit_timestamp() -> &'static str {
    env!("PHALA_GIT_COMMIT_TS")
}

pub fn git_revision_with_ts() -> &'static str {
    env!("PHALA_GIT_REVISION_WITH_TS")
}

#[test]
fn it_works() {
    assert!(git_revision_with_ts().starts_with(&git_revision()[..40]));
    assert!(git_revision_with_ts().ends_with(git_commit_timestamp()));
}

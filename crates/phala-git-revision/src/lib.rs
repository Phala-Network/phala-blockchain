#![no_std]

pub fn git_revision() -> &'static str {
    env!("PHALA_GIT_REVISION")
}

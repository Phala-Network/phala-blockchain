//! Immutable environment variables which are taken from the environment when the program starts
//! and never change.

#[ctor::ctor]
static VALUES: std::collections::HashMap<String, String> = std::env::vars().collect();

pub fn get(key: &str) -> Option<String> {
    VALUES.get(key).cloned()
}

#[test]
fn it_works() {
    let init = std::env::var("PATH").ok();
    std::env::set_var("PATH", "foo");
    assert_eq!(get("PATH"), init);
}
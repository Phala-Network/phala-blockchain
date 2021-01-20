# serde-name

[![serde-name on crates.io](https://img.shields.io/crates/v/serde-name)](https://crates.io/crates/serde-name)
[![Documentation (latest release)](https://docs.rs/serde-name/badge.svg)](https://docs.rs/serde-name/)
[![Documentation (master)](https://img.shields.io/badge/docs-master-brightgreen)](https://novifinancial.github.io/serde-reflection/serde_name/)
[![License](https://img.shields.io/badge/license-Apache-green.svg)](../LICENSE-APACHE)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](../LICENSE-MIT)

This crate provides a fast and reliable way to extract the Serde name of a Rust container.

```rust
#[derive(Deserialize)]
struct Foo {
  bar: Bar,
}

#[derive(Deserialize)]
#[serde(rename = "ABC")]
enum Bar { A, B, C }

assert_eq!(trace_name::<Foo>(), Some("Foo"));
assert_eq!(trace_name::<Bar>(), Some("ABC"));
assert_eq!(trace_name::<Option<Bar>>(), None);
```

## Contributing

See the [CONTRIBUTING](../CONTRIBUTING.md) file for how to help out.

## License

This project is available under the terms of either the [Apache 2.0 license](../LICENSE-APACHE) or the [MIT license](../LICENSE-MIT).

<!--
README.md is generated from README.tpl by cargo readme. To regenerate:

cargo install cargo-readme
cargo readme > README.md
-->

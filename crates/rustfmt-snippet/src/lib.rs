//! Format given Rust code snippet with rustfmt
//!
//! #Example
//! ```
//!    let formated = rustfmt_snippet::rustfmt(
//!        r#"fn main() {
//!        }"#,
//!    )
//!    .unwrap();
//!    assert_eq!(formated, "fn main() {}\n");
//! ```

/// Format given Rust code snippet with rustfmt
pub fn rustfmt(source: &str) -> std::io::Result<String> {
    use std::{
        io::Write,
        process::{Command, Stdio},
    };
    use tempfile::NamedTempFile;
    // Create a temporary file
    let mut tempfile: NamedTempFile = NamedTempFile::new()?;

    // Write the source code to the temporary file
    writeln!(tempfile, "{}", source)?;

    // Retrieve the path of the temporary file to pass it to rustfmt
    let path = tempfile.path();
    let path = format!("{}", path.display());
    let proc = Command::new("rustfmt")
        .args([path.as_str(), "--emit", "stdout", "--edition", "2018"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    proc.stdin
        .as_ref()
        .expect("It always exists")
        .write_all(source.as_bytes())?;
    let output = proc.wait_with_output().unwrap();
    let result = String::from_utf8_lossy(output.stdout.as_slice()).to_string();
    let pos = result.find("\n\n").unwrap() + 2;
    Ok(result[pos..].to_string())
}

/// Format given TokenStream with rustfmt
pub fn rustfmt_token_stream(stream: &proc_macro2::TokenStream) -> std::io::Result<String> {
    rustfmt(&format!("{stream}"))
}

#[test]
fn it_works() {
    let formated = rustfmt(
        r#"fn foo() {
        }"#,
    )
    .unwrap();
    assert_eq!(formated, "fn foo() {}\n");
}

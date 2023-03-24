use std::process::Command;

fn main() {
    export_git_revision();
}

fn export_git_revision() {
    let cmd = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .unwrap()
        .stdout;
    let revision = String::from_utf8_lossy(&cmd);
    let revision = revision.trim();
    let dirty = !Command::new("git")
        .args(["diff", "HEAD", "--quiet"])
        .output()
        .unwrap()
        .status
        .success();
    let tail = if dirty { "-dirty" } else { "" };
    if revision.is_empty() {
        println!("cargo:warning=⚠️ Failed to get git revision for pRuntime.");
        println!("cargo:warning=⚠️ Please ensure you have git installed and are compiling from a git repository.");
    }
    println!("cargo:rustc-env=PHALA_GIT_REVISION={revision}{tail}");
    println!("cargo:rerun-if-changed=always-rerun");
}

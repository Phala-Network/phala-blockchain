use std::process::Command;

fn main() {
    use tera::{Context, Tera};

    let tera = Tera::new("proto/*.proto").unwrap();

    let tmpdir = tempdir::TempDir::new("rendered_proto").unwrap();
    let render_dir = tmpdir.path();

    for tmpl in tera.templates.keys() {
        println!("cargo:rerun-if-changed=proto/{}", tmpl);
        let render_output = std::fs::File::create(render_dir.join(tmpl)).unwrap();
        tera.render_to(tmpl, &Context::new(), render_output)
            .unwrap();
    }

    let out_dir = "./src/proto_generated";

    let mut builder = prpc_build::configure()
        .out_dir(out_dir)
        .mod_prefix("crate::prpc::")
        .disable_package_emission();
    for r#type in [
        "InitRuntimeResponse",
        "Attestation",
        "TokenomicInfo",
        "WorkerState",
        "BenchState",
        "MiningState",
    ] {
        builder = builder.type_attribute(
            r#type,
            "#[cfg_attr(feature = \"serde\", derive(::serde::Serialize, ::serde::Deserialize))]",
        )
    }
    builder
        .compile(&["pruntime_rpc.proto"], &[render_dir])
        .unwrap();
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
    println!("cargo:rustc-env=PHALA_GIT_REVISION={}{}", revision, tail);
    println!("cargo:rerun-if-changed=always-rerun");
}

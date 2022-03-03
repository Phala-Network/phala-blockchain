
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

    prpc_build::configure()
        .out_dir(out_dir)
        .async_client(false)
        .async_server(true)
        .mod_prefix("crate::prpc::")
        .disable_package_emission()
        .compile(&["podtracker_rpc.proto"], &[render_dir])
        .unwrap();
}

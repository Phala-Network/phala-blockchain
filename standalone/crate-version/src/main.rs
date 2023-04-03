use clap::Parser;
use serde::Deserialize;

#[derive(Parser)]
struct Opts {
    #[clap(short = 'n', long, default_value = "2")]
    n_parts: usize,
    tomlfile: String,
}

#[derive(Deserialize)]
struct Crate {
    package: Package,
}

#[derive(Deserialize)]
struct Package {
    version: String,
}

fn main() {
    let opts = Opts::parse();
    let toml = std::fs::read_to_string(opts.tomlfile).expect("Failed to read Cargo.toml");
    let config: Crate = toml::from_str(&toml).expect("Failed to parse Cargo.toml");
    let ver_parts: Vec<_> = config
        .package
        .version
        .split('.')
        .take(opts.n_parts)
        .collect();
    println!("{}", ver_parts.join("."));
}

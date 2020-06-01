use std::process::Command;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use serde_json::{Value as JValue};
use toml_edit::{Document, Table, InlineTable, Value, value};

fn find_tomls() -> Vec<String> {
    let result = Command::new("cargo")
        .args(&["metadata", "--offline", "--no-deps", "--quiet"])
        .output()
        .expect("failed to execute process");

    let metadata: JValue = serde_json::from_slice(&result.stdout)
        .expect("bad metadata output");
    let tomls = metadata["packages"]
        .as_array().expect("no packages")
        .iter().map(|p| {
            p.as_object().unwrap()["manifest_path"].as_str().unwrap().to_string()
        }).collect();
    tomls
}

struct Update {
    known_version: HashMap<String, String>,
    version_map: HashMap<String, String>,
    num_updated: usize,
}

impl Update {
    fn new() -> Self {
        Update {
            known_version: Default::default(),
            version_map: Default::default(),
            num_updated: 0
        }
    }

    fn remember_version(&mut self, from: &str, to: &str) {
        if from != to {
            if let Some(orig) = self.version_map.insert(from.to_string(), to.to_string()) {
                if &orig != to {
                    panic!("Multiple version map [{}] => ({}, {})", from, orig, to);
                }
            }
        }
    }

    fn load_version(&mut self, base: &Path, path: &str) -> String {
        let mut path_buf = PathBuf::new();
        path_buf.push(base);
        path_buf.push(path);
        path_buf.push("Cargo.toml");
        let target_path = path_buf.as_path().canonicalize().unwrap();
        let target_string = target_path.to_str().unwrap().to_string();
        println!("\tat: {}", target_string);

        if let Some(ver) = self.known_version.get(&target_string) {
            return ver.clone();
        }

        let toml = std::fs::read_to_string(target_path)
            .expect("failed to load toml");
        let doc = toml.parse::<Document>()
            .expect("invalid toml");
        let ver = doc["package"]["version"].as_str().expect("no version").to_string();
        self.known_version.insert(target_string, ver.clone());

        ver
    }

    fn update_table(&mut self, base: &Path, k: &str, dep: &mut Table) {
        if dep.contains_key("path") && dep.contains_key("version") {
            println!("\tfound key: {}", k);
            let path = dep.get("path").unwrap().as_str().unwrap();
            let old_ver = dep.get("version").unwrap().as_str().unwrap();
            let ver = self.load_version(base, path);
            self.remember_version(old_ver, &ver);
            *dep.entry("version") = value(ver);
        }

    }

    fn update_inline_table(&mut self, base: &Path, k: &str, dep: &mut InlineTable) {
        if dep.contains_key("path") && dep.contains_key("version") {
            println!("\tfound key: {}", k);
            let path = dep.get("path").unwrap().as_str().unwrap();
            let old_ver = dep.get("version").unwrap().as_str().unwrap();
            let ver = self.load_version(base, path);
            self.remember_version(old_ver, &ver);
            *dep.get_mut("version").unwrap() = Value::from(ver);
            dep.fmt();
        }
    }

    fn update_deps(&mut self, base: &Path, deps: &mut Table) {
        let keys: Vec<String> = deps.iter().map(|(k,_)| k.to_string()).collect();
        for k in keys.iter() {
            if let Some(dep) = deps.entry(&k).as_table_mut() {
                self.update_table(base, &k, dep);
            } else if let Some(dep) = deps.entry(&k).as_inline_table_mut() {
                self.update_inline_table(base, &k, dep);
            }
        }
    }

    fn process_toml(&mut self, path: &String) {
        println!("# processing {}", path);
        let base_path = Path::new(path).parent().unwrap();
        // Read
        let toml = std::fs::read_to_string(path)
            .expect("failed to load toml");
        let mut doc = toml.parse::<Document>()
            .expect("invalid toml");
        // Update deps
        for section in &["dependencies", "dev-dependencies", "build-dependencies"] {
            if let Some(deps) = doc[*section].as_table_mut() {
                self.update_deps(base_path, deps);
            }
        }
        // Update conditional deps
        if doc["target"]["cfg(target_arch=\"x86_64\")"]["dependencies"].is_table() {
            let deps = doc["target"]["cfg(target_arch=\"x86_64\")"]["dependencies"].as_table_mut().unwrap();
            self.update_deps(base_path, deps);
        }
        // Update self version
        let version_val = &mut doc["package"]["version"];
        let version = version_val.as_str().expect("no version").to_string();
        if self.version_map.contains_key(&version) {
            let new_version = self.version_map[&version].clone();
            *version_val = value(new_version);
        }
        // Write
        let updated_toml = doc.to_string();
        if toml != updated_toml {
            std::fs::write(path, updated_toml)
                .expect("Failed to save updated toml");
            // println!("<<{}>>", doc.to_string());
            println!("> updated: {}", path);
            self.num_updated += 1;
        }
    }
}

fn main() {
    let tomls = find_tomls();
    println!("tomls: {:?}", tomls);

    let mut update = Update::new();
    for toml in &tomls {
        update.process_toml(&toml);
    }

    println!("Updated files: {} / {}", update.num_updated, tomls.len());
}

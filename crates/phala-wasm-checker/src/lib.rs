pub use error::ParseError;

use wasmparser::{Parser, Payload};

mod error;

#[derive(Default)]
pub struct WasmInfo {
    pub num_instructions: u32,
    pub num_functions: u32,
    pub const_data_size: usize,
}

impl WasmInfo {
    pub fn estimate_wasmi_memory_cost(&self) -> usize {
        // Each instruction takes 16 bytes in wasmi's memory
        self.num_instructions as usize * 16 + self.const_data_size
    }
}

pub fn wasm_info(data: &[u8]) -> Result<WasmInfo, ParseError> {
    let parser = Parser::new(0);
    let mut stats = WasmInfo::default();

    for payload in parser.parse_all(data) {
        match payload? {
            Payload::DataSection(data) => {
                for entry in data {
                    stats.const_data_size += entry?.data.len();
                }
            }
            Payload::CodeSectionEntry(body) => {
                stats.num_functions += 1;
                let mut reader = body.get_operators_reader()?;
                while !reader.eof() {
                    let _op = reader.read()?;
                    stats.num_instructions += 1;
                }
            }
            Payload::End(_) => {
                break;
            }
            _ => {}
        }
    }
    Ok(stats)
}

#[test]
fn test_invalid_wasm() {
    let wasm = b"foo";
    let info = wasm_info(wasm);
    assert!(info.is_err());
    let err = dbg!(info.err());
    println!("{}", err.unwrap());
}

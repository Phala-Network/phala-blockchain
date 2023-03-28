pub use error::ParseError;

use wasmparser::{Parser, Payload};

mod error;

pub struct WasmInfo {
    pub num_instructions: u32,
    pub num_functions: u32,
    pub const_data_size: usize,
}

impl WasmInfo {
    pub fn new() -> Self {
        WasmInfo {
            num_instructions: 0,
            num_functions: 0,
            const_data_size: 0,
        }
    }
}

impl WasmInfo {
    pub fn estimate_wasmi_memory_cost(&self) -> usize {
        // Each instruction takes 16 bytes in wasmi's memory
        self.num_instructions as usize * 16 + self.const_data_size
    }
}

pub fn wasm_info(data: &[u8]) -> Result<WasmInfo, ParseError> {
    let parser = Parser::new(0);
    let mut stats = WasmInfo::new();

    for payload in parser.parse_all(data) {
        match payload? {
            Payload::Version { .. } => {}
            Payload::TypeSection(_) => {}
            Payload::ImportSection(_) => {}
            Payload::FunctionSection(_) => {}
            Payload::TableSection(_) => {}
            Payload::MemorySection(_) => {}
            Payload::TagSection(_) => {}
            Payload::GlobalSection(_) => {}
            Payload::ExportSection(_) => {}
            Payload::StartSection { .. } => {}
            Payload::ElementSection(_) => {}
            Payload::DataCountSection { .. } => {}
            Payload::DataSection(data) => {
                for entry in data {
                    stats.const_data_size += entry?.data.len();
                }
            }
            Payload::CodeSectionStart { .. } => {}
            Payload::CodeSectionEntry(body) => {
                stats.num_functions += 1;
                let mut reader = body.get_operators_reader()?;
                while !reader.eof() {
                    let _op = reader.read()?;
                    stats.num_instructions += 1;
                }
            }
            Payload::ModuleSection { .. } => {}
            Payload::InstanceSection(_) => {}
            Payload::CoreTypeSection(_) => {}
            Payload::ComponentSection { .. } => {}
            Payload::ComponentInstanceSection(_) => {}
            Payload::ComponentAliasSection(_) => {}
            Payload::ComponentTypeSection(_) => {}
            Payload::ComponentCanonicalSection(_) => {}
            Payload::ComponentStartSection { .. } => {}
            Payload::ComponentImportSection(_) => {}
            Payload::ComponentExportSection(_) => {}
            Payload::CustomSection(_) => {}
            Payload::UnknownSection { .. } => {}
            Payload::End(_) => {
                break;
            }
        }
    }
    Ok(stats)
}

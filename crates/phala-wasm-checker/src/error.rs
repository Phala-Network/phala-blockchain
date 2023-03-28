use std::error::Error;

#[derive(Debug)]
pub enum ParseError {
    WasmError(wasmparser::BinaryReaderError),
}
impl Error for ParseError {}
impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ParseError::WasmError(e) => write!(f, "Wasm error: {}", e),
        }
    }
}
impl From<wasmparser::BinaryReaderError> for ParseError {
    fn from(e: wasmparser::BinaryReaderError) -> Self {
        ParseError::WasmError(e)
    }
}

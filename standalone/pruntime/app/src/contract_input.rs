use serde_json::{Map, Value};

#[derive(Serialize, Deserialize)]
pub struct ContractInput {
    pub input: Map<String, Value>,
    pub nonce: Map<String, Value>,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}


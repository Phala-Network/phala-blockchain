use sidevm::env::messages::{AccountId, Metric, SystemMessage, H256};
use std::{collections::VecDeque, ops::Deref};
use this_crate::VersionTuple;

pub struct Buffer {
    next_sequence: u64,
    capacity: usize,
    current_size: usize,
    records: VecDeque<Box<Record>>,
}

struct Record {
    contract_id: String,
    entry_contract: String,
    message: Message,
    size: usize,
    sequence: u64,
    block_number: Option<u32>,
}

enum Message {
    Origin(SerMessage),
    Encoded(String),
}

impl Record {
    fn encoded(&mut self) -> &str {
        if let Message::Origin(message) = &self.message {
            #[derive(serde::Serialize)]
            struct MessageWrapper<'a> {
                sequence: u64,
                #[serde(flatten)]
                message: &'a SerMessage,
            }
            let wrapped = MessageWrapper {
                sequence: self.sequence,
                message,
            };
            let s = serde_json::to_string(&wrapped).expect("Failed to serialize message");
            self.message = Message::Encoded(s);
        }
        match &self.message {
            Message::Encoded(s) => s.as_str(),
            Message::Origin(_) => unreachable!(),
        }
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct SerInfo {
    program_version: VersionTuple,
    next_sequence: u64,
    memory_capacity: usize,
    memory_usage: usize,
    current_number_of_records: usize,
    estimated_current_size: usize,
}

struct HexSer<T>(T);

impl<T> From<T> for HexSer<T> {
    fn from(it: T) -> Self {
        Self(it)
    }
}

impl<T> Deref for HexSer<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: AsRef<[u8]>> serde::Serialize for HexSer<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let hex = format!("0x{}", hex_fmt::HexFmt(self.0.as_ref()));
        hex.serialize(serializer)
    }
}

#[derive(serde::Serialize)]
#[serde(tag = "type")]
enum SerMessage {
    #[serde(rename_all = "camelCase")]
    Log {
        block_number: u32,
        contract: HexSer<AccountId>,
        entry: HexSer<AccountId>,
        exec_mode: String,
        #[serde(rename = "timestamp")]
        timestamp_ms: u64,
        level: u8,
        message: String,
    },
    #[serde(rename_all = "camelCase")]
    Event {
        block_number: u32,
        contract: HexSer<AccountId>,
        topics: Vec<HexSer<H256>>,
        payload: HexSer<Vec<u8>>,
    },
    #[serde(rename_all = "camelCase")]
    MessageOutput {
        block_number: u32,
        origin: HexSer<AccountId>,
        contract: HexSer<AccountId>,
        nonce: HexSer<Vec<u8>>,
        output: HexSer<Vec<u8>>,
    },
    TooLarge,
    #[serde(rename_all = "camelCase")]
    QueryIn {
        user: HexSer<[u8; 8]>,
    },
}

impl SerMessage {
    fn size(&self) -> usize {
        let payload = match &self {
            SerMessage::Log { message, .. } => message.len(),
            SerMessage::Event {
                topics, payload, ..
            } => {
                let topics_len: usize = topics.iter().map(|x| x.len() * 2).sum();
                topics_len + payload.len() * 2
            }
            SerMessage::MessageOutput { nonce, output, .. } => nonce.len() * 2 + output.len() * 2,
            SerMessage::TooLarge => 0,
            SerMessage::QueryIn { user } => user.len() * 2,
        };
        128 + payload
    }

    fn block_number(&self) -> Option<u32> {
        match self {
            SerMessage::Log { block_number, .. }
            | SerMessage::Event { block_number, .. }
            | SerMessage::MessageOutput { block_number, .. } => Some(*block_number),
            SerMessage::TooLarge | SerMessage::QueryIn { .. } => None,
        }
    }
}

impl From<SystemMessage> for SerMessage {
    fn from(it: SystemMessage) -> Self {
        match it {
            SystemMessage::PinkLog {
                block_number,
                contract,
                entry,
                exec_mode,
                timestamp_ms,
                level,
                message,
            } => Self::Log {
                block_number,
                contract: contract.into(),
                entry: entry.into(),
                exec_mode,
                timestamp_ms,
                level,
                message,
            },
            SystemMessage::PinkEvent {
                block_number,
                contract,
                topics,
                payload,
            } => Self::Event {
                block_number,
                contract: contract.into(),
                topics: topics.into_iter().map(Into::into).collect(),
                payload: payload.into(),
            },
            SystemMessage::PinkMessageOutput {
                block_number,
                origin,
                contract,
                nonce,
                output,
            } => Self::MessageOutput {
                block_number,
                origin: origin.into(),
                contract: contract.into(),
                nonce: nonce.into(),
                output: output.into(),
            },
            SystemMessage::Metric(Metric::PinkQueryIn(user)) => {
                Self::QueryIn { user: HexSer(user) }
            }
        }
    }
}

fn hex(data: &[u8]) -> String {
    format!("0x{}", hex_fmt::HexFmt(data))
}

fn contract_id_of(sysmessage: &SystemMessage) -> String {
    let id = match sysmessage {
        SystemMessage::PinkLog { contract, .. } => contract,
        SystemMessage::PinkEvent { contract, .. } => contract,
        SystemMessage::PinkMessageOutput { contract, .. } => contract,
        SystemMessage::Metric(_) => return "<metric>".into(),
    };
    hex(id)
}

fn entry_of(sysmessage: &SystemMessage) -> String {
    let id = match sysmessage {
        SystemMessage::PinkLog { entry, .. } => entry,
        SystemMessage::PinkEvent { contract, .. } => contract,
        SystemMessage::PinkMessageOutput { contract, .. } => contract,
        SystemMessage::Metric(_) => return "<metric>".into(),
    };
    hex(id)
}

impl Buffer {
    pub fn new(capacity: usize) -> Self {
        Buffer {
            next_sequence: 0,
            capacity,
            current_size: 0,
            records: Default::default(),
        }
    }

    pub fn push(&mut self, message: SystemMessage) {
        let contract_id = contract_id_of(&message);
        let entry_contract = entry_of(&message);
        let mut message: SerMessage = message.into();
        let mut size = message.size();
        let block_number = message.block_number();
        if size > self.capacity {
            message = SerMessage::TooLarge;
            size = message.size();
        }
        while self.capacity < crate::allocator::mem_usage() + size && !self.records.is_empty() {
            self.pop();
        }
        self.current_size += size;
        self.records.push_back(Box::new(Record {
            contract_id,
            entry_contract,
            message: Message::Origin(message),
            size,
            sequence: self.next_sequence,
            block_number,
        }));
        self.next_sequence += 1;
    }

    fn pop(&mut self) -> Option<Record> {
        let rec = self.records.pop_front()?;
        self.current_size -= rec.size;
        Some(*rec)
    }

    pub fn get_records(
        &mut self,
        contract: &str,
        from: i64,
        count: u64,
        block_number: Option<u32>,
    ) -> String {
        let count = if count == 0 { u64::MAX } else { count };
        let mut result: String = "{\"records\":[".into();
        let mut n = 0_u64;
        let mut next_seq = 0_u64;
        let from = if from < 0 {
            self.next_sequence + from as u64
        } else {
            from as u64
        };
        for rec in self.records.iter_mut() {
            next_seq = rec.sequence + 1;
            if rec.sequence < from {
                continue;
            }
            if let Some(block_number) = block_number {
                if rec.block_number != Some(block_number) {
                    continue;
                }
            }
            if contract.is_empty() || rec.contract_id == contract || rec.entry_contract == contract
            {
                if n > 0 {
                    result.push(',');
                }
                result.push_str(rec.encoded());
                n += 1;
                if n >= count {
                    break;
                }
            }
        }
        result.push_str(&format!(r#"],"next":{next_seq}}}"#));
        result
    }

    pub fn get_info(&self) -> String {
        let info = SerInfo {
            next_sequence: self.next_sequence,
            program_version: this_crate::version_tuple!(),
            memory_capacity: self.capacity,
            memory_usage: crate::allocator::mem_usage(),
            current_number_of_records: self.records.len(),
            estimated_current_size: self.current_size,
        };
        serde_json::to_string(&info).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pretty(s: &str) -> String {
        let v: serde_json::Value = serde_json::from_str(s).unwrap();
        serde_json::to_string_pretty(&v).unwrap()
    }

    fn test_buffer(cap: usize) -> Buffer {
        let mut buffer = Buffer::new(cap);
        buffer.push(SystemMessage::PinkLog {
            block_number: 0,
            contract: [1u8; 32],
            timestamp_ms: 1,
            level: 0,
            message: "hello".into(),
            entry: [1u8; 32],
            exec_mode: "query".into(),
        });
        buffer.push(SystemMessage::PinkEvent {
            block_number: 1,
            contract: [1; 32],
            topics: vec![[2; 32], [3; 32]],
            payload: vec![1, 2, 3, 4],
        });
        buffer.push(SystemMessage::PinkMessageOutput {
            block_number: 2,
            origin: [1; 32],
            contract: [2; 32],
            nonce: vec![1, 2, 3, 4, 5],
            output: vec![5, 4, 3, 2, 1],
        });
        buffer
    }

    #[test]
    fn it_works() {
        let mut buffer = test_buffer(1024);
        insta::assert_display_snapshot!(pretty(&buffer.get_records("".into(), 0, 0, None)));
    }

    #[test]
    fn it_can_rotate() {
        let mut buffer = test_buffer(256);
        insta::assert_display_snapshot!(pretty(&buffer.get_records("".into(), 0, 0, None)));
    }

    #[test]
    fn it_can_filter_by_contract_id() {
        let mut buffer = test_buffer(1024);
        let contract = [1; 32];
        insta::assert_display_snapshot!(pretty(&buffer.get_records(&hex(&contract), 0, 0, None)));
    }

    #[test]
    fn it_can_query_with_from() {
        let mut buffer = test_buffer(1024);
        insta::assert_display_snapshot!(pretty(&buffer.get_records("".into(), 1, 0, None)));
        insta::assert_display_snapshot!(pretty(&buffer.get_records("".into(), 4, 0, None)));
    }

    #[test]
    fn it_can_query_with_count_limit() {
        let mut buffer = test_buffer(1024);
        insta::assert_display_snapshot!(pretty(&buffer.get_records("".into(), 0, 1, None)));
    }

    #[test]
    fn it_can_query_with_all_conditions() {
        let mut buffer = test_buffer(1024);
        buffer.push(SystemMessage::PinkEvent {
            block_number: 1,
            contract: [1; 32],
            topics: vec![],
            payload: vec![1],
        });
        insta::assert_display_snapshot!(pretty(&buffer.get_records(&hex(&[1; 32]), 1, 1, None)));
    }
}

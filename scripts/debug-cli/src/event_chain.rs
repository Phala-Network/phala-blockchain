use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use anyhow::{anyhow, Result};
use pink::types::EventsBlock;

use codec::Decode;

fn decode_events_block(bytes: &[u8]) -> Result<EventsBlock> {
    Ok(Decode::decode(&mut &bytes[..])?)
}

fn try_decode_log_line(line: &str) -> Result<EventsBlock> {
    if !line.contains("event_chain") {
        anyhow::bail!("Not a log line")
    };
    let payload = line
        .split("payload=")
        .nth(1)
        .ok_or_else(|| anyhow!("Invalid log line"))?;
    let payload = hex::decode(payload)?;
    decode_events_block(&payload)
}

fn process_log_reader(reader: impl BufRead) -> impl Iterator<Item = Result<EventsBlock>> {
    reader.lines().map(|line_result| {
        line_result
            .map_err(anyhow::Error::from)
            .and_then(|line| try_decode_log_line(&line))
    })
}

pub(crate) fn process_log_file(
    path: impl AsRef<Path>,
) -> Result<impl Iterator<Item = Result<EventsBlock>>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    Ok(process_log_reader(reader))
}

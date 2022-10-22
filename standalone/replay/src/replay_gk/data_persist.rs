use super::EventRecord;
use anyhow::Result;
use chrono::TimeZone as _;
use phactory::gk;
use sqlx::types::Decimal;
use sqlx::{postgres::PgPoolOptions, Row};
use std::time::Duration;
use tokio::sync::mpsc;

pub(super) async fn run_persist(mut rx: mpsc::Receiver<EventRecord>, uri: &str) {
    log::info!("Connecting to {}", uri);

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(uri)
        .await
        .expect("Connect to database failed");

    let mut stopped = false;

    while !stopped {
        let mut records = vec![];
        loop {
            match tokio::time::timeout(Duration::from_secs(2), rx.recv()).await {
                Ok(Some(record)) => {
                    records.push(record);

                    const BATCH_SIZE: usize = 1000;
                    if records.len() >= BATCH_SIZE {
                        break;
                    }
                }
                Ok(None) => {
                    log::info!("data channel closed");
                    stopped = true;
                    break;
                }
                Err(_) => {
                    // Did not receive anything for 2 seconds,
                    break;
                }
            };
        }
        if !records.is_empty() {
            log::info!("Inserting {} records.", records.len());
            'try_insert: loop {
                match insert_records(&pool, &records).await {
                    Ok(()) => {
                        break;
                    }
                    Err(err) => {
                        log::error!("Insert {} records error.", records.len());
                        log::error!("{}", err);
                        match get_last_sequence(&pool).await {
                            Ok(last_sequence) => {
                                log::info!("last_sequence={}", last_sequence);
                                if last_sequence
                                    >= records.last().expect("records can not be empty").sequence
                                {
                                    log::info!("Insert succeeded, let's move on");
                                    break;
                                }
                                records.retain(|r| r.sequence > last_sequence);
                                log::info!("Insert records failed, try again");
                                continue 'try_insert;
                            }
                            Err(err) => {
                                // Error, let's try to insert again later.
                                let delay = 5;
                                log::error!("{}", err);
                                log::error!("Try again in {}s", delay);
                                tokio::time::sleep(Duration::from_secs(delay)).await;
                                continue 'try_insert;
                            }
                        }
                    }
                }
            }
        }
    }
}

async fn insert_records(pool: &sqlx::Pool<sqlx::Postgres>, records: &[EventRecord]) -> Result<()> {
    // Current version of sqlx does not support bulk insertion, so we have to do it manually.
    let mut sequences = vec![];
    let mut pubkeys = vec![];
    let mut block_numbers = vec![];
    let mut timestamps = vec![];
    let mut events = vec![];
    let mut vs = vec![];
    let mut ps = vec![];
    let mut payouts = vec![];

    let last_seq = get_last_sequence(pool).await?;

    for rec in records {
        if rec.sequence <= last_seq {
            continue;
        }
        sequences.push(rec.sequence);
        pubkeys.push(rec.pubkey.0.to_vec());
        block_numbers.push(rec.block_number);
        timestamps.push(chrono::Utc.timestamp_millis(rec.time_ms as _));
        events.push(rec.event.event_string());
        vs.push(cvt_fp(rec.v));
        ps.push(cvt_fp(rec.p));
        payouts.push(cvt_fp(rec.event.payout()));
    }

    if sequences.is_empty() {
        return Ok(());
    }

    sqlx::query(
        r#"
        INSERT INTO worker_finance_events
            (sequence, pubkey, block, time, event, v, p, payout)
        SELECT *
        FROM UNNEST($1, $2, $3, $4, $5, $6, $7, $8)
        ON CONFLICT (time, sequence)
        DO UPDATE
        SET (pubkey, block, event, v, p, payout) = (
            EXCLUDED.pubkey, EXCLUDED.block, EXCLUDED.event, EXCLUDED.v, EXCLUDED.p,
            EXCLUDED.payout
        )
        "#,
    )
    .bind(&sequences)
    .bind(&pubkeys)
    .bind(&block_numbers)
    .bind(&timestamps)
    .bind(&events)
    .bind(&vs)
    .bind(&ps)
    .bind(&payouts)
    .execute(pool)
    .await?;

    log::debug!("Inserted {} records.", records.len());

    Ok(())
}

fn cvt_fp(v: gk::FixedPoint) -> Decimal {
    Decimal::from_i128_with_scale((v * 10000000000).to_num(), 10)
}

async fn get_last_sequence(pool: &sqlx::Pool<sqlx::Postgres>) -> Result<i64> {
    let latest_row =
        sqlx::query("SELECT sequence FROM worker_finance_events ORDER BY sequence DESC LIMIT 1")
            .fetch_optional(pool)
            .await?;
    Ok(latest_row.map_or(0, |row| row.get(0)))
}

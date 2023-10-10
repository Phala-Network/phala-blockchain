use pink::types::AccountId;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, sync::Mutex};

/// Represents outgoing HTTP requests statistics.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct HttpGlobolCounters {
    /// Global HTTP counters for all contracts.
    pub global: HttpCoutners,
    /// HTTP counters grouped by contract account ID.
    pub by_contract: BTreeMap<AccountId, HttpCoutners>,
}

/// Represents HTTP counters for a single contract or globally.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct HttpCoutners {
    /// Time of the latest HTTP activitiy.
    latest_activity: u64,
    /// Number of HTTP requests.
    pub requests: u64,
    /// Number of failed HTTP requests.
    pub failures: u64,
    /// HTTP counters grouped by status code.
    pub by_status_code: BTreeMap<u16, u64>,
}

static HTTP_COUNTERS: once_cell::sync::OnceCell<Mutex<HttpGlobolCounters>> =
    once_cell::sync::OnceCell::new();

pub(super) fn counters() -> &'static Mutex<HttpGlobolCounters> {
    HTTP_COUNTERS.get_or_init(|| Mutex::new(HttpGlobolCounters::default()))
}

pub(super) fn add(contract: AccountId, status_code: u16) {
    let success = status_code / 100 == 2;
    let mut counters = counters().lock().unwrap();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    counters.global.latest_activity = now;
    counters.global.requests += 1;
    if !success {
        counters.global.failures += 1;
    }
    if status_code != 0 {
        *counters
            .global
            .by_status_code
            .entry(status_code)
            .or_insert(0) += 1;
    }

    let counters = counters
        .by_contract
        .entry(contract)
        .or_default();
    counters.latest_activity = now;
    counters.requests += 1;
    if !success {
        counters.failures += 1;
    }
    if status_code != 0 {
        *counters.by_status_code.entry(status_code).or_insert(0) += 1;
    }
}

pub(crate) fn stats() -> HttpGlobolCounters {
    counters().lock().unwrap().clone()
}

pub(crate) fn stats_for(contract: &AccountId) -> HttpCoutners {
    counters()
        .lock()
        .unwrap()
        .by_contract
        .get(contract)
        .cloned()
        .unwrap_or_default()
}

pub(crate) fn stats_global() -> HttpCoutners {
    counters().lock().unwrap().global.clone()
}

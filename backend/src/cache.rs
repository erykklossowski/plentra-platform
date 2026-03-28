use chrono::{DateTime, Utc};
use dashmap::DashMap;

#[derive(Clone)]
pub struct CachedEntry {
    pub data: serde_json::Value,
    pub fetched_at: DateTime<Utc>,
    pub ttl_seconds: u64,
}

impl CachedEntry {
    pub fn is_expired(&self) -> bool {
        let elapsed = Utc::now()
            .signed_duration_since(self.fetched_at)
            .num_seconds();
        elapsed > self.ttl_seconds as i64
    }
}

#[derive(Clone)]
pub struct Cache {
    store: DashMap<String, CachedEntry>,
}

impl Cache {
    pub fn new() -> Self {
        Self {
            store: DashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<CachedEntry> {
        self.store.get(key).and_then(|entry| {
            if entry.is_expired() {
                None
            } else {
                Some(entry.clone())
            }
        })
    }

    pub fn get_stale(&self, key: &str) -> Option<CachedEntry> {
        self.store.get(key).map(|entry| entry.clone())
    }

    pub fn set(&self, key: String, data: serde_json::Value, ttl: u64) {
        self.store.insert(
            key,
            CachedEntry {
                data,
                fetched_at: Utc::now(),
                ttl_seconds: ttl,
            },
        );
    }
}

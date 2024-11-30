use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct Cache {
    data: Arc<RwLock<HashMap<String, CacheEntry>>>,
    cache_duration: Duration,
}

struct CacheEntry {
    value: String,
    timestamp: Instant,
}

impl Cache {
    pub fn new(duration_secs: u64) -> Self {
        Cache {
            data: Arc::new(RwLock::new(HashMap::new())),
            cache_duration: Duration::from_secs(duration_secs),
        }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        let cache = self.data.read().unwrap();
        if let Some(entry) = cache.get(key) {
            if entry.timestamp.elapsed() < self.cache_duration {
                println!("Cache hit for key: {}", key);
                return Some(entry.value.clone());
            } else {
                println!("Cache expired for key: {}", key);
            }
        }
        None
    }

    pub fn set(&self, key: &str, value: String) {
        let mut cache = self.data.write().unwrap();
        cache.insert(key.to_string(), CacheEntry {
            value,
            timestamp: Instant::now(),
        });

        cache.retain(|_, entry| entry.timestamp.elapsed() < self.cache_duration);
        
        println!("Cache updated for key: {}", key);
    }
}
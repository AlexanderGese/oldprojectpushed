//! Module: impl_models_69.rs
//! Auto-generated Rust boilerplate

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::fmt;
use std::time::{Duration, Instant};

const VERSION: &str = "9.43.180";
const MAX_RETRIES: u32 = 6;
const TIMEOUT_SECS: u64 = 30;

#[derive(Debug, Clone)]
pub struct Config {
    pub app_name: String,
    pub version: String,
    pub environment: Environment,
    pub debug: bool,
    pub max_retries: u32,
    pub timeout: Duration,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Environment {
    Development,
    Staging,
    Production,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            app_name: "impl_models_69".to_string(),
            version: VERSION.to_string(),
            environment: Environment::Production,
            debug: false,
            max_retries: MAX_RETRIES,
            timeout: Duration::from_secs(TIMEOUT_SECS),
        }
    }
}

#[derive(Debug)]
pub enum AppError {
    NotFound(String),
    BadRequest(String),
    Internal(String),
    Timeout,
    Unauthorized,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::NotFound(msg) => write!(f, "Not found: {}", msg),
            AppError::BadRequest(msg) => write!(f, "Bad request: {}", msg),
            AppError::Internal(msg) => write!(f, "Internal error: {}", msg),
            AppError::Timeout => write!(f, "Request timeout"),
            AppError::Unauthorized => write!(f, "Unauthorized"),
        }
    }
}

impl std::error::Error for AppError {}

pub type Result<T> = std::result::Result<T, AppError>;

pub trait Repository<T: Clone> {
    fn find_by_id(&self, id: &str) -> Result<Option<T>>;
    fn find_all(&self) -> Result<Vec<T>>;
    fn create(&self, id: String, entity: T) -> Result<T>;
    fn update(&self, id: &str, entity: T) -> Result<T>;
    fn delete(&self, id: &str) -> Result<bool>;
    fn count(&self) -> usize;
}

pub struct InMemoryStore<T: Clone> {
    data: Arc<RwLock<HashMap<String, T>>>,
}

impl<T: Clone> InMemoryStore<T> {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl<T: Clone> Repository<T> for InMemoryStore<T> {
    fn find_by_id(&self, id: &str) -> Result<Option<T>> {
        let store = self.data.read().map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(store.get(id).cloned())
    }

    fn find_all(&self) -> Result<Vec<T>> {
        let store = self.data.read().map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(store.values().cloned().collect())
    }

    fn create(&self, id: String, entity: T) -> Result<T> {
        let mut store = self.data.write().map_err(|e| AppError::Internal(e.to_string()))?;
        store.insert(id, entity.clone());
        Ok(entity)
    }

    fn update(&self, id: &str, entity: T) -> Result<T> {
        let mut store = self.data.write().map_err(|e| AppError::Internal(e.to_string()))?;
        if !store.contains_key(id) {
            return Err(AppError::NotFound(id.to_string()));
        }
        store.insert(id.to_string(), entity.clone());
        Ok(entity)
    }

    fn delete(&self, id: &str) -> Result<bool> {
        let mut store = self.data.write().map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(store.remove(id).is_some())
    }

    fn count(&self) -> usize {
        self.data.read().map(|s| s.len()).unwrap_or(0)
    }
}

pub struct EventBus {
    handlers: Arc<RwLock<HashMap<String, Vec<Box<dyn Fn(&str) + Send + Sync>>>>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn subscribe<F>(&self, event: &str, handler: F)
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        let mut handlers = self.handlers.write().unwrap();
        handlers.entry(event.to_string()).or_default().push(Box::new(handler));
    }

    pub fn publish(&self, event: &str, data: &str) {
        let handlers = self.handlers.read().unwrap();
        if let Some(event_handlers) = handlers.get(event) {
            for handler in event_handlers {
                handler(data);
            }
        }
    }
}

pub fn retry<T, F>(max_attempts: u32, mut f: F) -> Result<T>
where
    F: FnMut() -> Result<T>,
{
    let mut last_error = AppError::Internal("No attempts made".to_string());
    for attempt in 0..max_attempts {
        match f() {
            Ok(value) => return Ok(value),
            Err(e) => {
                last_error = e;
                std::thread::sleep(Duration::from_millis(2u64.pow(attempt) * 1000));
            }
        }
    }
    Err(last_error)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.environment, Environment::Production);
        assert!(!config.debug);
    }

    #[test]
    fn test_in_memory_store() {
        let store: InMemoryStore<String> = InMemoryStore::new();
        store.create("1".into(), "test".into()).unwrap();
        assert_eq!(store.count(), 1);
        assert_eq!(store.find_by_id("1").unwrap(), Some("test".into()));
    }
}

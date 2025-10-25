use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEvent {
    pub did: String,
    pub event: String,
    pub timestamp: DateTime<Utc>,
}

pub struct AuditLogService;

impl AuditLogService {
    pub fn new() -> Self {
        Self
    }

    pub fn log(&self, did: &str, event: &str) {
        let log_event = AuditLogEvent {
            did: did.to_string(),
            event: event.to_string(),
            timestamp: Utc::now(),
        };

        // In a real implementation, this would write to a dedicated, immutable log.
        // For now, we'll just log to the console.
        println!("{}", serde_json::to_string(&log_event).unwrap());
    }
}

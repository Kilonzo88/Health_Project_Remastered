use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::database::Database;
use crate::models::AuditLog;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEvent {
    pub did: String,
    pub event: String,
    pub timestamp: DateTime<Utc>,
}

pub struct AuditLogService {
    db: Arc<Database>,
}

impl AuditLogService {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    pub async fn log(&self, did: &str, action: &str, details: Option<serde_json::Value>) {
        let log_entry = AuditLog {
            id: None,
            did: did.to_string(),
            action: action.to_string(),
            timestamp: Utc::now(),
            details,
            is_anchored: false,
            anchor_batch_id: None,
        };

        if let Err(e) = self.db.create_audit_log(&log_entry).await {
            // In a real-world scenario, you might want more robust error handling,
            // like a fallback to logging to a file or a different service.
            eprintln!("Failed to write audit log to database: {}", e);
        }
    }
}

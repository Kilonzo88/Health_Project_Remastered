
use std::sync::Arc;
use crate::config::Config;
use crate::database::Database;
use crate::models::*;
use crate::auditing::AuditLogService;

// --- PatientService ---
pub struct PatientService {
    db: Arc<Database>,
    config: Arc<Config>,
    audit_log_service: Arc<AuditLogService>,
}

impl PatientService {
    pub fn new(db: Arc<Database>, config: Arc<Config>, audit_log_service: Arc<AuditLogService>) -> Self {
        Self { db, config, audit_log_service }
    }
    pub async fn get_patient(&self, did: &str) -> anyhow::Result<Option<Patient>> {
        self.audit_log_service.log(did, "get_patient", None).await;
        self.db.get_patient_by_did(did, &self.config.ipfs_encryption_key).await
    }
}

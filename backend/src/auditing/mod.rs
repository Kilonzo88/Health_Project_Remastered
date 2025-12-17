pub mod audit_log;

use std::sync::Arc;
use anyhow::Result;
use rs_merkle::{MerkleTree, algorithms::Sha256 as MerkleSha256};
use sha2::{Digest, Sha256};
use bson::oid::ObjectId;

use crate::database::Database;
use crate::services::hedera::HealthcareHederaService;

pub use audit_log::AuditLogService;

pub struct AuditingService {
    db: Arc<Database>,
    hedera_service: Arc<HealthcareHederaService>,
}

impl AuditingService {
    pub fn new(db: Arc<Database>, hedera_service: Arc<HealthcareHederaService>) -> Self {
        Self { db, hedera_service }
    }

    pub async fn anchor_audit_logs(&self) -> Result<()> {
        let logs = self.db.get_unanchored_audit_logs().await?;
        if logs.is_empty() {
            println!("No new audit logs to anchor.");
            return Ok(());
        }

        println!("Found {} new audit logs to anchor.", logs.len());

        let log_ids: Vec<ObjectId> = logs.iter().map(|log| log.id.unwrap()).collect();

        let leaf_hashes: Vec<[u8; 32]> = logs
            .iter()
            .map(|log| {
                let serialized_log = serde_json::to_string(log).unwrap();
                let mut hasher = Sha256::new();
                hasher.update(serialized_log.as_bytes());
                let result = hasher.finalize();
                let mut hash = [0u8; 32];
                hash.copy_from_slice(&result);
                hash
            })
            .collect();

        let merkle_tree = MerkleTree::<MerkleSha256>::from_leaves(&leaf_hashes);
        let merkle_root = merkle_tree
            .root()
            .ok_or_else(|| anyhow::anyhow!("Failed to get Merkle root"))?;

        println!(
            "Calculated Merkle root: {:?}, batch size: {}",
            merkle_root,
            logs.len()
        );

        // Call hedera_service to anchor the root
        let transaction_record = self
            .hedera_service
            .anchor_log_batch(merkle_root, logs.len() as u64)
            .await?;
        println!(
            "Successfully anchored log batch. Transaction ID: {:?}",
            transaction_record.transaction_id
        );

        // Mark logs as anchored in the database
        let anchor_batch_id = ObjectId::new();
        self.db
            .mark_logs_as_anchored(&log_ids, anchor_batch_id)
            .await?;
        println!(
            "Successfully marked {} logs as anchored with batch ID: {}",
            log_ids.len(),
            anchor_batch_id
        );

        Ok(())
    }
}

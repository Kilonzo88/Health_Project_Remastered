
use anyhow::anyhow;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;

use crate::config::Config;
use crate::database::Database;
use crate::ipfs::IpfsClient;
use crate::models::*;
use crate::auditing::AuditLogService;
use crate::api::handlers::{CreateEncounterRequest};
use crate::fhir::FhirManager;
use crate::utils;

// --- EncounterService ---
pub struct EncounterService {
    db: Arc<Database>,
    ipfs_client: Arc<IpfsClient>,
    config: Arc<Config>,
    audit_log_service: Arc<AuditLogService>,
}

impl EncounterService {
    pub fn new(db: Arc<Database>, ipfs_client: Arc<IpfsClient>, config: Arc<Config>, audit_log_service: Arc<AuditLogService>) -> Self {
        Self { db, ipfs_client, config, audit_log_service }
    }

    pub async fn create_encounter(&self, request: CreateEncounterRequest) -> anyhow::Result<Encounter> {
        let fhir_encounter = FhirEncounter {
            resource_type: "Encounter".to_string(),
            id: Uuid::new_v4().to_string(),
            status: "in-progress".to_string(),
            class: request.class,
            subject: FhirReference { reference: format!("Patient/{}", request.patient_did), display: None },
            participant: vec![FhirEncounterParticipant {
                individual: Some(FhirReference { reference: format!("Practitioner/{}", request.practitioner_did), display: None }),
                ..Default::default()
            }],
            period: request.period,
            reason_code: request.reason_code,
        };
        let encounter = Encounter {
            id: None,
            patient_did: request.patient_did.clone(),
            practitioner_did: request.practitioner_did.clone(),
            fhir_encounter,
            status: EncounterStatus::Active,
            final_bundle_ipfs_hash: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let encounter_id = self.db.create_encounter(&encounter).await?;
        self.audit_log_service.log(&request.patient_did, &format!("create_encounter: {}", encounter_id), None).await;
        let mut created_encounter = encounter;
        created_encounter.id = Some(encounter_id);
        Ok(created_encounter)
    }

    pub async fn finalize_encounter(&self, encounter_id: &str) -> anyhow::Result<String> {
        let encounter_oid = bson::oid::ObjectId::parse_str(encounter_id)?;
        let encounter = self.db.get_encounter(encounter_oid).await?.ok_or_else(|| anyhow!("Encounter not found"))?;
        if let EncounterStatus::Finalized = encounter.status {
            return Err(anyhow!("Encounter already finalized"));
        }
        let patient = self.db.get_patient_by_did(&encounter.patient_did, &self.config.ipfs_encryption_key).await?.ok_or_else(|| anyhow!("Patient not found"))?;
        self.audit_log_service.log(&encounter.patient_did, &format!("finalize_encounter: {}", encounter_id), None).await;
        let observations = self.db.get_observations_for_encounter(encounter_id).await?;
        let conditions = self.db.get_conditions_for_encounter(encounter_id).await?;
        let medication_requests = self.db.get_medication_requests_for_encounter(encounter_id).await?;
        let mut resources: Vec<serde_json::Value> = vec![json!(encounter.fhir_encounter)];
        resources.extend(observations.into_iter().map(|r| json!(r)));
        resources.extend(conditions.into_iter().map(|r| json!(r)));
        resources.extend(medication_requests.into_iter().map(|r| json!(r)));
        let mut bundle = FhirManager::create_patient_bundle(&patient, resources)?;
        bundle.bundle[ "signature" ] = json!({
            "type": [{"system": "urn:iso-astm:E1762-95:2013", "code": "1.2.840.10065.1.12.1.1", "display": "Author's Signature"}],
            "when": Utc::now().to_rfc3339(),
            "who": {"reference": format!("Practitioner/{}", encounter.practitioner_did)},
            "data": "(placeholder_signature_data)",
            "sigFormat": "application/jose+json"
        });
        let bundle_json_string = serde_json::to_string(&bundle.bundle)?;
        let encrypted_bundle = utils::encrypt(bundle_json_string.as_bytes(), &self.config.ipfs_encryption_key)?;

        let ipfs_hash = self.ipfs_client.add_file(encrypted_bundle.as_bytes(), None).await?;
        self.db.finalize_encounter(encounter_oid, &ipfs_hash).await?;
        Ok(ipfs_hash)
    }
}

use anyhow::{anyhow, Result};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use serde_json::json;

use crate::audit_log::AuditLogService;
use crate::auth::AuthClaims;
use crate::config::Config;
use crate::database::Database;
use crate::did::DidManager;
use crate::fhir::FhirManager;
use crate::handlers::{CreateEncounterRequest, IssueCredentialRequest, RegisterRequest};
use crate::hedera::{HealthcareHederaService, HederaClient};
use crate::ipfs::IpfsClient;
use crate::models::*;
use crate::utils;

// --- AuthService ---
pub struct AuthService {
    db: Arc<Database>,
    hedera_client: Arc<HederaClient>,
    config: Arc<Config>,
    audit_log_service: Arc<AuditLogService>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegistrationResponse {
    pub user: Patient,
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InitiateAuthResponse {
    pub user_exists: bool,
}

impl AuthService {
    pub fn new(db: Arc<Database>, hedera_client: Arc<HederaClient>, config: Arc<Config>, audit_log_service: Arc<AuditLogService>) -> Self {
        Self { db, hedera_client, config, audit_log_service }
    }

    pub async fn initiate_auth(&self, email: &str) -> anyhow::Result<InitiateAuthResponse> {
        let patient = self.db.get_patient_by_email(email, &self.config.ipfs_encryption_key).await?;
        Ok(InitiateAuthResponse {
            user_exists: patient.is_some(),
        })
    }

    pub async fn register_new_user(&self, request: RegisterRequest) -> anyhow::Result<RegistrationResponse> {
        let did = DidManager::create_did(&self.hedera_client, &request.public_key_hex, &self.config.hedera_network).await?;
        let fhir_patient = FhirPatient {
            resource_type: "Patient".to_string(),
            id: Uuid::new_v4().to_string(),
            name: vec![FhirHumanName {
                r#use: Some("official".to_string()),
                family: Some(request.name.clone()),
                given: vec![request.name.clone()],
                ..Default::default()
            }],
            telecom: vec![FhirContactPoint {
                system: "email".to_string(),
                value: request.email.clone(),
                r#use: Some("home".to_string()),
            }],
            ..Default::default()
        };
        let patient = Patient {
            id: None,
            did: did.clone(),
            fhir_patient,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        self.db.create_patient(&patient, &self.config.ipfs_encryption_key).await?;
        self.audit_log_service.log(&did, "register_new_user");
        let expiration = Utc::now()
            .checked_add_signed(Duration::seconds(self.config.jwt_expiration_seconds))
            .expect("valid timestamp")
            .timestamp();
        let claims = AuthClaims {
            sub: did.clone(),
            exp: expiration as usize,
        };
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.config.jwt_secret.as_ref()),
        )?;
        Ok(RegistrationResponse { user: patient, token })
    }
}

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
        self.audit_log_service.log(did, "get_patient");
        self.db.get_patient_by_did(did, &self.config.ipfs_encryption_key).await
    }
}

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
        self.audit_log_service.log(&request.patient_did, &format!("create_encounter: {}", encounter_id));
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
        self.audit_log_service.log(&encounter.patient_did, &format!("finalize_encounter: {}", encounter_id));
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

// --- VerifiableCredentialService ---
pub struct VerifiableCredentialService {
    db: Arc<Database>,
    ipfs_client: Arc<IpfsClient>,
    hedera_service: Arc<HealthcareHederaService>,
    audit_log_service: Arc<AuditLogService>,
}

impl VerifiableCredentialService {
    pub fn new(db: Arc<Database>, ipfs_client: Arc<IpfsClient>, hedera_service: Arc<HealthcareHederaService>, audit_log_service: Arc<AuditLogService>) -> Self {
        Self { db, ipfs_client, hedera_service, audit_log_service }
    }

    pub async fn issue_credential(&self, request: IssueCredentialRequest) -> anyhow::Result<String> {
        self.audit_log_service.log(&request.subject_did, &format!("issue_credential: {}", request.credential_type));
        // ... implementation
        Ok("".to_string())
    }
}


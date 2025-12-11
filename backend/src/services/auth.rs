
use anyhow::anyhow;
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use rand::{Rng, RngCore};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use serde_json::json;

use crate::auditing::{AuditLogService, AuditingService};
use crate::api::middleware::auth::AuthClaims;
use crate::config::Config;
use crate::database::Database;
use crate::did::DidManager;
use crate::fhir::FhirManager;
use crate::api::handlers::{CreateEncounterRequest, IssueCredentialRequest, RegisterRequest, GoogleAuthRequest, PhoneAuthInitiateRequest, PhoneAuthVerifyRequest};
use crate::hedera::{HealthcareHederaService, HederaClient};
use crate::ipfs::IpfsClient;
use crate::models::*;
use crate::twilio::TwilioService;
use crate::utils;

#[cfg(not(feature = "test"))]
use google_jwt_signin::Client;
#[cfg(feature = "test")]
use mockall::automock;

// --- AuthService ---
#[cfg_attr(feature = "test", automock)]
pub trait AuthService: Send + Sync {
    fn new(db: Arc<Database>, hedera_client: Arc<HederaClient>, config: Arc<Config>, audit_log_service: Arc<AuditLogService>, twilio_service: Arc<TwilioService>) -> Self where Self: Sized;
    async fn initiate_auth(&self, email: &str) -> anyhow::Result<InitiateAuthResponse>;
    async fn register_new_user(&self, request: RegisterRequest) -> anyhow::Result<RegistrationResponse>;
    async fn authenticate_with_google(&self, request: GoogleAuthRequest) -> anyhow::Result<RegistrationResponse>;
    async fn verify_google_token(&self, token: &str) -> anyhow::Result<google_jwt_signin::Claims>;
    async fn initiate_phone_auth(&self, request: PhoneAuthInitiateRequest) -> anyhow::Result<()>;
    async fn verify_phone_auth(&self, request: PhoneAuthVerifyRequest) -> anyhow::Result<RegistrationResponse>;
}

pub struct AuthServiceImpl {
    db: Arc<Database>,
    hedera_client: Arc<HederaClient>,
    config: Arc<Config>,
    audit_log_service: Arc<AuditLogService>,
    twilio_service: Arc<TwilioService>,
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

impl AuthService for AuthServiceImpl {
    fn new(db: Arc<Database>, hedera_client: Arc<HederaClient>, config: Arc<Config>, audit_log_service: Arc<AuditLogService>, twilio_service: Arc<TwilioService>) -> Self {
        Self { db, hedera_client, config, audit_log_service, twilio_service }
    }

    async fn initiate_auth(&self, email: &str) -> anyhow::Result<InitiateAuthResponse> {
        let patient = self.db.get_patient_by_email(email, &self.config.ipfs_encryption_key).await?;
        Ok(InitiateAuthResponse {
            user_exists: patient.is_some(),
        })
    }

    async fn register_new_user(&self, request: RegisterRequest) -> anyhow::Result<RegistrationResponse> {
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
        self.audit_log_service.log(&did, "register_new_user", None).await;
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

    #[cfg(not(feature = "test"))]
    async fn authenticate_with_google(&self, request: GoogleAuthRequest) -> anyhow::Result<RegistrationResponse> {

        let client = Client::new(&self.config.google_client_id);
        let id_token = client.verify_id_token(&request.id_token)?;
        let email = id_token.payload.email.as_ref().unwrap();

        if let Some(patient) = self.db.get_patient_by_email(email, &self.config.ipfs_encryption_key).await? {

        let claims = self.verify_google_token(&request.id_token).await?;
        let email = claims.email.as_ref().unwrap();

        if let Some(patient) = self.db.get_patient_by_email(email, &self.config.ipfs_encryption_key).await? {
            tracing::debug!("Existing user {} authenticated with Google. DID: {}", email, patient.did);
            let expiration = Utc::now()
                .checked_add_signed(Duration::seconds(self.config.jwt_expiration_seconds))
                .expect("valid timestamp")
                .timestamp();
            let claims = AuthClaims {
                sub: patient.did.clone(),
                exp: expiration as usize,
            };
            let token = encode(
                &Header::default(),
                &claims,
                &EncodingKey::from_secret(self.config.jwt_secret.as_ref()),
            )?;
            Ok(RegistrationResponse { user: patient, token })
        } else {
            // Create a new user
            let mut public_key_bytes = [0u8; 32];
            rand::thread_rng().fill_bytes(&mut public_key_bytes);
            let public_key_hex = hex::encode(public_key_bytes);
            let did = DidManager::create_did(&self.hedera_client, &public_key_hex, &self.config.hedera_network).await?;
            let fhir_patient = FhirPatient {
                resource_type: "Patient".to_string(),
                id: Uuid::new_v4().to_string(),
                name: vec![FhirHumanName {
                    r#use: Some("official".to_string()),

                    family: id_token.payload.name.clone(),
                    given: vec![id_token.payload.name.clone().unwrap_or_default()],

                    family: claims.family_name.clone(),
                    given: vec![claims.given_name.clone().unwrap_or_default()],
                    ..Default::default()
                }],
                telecom: vec![FhirContactPoint {
                    system: "email".to_string(),
                    value: email.to_string(),
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
            self.audit_log_service.log(&did, "register_new_user_google", None).await;


            tracing::debug!("New user {} registered with Google. DID: {}", email, did);

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

    async fn initiate_phone_auth(&self, request: PhoneAuthInitiateRequest) -> anyhow::Result<()> {
        let otp = format!("{:06}", rand::thread_rng().gen_range(0..1_000_000));
        let otp_record = Otp {
            id: None,
            phone_number: request.phone_number.clone(),
            otp: otp.clone(),
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::minutes(5),
        };
        self.db.create_otp(&otp_record).await?;
        self.twilio_service.send_otp(&request.phone_number, &otp)?;
        Ok(())
    }

    async fn verify_phone_auth(&self, request: PhoneAuthVerifyRequest) -> anyhow::Result<RegistrationResponse> {
        let otp_record = self.db.get_otp(&request.phone_number, &request.otp).await?;

        if let Some(otp_record) = otp_record {
            if otp_record.expires_at < Utc::now() {
                return Err(anyhow!("OTP has expired"));
            }

            // For simplicity, we'll use the phone number to find the user.
            // In a real application, you might want to have a separate way to link phone numbers to users.
            let patient = self.db.get_patient_by_phone(&request.phone_number, &self.config.ipfs_encryption_key).await?;

            if let Some(patient) = patient {
                let expiration = Utc::now()
                    .checked_add_signed(Duration::seconds(self.config.jwt_expiration_seconds))
                    .expect("valid timestamp")
                    .timestamp();
                let claims = AuthClaims {
                    sub: patient.did.clone(),
                    exp: expiration as usize,
                };
                let token = encode(
                    &Header::default(),
                    &claims,
                    &EncodingKey::from_secret(self.config.jwt_secret.as_ref()),
                )?;
                Ok(RegistrationResponse { user: patient, token })
            } else {
                // Create a new user
                let mut public_key_bytes = [0u8; 32];
                rand::thread_rng().fill_bytes(&mut public_key_bytes);
                let public_key_hex = hex::encode(public_key_bytes);
                let did = DidManager::create_did(&self.hedera_client, &public_key_hex, &self.config.hedera_network).await?;
                let fhir_patient = FhirPatient {
                    resource_type: "Patient".to_string(),
                    id: Uuid::new_v4().to_string(),
                    telecom: vec![FhirContactPoint {
                        system: "phone".to_string(),
                        value: request.phone_number.clone(),
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
                self.audit_log_service.log(&did, "register_new_user_phone", None).await;
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
        } else {
            Err(anyhow!("Invalid OTP"))
        }
    }

    async fn verify_google_token(&self, token: &str) -> anyhow::Result<google_jwt_signin::Claims> {
        let client = Client::new(&self.config.google_client_id);
        let id_token = client.verify_id_token(token)?;
        Ok(id_token.payload)
    }
}
}
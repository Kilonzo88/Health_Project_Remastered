use anyhow::{anyhow, Context, Result};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use rand::{Rng, RngCore};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use tracing;
use hex;

use crate::auditing::AuditLogService;
use crate::api::middleware::jwt_auth::AuthClaims;
use crate::config::Config;
use crate::database::Database;
use crate::services::did::DidManager;
use crate::api::handlers::{RegisterRequest, GoogleAuthRequest, PhoneAuthInitiateRequest, PhoneAuthVerifyRequest};
use crate::services::hedera::HederaClient;
use crate::models::*;
use crate::services::email::EmailService;
use crate::services::twilio::TwilioService;

#[cfg(not(feature = "test"))]
use google_jwt_signin::Client;
#[cfg(feature = "test")]
use mockall::automock;

// --- AuthService ---
#[cfg_attr(feature = "test", automock)]
pub trait AuthService: Send + Sync {
    fn new(
        db: Arc<Database>,
        hedera_client: Arc<HederaClient>,
        config: Arc<Config>,
        audit_log_service: Arc<AuditLogService>,
        twilio_service: Arc<TwilioService>,
        email_service: Arc<EmailService>,
    ) -> Self
    where
        Self: Sized;
    async fn initiate_auth(&self, email: &str) -> anyhow::Result<InitiateAuthResponse>;
    async fn register_new_user(&self, request: RegisterRequest) -> anyhow::Result<RegistrationResponse>;
    async fn authenticate_with_google(&self, request: GoogleAuthRequest) -> Result<RegistrationResponse>;
    async fn verify_google_token(&self, id_token: &str) -> Result<String>;
    async fn get_patient_by_did(&self, did: &str) -> Result<Patient>;
    async fn initiate_phone_auth(&self, request: PhoneAuthInitiateRequest) -> anyhow::Result<()>;
    async fn verify_phone_auth(&self, request: PhoneAuthVerifyRequest) -> anyhow::Result<RegistrationResponse>;
}

pub struct AuthServiceImpl {
    db: Arc<Database>,
    hedera_client: Arc<HederaClient>,
    config: Arc<Config>,
    audit_log_service: Arc<AuditLogService>,
    twilio_service: Arc<TwilioService>,
    email_service: Arc<EmailService>,
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

#[derive(Debug)]
struct GoogleUserInfo {
    email: String,
    name: String,
    family_name: Option<String>,
    given_name: Option<String>,
}

impl AuthService for AuthServiceImpl {
    fn new(
        db: Arc<Database>,
        hedera_client: Arc<HederaClient>,
        config: Arc<Config>,
        audit_log_service: Arc<AuditLogService>,
        twilio_service: Arc<TwilioService>,
        email_service: Arc<EmailService>,
    ) -> Self {
        Self {
            db,
            hedera_client,
            config,
            audit_log_service,
            twilio_service,
            email_service,
        }
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

        // Generate verification token and expiration time
        let verification_token = Uuid::new_v4().to_string();
        let verification_token_expires = Utc::now() + Duration::hours(24);

        let patient = Patient {
            id: None,
            did: did.clone(),
            fhir_patient,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            email_verified: false,
            verification_token: Some(verification_token.clone()),
            verification_token_expires: Some(verification_token_expires),
        };

        self.db.create_patient(&patient, &self.config.ipfs_encryption_key).await?;
        self.audit_log_service.log(&did, "register_new_user", None).await;

        // --- Send verification and welcome emails (fire and forget) ---
        self.email_service
            .send_verification_email(&request.email, &request.name, &verification_token);
        self.email_service
            .send_welcome_email(&request.email, &request.name);

        let token = self.generate_jwt_for_patient(&patient)?;

        Ok(RegistrationResponse { user: patient, token })
    }

    /// Main entry point for Google authentication
    /// 
    /// Flow: Google ID Token → Verify → Find/Create Patient → Generate JWT
    async fn authenticate_with_google(
        &self,
        request: GoogleAuthRequest,
    ) -> Result<RegistrationResponse> {
        // Step 1: Verify Google token and extract user info
        let user_info = self
            .verify_google_token_internal(&request.id_token)
            .await
            .context("Failed to verify Google token")?;

        // Step 2: Find existing patient or create new one
        let patient = self
            .find_or_create_patient(&user_info)
            .await
            .context("Failed to find or create patient")?;

        // Step 3: Generate JWT token with patient's DID
        let token = self
            .generate_jwt_for_patient(&patient)
            .context("Failed to generate JWT")?;

        Ok(RegistrationResponse { user: patient, token })
    }

    async fn verify_google_token(&self, id_token: &str) -> Result<String> {
        let user_info = self
            .verify_google_token_internal(id_token)
            .await
            .context("Failed to verify Google token")?;
        Ok(user_info.email)
    }

    /// Get patient by their DID (used by middleware to load user from JWT)
    async fn get_patient_by_did(&self, did: &str) -> Result<Patient> {
        self.db
            .get_patient_by_did(did, &self.config.ipfs_encryption_key)
            .await?
            .ok_or_else(|| anyhow!("Patient not found for DID: {}", did))
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
                    email_verified: true,
                    verification_token: None,
                    verification_token_expires: None,
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

}

impl AuthServiceImpl {
    
    // --- Private Helper Methods ---

    /// Verify Google ID token and extract user information
    #[cfg(not(feature = "test"))]
    async fn verify_google_token_internal(&self, id_token: &str) -> Result<GoogleUserInfo> {
        let client = Client::new(&self.config.google_client_id);
        let verified_token = client
            .verify_id_token(id_token)
            .map_err(|e| anyhow!("Invalid Google token: {}", e))?;

        let payload = verified_token.payload;
        let email = payload
            .email
            .ok_or_else(|| anyhow!("Google token missing email"))?;

        Ok(GoogleUserInfo {
            email,
            name: payload.name.unwrap_or_default(),
            given_name: payload.given_name,
            family_name: payload.family_name,
        })
    }

    #[cfg(feature = "test")]
    async fn verify_google_token_internal(&self, _id_token: &str) -> Result<GoogleUserInfo> {
        Ok(GoogleUserInfo {
            email: "test@example.com".to_string(),
            name: "Test User".to_string(),
            given_name: Some("Test".to_string()),
            family_name: Some("User".to_string()),
        })
    }

    /// Find existing patient by email or create new one
    async fn find_or_create_patient(&self, user_info: &GoogleUserInfo) -> Result<Patient> {
        match self
            .db
            .get_patient_by_email(&user_info.email, &self.config.ipfs_encryption_key)
            .await?
        {
            Some(patient) => {
                tracing::info!(
                    email = %user_info.email,
                    did = %patient.did,
                    "Existing user authenticated with Google"
                );
                Ok(patient)
            }
            None => {
                tracing::info!(email = %user_info.email, "Creating new user via Google auth");
                self.create_new_patient(user_info).await
            }
        }
    }

    /// Create a new patient with Hedera DID
    async fn create_new_patient(&self, user_info: &GoogleUserInfo) -> Result<Patient> {
        // Generate random public key for DID creation
        let public_key_hex = generate_random_public_key();

        // Create DID on Hedera network
        let did = DidManager::create_did(
            &self.hedera_client,
            &public_key_hex,
            &self.config.hedera_network,
        )
        .await
        .context("Failed to create Hedera DID")?;

        tracing::debug!(did = %did, "Created Hedera DID for new user");

        // Build FHIR-compliant patient record
        let fhir_patient = build_fhir_patient(user_info);

        // Create patient object (Google-sign-in users are considered email-verified)
        let patient = Patient {
            id: None,
            did: did.clone(),
            fhir_patient,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            email_verified: true,
            verification_token: None,
            verification_token_expires: None,
        };

        // Persist to database
        self.db
            .create_patient(&patient, &self.config.ipfs_encryption_key)
            .await
            .context("Failed to save patient to database")?;

        // Audit log
        self.audit_log_service
            .log(&did, "google_auth_new_user", None)
            .await;

        Ok(patient)
    }

    /// Generate JWT token with patient's DID as subject
    fn generate_jwt_for_patient(&self, patient: &Patient) -> Result<String> {
        let expiration = Utc::now()
            .checked_add_signed(Duration::seconds(self.config.jwt_expiration_seconds))
            .ok_or_else(|| anyhow!("Invalid expiration time"))?
            .timestamp();

        let claims = AuthClaims {
            sub: patient.did.clone(), // DID goes in the JWT subject
            exp: expiration as usize,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.config.jwt_secret.as_ref()),
        )
        .map_err(Into::into)
    }
}

// --- Utility Functions ---

/// Generate a random 32-byte public key for DID creation
fn generate_random_public_key() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

/// Build FHIR-compliant patient resource from Google user info
fn build_fhir_patient(user_info: &GoogleUserInfo) -> FhirPatient {
    FhirPatient {
        resource_type: "Patient".to_string(),
        id: Uuid::new_v4().to_string(),
        name: vec![FhirHumanName {
            r#use: Some("official".to_string()),
            family: user_info.family_name.clone(),
            given: vec![user_info
                .given_name
                .clone()
                .unwrap_or_else(|| user_info.name.clone())],
            ..Default::default()
        }],
        telecom: vec![FhirContactPoint {
            system: "email".to_string(),
            value: user_info.email.clone(),
            r#use: Some("home".to_string()),
        }],
        ..Default::default()
    }
}
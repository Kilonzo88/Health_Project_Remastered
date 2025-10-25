use axum::{
    extract::{Path, State, Extension},
    http::StatusCode,
    response::Json,
};
use anyhow::Result;
use serde::Deserialize;

use crate::models::*;
use crate::services::*;
use crate::state::AppState;
use crate::auth::AuthContext;
use std::sync::Arc;


// --- Auth Handlers ---
#[derive(Debug, Clone, Deserialize)]
pub struct InitiateAuthRequest {
    pub email: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RegisterRequest {
    pub name: String,
    pub email: String,
    pub public_key_hex: String,
}

#[axum::debug_handler]
pub async fn auth_initiate(
    State(state): State<Arc<AppState>>,
    Json(request): Json<InitiateAuthRequest>,
) -> Result<Json<ApiResponse<InitiateAuthResponse>>, StatusCode> {
    let auth_service = AuthService::new(state.database.clone(), state.hedera_client.clone(), state.config.clone(), state.audit_log_service.clone());

    match auth_service.initiate_auth(&request.email).await {
        Ok(response) => Ok(Json(ApiResponse::success(response))),
        Err(e) => {
            tracing::error!("Failed to initiate auth: {}", e);
            Ok(Json(ApiResponse::error(e.to_string())))
        }
    }
}

#[axum::debug_handler]
pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(request): Json<RegisterRequest>,
) -> Result<Json<ApiResponse<RegistrationResponse>>, StatusCode> {
    let auth_service = AuthService::new(state.database.clone(), state.hedera_client.clone(), state.config.clone(), state.audit_log_service.clone());

    match auth_service.register_new_user(request).await {
        Ok(response) => Ok(Json(ApiResponse::success(response))),
        Err(e) => {
            tracing::error!("Failed to register user: {}", e);
            Ok(Json(ApiResponse::error(e.to_string())))
        }
    }
}


#[axum::debug_handler]
pub async fn step_up_auth() -> Result<Json<ApiResponse<String>>, StatusCode> {
    // In a real implementation, this would involve re-authenticating the user
    // and creating a high-assurance session.
    Ok(Json(ApiResponse::success("Step-up authentication successful".to_string())))
}


// --- Patient Handlers ---
#[axum::debug_handler]
pub async fn get_patient(
    State(state): State<Arc<AppState>>,
    Path(patient_did): Path<String>,
) -> Result<Json<ApiResponse<Option<Patient>>>, StatusCode> {
    let patient_service = PatientService::new(state.database.clone(), state.config.clone(), state.audit_log_service.clone());
    
    match patient_service.get_patient(&patient_did).await {
        Ok(patient) => Ok(Json(ApiResponse::success(patient))),
        Err(e) => {
            tracing::error!("Failed to get patient: {}", e);
            Ok(Json(ApiResponse::error(e.to_string())))
        }
    }
}


// --- Encounter Handlers ---
#[derive(Debug, Clone, Deserialize)]
pub struct CreateEncounterRequest {
    pub patient_did: String,
    pub practitioner_did: String,
    pub class: FhirCoding,
    pub reason_code: Vec<FhirCodeableConcept>,
    pub period: FhirPeriod,
}

#[axum::debug_handler]
pub async fn create_encounter(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateEncounterRequest>,
) -> Result<Json<ApiResponse<Encounter>>, StatusCode> {
    let encounter_service = EncounterService::new(state.database.clone(), state.ipfs_client.clone(), state.config.clone(), state.audit_log_service.clone());

    match encounter_service.create_encounter(request).await {
        Ok(encounter) => Ok(Json(ApiResponse::success(encounter))),
        Err(e) => {
            tracing::error!("Failed to create encounter: {}", e);
            Ok(Json(ApiResponse::error(e.to_string())))
        }
    }
}

#[axum::debug_handler]
pub async fn finalize_encounter(
    State(state): State<Arc<AppState>>,
    Path(encounter_id): Path<String>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    let encounter_service = EncounterService::new(state.database.clone(), state.ipfs_client.clone(), state.config.clone(), state.audit_log_service.clone());

    match encounter_service.finalize_encounter(&encounter_id).await {
        Ok(ipfs_hash) => Ok(Json(ApiResponse::success(ipfs_hash))),
        Err(e) => {
            tracing::error!("Failed to finalize encounter: {}", e);
            Ok(Json(ApiResponse::error(e.to_string())))
        }
    }
}


// --- Verifiable Credential Handlers ---
#[derive(Debug, Clone, Deserialize)]
pub struct IssueCredentialRequest {
    pub subject_did: String,
    pub credential_type: String,
    pub issuer: String,
    pub expires_at: Option<u64>,
    pub metadata: String,
}

#[axum::debug_handler]
pub async fn issue_credential(
    State(state): State<Arc<AppState>>,
    Json(request): Json<IssueCredentialRequest>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    let vc_service = VerifiableCredentialService::new(
        state.database.clone(),
        state.ipfs_client.clone(),
        state.hedera_service.clone(),
        state.audit_log_service.clone(),
    );

    match vc_service.issue_credential(request).await {
        Ok(transaction_id) => Ok(Json(ApiResponse::success(transaction_id))),
        Err(e) => {
            tracing::error!("Failed to issue credential: {}", e);
            Ok(Json(ApiResponse::error(e.to_string())))
        }
    }
}

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;

use crate::models::*;
use crate::services::*;
use crate::services::auth::EmailVerificationResponse;
use crate::state::AppState;
use std::sync::Arc;
use crate::services::ask_gemini;


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

#[derive(Debug, Clone, Deserialize)]
pub struct GoogleAuthRequest {
    pub id_token: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PhoneAuthInitiateRequest {
    pub phone_number: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PhoneAuthVerifyRequest {
    pub phone_number: String,
    pub otp: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChatRequest {
    pub prompt: String,
}

#[axum::debug_handler]
pub async fn auth_initiate(
    State(state): State<Arc<AppState<AuthServiceImpl>>>,
    Json(request): Json<InitiateAuthRequest>,
) -> Result<Json<ApiResponse<InitiateAuthResponse>>, StatusCode> {
    match state.auth_service.initiate_auth(&request.email).await {
        Ok(response) => Ok(Json(ApiResponse::success(response))),
        Err(e) => {
            tracing::error!("Failed to initiate auth: {}", e);
            Ok(Json(ApiResponse::error(e.to_string())))
        }
    }
}

#[axum::debug_handler]
pub async fn register(
    State(state): State<Arc<AppState<AuthServiceImpl>>>,
    Json(request): Json<RegisterRequest>,
) -> Result<Json<ApiResponse<RegistrationResponse>>, StatusCode> {
    match state.auth_service.register_new_user(request).await {
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

#[axum::debug_handler]
pub async fn auth_google(
    State(state): State<Arc<AppState<AuthServiceImpl>>>,
    Json(request): Json<GoogleAuthRequest>,
) -> Result<Json<ApiResponse<RegistrationResponse>>, StatusCode> {
    match state.auth_service.authenticate_with_google(request).await {
        Ok(response) => Ok(Json(ApiResponse::success(response))),
        Err(e) => {
            tracing::error!("Failed to authenticate with Google: {}", e);
            Ok(Json(ApiResponse::error(e.to_string())))
        }
    }
}

#[axum::debug_handler]
pub async fn auth_phone_initiate(
    State(state): State<Arc<AppState<AuthServiceImpl>>>,
    Json(request): Json<PhoneAuthInitiateRequest>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    match state.auth_service.initiate_phone_auth(request).await {
        Ok(_) => Ok(Json(ApiResponse::success("OTP sent successfully".to_string()))),
        Err(e) => {
            tracing::error!("Failed to initiate phone auth: {}", e);
            Ok(Json(ApiResponse::error(e.to_string())))
        }
    }
}

#[axum::debug_handler]
pub async fn auth_phone_verify(
    State(state): State<Arc<AppState<AuthServiceImpl>>>,
    Json(request): Json<PhoneAuthVerifyRequest>,
) -> Result<Json<ApiResponse<RegistrationResponse>>, StatusCode> {
    match state.auth_service.verify_phone_auth(request).await {
        Ok(response) => Ok(Json(ApiResponse::success(response))),
        Err(e) => {
            tracing::error!("Failed to verify phone auth: {}", e);
            Ok(Json(ApiResponse::error(e.to_string())))
        }
    }
}

#[axum::debug_handler]
pub async fn verify_email(
    State(state): State<Arc<AppState<AuthServiceImpl>>>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<ApiResponse<EmailVerificationResponse>>, StatusCode> {
    let token = params.get("token")
        .ok_or_else(|| {
            tracing::error!("Missing verification token in query parameters");
            StatusCode::BAD_REQUEST
        })?;

    match state.auth_service.verify_email(token).await {
        Ok(response) => Ok(Json(ApiResponse::success(response))),
        Err(e) => {
            tracing::error!("Failed to verify email: {}", e);
            Ok(Json(ApiResponse::error(e.to_string())))
        }
    }
}

#[axum::debug_handler]
pub async fn chat(
    State(state): State<Arc<AppState<AuthServiceImpl>>>,
    Json(request): Json<ChatRequest>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    match ask_gemini(&request.prompt, &state.config).await {
        Ok(response) => Ok(Json(ApiResponse::success(response))),
        Err(e) => {
            tracing::error!("Failed to ask Gemini: {}", e);
            Ok(Json(ApiResponse::error(e.to_string())))
        }
    }
}


// --- Patient Handlers ---
#[axum::debug_handler]
pub async fn get_patient(
    State(state): State<Arc<AppState<AuthServiceImpl>>>,
    Path(patient_did): Path<String>,
) -> Result<Json<ApiResponse<Option<Patient>>>, StatusCode> {
    match state.patient_service.get_patient(&patient_did).await {
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
    State(state): State<Arc<AppState<AuthServiceImpl>>>,
    Json(request): Json<CreateEncounterRequest>,
) -> Result<Json<ApiResponse<Encounter>>, StatusCode> {
    match state.encounter_service.create_encounter(request).await {
        Ok(encounter) => Ok(Json(ApiResponse::success(encounter))),
        Err(e) => {
            tracing::error!("Failed to create encounter: {}", e);
            Ok(Json(ApiResponse::error(e.to_string())))
        }
    }
}

#[axum::debug_handler]
pub async fn finalize_encounter(
    State(state): State<Arc<AppState<AuthServiceImpl>>>,
    Path(encounter_id): Path<String>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    match state.encounter_service.finalize_encounter(&encounter_id).await {
        Ok(ipfs_hash) => Ok(Json(ApiResponse::success(ipfs_hash))),
        Err(e) => {
            tracing::error!("Failed to finalize encounter: {}", e);
            Ok(Json(ApiResponse::error(e.to_string())))
        }
    }
}


// --- Verifiable Credential Handlers ---

#[derive(Debug, Deserialize)]
pub struct GoogleToken {
    token: String,
}

#[axum::debug_handler]
pub async fn verify_google_token(
    State(state): State<Arc<AppState<AuthServiceImpl>>>,
    Json(token): Json<GoogleToken>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    match state.auth_service.verify_google_token(&token.token).await {
        Ok(email) => Ok(Json(ApiResponse::success(email))),
        Err(e) => {
            tracing::error!("Failed to verify Google token: {}", e);
            Ok(Json(ApiResponse::error("Invalid Google token".to_string())))
        }
    }
}

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
    State(state): State<Arc<AppState<AuthServiceImpl>>>,
    Json(request): Json<IssueCredentialRequest>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    match state.vc_service.issue_credential(request).await {
        Ok(transaction_id) => Ok(Json(ApiResponse::success(transaction_id))),
        Err(e) => {
            tracing::error!("Failed to issue credential: {}", e);
            Ok(Json(ApiResponse::error(e.to_string())))
        }
    }
}

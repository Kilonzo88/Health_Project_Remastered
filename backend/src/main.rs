use axum::{http::StatusCode, response::Json, routing::{get, post}, Router, middleware};
use std::sync::Arc;
use std::str::FromStr;
use tokio::time::{self, Duration};
use tower_http::cors::CorsLayer;
use tracing;
use shuttle_runtime::SecretStore;

use crate::hedera::ContractId;

mod auth_plus;
mod auth;
mod config;
mod database;
mod did;
mod fhir;
mod handlers;
mod hedera;
mod ipfs;
mod models;
mod services;
mod state;
mod utils;
mod audit_log;
mod auditing;
mod twilio;

use crate::audit_log::AuditLogService;
use crate::auditing::AuditingService;
use crate::auth::auth_middleware;
use crate::auth_plus::high_assurance_auth_middleware;
use config::Config;
use database::Database;
use handlers::*;
use ipfs::IpfsClient;
use hedera::{HederaClient, HealthcareHederaService};
use state::AppState;
use services::{AuthService, AuthServiceImpl};
use twilio::TwilioService;

#[shuttle_runtime::main]
async fn main(
    #[shuttle_runtime::Secrets] secrets: SecretStore,
) -> shuttle_axum::ShuttleAxum {

    // Load configuration from secrets
    let config = Arc::new(Config {
        database_url: secrets.get("DATABASE_URL").expect("DATABASE_URL must be set"),
        hedera_network: secrets.get("HEDERA_NETWORK").unwrap_or_else(|| "testnet".to_string()),
        hedera_account_id: secrets.get("HEDERA_ACCOUNT_ID").expect("HEDERA_ACCOUNT_ID must be set"),
        hedera_private_key: secrets.get("HEDERA_PRIVATE_KEY").expect("HEDERA_PRIVATE_KEY must be set"),
        ipfs_url: secrets.get("IPFS_URL").unwrap_or_else(|| "http://localhost:5001".to_string()),
        jwt_secret: secrets.get("JWT_SECRET").expect("JWT_SECRET must be set"),
        jwt_expiration_seconds: secrets.get("JWT_EXPIRATION_SECONDS").unwrap_or_else(|| "86400".to_string()).parse().expect("Invalid JWT_EXPIRATION_SECONDS"),
        ipfs_encryption_key: secrets.get("IPFS_ENCRYPTION_KEY").expect("IPFS_ENCRYPTION_KEY must be set"),
        server_port: secrets.get("SERVER_PORT").unwrap_or_else(|| "3000".to_string()).parse().expect("Invalid SERVER_PORT"),
        healthcare_access_control_contract_id: secrets.get("HEALTHCARE_ACCESS_CONTROL_CONTRACT_ID").expect("HEALTHCARE_ACCESS_CONTROL_CONTRACT_ID must be set"),
        verifiable_credentials_contract_id: secrets.get("VERIFIABLE_CREDENTIALS_CONTRACT_ID").expect("VERIFIABLE_CREDENTIALS_CONTRACT_ID must be set"),
        audit_trail_contract_id: secrets.get("AUDIT_TRAIL_CONTRACT_ID").expect("AUDIT_TRAIL_CONTRACT_ID must be set"),
        google_client_id: secrets.get("GOOGLE_CLIENT_ID").expect("GOOGLE_CLIENT_ID must be set"),
        twilio_account_sid: secrets.get("TWILIO_ACCOUNT_SID").expect("TWILIO_ACCOUNT_SID must be set"),
        twilio_auth_token: secrets.get("TWILIO_AUTH_TOKEN").expect("TWILIO_AUTH_TOKEN must be set"),
        twilio_phone_number: secrets.get("TWILIO_PHONE_NUMBER").expect("TWILIO_PHONE_NUMBER must be set"),
        gemini_api_key: secrets.get("GEMINI_API_KEY").expect("GEMINI_API_KEY must be set"),
    });

    // Initialize database
    let database = Arc::new(Database::new(&config.database_url).await.expect("Failed to connect to database"));

    // Initialize IPFS client
    let ipfs_client = Arc::new(IpfsClient::new(&config.ipfs_url));

    // Initialize Hedera client
    let hedera_client = Arc::new(HederaClient::new(&config.hedera_account_id, &config.hedera_private_key, &config.hedera_network).expect("Failed to create Hedera client"));
    let mut hedera_service = HealthcareHederaService::new((*hedera_client).clone());

    // --- Configure Contracts ---
    let access_control_contract_id = ContractId::from_str(
        &config.healthcare_access_control_contract_id
    ).expect("Failed to parse access control contract ID");
    let credentials_contract_id = ContractId::from_str(
        &config.verifiable_credentials_contract_id
    ).expect("Failed to parse credentials contract ID");
    let audit_trail_contract_id = ContractId::from_str(
        &config.audit_trail_contract_id
    ).expect("Failed to parse audit trail contract ID");

    hedera_service.set_contract_ids(
        access_control_contract_id,
        credentials_contract_id,
        audit_trail_contract_id,
    );

    let hedera_service = Arc::new(hedera_service);

    // Initialize services
    let audit_log_service = Arc::new(AuditLogService::new(database.clone()));
    let auditing_service = Arc::new(AuditingService::new(database.clone(), hedera_service.clone()));
    let twilio_service = Arc::new(TwilioService::new(&config));
    let auth_service = Arc::new(AuthServiceImpl::new(database.clone(), hedera_client.clone(), config.clone(), audit_log_service.clone(), twilio_service.clone()));
    
    let app_state = Arc::new(AppState {
        database: database.clone(),
        config: config.clone(),
        ipfs_client,
        hedera_client,
        hedera_service,
        audit_log_service,
        auditing_service: auditing_service.clone(),
        auth_service,
        twilio_service,
    });

    // --- Protected Routes ---
    let protected_routes = Router::new()
        .route("/api/patients/:id", get(get_patient))
        .route("/api/encounters", post(create_encounter))
        .route("/api/encounters/:id/finalize", post(finalize_encounter))
        .route_layer(middleware::from_fn_with_state(app_state.clone(), auth_middleware));

    // --- Protected High Assurance Routes ---
    let protected_high_assurance_routes = Router::new()
        .route("/api/credentials/issue", post(issue_credential))
        .route_layer(middleware::from_fn_with_state(app_state.clone(), high_assurance_auth_middleware));

    // --- Public Routes ---
    let public_routes = Router::new()
        .route("/health", get(health_check))
        .route("/api/auth/initiate", post(auth_initiate))
        .route("/api/auth/register", post(register))
        .route("/api/auth/step-up", post(step_up_auth))
        .route("/api/auth/google", post(auth_google))
        .route("/api/auth/phone/initiate", post(auth_phone_initiate))
        .route("/api/auth/phone/verify", post(auth_phone_verify))
        .route("/api/chat", post(chat));

    // --- Build Application ---
    let app = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .merge(protected_high_assurance_routes)
        .layer(CorsLayer::permissive())
        .with_state(app_state.clone());

    Ok(app.into())
}

async fn health_check() -> Result<Json<serde_json::Value>, StatusCode> {
    Ok(Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now()
    })))
}


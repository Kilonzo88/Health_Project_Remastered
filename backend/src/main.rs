use axum::{http::StatusCode, response::Json, routing::{get, post}, Router, middleware};
use std::sync::Arc;
use std::str::FromStr;
use tokio::net::TcpListener;
use tokio::time::{self, Duration};
use tower_http::cors::CorsLayer;
use tracing_subscriber;
use dotenv;
use crate::services::hedera::ContractId;

#[cfg(feature = "tls")]
use axum_server::{tls_rustls::RustlsConfig, bind_rustls};

mod api;
mod services;
mod models;
// mod auth;
mod utils;
mod auditing;
mod database;
mod config;
mod state;

use crate::auditing::{audit_log::AuditLogService, AuditingService};
// use crate::auth::auth_middleware;
// use crate::auth::high_assurance_auth_middleware;
use crate::config::Config;
use crate::database::Database;
use crate::api::handlers::*;
use crate::services::ipfs::IpfsClient;
use crate::services::hedera::{HederaClient, HealthcareHederaService};
use crate::state::AppState;
use crate::services::{AuthService, AuthServiceImpl, PatientService, EncounterService, VerifiableCredentialService};
use crate::services::twilio::TwilioService;
use crate::api::middleware::auth::{auth_middleware, high_assurance_auth_middleware};


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::from_path("../.env").ok();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("healthcare_backend=debug,tower_http=debug")
        .init();

    // Load configuration
    let config = Arc::new(Config::load()?);
    
    // Initialize database with retry logic
    let database = loop {
        match Database::new(&config.database_url).await {
            Ok(db) => {
                tracing::info!("Successfully connected to the database.");
                break Arc::new(db);
            }
            Err(e) => {
                tracing::error!("Failed to connect to database: {}. Retrying in 5 seconds...", e);
                time::sleep(Duration::from_secs(5)).await;
            }
        }
    };

    // Initialize IPFS client
    let ipfs_client = Arc::new(IpfsClient::new(&config.ipfs_url));

    // Initialize Hedera client
    let hedera_client = Arc::new(HederaClient::new(&config.hedera_account_id, &config.hedera_private_key, &config.hedera_network)?);
    let mut hedera_service = HealthcareHederaService::new((*hedera_client).clone());

    // --- Configure Contracts ---
    let access_control_contract_id = ContractId::from_str(
        &config.healthcare_access_control_contract_id
    )?;
    let credentials_contract_id = ContractId::from_str(
        &config.verifiable_credentials_contract_id
    )?;
    let audit_trail_contract_id = ContractId::from_str(
        &config.audit_trail_contract_id
    )?;

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
    let patient_service = Arc::new(PatientService::new(database.clone(), config.clone(), audit_log_service.clone()));
    let encounter_service = Arc::new(EncounterService::new(database.clone(), ipfs_client.clone(), config.clone(), audit_log_service.clone()));
    let vc_service = Arc::new(VerifiableCredentialService::new(database.clone(), ipfs_client.clone(), hedera_service.clone(), audit_log_service.clone()));
    
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
        patient_service,
        encounter_service,
        vc_service,
    });

    // --- Spawn Background Tasks ---
    let audit_handle = tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(3600)); // Anchor logs every hour
        loop {
            interval.tick().await;
            tracing::info!("Running periodic audit log anchoring...");
            if let Err(e) = auditing_service.anchor_audit_logs().await {
                tracing::error!("Failed to anchor audit logs: {}", e);
            }
        }
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
        .route("/api/auth/google/verify", post(verify_google_token))
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

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], app_state.config.server_port));

    if app_state.config.use_tls {
        #[cfg(feature = "tls")]
        {
            // Configure TLS
            let tls_config = axum_server::tls_rustls::RustlsConfig::from_pem_file(
                "cert.pem",
                "key.pem",
            )
            .await?;

            tracing::info!("Server running on https://{}", addr);
            
            axum_server::bind_rustls(addr, tls_config)
                .serve(app.into_make_service())
                .await?;
        }
        #[cfg(not(feature = "tls"))]
        {
            tracing::warn!("TLS is enabled in the configuration, but the `tls` feature is not compiled. Falling back to HTTP.");
            let listener = TcpListener::bind(addr).await?;
            axum::serve(listener, app.into_make_service()).await?;
        }
    } else {
        tracing::info!("Server running on http://{}", addr);
        let listener = TcpListener::bind(addr).await?;
        axum::serve(listener, app.into_make_service()).await?;
    }
    
    // Cleanly shut down background tasks
    audit_handle.abort();

    Ok(())
}

async fn health_check() -> Result<Json<serde_json::Value>, StatusCode> {
    Ok(Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now()
    })))
}


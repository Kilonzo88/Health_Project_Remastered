use axum::{http::StatusCode, response::Json, routing::{get, post}, Router, middleware};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing_subscriber;

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

use crate::audit_log::AuditLogService;
use crate::auth::auth_middleware;
use crate::auth_plus::high_assurance_auth_middleware;
use config::Config;
use database::Database;
use handlers::*;
use ipfs::IpfsClient;
use hedera::{HederaClient, HealthcareHederaService};
use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("healthcare_backend=debug,tower_http=debug")
        .init();

    // Load configuration
    let config = Config::load()?;
    
    // Initialize database
    let database = Database::new(&config.database_url).await?;

    // Initialize IPFS client
    let ipfs_client = Arc::new(IpfsClient::new(&config.ipfs_url));

    // Initialize Hedera client
    let hedera_client = Arc::new(HederaClient::new(&config.hedera_account_id, &config.hedera_private_key)?);
    let hedera_service = Arc::new(HealthcareHederaService::new((*hedera_client).clone()));

    // Initialize audit log service
    let audit_log_service = Arc::new(AuditLogService::new());
    
    // Initialize services
    let app_state = Arc::new(AppState {
        database: Arc::new(database),
        config: Arc::new(config),
        ipfs_client,
        hedera_client,
        hedera_service,
        audit_log_service,
    });

    // --- Protected Routes ---
    // These routes require a valid JWT
    let protected_routes = Router::new()
        .route("/api/patients/:id", get(get_patient))
        .route("/api/encounters", post(create_encounter))
        .route("/api/encounters/:id/finalize", post(finalize_encounter))
        .route_layer(middleware::from_fn_with_state(app_state.clone(), auth_middleware));

    // --- Protected High Assurance Routes ---
    // These routes require a high-assurance session
    let protected_high_assurance_routes = Router::new()
        .route("/api/credentials/issue", post(issue_credential))
        .route_layer(middleware::from_fn_with_state(app_state.clone(), high_assurance_auth_middleware));

    // --- Public Routes ---
    // These routes are accessible without authentication
    let public_routes = Router::new()
        .route("/health", get(health_check))
        .route("/api/auth/initiate", post(auth_initiate))
        .route("/api/auth/register", post(register))
        .route("/api/auth/step-up", post(step_up_auth));

use axum_server::bind_rustls;
use axum_server::tls_rustls::RustlsConfig;

    // --- Build Application ---
    let app = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .merge(protected_high_assurance_routes)
        .layer(CorsLayer::permissive())
        .with_state(app_state.clone());

    // Configure TLS
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], app_state.config.server_port));
    let tls_config = RustlsConfig::from_pem_file(
        "/home/dk/health_remastered/backend/cert.pem",
        "/home/dk/health_remastered/backend/key.pem",
    )
    .await?;

    tracing::info!("Server running on https://{}", addr);
    
    bind_rustls(addr, tls_config)
        .serve(app.into_make_service())
        .await?;
    
    Ok(())
}

async fn health_check() -> Result<Json<serde_json::Value>, StatusCode> {
    Ok(Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now()
    })))
}

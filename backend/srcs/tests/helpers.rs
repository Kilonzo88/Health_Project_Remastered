use super::*;
use std::sync::Arc;
use axum::Router;

use health_remastered::{
    config::Config,
    database::Database,
    state::AppState,
    services::{PatientService, EncounterService, VerifiableCredentialService, AuthService},
    handlers::{*},
    hedera::{HederaClient, HealthcareHederaService},
    ipfs::IpfsClient,
    audit_log::AuditLogService,
    auditing::AuditingService,
};

pub async fn create_app<T: AuthService + 'static>(auth_service: Arc<T>) -> Router {
    let config = Arc::new(Config::load().unwrap());
    let database = Arc::new(Database::new(&config.database_url).await.unwrap());
    let ipfs_client = Arc::new(IpfsClient::new(&config.ipfs_url));
    let hedera_client = Arc::new(HederaClient::new(&config.hedera_account_id, &config.hedera_private_key, &config.hedera_network).unwrap());
    let mut hedera_service = HealthcareHederaService::new(hedera_client.clone());
    let access_control_contract_id = hedera::ContractId::from_str(&config.healthcare_access_control_contract_id.as_deref().unwrap()).unwrap();
    let credentials_contract_id = hedera::ContractId::from_str(&config.verifiable_credentials_contract_id.as_deref().unwrap()).unwrap();
    let audit_trail_contract_id = hedera::ContractId::from_str(&config.audit_trail_contract_id.as_deref().unwrap()).unwrap();
    hedera_service.set_contract_ids(
        access_control_contract_id,
        credentials_contract_id,
        audit_trail_contract_id,
    );
    let hedera_service = Arc::new(hedera_service);
    let audit_log_service = Arc::new(AuditLogService::new(database.clone()));
    let auditing_service = Arc::new(AuditingService::new(database.clone(), hedera_service.clone()));

    let app_state = Arc::new(AppState {
        database: database.clone(),
        config: config.clone(),
        ipfs_client,
        hedera_client,
        hedera_service,
        audit_log_service,
        auditing_service: auditing_service.clone(),
        auth_service,
    });

    Router::new()
        .route("/api/auth/google", post(auth_google))
        .with_state(app_state)
}

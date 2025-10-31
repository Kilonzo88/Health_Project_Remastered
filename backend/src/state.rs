use crate::audit_log::AuditLogService;
use crate::auditing::AuditingService;
use crate::config::Config;
use crate::database::Database;
use crate::hedera::{HederaClient, HealthcareHederaService};
use crate::ipfs::IpfsClient;
use crate::services::AuthService;
use crate::twilio::TwilioService;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState<T: AuthService> {
    pub database: Arc<Database>,
    pub config: Arc<Config>,
    pub ipfs_client: Arc<IpfsClient>,
    pub hedera_client: Arc<HederaClient>,
    pub hedera_service: Arc<HealthcareHederaService>,
    pub audit_log_service: Arc<AuditLogService>,
    pub auditing_service: Arc<AuditingService>,
    pub auth_service: Arc<T>,
    pub twilio_service: Arc<TwilioService>,
}

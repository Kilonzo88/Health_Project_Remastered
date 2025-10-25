
use crate::audit_log::AuditLogService;
use crate::config::Config;
use crate::database::Database;
use crate::hedera::{HederaClient, HealthcareHederaService};
use crate::ipfs::IpfsClient;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub database: Arc<Database>,
    pub config: Arc<Config>,
    pub ipfs_client: Arc<IpfsClient>,
    pub hedera_client: Arc<HederaClient>,
    pub hedera_service: Arc<HealthcareHederaService>,
    pub audit_log_service: Arc<AuditLogService>,
}

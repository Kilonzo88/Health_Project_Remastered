use std::sync::Arc;

use crate::auditing::{AuditLogService, AuditingService};
use crate::config::Config;
use crate::database::Database;
use crate::services::ipfs::IpfsClient;
use crate::services::hedera::{HederaClient, HealthcareHederaService};
use crate::services::{AuthServiceImpl, PatientService, EncounterService, VerifiableCredentialService};
use crate::services::twilio::TwilioService;

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
    pub patient_service: Arc<PatientService>,
    pub encounter_service: Arc<EncounterService>,
    pub vc_service: Arc<VerifiableCredentialService>,
}

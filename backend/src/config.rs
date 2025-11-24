use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub hedera_network: String,
    pub hedera_account_id: String,
    pub hedera_private_key: String,
    pub ipfs_url: String,
    pub jwt_secret: String,
    pub jwt_expiration_seconds: i64,
    pub ipfs_encryption_key: String,
    pub server_port: u16,
    pub healthcare_access_control_contract_id: String,
    pub verifiable_credentials_contract_id: String,
    pub audit_trail_contract_id: String,
    pub google_client_id: String,
    pub twilio_account_sid: String,
    pub twilio_auth_token: String,
    pub twilio_phone_number: String,
    pub gemini_api_key: String,
}

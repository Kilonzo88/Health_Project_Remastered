use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmtpConfig {
    pub server: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub from_email: String,
}

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
    pub use_tls: bool,
    pub frontend_base_url: String,
    pub smtp: SmtpConfig, // Added SmtpConfig here
}

impl Config {
    pub fn load() -> Result<Self> {
        dotenv::dotenv().ok();
        
        Ok(Config {
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            hedera_network: env::var("HEDERA_NETWORK").expect("HEDERA_NETWORK must be set"),
            hedera_account_id: env::var("HEDERA_ACCOUNT_ID")
                .expect("HEDERA_ACCOUNT_ID must be set"),
            hedera_private_key: env::var("HEDERA_PRIVATE_KEY")
                .expect("HEDERA_PRIVATE_KEY must be set"),
            ipfs_url: env::var("IPFS_URL").expect("IPFS_URL must be set"),
            jwt_secret: env::var("JWT_SECRET")
                .expect("JWT_SECRET must be set"),
            jwt_expiration_seconds: env::var("JWT_EXPIRATION_SECONDS")
                .expect("JWT_EXPIRATION_SECONDS must be set")
                .parse()
                .expect("Invalid JWT_EXPIRATION_SECONDS"),
            ipfs_encryption_key: env::var("IPFS_ENCRYPTION_KEY")
                .expect("IPFS_ENCRYPTION_KEY must be set"),
            server_port: env::var("SERVER_PORT")
                .expect("SERVER_PORT must be set")
                .parse()
                .expect("Invalid SERVER_PORT"),
            healthcare_access_control_contract_id: env::var("HEALTHCARE_ACCESS_CONTROL_CONTRACT_ID")
                .expect("HEALTHCARE_ACCESS_CONTROL_CONTRACT_ID must be set"),
            verifiable_credentials_contract_id: env::var("VERIFIABLE_CREDENTIALS_CONTRACT_ID")
                .expect("VERIFIABLE_CREDENTIALS_CONTRACT_ID must be set"),
            audit_trail_contract_id: env::var("AUDIT_TRAIL_CONTRACT_ID")
                .expect("AUDIT_TRAIL_CONTRACT_ID must be set"),
            google_client_id: env::var("GOOGLE_CLIENT_ID").expect("GOOGLE_CLIENT_ID must be set"),
            twilio_account_sid: env::var("TWILIO_ACCOUNT_SID").expect("TWILIO_ACCOUNT_SID must be set"),
            twilio_auth_token: env::var("TWILIO_AUTH_TOKEN").expect("TWILIO_AUTH_TOKEN must be set"),
            twilio_phone_number: env::var("TWILIO_PHONE_NUMBER").expect("TWILIO_PHONE_NUMBER must be set"),
            gemini_api_key: env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY must be set"),
            use_tls: env::var("USE_TLS")
                .expect("USE_TLS must be set")
                .parse()
                .expect("Invalid USE_TLS value"),
            frontend_base_url: env::var("FRONTEND_BASE_URL").expect("FRONTEND_BASE_URL must be set"),
            smtp: SmtpConfig { // Populating SmtpConfig
                server: env::var("SMTP_SERVER").expect("SMTP_SERVER must be set"),
                port: env::var("SMTP_PORT")
                    .expect("SMTP_PORT must be set")
                    .parse()
                    .expect("Invalid SMTP_PORT"),
                username: env::var("SMTP_USERNAME").expect("SMTP_USERNAME must be set"),
                password: env::var("SMTP_PASSWORD").expect("SMTP_PASSWORD must be set"),
                from_email: env::var("SMTP_FROM_EMAIL").expect("SMTP_FROM_EMAIL must be set"),
            },
        })
    }
}

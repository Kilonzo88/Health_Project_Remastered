use anyhow::Result;
use serde::{Deserialize, Serialize};
use crate::services::hedera::HederaClient;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DidDocument {
    #[serde(rename = "@context")]
    pub context: Vec<String>,
    pub id: String,
    pub verification_method: Vec<VerificationMethod>,
    pub authentication: Vec<String>,
    pub assertion_method: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationMethod {
    pub id: String,
    #[serde(rename = "type")]
    pub verification_type: String,
    pub controller: String,
    #[serde(rename = "publicKeyMultibase")]
    pub public_key_multibase: String,
}

pub struct DidManager;

impl DidManager {
    pub async fn create_did(hedera_client: &HederaClient, public_key_hex: &str, network: &str) -> Result<String> {
        // 1. Construct the DID string before creating the document
        // This is a temporary placeholder until we get the file ID
        let temp_did = format!("did:hedera:{}:_placeholder_", network);

        // 2. Create the DID Document
        let verification_method_id = format!("{}#key-1", temp_did);
        let doc = DidDocument {
            context: vec![
                "https://www.w3.org/ns/did/v1".to_string(),
                "https://w3id.org/security/suites/ed25519-2020/v1".to_string(),
            ],
            id: temp_did.clone(),
            verification_method: vec![VerificationMethod {
                id: verification_method_id.clone(),
                verification_type: "Ed25519VerificationKey2020".to_string(),
                controller: temp_did.clone(),
                public_key_multibase: format!("z{}", public_key_hex),
            }],
            authentication: vec![verification_method_id.clone()],
            assertion_method: vec![verification_method_id.clone()],
        };

        // 3. Serialize the document and store it on Hedera File Service
        let doc_json = serde_json::to_vec(&doc)?;
        let file_id = hedera_client.create_file(&doc_json).await?;

        // 4. Construct the final, correct DID using the new File ID
        let final_did = format!("did:hedera:{}:{}", network, file_id);

        // 5. Update the document in memory with the correct DID
        let mut final_doc = doc;
        final_doc.id = final_did.clone();
        final_doc.verification_method[0].id = format!("{}#key-1", final_did);
        final_doc.verification_method[0].controller = final_did.clone();
        final_doc.authentication[0] = format!("{}#key-1", final_did);
        final_doc.assertion_method[0] = format!("{}#key-1", final_did);

        // 6. Serialize the final document and update the file on Hedera
        let final_doc_json = serde_json::to_vec(&final_doc)?;
        hedera_client.update_file(file_id, &final_doc_json).await?;

        Ok(final_did)
    }
}

#[cfg(test)]
mod tests {
    // Note: These tests would require a live Hedera client and network, 
    // so they are commented out. Integration tests would be needed.

    // use super::*;
    // use crate::config::Config;

    // #[tokio::test]
    // async fn test_did_hedera_creation() {
    //     let config = Config::load().unwrap();
    //     let hedera_client = HederaClient::new(&config.hedera_account_id, &config.hedera_private_key).unwrap();
        
    //     // A sample ed25519 public key in hex format (32 bytes)
    //     let public_key_hex = "6e85794657c6fa4c1518c6a92c145955b839a1823c69c856d042eb433a91d434";

    //     let did = DidManager::create_did(&hedera_client, public_key_hex, &config.hedera_network).await.unwrap();
        
    //     assert!(did.starts_with("did:hedera:testnet:"));
    //     println!("Created DID: {}", did);
    // }
}
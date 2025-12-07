use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct IpfsClient {
    client: Client,
    base_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IpfsResponse {
    pub name: String,
    pub hash: String,
    pub size: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IpfsPinResponse {
    pub pins: Vec<String>,
    pub progress: Option<u32>,
}

impl IpfsClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.to_string(),
        }
    }

    /// Add a file to IPFS
    pub async fn add_file(&self, content: &[u8], filename: Option<&str>) -> Result<String> {
        let url = format!("{}/api/v0/add", self.base_url);
        
        let mut form = reqwest::multipart::Form::new()
            .part("file", reqwest::multipart::Part::bytes(content.to_vec()));
        
        if let Some(name) = filename {
            form = form.part("filename", reqwest::multipart::Part::text(name.to_string()));
        }

        let response = self.client
            .post(&url)
            .multipart(form)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("IPFS add failed: {}", response.status()));
        }

        let ipfs_response: IpfsResponse = response.json().await?;
        Ok(ipfs_response.hash)
    }

    /// Add a JSON object to IPFS
    pub async fn add_json<T: Serialize>(&self, data: &T, filename: Option<&str>) -> Result<String> {
        let json_string = serde_json::to_string_pretty(data)?;
        self.add_file(json_string.as_bytes(), filename).await
    }

    /// Retrieve a file from IPFS
    pub async fn get_file(&self, hash: &str) -> Result<Vec<u8>> {
        let url = format!("{}/api/v0/cat/{}", self.base_url, hash);
        
        let response = self.client
            .post(&url)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("IPFS get failed: {}", response.status()));
        }

        let content = response.bytes().await?;
        Ok(content.to_vec())
    }

    /// Retrieve and parse a JSON object from IPFS
    pub async fn get_json<T: for<'de> Deserialize<'de>>(&self, hash: &str) -> Result<T> {
        let content = self.get_file(hash).await?;
        let json_string = String::from_utf8(content)?;
        Ok(serde_json::from_str(&json_string)?)
    }

    /// Pin a file to IPFS (ensure it stays available)
    pub async fn pin_add(&self, hash: &str) -> Result<Vec<String>> {
        let url = format!("{}/api/v0/pin/add/{}", self.base_url, hash);
        
        let response = self.client
            .post(&url)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("IPFS pin add failed: {}", response.status()));
        }

        let pin_response: IpfsPinResponse = response.json().await?;
        Ok(pin_response.pins)
    }

    /// Unpin a file from IPFS
    pub async fn pin_rm(&self, hash: &str) -> Result<Vec<String>> {
        let url = format!("{}/api/v0/pin/rm/{}", self.base_url, hash);
        
        let response = self.client
            .post(&url)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("IPFS pin rm failed: {}", response.status()));
        }

        let pin_response: IpfsPinResponse = response.json().await?;
        Ok(pin_response.pins)
    }

    /// Check if a file is pinned
    pub async fn pin_ls(&self, hash: Option<&str>) -> Result<HashMap<String, String>> {
        let url = if let Some(h) = hash {
            format!("{}/api/v0/pin/ls/{}", self.base_url, h)
        } else {
            format!("{}/api/v0/pin/ls", self.base_url)
        };
        
        let response = self.client
            .post(&url)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("IPFS pin ls failed: {}", response.status()));
        }

        let pin_list: HashMap<String, String> = response.json().await?;
        Ok(pin_list)
    }

    /// Get file information
    pub async fn stat(&self, hash: &str) -> Result<IpfsResponse> {
        let url = format!("{}/api/v0/object/stat/{}", self.base_url, hash);
        
        let response = self.client
            .post(&url)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("IPFS stat failed: {}", response.status()));
        }

        let stat_response: IpfsResponse = response.json().await?;
        Ok(stat_response)
    }

    /// Store a verifiable credential on IPFS
    pub async fn store_credential(&self, credential: &crate::models::VerifiableCredential) -> Result<String> {
        let filename = format!("credential_{}.json", credential.issuer);
        self.add_json(credential, Some(&filename)).await
    }

    /// Retrieve a verifiable credential from IPFS
    pub async fn get_credential(&self, hash: &str) -> Result<crate::models::VerifiableCredential> {
        self.get_json(hash).await
    }

    /// Store a FHIR bundle on IPFS
    pub async fn store_fhir_bundle(&self, bundle: &crate::models::FhirBundle) -> Result<String> {
        let filename = format!("fhir_bundle_{}.json", bundle.patient_did);
        self.add_json(&bundle.bundle, Some(&filename)).await
    }

    /// Retrieve a FHIR bundle from IPFS
    pub async fn get_fhir_bundle(&self, hash: &str) -> Result<serde_json::Value> {
        self.get_json(hash).await
    }

    /// Store encrypted patient data on IPFS
    pub async fn store_encrypted_data(&self, data: &[u8], patient_did: &str) -> Result<String> {
        let filename = format!("encrypted_data_{}.bin", patient_did);
        self.add_file(data, Some(&filename)).await
    }

    /// Retrieve encrypted patient data from IPFS
    pub async fn get_encrypted_data(&self, hash: &str) -> Result<Vec<u8>> {
        self.get_file(hash).await
    }

    /// Check if IPFS node is available
    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/api/v0/version", self.base_url);
        
        match self.client.post(&url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    /// Get IPFS node version
    pub async fn get_version(&self) -> Result<String> {
        let url = format!("{}/api/v0/version", self.base_url);
        
        let response = self.client
            .post(&url)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("IPFS version check failed: {}", response.status()));
        }

        let version_info: serde_json::Value = response.json().await?;
        Ok(version_info["Version"].as_str().unwrap_or("unknown").to_string())
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[tokio::test]
//     async fn test_ipfs_client_creation() {
//         let client = IpfsClient::new("http://localhost:5001");
//         assert_eq!(client.base_url, "http://localhost:5001");
//     }

//     #[tokio::test]
//     async fn test_health_check() {
//         let client = IpfsClient::new("http://localhost:5001");
//         // This test will only pass if IPFS is running locally
//         let is_healthy = client.health_check().await.unwrap_or(false);
//         // We can't assert true here since IPFS might not be running in tests
//         println!("IPFS health check: {}", is_healthy);
//     }
// }

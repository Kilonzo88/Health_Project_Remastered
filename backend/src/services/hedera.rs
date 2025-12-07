use anyhow::Result;
use hedera::{
    Client,
    FileCreateTransaction,
    FileUpdateTransaction,
    ContractCreateTransaction,
    ContractFunctionParameters,
    PrivateKey,
    AccountId,
    Hbar,
    FileId,
    ContractExecuteTransaction,
    ContractCallQuery,
    TransactionRecordQuery,
    TransactionRecord,
};

// Re-export types needed by crate root to avoid name collisions with our module name
pub use hedera::ContractId;

#[derive(Debug, Clone)]
pub struct HederaClient {
    client: Client,
    operator_private_key: PrivateKey,
}

impl HederaClient {
    pub fn new(account_id: &str, private_key: &str, network: &str) -> Result<Self> {
        let account_id: AccountId = account_id.parse()?;
        let private_key: PrivateKey = private_key.parse()?;

        let client = match network {
            "mainnet" => Client::for_mainnet(),
            "previewnet" => Client::for_previewnet(),
            _ => Client::for_testnet(),
        };
        client.set_operator(account_id, private_key.clone());

        Ok(Self { client, operator_private_key: private_key })
    }

    pub async fn create_contract(&self, bytecode: &[u8]) -> Result<ContractId> {
        // 1. Create a file on Hedera for the contract bytecode
        let mut file_tx = FileCreateTransaction::new();
        file_tx.keys([self.operator_private_key.public_key()])
            .contents(bytecode)
            .max_transaction_fee(Hbar::new(2));
        
        let signed_tx = file_tx.freeze_with(&self.client)?.sign(self.operator_private_key.clone());
        let tx_response = signed_tx.execute(&self.client).await?;
        let receipt = tx_response.get_receipt(&self.client).await?;
        let file_id = receipt.file_id.ok_or_else(|| anyhow::anyhow!("File ID not found in receipt "))?;

        // 2. Create the smart contract
        let mut contract_tx = ContractCreateTransaction::new();
        contract_tx.bytecode_file_id(file_id)
            .gas(100_000)
            .max_transaction_fee(Hbar::new(16));

        let contract_response = contract_tx.execute(&self.client).await?;
        let contract_receipt = contract_response.get_receipt(&self.client).await?;
        let contract_id = contract_receipt.contract_id.ok_or_else(|| anyhow::anyhow!("Contract ID not found in receipt "))?;

        tracing::info!("Successfully created contract with ID: {}", contract_id);

        Ok(contract_id)
    }

    pub async fn call_contract(
        &self,
        contract_id: &ContractId,
        function_name: &str,
        parameters: ContractFunctionParameters,
    ) -> Result<TransactionRecord> {
        let mut tx = ContractExecuteTransaction::new();
        tx.contract_id(*contract_id)
            .gas(100_000)
            .function(function_name)
            .function_parameters(parameters.to_bytes(None))
            .max_transaction_fee(Hbar::new(2));

        let tx_response = tx.execute(&self.client).await?;
        let record = TransactionRecordQuery::new()
            .transaction_id(tx_response.transaction_id)
            .execute(&self.client)
            .await?;

        Ok(record)
    }

    pub async fn query_contract(
        &self,
        contract_id: &ContractId,
        function_name: &str,
        parameters: ContractFunctionParameters,
    ) -> Result<Vec<u8>> {
        let mut query = ContractCallQuery::new();
        query.contract_id(*contract_id)
            .gas(100_000)
            .function(function_name)
            .function_parameters(parameters.to_bytes(None));

        let result = query.execute(&self.client).await?;
        Ok(result.as_bytes().to_vec())
    }

    pub async fn create_file(&self, contents: &[u8]) -> Result<FileId> {
        let mut file_tx = FileCreateTransaction::new();
        file_tx.keys([self.operator_private_key.public_key()])
            .contents(contents.to_vec())
            .max_transaction_fee(Hbar::new(2));

        let signed_tx = file_tx.freeze_with(&self.client)?.sign(self.operator_private_key.clone());
        let tx_response = signed_tx.execute(&self.client).await?;
        let receipt = tx_response.get_receipt(&self.client).await?;
        let file_id = receipt.file_id.ok_or_else(|| anyhow::anyhow!("File ID not found in receipt "))?;

        Ok(file_id)
    }

    pub async fn update_file(&self, file_id: FileId, contents: &[u8]) -> Result<()> {
        let mut file_tx = FileUpdateTransaction::new();
        file_tx.file_id(file_id)
            .contents(contents.to_vec())
            .max_transaction_fee(Hbar::new(2));

        let signed_tx = file_tx.freeze_with(&self.client)?.sign(self.operator_private_key.clone());
        let tx_response = signed_tx.execute(&self.client).await?;
        tx_response.get_receipt(&self.client).await?;

        Ok(())
    }
}

pub struct HealthcareHederaService {
    client: HederaClient,
    access_control_contract: Option<ContractId>,
    credentials_contract: Option<ContractId>,
    audit_trail_contract: Option<ContractId>,
}

impl HealthcareHederaService {
    pub fn new(client: HederaClient) -> Self {
        Self {
            client,
            access_control_contract: None,
            credentials_contract: None,
            audit_trail_contract: None,
        }
    }

    pub fn set_contract_ids(
        &mut self,
        access_control: ContractId,
        credentials: ContractId,
        audit_trail: ContractId,
    ) {
        self.access_control_contract = Some(access_control);
        self.credentials_contract = Some(credentials);
        self.audit_trail_contract = Some(audit_trail);
    }

    pub async fn deploy_access_control_contract(&mut self, bytecode: &[u8]) -> Result<ContractId> {
        let contract_id = self.client.create_contract(bytecode).await?;
        self.access_control_contract = Some(contract_id.clone());
        Ok(contract_id)
    }

    pub async fn deploy_credentials_contract(&mut self, bytecode: &[u8]) -> Result<ContractId> {
        let contract_id = self.client.create_contract(bytecode).await?;
        self.credentials_contract = Some(contract_id.clone());
        Ok(contract_id)
    }

    pub async fn deploy_audit_trail_contract(&mut self, bytecode: &[u8]) -> Result<ContractId> {
        let contract_id = self.client.create_contract(bytecode).await?;
        self.audit_trail_contract = Some(contract_id.clone());
        Ok(contract_id)
    }

    pub async fn anchor_log_batch(&self, root_hash: [u8; 32], batch_size: u64) -> Result<TransactionRecord> {
        if let Some(contract_id) = &self.audit_trail_contract {
            let mut params = ContractFunctionParameters::new();
            params.add_bytes(&root_hash);
            params.add_uint64(batch_size);

            self.client.call_contract(contract_id, "anchorLogBatch", params).await
        } else {
            Err(anyhow::anyhow!("AuditTrail contract not deployed"))
        }
    }

    pub async fn store_credential(
        &self,
        subject_did: &str,
        credential_type: &str,
        ipfs_hash: &str,
        expires_at: Option<u64>,
        metadata: &str,
    ) -> Result<TransactionRecord> {
        if let Some(contract_id) = &self.credentials_contract {
            let mut params = ContractFunctionParameters::new();
            params.add_string(subject_did);
            params.add_string(credential_type);
            params.add_string(ipfs_hash);
            params.add_uint64(expires_at.unwrap_or(0));
            params.add_string(metadata);

            self.client.call_contract(contract_id, "storeCredential", params).await
        } else {
            Err(anyhow::anyhow!("Credentials contract not deployed "))
        }
    }

    pub async fn verify_credential(&self, credential_hash: &[u8]) -> Result<bool> {
        if let Some(contract_id) = &self.credentials_contract {
            let mut params = ContractFunctionParameters::new();
            params.add_bytes(credential_hash);

            let result = self.client.query_contract(contract_id, "verifyCredential", params).await?;

            // The result is a boolean encoded as a 32-byte array.
            Ok(result[31] == 1)
        } else {
            Err(anyhow::anyhow!("Credentials contract not deployed "))
        }
    }
}

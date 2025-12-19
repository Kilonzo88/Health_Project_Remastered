use anyhow::Result;
use mongodb::{Client, Database as MongoDatabase, Collection};
use futures_util::stream::TryStreamExt;
use bson::{oid::ObjectId, doc, DateTime};
use sha2::{Digest, Sha256};

use crate::models::*;
use crate::utils::{encrypt, decrypt};

pub struct Database {
    pub client: Client,
    pub db: MongoDatabase,
}

impl Database {
    pub async fn new(uri: &str) -> Result<Self> {
        let client = Client::with_uri_str(uri).await?;
        let db = client.database("healthcare");
        
        Self::create_indexes(&db).await?;
        
        Ok(Database { client, db })
    }

    async fn create_indexes(db: &MongoDatabase) -> Result<()> {
        // Patient indexes
        let patients: Collection<EncryptedPatient> = db.collection("patients");
        patients.create_index(
            mongodb::IndexModel::builder()
                .keys(doc! { "did": 1 })
                .options(mongodb::options::IndexOptions::builder().unique(true).build())
                .build(),
            None,
        ).await?;
        patients.create_index(
            mongodb::IndexModel::builder()
                .keys(doc! { "email_hash": 1 })
                .build(),
            None,
        ).await?;

        // Practitioner indexes
        let practitioners: Collection<Practitioner> = db.collection("practitioners");
        practitioners.create_index(
            mongodb::IndexModel::builder()
                .keys(doc! { "did": 1 })
                .options(mongodb::options::IndexOptions::builder().unique(true).build())
                .build(),
            None,
        ).await?;

        // Encounter indexes
        let encounters: Collection<Encounter> = db.collection("encounters");
        encounters.create_index(
            mongodb::IndexModel::builder()
                .keys(doc! { "patient_did": 1, "status": 1 })
                .build(),
            None,
        ).await?;

        // Prescription indexes
        let prescriptions: Collection<Prescription> = db.collection("prescriptions");
        prescriptions.create_index(
            mongodb::IndexModel::builder()
                .keys(doc! { "patient_did": 1 })
                .build(),
            None,
        ).await?;

        // Access control indexes
        let access_controls: Collection<AccessControl> = db.collection("access_controls");
        access_controls.create_index(
            mongodb::IndexModel::builder()
                .keys(doc! { "patient_did": 1, "grantee_did": 1 })
                .options(mongodb::options::IndexOptions::builder().unique(true).build())
                .build(),
            None,
        ).await?;

        // Verifiable Credential indexes
        let credentials: Collection<VerifiableCredential> = db.collection("verifiable_credentials");
        credentials.create_index(
            mongodb::IndexModel::builder()
                .keys(doc! { "subject_did": 1 })
                .build(),
            None,
        ).await?;

        // Audit Log indexes
        let audit_logs: Collection<AuditLog> = db.collection("audit_logs");
        audit_logs.create_index(
            mongodb::IndexModel::builder()
                .keys(doc! { "is_anchored": 1 })
                .build(),
            None,
        ).await?;

        // OTP indexes
        let otps: Collection<Otp> = db.collection("otps");
        otps.create_index(
            mongodb::IndexModel::builder()
                .keys(doc! { "phone_number": 1, "otp": 1 })
                .build(),
            None,
        ).await?;

        Ok(())
    }

    // Patient operations
    pub async fn create_patient(&self, patient: &Patient, encryption_key: &str) -> Result<()> {
        let collection: Collection<EncryptedPatient> = self.db.collection("patients");
        let fhir_patient_json = serde_json::to_string(&patient.fhir_patient)?;
        let encrypted_fhir_patient = encrypt(fhir_patient_json.as_bytes(), encryption_key)?;

        let email = patient.fhir_patient.telecom.iter().find(|c| c.system == "email").map(|c| c.value.as_str()).unwrap_or("");
        let mut hasher = Sha256::new();
        hasher.update(email.as_bytes());
        let email_hash = format!("{:x}", hasher.finalize());

        let encrypted_patient = EncryptedPatient {
            id: None,
            did: patient.did.clone(),
            encrypted_fhir_patient,
            email_hash,
            created_at: patient.created_at,
            updated_at: patient.updated_at,
            email_verified: patient.email_verified,
            verification_token: patient.verification_token.clone(),
            verification_token_expires: patient.verification_token_expires,
        };

        collection.insert_one(encrypted_patient, None).await?;
        Ok(())
    }

    pub async fn get_patient_by_did(&self, did: &str, encryption_key: &str) -> Result<Option<Patient>> {
        let collection: Collection<EncryptedPatient> = self.db.collection("patients");
        let filter = doc! { "did": did };
        if let Some(encrypted_patient) = collection.find_one(filter, None).await? {
            let decrypted_fhir_patient_json = decrypt(&encrypted_patient.encrypted_fhir_patient, encryption_key)?;
            let fhir_patient: FhirPatient = serde_json::from_slice(&decrypted_fhir_patient_json)?;

            let patient = Patient {
                id: encrypted_patient.id,
                did: encrypted_patient.did,
                fhir_patient,
                created_at: encrypted_patient.created_at,
                updated_at: encrypted_patient.updated_at,
                email_verified: encrypted_patient.email_verified,
                verification_token: encrypted_patient.verification_token,
                verification_token_expires: encrypted_patient.verification_token_expires,
            };
            Ok(Some(patient))
        } else {
            Ok(None)
        }
    }

    pub async fn get_patient_by_email(&self, email: &str, encryption_key: &str) -> Result<Option<Patient>> {
        let mut hasher = Sha256::new();
        hasher.update(email.as_bytes());
        let email_hash = format!("{:x}", hasher.finalize());

        let collection: Collection<EncryptedPatient> = self.db.collection("patients");
        let filter = doc! { "email_hash": email_hash };
        if let Some(encrypted_patient) = collection.find_one(filter, None).await? {
            let decrypted_fhir_patient_json = decrypt(&encrypted_patient.encrypted_fhir_patient, encryption_key)?;
            let fhir_patient: FhirPatient = serde_json::from_slice(&decrypted_fhir_patient_json)?;

            let patient = Patient {
                id: encrypted_patient.id,
                did: encrypted_patient.did,
                fhir_patient,
                created_at: encrypted_patient.created_at,
                updated_at: encrypted_patient.updated_at,
                email_verified: encrypted_patient.email_verified,
                verification_token: encrypted_patient.verification_token,
                verification_token_expires: encrypted_patient.verification_token_expires,
            };
            Ok(Some(patient))
        } else {
            Ok(None)
        }
    }

    pub async fn get_patient_by_phone(&self, phone_number: &str, encryption_key: &str) -> Result<Option<Patient>> {
        let collection: Collection<EncryptedPatient> = self.db.collection("patients");
        // This is inefficient, as it requires decrypting all patients. 
        // A better approach would be to store a hash of the phone number, similar to the email.
        let mut cursor = collection.find(None, None).await?;
        while let Some(encrypted_patient) = cursor.try_next().await? {
            let decrypted_fhir_patient_json = decrypt(&encrypted_patient.encrypted_fhir_patient, encryption_key)?;
            let fhir_patient: FhirPatient = serde_json::from_slice(&decrypted_fhir_patient_json)?;

            if fhir_patient.telecom.iter().any(|c| c.system == "phone" && c.value == phone_number) {
                let patient = Patient {
                    id: encrypted_patient.id,
                    did: encrypted_patient.did,
                    fhir_patient,
                    created_at: encrypted_patient.created_at,
                    updated_at: encrypted_patient.updated_at,
                    email_verified: encrypted_patient.email_verified,
                    verification_token: encrypted_patient.verification_token,
                    verification_token_expires: encrypted_patient.verification_token_expires,
                };
                return Ok(Some(patient));
            }
        }
        Ok(None)
    }

    pub async fn find_patient_by_verification_token(&self, token: &str, encryption_key: &str) -> Result<Option<Patient>> {
        let collection: Collection<EncryptedPatient> = self.db.collection("patients");
        let filter = doc! { "verification_token": token };
        if let Some(encrypted_patient) = collection.find_one(filter, None).await? {
            let decrypted_fhir_patient_json = decrypt(&encrypted_patient.encrypted_fhir_patient, encryption_key)?;
            let fhir_patient: FhirPatient = serde_json::from_slice(&decrypted_fhir_patient_json)?;

            let patient = Patient {
                id: encrypted_patient.id,
                did: encrypted_patient.did,
                fhir_patient,
                created_at: encrypted_patient.created_at,
                updated_at: encrypted_patient.updated_at,
                email_verified: encrypted_patient.email_verified,
                verification_token: encrypted_patient.verification_token,
                verification_token_expires: encrypted_patient.verification_token_expires,
            };
            Ok(Some(patient))
        } else {
            Ok(None)
        }
    }

    pub async fn set_patient_email_verified(&self, did: &str, verified: bool) -> Result<()> {
        let collection: Collection<EncryptedPatient> = self.db.collection("patients");
        let filter = doc! { "did": did };
        let update = doc! {
            "$set": {
                "email_verified": verified,
                "updated_at": DateTime::now(),
            },
            "$unset": {
                "verification_token": "",
                "verification_token_expires": "",
            }
        };
        collection.update_one(filter, update, None).await?;
        Ok(())
    }

    // Practitioner operations
    pub async fn create_practitioner(&self, practitioner: &Practitioner) -> Result<()> {
        let collection: Collection<Practitioner> = self.db.collection("practitioners");
        collection.insert_one(practitioner, None).await?;
        Ok(())
    }

    pub async fn get_practitioner_by_did(&self, did: &str) -> Result<Option<Practitioner>> {
        let collection: Collection<Practitioner> = self.db.collection("practitioners");
        let filter = doc! { "did": did };
        Ok(collection.find_one(filter, None).await?)
    }

    // Encounter Operations
    pub async fn create_encounter(&self, encounter: &Encounter) -> Result<ObjectId> {
        let collection: Collection<Encounter> = self.db.collection("encounters");
        let result = collection.insert_one(encounter, None).await?;
        Ok(result.inserted_id.as_object_id().unwrap())
    }

    pub async fn get_encounter(&self, encounter_id: ObjectId) -> Result<Option<Encounter>> {
        let collection: Collection<Encounter> = self.db.collection("encounters");
        Ok(collection.find_one(doc! { "_id": encounter_id }, None).await?)
    }

    pub async fn get_observations_for_encounter(&self, encounter_id: &str) -> Result<Vec<FhirObservation>> {
        let collection: Collection<FhirObservation> = self.db.collection("observations");
        let filter = doc! { "encounter.reference": format!("Encounter/{}", encounter_id) };
        let cursor = collection.find(filter, None).await?;
        Ok(cursor.try_collect().await?)
    }

    pub async fn get_conditions_for_encounter(&self, encounter_id: &str) -> Result<Vec<FhirCondition>> {
        let collection: Collection<FhirCondition> = self.db.collection("conditions");
        let filter = doc! { "encounter.reference": format!("Encounter/{}", encounter_id) };
        let cursor = collection.find(filter, None).await?;
        Ok(cursor.try_collect().await?)
    }

    pub async fn get_medication_requests_for_encounter(&self, encounter_id: &str) -> Result<Vec<FhirMedicationRequest>> {
        let collection: Collection<FhirMedicationRequest> = self.db.collection("medication_requests");
        let filter = doc! { "encounter.reference": format!("Encounter/{}", encounter_id) };
        let cursor = collection.find(filter, None).await?;
        Ok(cursor.try_collect().await?)
    }

    pub async fn finalize_encounter(&self, encounter_id: ObjectId, ipfs_hash: &str) -> Result<()> {
        let collection: Collection<Encounter> = self.db.collection("encounters");
        let filter = doc! { "_id": encounter_id };
        let update = doc! { "$set": { 
            "status": "Finalized", 
            "final_bundle_ipfs_hash": ipfs_hash,
            "updated_at": DateTime::now()
        } };
        collection.update_one(filter, update, None).await?;
        Ok(())
    }

    // Prescription operations
    pub async fn create_prescription(&self, prescription: &Prescription) -> Result<()> {
        let collection: Collection<Prescription> = self.db.collection("prescriptions");
        collection.insert_one(prescription, None).await?;
        Ok(())
    }

    pub async fn get_prescriptions_by_patient(&self, patient_did: &str) -> Result<Vec<Prescription>> {
        let collection: Collection<Prescription> = self.db.collection("prescriptions");
        let filter = doc! { "patient_did": patient_did };
        let cursor = collection.find(filter, None).await?;
        Ok(cursor.try_collect().await?)
    }

    // Access control operations
    pub async fn grant_access(&self, access_control: &AccessControl) -> Result<()> {
        let collection: Collection<AccessControl> = self.db.collection("access_controls");
        collection.insert_one(access_control, None).await?;
        Ok(())
    }

    pub async fn check_access(&self, patient_did: &str, grantee_did: &str) -> Result<bool> {
        let collection: Collection<AccessControl> = self.db.collection("access_controls");
        let filter = doc! { 
            "patient_did": patient_did, 
            "grantee_did": grantee_did,
            "active": true
        };
        Ok(collection.find_one(filter, None).await?.is_some())
    }

    // FHIR Bundle operations
    pub async fn create_fhir_bundle(&self, bundle: &FhirBundle) -> Result<()> {
        let collection: Collection<FhirBundle> = self.db.collection("fhir_bundles");
        collection.insert_one(bundle, None).await?;
        Ok(())
    }

    pub async fn get_fhir_bundle(&self, patient_did: &str) -> Result<Option<FhirBundle>> {
        let collection: Collection<FhirBundle> = self.db.collection("fhir_bundles");
        let filter = doc! { "patient_did": patient_did };
        Ok(collection.find_one(filter, None).await?)
    }

    // Verifiable Credential operations
    pub async fn create_verifiable_credential(&self, credential: &VerifiableCredential) -> Result<()> {
        let collection: Collection<VerifiableCredential> = self.db.collection("verifiable_credentials");
        collection.insert_one(credential, None).await?;
        Ok(())
    }

    // Audit Log operations
    pub async fn create_audit_log(&self, log: &AuditLog) -> Result<()> {
        let collection: Collection<AuditLog> = self.db.collection("audit_logs");
        collection.insert_one(log, None).await?;
        Ok(())
    }

    pub async fn get_unanchored_audit_logs(&self) -> Result<Vec<AuditLog>> {
        let collection: Collection<AuditLog> = self.db.collection("audit_logs");
        let filter = doc! { "is_anchored": false };
        let cursor = collection.find(filter, None).await?;
        Ok(cursor.try_collect().await?)
    }

    pub async fn mark_logs_as_anchored(&self, log_ids: &[ObjectId], anchor_batch_id: ObjectId) -> Result<()> {
        let collection: Collection<AuditLog> = self.db.collection("audit_logs");
        let filter = doc! { "_id": { "$in": log_ids } };
        let update = doc! { "$set": { 
            "is_anchored": true, 
            "anchor_batch_id": anchor_batch_id 
        } };
        collection.update_many(filter, update, None).await?;
        Ok(())
    }

    // OTP operations
    pub async fn create_otp(&self, otp: &Otp) -> Result<()> {
        let collection: Collection<Otp> = self.db.collection("otps");
        collection.insert_one(otp, None).await?;
        Ok(())
    }

    pub async fn get_otp(&self, phone_number: &str, otp: &str) -> Result<Option<Otp>> {
        let collection: Collection<Otp> = self.db.collection("otps");
        let filter = doc! { "phone_number": phone_number, "otp": otp };
        Ok(collection.find_one(filter, None).await?)
    }
}

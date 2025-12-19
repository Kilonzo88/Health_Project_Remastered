use serde::{Deserialize, Serialize};
use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use mongodb::bson::doc;

// Core entity models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Patient {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub did: String,
    pub fhir_patient: FhirPatient,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub email_verified: bool,
    pub verification_token: Option<String>,
    pub verification_token_expires: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedPatient {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub did: String,
    pub encrypted_fhir_patient: String,
    pub email_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub email_verified: bool,
    pub verification_token: Option<String>,
    pub verification_token_expires: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Practitioner {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub did: String,
    pub fhir_practitioner: FhirPractitioner,
    pub license_verification: LicenseVerification,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Encounter {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub patient_did: String,
    pub practitioner_did: String,
    pub fhir_encounter: FhirEncounter,
    pub status: EncounterStatus,
    pub final_bundle_ipfs_hash: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EncounterStatus {
    Active,
    Finalized,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prescription {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub patient_did: String,
    pub practitioner_did: String,
    pub fhir_medication_request: FhirMedicationRequest,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessControl {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub patient_did: String,
    pub grantee_did: String,
    pub permissions: Vec<Permission>,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FhirBundle {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub patient_did: String,
    pub bundle: serde_json::Value,
    pub version: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Otp {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub phone_number: String,
    pub otp: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

// FHIR R4 Models
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FhirPatient {
    #[serde(rename = "resourceType")]
    pub resource_type: String,
    pub id: String,
    pub identifier: Vec<FhirIdentifier>,
    pub name: Vec<FhirHumanName>,
    pub gender: String,
    pub birth_date: String,
    pub address: Vec<FhirAddress>,
    pub telecom: Vec<FhirContactPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FhirPractitioner {
    #[serde(rename = "resourceType")]
    pub resource_type: String,
    pub id: String,
    pub identifier: Vec<FhirIdentifier>,
    pub name: Vec<FhirHumanName>,
    pub qualification: Vec<FhirPractitionerQualification>,
    pub telecom: Vec<FhirContactPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FhirMedicationRequest {
    #[serde(rename = "resourceType")]
    pub resource_type: String,
    pub id: String,
    pub status: String,
    pub intent: String,
    pub medication_codeable_concept: FhirCodeableConcept,
    pub subject: FhirReference,
    pub encounter: Option<FhirReference>,
    pub authored_on: String,
    pub requester: FhirReference,
    pub dosage_instruction: Vec<FhirDosageInstruction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FhirEncounter {
    #[serde(rename = "resourceType")]
    pub resource_type: String,
    pub id: String,
    pub status: String,
    pub class: FhirCoding,
    pub subject: FhirReference,
    pub participant: Vec<FhirEncounterParticipant>,
    pub period: FhirPeriod,
    pub reason_code: Vec<FhirCodeableConcept>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FhirObservation {
    #[serde(rename = "resourceType")]
    pub resource_type: String,
    pub id: String,
    pub status: String,
    pub category: Vec<FhirCodeableConcept>,
    pub code: FhirCodeableConcept,
    pub subject: FhirReference,
    pub encounter: Option<FhirReference>,
    pub effective_date_time: String,
    pub value_quantity: Option<FhirQuantity>,
    pub value_string: Option<String>,
    pub interpretation: Vec<FhirCodeableConcept>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FhirCondition {
    #[serde(rename = "resourceType")]
    pub resource_type: String,
    pub id: String,
    pub clinical_status: FhirCodeableConcept,
    pub verification_status: FhirCodeableConcept,
    pub category: Vec<FhirCodeableConcept>,
    pub code: FhirCodeableConcept,
    pub subject: FhirReference,
    pub encounter: Option<FhirReference>,
    pub onset_date_time: String,
    pub recorded_date: String,
}

// FHIR Common Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FhirIdentifier {
    #[serde(rename = "use")]
    pub use_field: Option<String>,
    #[serde(rename = "type")]
    pub identifier_type: Option<FhirCodeableConcept>,
    pub system: Option<String>,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FhirHumanName {
    #[serde(rename = "use")]
    pub r#use: Option<String>,
    pub family: Option<String>,
    pub given: Vec<String>,
    pub prefix: Vec<String>,
    pub suffix: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FhirAddress {
    #[serde(rename = "use")]
    pub r#use: Option<String>,
    pub line: Vec<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FhirContactPoint {
    pub system: String,
    pub value: String,
    #[serde(rename = "use")]
    pub r#use: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FhirCodeableConcept {
    pub coding: Vec<FhirCoding>,
    pub text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FhirCoding {
    pub system: Option<String>,
    pub code: Option<String>,
    pub display: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FhirReference {
    pub reference: String,
    pub display: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FhirPractitionerQualification {
    pub identifier: Vec<FhirIdentifier>,
    pub code: FhirCodeableConcept,
    pub period: Option<FhirPeriod>,
    pub issuer: Option<FhirReference>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FhirDosageInstruction {
    pub text: Option<String>,
    pub timing: Option<FhirTiming>,
    pub dose_and_rate: Vec<FhirDosageDoseAndRate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FhirDosageDoseAndRate {
    #[serde(rename = "type")]
    pub dose_type: Option<FhirCodeableConcept>,
    pub dose_quantity: Option<FhirQuantity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FhirQuantity {
    pub value: Option<f64>,
    pub unit: Option<String>,
    pub system: Option<String>,
    pub code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FhirTiming {
    pub repeat: Option<FhirTimingRepeat>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FhirTimingRepeat {
    pub frequency: Option<u32>,
    pub period: Option<f64>,
    pub period_unit: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FhirEncounterParticipant {
    #[serde(rename = "type")]
    pub participant_type: Vec<FhirCodeableConcept>,
    pub individual: Option<FhirReference>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FhirPeriod {
    pub start: Option<String>,
    pub end: Option<String>,
}

// License and Verification Models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseVerification {
    pub license_number: String,
    pub issuing_authority: String,
    pub issue_date: String,
    pub expiry_date: String,
    pub hedera_transaction_id: String,
    pub ipfs_hash: String,
    pub verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifiableCredential {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub subject_did: String,
    pub credential_type: String,
    pub issuer: String,
    pub issued_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub ipfs_hash: String,
    pub hedera_transaction_id: String,
    pub metadata: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub did: String,
    pub action: String,
    pub timestamp: DateTime<Utc>,
    pub details: Option<serde_json::Value>,
    pub is_anchored: bool,
    pub anchor_batch_id: Option<ObjectId>,
}


// Permission and Access Control
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Permission {
    Read,
    Write,
    Prescribe,
    ViewPrescriptions,
    ViewEncounters,
    ViewObservations,
}

// API Request/Response Models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePatientRequest {
    pub fhir_patient: FhirPatient,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePractitionerRequest {
    pub fhir_practitioner: FhirPractitioner,
    pub license_verification: LicenseVerification,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePrescriptionRequest {
    pub patient_did: String,
    pub medication_request: FhirMedicationRequest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrantAccessRequest {
    pub patient_did: String,
    pub grantee_did: String,
    pub permissions: Vec<Permission>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub timestamp: DateTime<Utc>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: Utc::now(),
        }
    }

    pub fn error(error: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
            timestamp: Utc::now(),
        }
    }
}

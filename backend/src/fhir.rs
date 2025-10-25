use anyhow::Result;
use serde_json::{json, Value};
use chrono::Utc;
use uuid::Uuid;

use crate::models::*;

pub struct FhirManager;

impl FhirManager {
    /// Create a FHIR Bundle containing all resources for a patient
    pub fn create_patient_bundle(patient: &Patient, resources: Vec<Value>) -> Result<FhirBundle> {
        let mut bundle_entries = vec![
            json!({
                "resource": patient.fhir_patient
            })
        ];

        // Add all other resources
        for resource in resources {
            bundle_entries.push(json!({
                "resource": resource
            }));
        }

        let bundle = json!({
            "resourceType": "Bundle",
            "id": Uuid::new_v4().to_string(),
            "type": "document",
            "timestamp": Utc::now().to_rfc3339(),
            "entry": bundle_entries
        });

        Ok(FhirBundle {
            id: None,
            patient_did: patient.did.clone(),
            bundle,
            version: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    // /// Create a FHIR Patient resource
    // pub fn create_patient_resource(
    //     _did: &str,
    //     identifier: Vec<FhirIdentifier>,
    //     name: Vec<FhirHumanName>,
    //     gender: &str,
    //     birth_date: &str,
    //     address: Vec<FhirAddress>,
    //     telecom: Vec<FhirContactPoint>,
    // ) -> FhirPatient {
    //     FhirPatient {
    //         resource_type: "Patient".to_string(),
    //         id: Uuid::new_v4().to_string(),
    //         identifier,
    //         name,
    //         gender: gender.to_string(),
    //         birth_date: birth_date.to_string(),
    //         address,
    //         telecom,
    //     }
    // }

    // /// Create a FHIR Practitioner resource
    // pub fn create_practitioner_resource(
    //     _did: &str,
    //     identifier: Vec<FhirIdentifier>,
    //     name: Vec<FhirHumanName>,
    //     qualification: Vec<FhirPractitionerQualification>,
    //     telecom: Vec<FhirContactPoint>,
    // ) -> FhirPractitioner {
    //     FhirPractitioner {
    //         resource_type: "Practitioner".to_string(),
    //         id: Uuid::new_v4().to_string(),
    //         identifier,
    //         name,
    //         qualification,
    //         telecom,
    //     }
    // }

    // /// Create a FHIR MedicationRequest (prescription)
    // pub fn create_medication_request(
    //     patient_did: &str,
    //     practitioner_did: &str,
    //     medication_code: FhirCodeableConcept,
    //     dosage_instructions: Vec<FhirDosageInstruction>,
    // ) -> FhirMedicationRequest {
    //     FhirMedicationRequest {
    //         resource_type: "MedicationRequest".to_string(),
    //         id: Uuid::new_v4().to_string(),
    //         status: "active".to_string(),
    //         intent: "order".to_string(),
    //         medication_codeable_concept: medication_code,
    //         subject: FhirReference {
    //             reference: format!("Patient/{}", patient_did),
    //             display: None,
    //         },
    //         authored_on: Utc::now().to_rfc3339(),
    //         requester: FhirReference {
    //             reference: format!("Practitioner/{}", practitioner_did),
    //             display: None,
    //         },
    //         dosage_instruction: dosage_instructions,
    //     }
    // }

    // /// Create a FHIR Encounter
    // pub fn create_encounter(
    //     patient_did: &str,
    //     practitioner_did: &str,
    //     encounter_class: FhirCoding,
    //     reason_codes: Vec<FhirCodeableConcept>,
    //     start_time: &str,
    //     end_time: Option<&str>,
    // ) -> FhirEncounter {
    //     FhirEncounter {
    //         resource_type: "Encounter".to_string(),
    //         id: Uuid::new_v4().to_string(),
    //         status: "finished".to_string(),
    //         class: encounter_class,
    //         subject: FhirReference {
    //             reference: format!("Patient/{}", patient_did),
    //             display: None,
    //         },
    //         participant: vec![FhirEncounterParticipant {
    //             participant_type: vec![FhirCodeableConcept {
    //                 coding: vec![FhirCoding {
    //                     system: Some("http://terminology.hl7.org/CodeSystem/v3-ParticipationType".to_string()),
    //                     code: Some("PPRF".to_string()),
    //                     display: Some("Primary Performer".to_string()),
    //                 }],
    //                 text: None,
    //             }],
    //             individual: Some(FhirReference {
    //                 reference: format!("Practitioner/{}", practitioner_did),
    //                 display: None,
    //             }),
    //         }],
    //         period: FhirPeriod {
    //             start: Some(start_time.to_string()),
    //             end: end_time.map(|s| s.to_string()),
    //         },
    //         reason_code: reason_codes,
    //     }
    // }

    // /// Create a FHIR Observation
    // pub fn create_observation(
    //     patient_did: &str,
    //     observation_code: FhirCodeableConcept,
    //     category: Vec<FhirCodeableConcept>,
    //     value_quantity: Option<FhirQuantity>,
    //     value_string: Option<String>,
    //     interpretation: Vec<FhirCodeableConcept>,
    //     effective_time: &str,
    // ) -> FhirObservation {
    //     FhirObservation {
    //         resource_type: "Observation".to_string(),
    //         id: Uuid::new_v4().to_string(),
    //         status: "final".to_string(),
    //         category,
    //         code: observation_code,
    //         subject: FhirReference {
    //             reference: format!("Patient/{}", patient_did),
    //             display: None,
    //         },
    //         effective_date_time: effective_time.to_string(),
    //         value_quantity,
    //         value_string,
    //         interpretation,
    //     }
    // }

    // /// Create a FHIR Condition
    // pub fn create_condition(
    //     patient_did: &str,
    //     condition_code: FhirCodeableConcept,
    //     category: Vec<FhirCodeableConcept>,
    //     onset_time: &str,
    //     recorded_time: &str,
    // ) -> FhirCondition {
    //     FhirCondition {
    //         resource_type: "Condition".to_string(),
    //         id: Uuid::new_v4().to_string(),
    //         clinical_status: FhirCodeableConcept {
    //             coding: vec![FhirCoding {
    //                 system: Some("http://terminology.hl7.org/CodeSystem/condition-clinical".to_string()),
    //                 code: Some("active".to_string()),
    //                 display: Some("Active".to_string()),
    //             }],
    //             text: None,
    //         },
    //         verification_status: FhirCodeableConcept {
    //             coding: vec![FhirCoding {
    //                 system: Some("http://terminology.hl7.org/CodeSystem/condition-ver-status".to_string()),
    //                 code: Some("confirmed".to_string()),
    //                 display: Some("Confirmed".to_string()),
    //             }],
    //             text: None,
    //         },
    //         category,
    //         code: condition_code,
    //         subject: FhirReference {
    //             reference: format!("Patient/{}", patient_did),
    //             display: None,
    //         },
    //         onset_date_time: onset_time.to_string(),
    //         recorded_date: recorded_time.to_string(),
    //     }
    // }

    // /// Validate FHIR resource against basic FHIR R4 rules
    // pub fn validate_resource(_resource: &Value) -> Result<()> {
    //     // Check for required fields
    //     // if !resource.is_object() {
    //     //     return Err(anyhow::anyhow!("Resource must be a JSON object"));
    //     // }

    //     // let obj = resource.as_object().unwrap();
        
    //     // if !obj.contains_key("resourceType") {
    //     //     return Err(anyhow::anyhow!("Resource must have 'resourceType' field"));
    //     // }

    //     // if !obj.contains_key("id") {
    //     //     return Err(anyhow::anyhow!("Resource must have 'id' field"));
    //     // }

    //     // Additional validation can be added here
    //     // For now, we'll do basic structure validation
    //     Ok(())
    // }

    // /// Convert FHIR resource to JSON string
    // pub fn resource_to_json(resource: &Value) -> Result<String> {
    //     Ok(serde_json::to_string_pretty(resource)?)
    // }

    // /// Parse FHIR resource from JSON string
    // pub fn resource_from_json(json_str: &str) -> Result<Value> {
    //     Ok(serde_json::from_str(json_str)?)
    // }
}

// Common FHIR code systems and values
// pub struct FhirCodeSystems;

// impl FhirCodeSystems {
//     pub fn loinc() -> &'static str {
//         "http://loinc.org"
//     }

//     pub fn snomed() -> &'static str {
//         "http://snomed.info/sct"
//     }

//     pub fn icd10() -> &'static str {
//         "http://hl7.org/fhir/sid/icd-10-cm"
//     }

//     pub fn rxnorm() -> &'static str {
//         "http://www.nlm.nih.gov/research/umls/rxnorm"
//     }

//     pub fn npi() -> &'static str {
//         "http://hl7.org/fhir/sid/us-npi"
//     }

//     pub fn ssn() -> &'static str {
//         "http://hl7.org/fhir/sid/us-ssn"
//     }
// }

// // Common medication codes
// pub struct MedicationCodes;

// impl MedicationCodes {
//     pub fn aspirin() -> FhirCodeableConcept {
//         FhirCodeableConcept {
//             coding: vec![FhirCoding {
//                 system: Some(FhirCodeSystems::rxnorm().to_string()),
//                 code: Some("1191".to_string()),
//                 display: Some("Aspirin".to_string()),
//             }],
//             text: Some("Aspirin".to_string()),
//         }
//     }

//     pub fn metformin() -> FhirCodeableConcept {
//         FhirCodeableConcept {
//             coding: vec![FhirCoding {
//                 system: Some(FhirCodeSystems::rxnorm().to_string()),
//                 code: Some("6809".to_string()),
//                 display: Some("Metformin".to_string()),
//             }],
//             text: Some("Metformin".to_string()),
//         }
//     }
// }

// // Common observation codes
// pub struct ObservationCodes;

// impl ObservationCodes {
//     pub fn blood_pressure() -> FhirCodeableConcept {
//         FhirCodeableConcept {
//             coding: vec![FhirCoding {
//                 system: Some(FhirCodeSystems::loinc().to_string()),
//                 code: Some("85354-9".to_string()),
//                 display: Some("Blood pressure panel".to_string()),
//             }],
//             text: Some("Blood Pressure".to_string()),
//         }
//     }

//     pub fn body_temperature() -> FhirCodeableConcept {
//         FhirCodeableConcept {
//             coding: vec![FhirCoding {
//                 system: Some(FhirCodeSystems::loinc().to_string()),
//                 code: Some("8310-5".to_string()),
//                 display: Some("Body temperature".to_string()),
//             }],
//             text: Some("Body Temperature".to_string()),
//         }
//     }

//     pub fn heart_rate() -> FhirCodeableConcept {
//         FhirCodeableConcept {
//             coding: vec![FhirCoding {
//                 system: Some(FhirCodeSystems::loinc().to_string()),
//                 code: Some("8867-4".to_string()),
//                 display: Some("Heart rate".to_string()),
//             }],
//             text: Some("Heart Rate".to_string()),
//         }
//     }
// }

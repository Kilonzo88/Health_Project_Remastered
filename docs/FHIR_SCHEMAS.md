# FHIR R4 Schemas and Data Models

## Overview

The Healthcare dApp implements FHIR R4 (Fast Healthcare Interoperability Resources) specification for healthcare data exchange. This document outlines the FHIR resources used and their schemas.

## Core FHIR Resources

### Patient Resource

The Patient resource represents demographic and administrative information about a person receiving care.

```json
{
  "resourceType": "Patient",
  "id": "patient-123",
  "identifier": [
    {
      "use": "usual",
      "type": {
        "coding": [
          {
            "system": "http://hl7.org/fhir/sid/us-ssn",
            "code": "SS",
            "display": "Social Security Number"
          }
        ]
      },
      "system": "http://hl7.org/fhir/sid/us-ssn",
      "value": "123-45-6789"
    }
  ],
  "name": [
    {
      "use": "official",
      "family": "Doe",
      "given": ["John", "Michael"],
      "prefix": ["Mr."],
      "suffix": ["Jr."]
    }
  ],
  "gender": "male",
  "birthDate": "1990-01-15",
  "address": [
    {
      "use": "home",
      "line": ["123 Main Street", "Apt 4B"],
      "city": "Anytown",
      "state": "CA",
      "postalCode": "12345",
      "country": "US"
    }
  ],
  "telecom": [
    {
      "system": "phone",
      "value": "+1-555-123-4567",
      "use": "mobile"
    },
    {
      "system": "email",
      "value": "john.doe@example.com",
      "use": "home"
    }
  ],
  "maritalStatus": {
    "coding": [
      {
        "system": "http://terminology.hl7.org/CodeSystem/v3-MaritalStatus",
        "code": "M",
        "display": "Married"
      }
    ]
  }
}
```

### Practitioner Resource

The Practitioner resource represents a person who is directly or indirectly involved in the provisioning of healthcare.

```json
{
  "resourceType": "Practitioner",
  "id": "practitioner-123",
  "identifier": [
    {
      "use": "official",
      "type": {
        "coding": [
          {
            "system": "http://terminology.hl7.org/CodeSystem/v2-0203",
            "code": "NPI",
            "display": "National Provider Identifier"
          }
        ]
      },
      "system": "http://hl7.org/fhir/sid/us-npi",
      "value": "1234567890"
    }
  ],
  "name": [
    {
      "use": "official",
      "family": "Smith",
      "given": ["Jane", "Elizabeth"],
      "prefix": ["Dr."],
      "suffix": ["MD"]
    }
  ],
  "qualification": [
    {
      "identifier": [
        {
          "system": "http://hl7.org/fhir/sid/us-npi",
          "value": "1234567890"
        }
      ],
      "code": {
        "coding": [
          {
            "system": "http://terminology.hl7.org/CodeSystem/v2-0360",
            "code": "MD",
            "display": "Doctor of Medicine"
          }
        ],
        "text": "Doctor of Medicine"
      },
      "period": {
        "start": "2020-01-15",
        "end": "2025-01-15"
      },
      "issuer": {
        "display": "California Medical Board"
      }
    }
  ],
  "telecom": [
    {
      "system": "phone",
      "value": "+1-555-987-6543",
      "use": "work"
    },
    {
      "system": "email",
      "value": "jane.smith@hospital.com",
      "use": "work"
    }
  ]
}
```

### MedicationRequest Resource

The MedicationRequest resource represents an order or request for both supply of the medication and the instructions for administration of the medication to a patient.

```json
{
  "resourceType": "MedicationRequest",
  "id": "med-request-123",
  "status": "active",
  "intent": "order",
  "medicationCodeableConcept": {
    "coding": [
      {
        "system": "http://www.nlm.nih.gov/research/umls/rxnorm",
        "code": "6809",
        "display": "Metformin"
      }
    ],
    "text": "Metformin 500mg"
  },
  "subject": {
    "reference": "Patient/patient-123",
    "display": "John Doe"
  },
  "authoredOn": "2023-10-15T10:30:00Z",
  "requester": {
    "reference": "Practitioner/practitioner-123",
    "display": "Dr. Jane Smith"
  },
  "dosageInstruction": [
    {
      "text": "Take 1 tablet twice daily with meals",
      "timing": {
        "repeat": {
          "frequency": 2,
          "period": 1,
          "periodUnit": "d"
        }
      },
      "doseAndRate": [
        {
          "type": {
            "coding": [
              {
                "system": "http://terminology.hl7.org/CodeSystem/dose-rate-type",
                "code": "calculated",
                "display": "Calculated"
              }
            ]
          },
          "doseQuantity": {
            "value": 1,
            "unit": "tablet",
            "system": "http://unitsofmeasure.org",
            "code": "tab"
          }
        }
      ]
    }
  ]
}
```

### Encounter Resource

The Encounter resource represents an interaction between a patient and healthcare provider(s) for the purpose of providing healthcare service(s) or assessing the health status of a patient.

```json
{
  "resourceType": "Encounter",
  "id": "encounter-123",
  "status": "finished",
  "class": {
    "system": "http://terminology.hl7.org/CodeSystem/v3-ActCode",
    "code": "AMB",
    "display": "ambulatory"
  },
  "subject": {
    "reference": "Patient/patient-123",
    "display": "John Doe"
  },
  "participant": [
    {
      "type": [
        {
          "coding": [
            {
              "system": "http://terminology.hl7.org/CodeSystem/v3-ParticipationType",
              "code": "PPRF",
              "display": "Primary Performer"
            }
          ]
        }
      ],
      "individual": {
        "reference": "Practitioner/practitioner-123",
        "display": "Dr. Jane Smith"
      }
    }
  ],
  "period": {
    "start": "2023-10-15T09:00:00Z",
    "end": "2023-10-15T10:00:00Z"
  },
  "reasonCode": [
    {
      "coding": [
        {
          "system": "http://snomed.info/sct",
          "code": "44054006",
          "display": "Diabetes mellitus"
        }
      ]
    }
  ]
}
```

### Observation Resource

The Observation resource represents a single observation or measurement made about a patient, device, or other subject.

```json
{
  "resourceType": "Observation",
  "id": "observation-123",
  "status": "final",
  "category": [
    {
      "coding": [
        {
          "system": "http://terminology.hl7.org/CodeSystem/observation-category",
          "code": "vital-signs",
          "display": "Vital Signs"
        }
      ]
    }
  ],
  "code": {
    "coding": [
      {
        "system": "http://loinc.org",
        "code": "8310-5",
        "display": "Body temperature"
      }
    ],
    "text": "Body Temperature"
  },
  "subject": {
    "reference": "Patient/patient-123",
    "display": "John Doe"
  },
  "effectiveDateTime": "2023-10-15T10:30:00Z",
  "valueQuantity": {
    "value": 98.6,
    "unit": "°F",
    "system": "http://unitsofmeasure.org",
    "code": "[degF]"
  },
  "interpretation": [
    {
      "coding": [
        {
          "system": "http://terminology.hl7.org/CodeSystem/v3-ObservationInterpretation",
          "code": "N",
          "display": "Normal"
        }
      ]
    }
  ]
}
```

### Condition Resource

The Condition resource represents a clinical condition, problem, diagnosis, or other event, situation, issue, or clinical concept that has risen to a level of concern.

```json
{
  "resourceType": "Condition",
  "id": "condition-123",
  "clinicalStatus": {
    "coding": [
      {
        "system": "http://terminology.hl7.org/CodeSystem/condition-clinical",
        "code": "active",
        "display": "Active"
      }
    ]
  },
  "verificationStatus": {
    "coding": [
      {
        "system": "http://terminology.hl7.org/CodeSystem/condition-ver-status",
        "code": "confirmed",
        "display": "Confirmed"
      }
    ]
  },
  "category": [
    {
      "coding": [
        {
          "system": "http://terminology.hl7.org/CodeSystem/condition-category",
          "code": "encounter-diagnosis",
          "display": "Encounter Diagnosis"
        }
      ]
    }
  ],
  "code": {
    "coding": [
      {
        "system": "http://snomed.info/sct",
        "code": "44054006",
        "display": "Diabetes mellitus"
      }
    ],
    "text": "Type 2 Diabetes"
  },
  "subject": {
    "reference": "Patient/patient-123",
    "display": "John Doe"
  },
  "onsetDateTime": "2023-01-15T00:00:00Z",
  "recordedDate": "2023-10-15T10:30:00Z"
}
```

## FHIR Bundle Structure

A FHIR Bundle is a container for a collection of resources. In our system, each patient has a Bundle containing all their health records.

```json
{
  "resourceType": "Bundle",
  "id": "bundle-123",
  "type": "document",
  "timestamp": "2023-10-15T10:30:00Z",
  "entry": [
    {
      "resource": {
        "resourceType": "Patient",
        "id": "patient-123",
        // ... Patient resource content
      }
    },
    {
      "resource": {
        "resourceType": "MedicationRequest",
        "id": "med-request-123",
        // ... MedicationRequest resource content
      }
    },
    {
      "resource": {
        "resourceType": "Observation",
        "id": "observation-123",
        // ... Observation resource content
      }
    }
  ]
}
```

## Code Systems and Terminologies

### LOINC (Logical Observation Identifiers Names and Codes)
Used for laboratory and clinical observations:
- `8310-5`: Body temperature
- `8867-4`: Heart rate
- `85354-9`: Blood pressure panel

### SNOMED CT (Systematized Nomenclature of Medicine Clinical Terms)
Used for clinical terms and diagnoses:
- `44054006`: Diabetes mellitus
- `38341003`: Hypertension
- `195967001`: Asthma

### RxNorm
Used for medication names and codes:
- `6809`: Metformin
- `1191`: Aspirin
- `7980`: Lisinopril

### ICD-10-CM
Used for diagnosis codes:
- `E11.9`: Type 2 diabetes mellitus without complications
- `I10`: Essential hypertension
- `J45.9`: Unspecified asthma

## Data Validation Rules

### Required Fields
- All resources must have `resourceType` and `id`
- Patient must have at least one `name` and `gender`
- Practitioner must have at least one `name` and `qualification`
- MedicationRequest must have `medicationCodeableConcept` and `subject`

### Format Validation
- Dates must be in ISO 8601 format
- DIDs must follow the `did:key:` format
- Phone numbers must be in E.164 format
- Email addresses must be valid email format

### Business Rules
- Patient age must be calculated from `birthDate`
- Practitioner license must not be expired
- Prescription status must be valid enum value
- Access permissions must be granted by patient

## Security Considerations

### Data Encryption
- All FHIR resources are encrypted at rest
- Sensitive fields are encrypted with AES-256
- Keys are managed through Hedera Key Management

### Access Control
- Patient data access is controlled by smart contracts
- Permissions are enforced at the API level
- All access is logged and auditable

### Privacy
- Patient consent is required for data sharing
- Data minimization principles are applied
- Right to be forgotten is supported

## Interoperability

### HL7 FHIR R4 Compliance
- Full compliance with FHIR R4 specification
- Support for all required and recommended elements
- Proper use of terminology systems

### Data Exchange
- Support for JSON and XML formats
- RESTful API following FHIR conventions
- Bulk data export capabilities

### Integration
- Standard FHIR endpoints
- OAuth 2.0 authentication
- SMART on FHIR app launch

## Testing and Validation

### FHIR Validation
- All resources are validated against FHIR schemas
- Terminology validation using terminology servers
- Business rule validation

### Data Quality
- Completeness checks
- Consistency validation
- Accuracy verification

### Performance
- Resource bundle optimization
- Caching strategies
- Query performance monitoring

// MongoDB initialization script for Healthcare dApp
// This script sets up the initial database structure and indexes

// Switch to the healthcare database
db = db.getSiblingDB('healthcare');

// Create collections with validation rules
db.createCollection('patients', {
  validator: {
    $jsonSchema: {
      bsonType: 'object',
      required: ['did', 'fhirPatient', 'createdAt', 'updatedAt'],
      properties: {
        did: {
          bsonType: 'string',
          pattern: '^did:key:.*',
          description: 'DID must be a valid did:key format'
        },
        fhirPatient: {
          bsonType: 'object',
          required: ['resourceType', 'id', 'name', 'gender'],
          properties: {
            resourceType: {
              bsonType: 'string',
              enum: ['Patient'],
              description: 'resourceType must be Patient'
            },
            id: {
              bsonType: 'string',
              description: 'Patient ID is required'
            },
            name: {
              bsonType: 'array',
              minItems: 1,
              description: 'At least one name is required'
            },
            gender: {
              bsonType: 'string',
              enum: ['male', 'female', 'other', 'unknown'],
              description: 'Gender must be a valid value'
            }
          }
        },
        createdAt: {
          bsonType: 'date',
          description: 'createdAt must be a date'
        },
        updatedAt: {
          bsonType: 'date',
          description: 'updatedAt must be a date'
        }
      }
    }
  }
});

db.createCollection('practitioners', {
  validator: {
    $jsonSchema: {
      bsonType: 'object',
      required: ['did', 'fhirPractitioner', 'licenseVerification', 'createdAt', 'updatedAt'],
      properties: {
        did: {
          bsonType: 'string',
          pattern: '^did:key:.*',
          description: 'DID must be a valid did:key format'
        },
        fhirPractitioner: {
          bsonType: 'object',
          required: ['resourceType', 'id', 'name', 'qualification'],
          properties: {
            resourceType: {
              bsonType: 'string',
              enum: ['Practitioner'],
              description: 'resourceType must be Practitioner'
            },
            id: {
              bsonType: 'string',
              description: 'Practitioner ID is required'
            },
            name: {
              bsonType: 'array',
              minItems: 1,
              description: 'At least one name is required'
            },
            qualification: {
              bsonType: 'array',
              minItems: 1,
              description: 'At least one qualification is required'
            }
          }
        },
        licenseVerification: {
          bsonType: 'object',
          required: ['licenseNumber', 'issuingAuthority', 'issueDate', 'expiryDate', 'verified'],
          properties: {
            licenseNumber: {
              bsonType: 'string',
              description: 'License number is required'
            },
            issuingAuthority: {
              bsonType: 'string',
              description: 'Issuing authority is required'
            },
            issueDate: {
              bsonType: 'string',
              description: 'Issue date is required'
            },
            expiryDate: {
              bsonType: 'string',
              description: 'Expiry date is required'
            },
            verified: {
              bsonType: 'bool',
              description: 'Verification status is required'
            }
          }
        },
        createdAt: {
          bsonType: 'date',
          description: 'createdAt must be a date'
        },
        updatedAt: {
          bsonType: 'date',
          description: 'updatedAt must be a date'
        }
      }
    }
  }
});

db.createCollection('prescriptions', {
  validator: {
    $jsonSchema: {
      bsonType: 'object',
      required: ['patientDid', 'practitionerDid', 'fhirMedicationRequest', 'createdAt', 'updatedAt'],
      properties: {
        patientDid: {
          bsonType: 'string',
          pattern: '^did:key:.*',
          description: 'Patient DID must be a valid did:key format'
        },
        practitionerDid: {
          bsonType: 'string',
          pattern: '^did:key:.*',
          description: 'Practitioner DID must be a valid did:key format'
        },
        fhirMedicationRequest: {
          bsonType: 'object',
          required: ['resourceType', 'id', 'status', 'intent', 'medicationCodeableConcept', 'subject'],
          properties: {
            resourceType: {
              bsonType: 'string',
              enum: ['MedicationRequest'],
              description: 'resourceType must be MedicationRequest'
            },
            id: {
              bsonType: 'string',
              description: 'MedicationRequest ID is required'
            },
            status: {
              bsonType: 'string',
              enum: ['draft', 'active', 'on-hold', 'cancelled', 'completed', 'entered-in-error', 'stopped', 'unknown'],
              description: 'Status must be a valid MedicationRequest status'
            },
            intent: {
              bsonType: 'string',
              enum: ['proposal', 'plan', 'order', 'original-order', 'reflex-order', 'filler-order', 'instance-order', 'option'],
              description: 'Intent must be a valid MedicationRequest intent'
            }
          }
        },
        createdAt: {
          bsonType: 'date',
          description: 'createdAt must be a date'
        },
        updatedAt: {
          bsonType: 'date',
          description: 'updatedAt must be a date'
        }
      }
    }
  }
});

db.createCollection('access_controls', {
  validator: {
    $jsonSchema: {
      bsonType: 'object',
      required: ['patientDid', 'granteeDid', 'permissions', 'active', 'createdAt'],
      properties: {
        patientDid: {
          bsonType: 'string',
          pattern: '^did:key:.*',
          description: 'Patient DID must be a valid did:key format'
        },
        granteeDid: {
          bsonType: 'string',
          pattern: '^did:key:.*',
          description: 'Grantee DID must be a valid did:key format'
        },
        permissions: {
          bsonType: 'array',
          items: {
            bsonType: 'string',
            enum: ['READ', 'WRITE', 'PRESCRIBE', 'VIEW_PRESCRIPTIONS', 'VIEW_ENCOUNTERS', 'VIEW_OBSERVATIONS']
          },
          description: 'Permissions must be valid enum values'
        },
        active: {
          bsonType: 'bool',
          description: 'Active status is required'
        },
        createdAt: {
          bsonType: 'date',
          description: 'createdAt must be a date'
        },
        expiresAt: {
          bsonType: 'date',
          description: 'expiresAt must be a date if provided'
        }
      }
    }
  }
});

db.createCollection('fhir_bundles', {
  validator: {
    $jsonSchema: {
      bsonType: 'object',
      required: ['patientDid', 'bundle', 'version', 'createdAt', 'updatedAt'],
      properties: {
        patientDid: {
          bsonType: 'string',
          pattern: '^did:key:.*',
          description: 'Patient DID must be a valid did:key format'
        },
        bundle: {
          bsonType: 'object',
          required: ['resourceType', 'id', 'type', 'timestamp', 'entry'],
          properties: {
            resourceType: {
              bsonType: 'string',
              enum: ['Bundle'],
              description: 'resourceType must be Bundle'
            },
            id: {
              bsonType: 'string',
              description: 'Bundle ID is required'
            },
            type: {
              bsonType: 'string',
              enum: ['document', 'message', 'transaction', 'transaction-response', 'batch', 'batch-response', 'history', 'searchset', 'collection'],
              description: 'Bundle type must be valid'
            },
            timestamp: {
              bsonType: 'string',
              description: 'Bundle timestamp is required'
            },
            entry: {
              bsonType: 'array',
              description: 'Bundle entries are required'
            }
          }
        },
        version: {
          bsonType: 'int',
          minimum: 1,
          description: 'Version must be a positive integer'
        },
        createdAt: {
          bsonType: 'date',
          description: 'createdAt must be a date'
        },
        updatedAt: {
          bsonType: 'date',
          description: 'updatedAt must be a date'
        }
      }
    }
  }
});

// Create indexes for better performance
print('Creating indexes...');

// Patient indexes
db.patients.createIndex({ "did": 1 }, { unique: true });
db.patients.createIndex({ "fhirPatient.identifier.value": 1 });
db.patients.createIndex({ "createdAt": 1 });
db.patients.createIndex({ "updatedAt": 1 });

// Practitioner indexes
db.practitioners.createIndex({ "did": 1 }, { unique: true });
db.practitioners.createIndex({ "fhirPractitioner.identifier.value": 1 });
db.practitioners.createIndex({ "licenseVerification.licenseNumber": 1 });
db.practitioners.createIndex({ "licenseVerification.verified": 1 });
db.practitioners.createIndex({ "createdAt": 1 });

// Prescription indexes
db.prescriptions.createIndex({ "patientDid": 1 });
db.prescriptions.createIndex({ "practitionerDid": 1 });
db.prescriptions.createIndex({ "fhirMedicationRequest.status": 1 });
db.prescriptions.createIndex({ "createdAt": 1 });
db.prescriptions.createIndex({ "patientDid": 1, "createdAt": -1 });

// Access control indexes
db.access_controls.createIndex({ "patientDid": 1, "granteeDid": 1 }, { unique: true });
db.access_controls.createIndex({ "patientDid": 1 });
db.access_controls.createIndex({ "granteeDid": 1 });
db.access_controls.createIndex({ "active": 1 });
db.access_controls.createIndex({ "expiresAt": 1 });
db.access_controls.createIndex({ "createdAt": 1 });

// FHIR bundle indexes
db.fhir_bundles.createIndex({ "patientDid": 1 }, { unique: true });
db.fhir_bundles.createIndex({ "version": 1 });
db.fhir_bundles.createIndex({ "createdAt": 1 });
db.fhir_bundles.createIndex({ "updatedAt": 1 });

// Create text indexes for search
db.patients.createIndex({
  "fhirPatient.name.family": "text",
  "fhirPatient.name.given": "text",
  "fhirPatient.identifier.value": "text"
});

db.practitioners.createIndex({
  "fhirPractitioner.name.family": "text",
  "fhirPractitioner.name.given": "text",
  "fhirPractitioner.identifier.value": "text",
  "licenseVerification.licenseNumber": "text"
});

db.prescriptions.createIndex({
  "fhirMedicationRequest.medicationCodeableConcept.text": "text"
});

print('Database initialization completed successfully!');
print('Collections created: patients, practitioners, prescriptions, access_controls, fhir_bundles');
print('Indexes created for optimal performance');
print('Validation rules applied to all collections');

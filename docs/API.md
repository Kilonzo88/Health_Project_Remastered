# Healthcare dApp API Documentation

## Base URL
```
http://localhost:3000
```

## Authentication
The API uses JWT tokens for authentication. Include the token in the Authorization header:
```
Authorization: Bearer <your_jwt_token>
```

## Response Format
All API responses follow this format:
```json
{
  "success": true,
  "data": { ... },
  "error": null,
  "timestamp": "2023-10-15T10:30:00Z"
}
```

## Error Handling
Errors are returned with appropriate HTTP status codes:
- `400` - Bad Request
- `401` - Unauthorized
- `403` - Forbidden
- `404` - Not Found
- `500` - Internal Server Error

## Endpoints

### Health Check

#### GET /health
Check if the API is running.

**Response:**
```json
{
  "status": "healthy",
  "timestamp": "2023-10-15T10:30:00Z"
}
```

### Patient Management

#### POST /api/patients
Create a new patient.

**Request Body:**
```json
{
  "fhirPatient": {
    "resourceType": "Patient",
    "id": "patient-123",
    "identifier": [
      {
        "system": "http://hl7.org/fhir/sid/us-ssn",
        "value": "123-45-6789"
      }
    ],
    "name": [
      {
        "given": ["John"],
        "family": "Doe",
        "use": "official"
      }
    ],
    "gender": "male",
    "birthDate": "1990-01-15",
    "address": [
      {
        "line": ["123 Main St"],
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
    ]
  }
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "id": "64f8a1b2c3d4e5f6a7b8c9d0",
    "did": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    "fhirPatient": { ... },
    "createdAt": "2023-10-15T10:30:00Z",
    "updatedAt": "2023-10-15T10:30:00Z"
  },
  "error": null,
  "timestamp": "2023-10-15T10:30:00Z"
}
```

#### GET /api/patients/:id
Get a patient by DID.

**Parameters:**
- `id` (string): Patient DID

**Response:**
```json
{
  "success": true,
  "data": {
    "id": "64f8a1b2c3d4e5f6a7b8c9d0",
    "did": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    "fhirPatient": { ... },
    "createdAt": "2023-10-15T10:30:00Z",
    "updatedAt": "2023-10-15T10:30:00Z"
  },
  "error": null,
  "timestamp": "2023-10-15T10:30:00Z"
}
```

#### GET /api/patients/:id/records
Get a patient's FHIR bundle.

**Parameters:**
- `id` (string): Patient DID

**Response:**
```json
{
  "success": true,
  "data": {
    "id": "64f8a1b2c3d4e5f6a7b8c9d0",
    "patientDid": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    "bundle": {
      "resourceType": "Bundle",
      "id": "bundle-123",
      "type": "document",
      "timestamp": "2023-10-15T10:30:00Z",
      "entry": [
        {
          "resource": { ... }
        }
      ]
    },
    "version": 1,
    "createdAt": "2023-10-15T10:30:00Z",
    "updatedAt": "2023-10-15T10:30:00Z"
  },
  "error": null,
  "timestamp": "2023-10-15T10:30:00Z"
}
```

#### POST /api/patients/:id/access
Grant access to a patient's data.

**Parameters:**
- `id` (string): Patient DID

**Request Body:**
```json
{
  "granteeDid": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "permissions": ["READ", "WRITE", "VIEW_PRESCRIPTIONS"],
  "expiresAt": "2024-10-15T10:30:00Z"
}
```

**Response:**
```json
{
  "success": true,
  "data": null,
  "error": null,
  "timestamp": "2023-10-15T10:30:00Z"
}
```

### Practitioner Management

#### POST /api/practitioners
Create a new practitioner.

**Request Body:**
```json
{
  "fhirPractitioner": {
    "resourceType": "Practitioner",
    "id": "practitioner-123",
    "identifier": [
      {
        "system": "http://hl7.org/fhir/sid/us-npi",
        "value": "1234567890"
      }
    ],
    "name": [
      {
        "given": ["Dr. Jane"],
        "family": "Smith",
        "prefix": ["Dr."],
        "use": "official"
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
  },
  "licenseVerification": {
    "licenseNumber": "MD123456",
    "issuingAuthority": "California Medical Board",
    "issueDate": "2020-01-15",
    "expiryDate": "2025-01-15",
    "hederaTransactionId": "0.0.123456@1640995200.123456789",
    "ipfsHash": "QmXxXxXxXxXxXxXxXxXxXxXxXxXxXxXxXxXxXxXxXxXxXxXxXx",
    "verified": true
  }
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "id": "64f8a1b2c3d4e5f6a7b8c9d0",
    "did": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    "fhirPractitioner": { ... },
    "licenseVerification": { ... },
    "createdAt": "2023-10-15T10:30:00Z",
    "updatedAt": "2023-10-15T10:30:00Z"
  },
  "error": null,
  "timestamp": "2023-10-15T10:30:00Z"
}
```

#### GET /api/practitioners/:id
Get a practitioner by DID.

**Parameters:**
- `id` (string): Practitioner DID

**Response:**
```json
{
  "success": true,
  "data": {
    "id": "64f8a1b2c3d4e5f6a7b8c9d0",
    "did": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    "fhirPractitioner": { ... },
    "licenseVerification": { ... },
    "createdAt": "2023-10-15T10:30:00Z",
    "updatedAt": "2023-10-15T10:30:00Z"
  },
  "error": null,
  "timestamp": "2023-10-15T10:30:00Z"
}
```

#### GET /api/practitioners/:id/verify
Verify a practitioner's credentials.

**Parameters:**
- `id` (string): Practitioner DID

**Response:**
```json
{
  "success": true,
  "data": true,
  "error": null,
  "timestamp": "2023-10-15T10:30:00Z"
}
```

### Prescription Management

#### POST /api/prescriptions
Create a new prescription.

**Request Body:**
```json
{
  "patientDid": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "medicationRequest": {
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
        "doseAndRate": [
          {
            "doseQuantity": {
              "value": 1,
              "unit": "tablet"
            }
          }
        ]
      }
    ]
  }
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "id": "64f8a1b2c3d4e5f6a7b8c9d0",
    "patientDid": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    "practitionerDid": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    "fhirMedicationRequest": { ... },
    "createdAt": "2023-10-15T10:30:00Z",
    "updatedAt": "2023-10-15T10:30:00Z"
  },
  "error": null,
  "timestamp": "2023-10-15T10:30:00Z"
}
```

#### GET /api/prescriptions/:id
Get a prescription by ID.

**Parameters:**
- `id` (string): Prescription ID

**Response:**
```json
{
  "success": true,
  "data": {
    "id": "64f8a1b2c3d4e5f6a7b8c9d0",
    "patientDid": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    "practitionerDid": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    "fhirMedicationRequest": { ... },
    "createdAt": "2023-10-15T10:30:00Z",
    "updatedAt": "2023-10-15T10:30:00Z"
  },
  "error": null,
  "timestamp": "2023-10-15T10:30:00Z"
}
```

#### GET /api/prescriptions?patient=:did
Get prescriptions for a patient.

**Query Parameters:**
- `patient` (string): Patient DID

**Response:**
```json
{
  "success": true,
  "data": [
    {
      "id": "64f8a1b2c3d4e5f6a7b8c9d0",
      "patientDid": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
      "practitionerDid": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
      "fhirMedicationRequest": { ... },
      "createdAt": "2023-10-15T10:30:00Z",
      "updatedAt": "2023-10-15T10:30:00Z"
    }
  ],
  "error": null,
  "timestamp": "2023-10-15T10:30:00Z"
}
```

### Access Control

#### POST /api/access/grant
Grant access to patient data.

**Request Body:**
```json
{
  "patientDid": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "granteeDid": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "permissions": ["READ", "WRITE", "VIEW_PRESCRIPTIONS"],
  "expiresAt": "2024-10-15T10:30:00Z"
}
```

**Response:**
```json
{
  "success": true,
  "data": null,
  "error": null,
  "timestamp": "2023-10-15T10:30:00Z"
}
```

#### DELETE /api/access/:patient/:grantee
Revoke access to patient data.

**Parameters:**
- `patient` (string): Patient DID
- `grantee` (string): Grantee DID

**Response:**
```json
{
  "success": true,
  "data": null,
  "error": null,
  "timestamp": "2023-10-15T10:30:00Z"
}
```

#### GET /api/access/:patient/:grantee
Check permissions for a grantee.

**Parameters:**
- `patient` (string): Patient DID
- `grantee` (string): Grantee DID

**Response:**
```json
{
  "success": true,
  "data": ["READ", "WRITE", "VIEW_PRESCRIPTIONS"],
  "error": null,
  "timestamp": "2023-10-15T10:30:00Z"
}
```

## Data Models

### Permission Enum
```typescript
enum Permission {
  READ = "READ",
  WRITE = "WRITE",
  PRESCRIBE = "PRESCRIBE",
  VIEW_PRESCRIPTIONS = "VIEW_PRESCRIPTIONS",
  VIEW_ENCOUNTERS = "VIEW_ENCOUNTERS",
  VIEW_OBSERVATIONS = "VIEW_OBSERVATIONS"
}
```

### User Types
```typescript
enum UserType {
  PATIENT = "PATIENT",
  PRACTITIONER = "PRACTITIONER",
  HOSPITAL = "HOSPITAL"
}
```

## Rate Limiting
The API implements rate limiting to prevent abuse:
- 100 requests per minute per IP
- 1000 requests per hour per authenticated user

## CORS
Cross-Origin Resource Sharing is enabled for:
- `http://localhost:3001` (Frontend)
- `http://localhost:3000` (Backend)

## Webhooks
The API supports webhooks for real-time notifications:
- Patient data access events
- Prescription creation events
- Access control changes
- License verification events

## SDKs
Official SDKs are available for:
- JavaScript/TypeScript
- Python
- Rust
- Go

## Support
For API support and questions:
- Check the troubleshooting guide
- Review the error logs
- Contact the development team

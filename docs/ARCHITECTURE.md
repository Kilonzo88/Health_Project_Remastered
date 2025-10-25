# Healthcare dApp Architecture

## Overview

The Healthcare dApp is a decentralized healthcare records management system built on blockchain technology, featuring immutable credential storage, secure patient data management, and decentralized access control.

## System Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   React Frontend │    │   Rust Backend  │    │   MongoDB       │
│   (Port 3001)   │◄──►│   (Port 3000)   │◄──►│   (Port 27017)  │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                │
                                ▼
                       ┌─────────────────┐
                       │   Hedera        │
                       │   Smart         │
                       │   Contracts     │
                       └─────────────────┘
                                │
                                ▼
                       ┌─────────────────┐
                       │   IPFS          │
                       │   (Port 5001)   │
                       └─────────────────┘
```

## Core Components

### 1. Frontend (React + TypeScript)
- **Location**: `/frontend`
- **Port**: 3001
- **Technology**: React 18, Material-UI, TypeScript
- **Features**:
  - Patient profile management
  - Practitioner credential verification
  - Prescription management
  - Access control interface
  - Real-time notifications

### 2. Backend (Rust + Axum)
- **Location**: `/backend`
- **Port**: 3000
- **Technology**: Rust, Axum web framework, MongoDB driver
- **Features**:
  - RESTful API endpoints
  - FHIR R4 data processing
  - DID management
  - Access control logic
  - Hedera blockchain integration
  - IPFS file storage

### 3. Database (MongoDB)
- **Port**: 27017
- **Collections**:
  - `patients`: Patient records and FHIR data
  - `practitioners`: Practitioner credentials and licenses
  - `prescriptions`: Medication requests and prescriptions
  - `access_controls`: Permission grants and access management
  - `fhir_bundles`: Complete FHIR resource bundles

### 4. Blockchain (Hedera Hashgraph)
- **Network**: Testnet (configurable)
- **Smart Contracts**:
  - `HealthcareAccessControl.sol`: Manages patient data access permissions
  - `VerifiableCredentials.sol`: Stores and verifies medical credentials
- **Features**:
  - Immutable credential storage
  - Decentralized access control
  - Audit trail for all medical interactions

### 5. Storage (IPFS)
- **Port**: 5001 (API), 8080 (Web UI)
- **Purpose**: Off-chain storage for large files and verifiable credentials
- **Features**:
  - Decentralized file storage
  - Content addressing
  - Pinning for persistence

## Data Flow

### Patient Registration
1. Patient creates DID using Rust backend
2. FHIR Patient resource is created and stored in MongoDB
3. Patient DID is registered on Hedera smart contract
4. Initial FHIR bundle is created and stored

### Practitioner Verification
1. Practitioner creates DID and uploads license
2. License is stored on IPFS
3. Verifiable credential is created and stored on Hedera
4. Practitioner record is created in MongoDB

### Prescription Issuance
1. Practitioner authenticates and verifies access to patient
2. FHIR MedicationRequest is created
3. Prescription is stored in MongoDB
4. Patient's FHIR bundle is updated
5. Access is logged on Hedera for audit

### Access Control
1. Patient grants access to practitioner/hospital
2. Permission is stored on Hedera smart contract
3. Access control record is created in MongoDB
4. Real-time access verification for all operations

## Security Features

### 1. Decentralized Identifiers (DIDs)
- Unique identifiers for all entities
- Cryptographic key pairs for authentication
- Self-sovereign identity management

### 2. Access Control
- Granular permissions (READ, WRITE, PRESCRIBE, etc.)
- Time-based access expiration
- Revocable access grants
- Blockchain-enforced permissions

### 3. Data Encryption
- All sensitive data encrypted at rest
- TLS encryption for data in transit
- Cryptographic signatures for data integrity

### 4. Audit Trail
- All operations logged on Hedera
- Immutable transaction history
- Compliance-ready audit logs

## FHIR R4 Compliance

The system implements FHIR R4 specification for healthcare data:

### Core Resources
- **Patient**: Demographics and contact information
- **Practitioner**: Healthcare provider information
- **MedicationRequest**: Prescriptions and medication orders
- **Encounter**: Healthcare visits and interactions
- **Observation**: Clinical measurements and findings
- **Condition**: Diagnoses and health conditions

### Bundle Management
- Complete FHIR bundles for each patient
- Version control for data changes
- Resource references and relationships

## API Endpoints

### Patient Endpoints
- `POST /api/patients` - Create patient
- `GET /api/patients/:id` - Get patient by DID
- `GET /api/patients/:id/records` - Get patient FHIR bundle
- `POST /api/patients/:id/access` - Grant access

### Practitioner Endpoints
- `POST /api/practitioners` - Create practitioner
- `GET /api/practitioners/:id` - Get practitioner by DID
- `GET /api/practitioners/:id/verify` - Verify practitioner credentials

### Prescription Endpoints
- `POST /api/prescriptions` - Create prescription
- `GET /api/prescriptions/:id` - Get prescription
- `GET /api/prescriptions?patient=:did` - Get patient prescriptions

### Access Control Endpoints
- `POST /api/access/grant` - Grant access
- `DELETE /api/access/:patient/:grantee` - Revoke access
- `GET /api/access/:patient/:grantee` - Check permissions

## Deployment

### Development
```bash
# Start all services
docker-compose up -d

# View logs
docker-compose logs -f

# Stop services
docker-compose down
```

### Production
- Use production Hedera mainnet
- Configure proper SSL certificates
- Set up monitoring and logging
- Implement backup strategies
- Configure load balancing

## Monitoring and Maintenance

### Health Checks
- Backend API health endpoint
- Database connectivity monitoring
- Hedera network status
- IPFS node availability

### Logging
- Structured logging with tracing
- Error tracking and alerting
- Performance monitoring
- Security event logging

### Backup Strategy
- MongoDB regular backups
- IPFS content pinning
- Hedera transaction archiving
- Configuration backup

## Future Enhancements

1. **Zero-Knowledge Proofs**: Privacy-preserving queries
2. **Cross-Chain Integration**: Multi-blockchain support
3. **AI/ML Integration**: Clinical decision support
4. **Mobile Applications**: Native mobile apps
5. **Interoperability**: HL7 FHIR R5 support
6. **Compliance**: HIPAA, GDPR compliance tools

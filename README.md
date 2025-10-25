# Decentralized Healthcare Records System (dApp)

A comprehensive healthcare records management system built on a hybrid Web2/Web3 architecture, featuring social logins, decentralized identity, and a robust data lifecycle for clinical records.

## 🏗️ Architecture Overview

The system is designed to provide a seamless user experience by integrating familiar social logins with a powerful decentralized backend. It uses a two-stage data management approach to balance efficiency and verifiability.

### Core Components

1.  **Dart Frontend**: The user interface for patients and practitioners.
2.  **Rust Backend**: The main API server handling business logic, DID creation, and data orchestration.
3.  **Web3Auth**: A third-party service used to link social logins (Google, etc.) to cryptographic key pairs, providing non-custodial key management for users.
4.  **Hedera**: The DLT layer used for creating Decentralized Identifiers (`did:hedera`) and anchoring Verifiable Credentials.
5.  **MongoDB**: The primary database for **live, operational data**. It stores active clinical encounters and individual FHIR resources as they are created and modified.
6.  **IPFS**: The decentralized storage layer for **finalized, immutable records**. At the end of a clinical visit, all related data is bundled, signed, and archived to IPFS.

### Technology Stack

-   **Frontend**: Dart (using a framework like Flutter)
-   **Backend**: Rust with Axum web framework
-   **Identity & Auth**: Web3Auth, `did:hedera`
-   **Database (Live Data)**: MongoDB
-   **Blockchain (Identity & VCs)**: Hedera Hashgraph
-   **Storage (Archived Data)**: IPFS

---
## Architectural Decisions

### Authentication

This project uses a custom, self-hosted Rust backend for all user authentication. This decision was made to ensure full control over user data, maintain data sovereignty (critical for healthcare applications), and avoid vendor lock-in with third-party authentication providers like Firebase Auth.

Frontend clients, such as the FlutterFlow mobile app, interact with the backend for authentication by:
1. Using native SDKs (e.g., Google Sign-In) on the device to obtain an `ID Token`.
2. Sending this `ID Token` to a dedicated endpoint on the custom backend.
3. The backend verifies the token, finds or creates a user, and returns a standard JWT session token that is used for all subsequent authenticated API calls.

## 🌊 Data Flow & User Journey

This architecture introduces a clear data lifecycle that separates "hot" operational data from "cold" verifiable records.

### 1. User Onboarding (Registration)

Instead of managing complex seed phrases, a new user registers via a familiar social login.

1.  **Social Login**: The user clicks "Continue with Google" on the Dart frontend.
2.  **Key Generation**: The Web3Auth service handles the OAuth flow and generates a non-custodial cryptographic key pair for the user, which is securely associated with their social account.
3.  **DID Creation**: The frontend sends the user\'s **public key** to the backend\'s `POST /api/auth/register` endpoint.
4.  **On-Chain Identity**: The backend creates a W3C-compliant DID Document, stores it on the **Hedera File Service**, and generates a permanent `did:hedera` for the user, which is saved in the database.
5.  **Session**: The backend returns a JWT, logging the user into the application.

### 2. Clinical Encounter Workflow

1.  **Start Encounter**: A practitioner starts a new visit, triggering a `POST /api/encounters` call. This creates an `Encounter` document in MongoDB with a status of `Active`.
2.  **Live Data Capture**: During the visit, the practitioner creates various clinical records (Observations, Conditions, etc.). Each of these is an individual FHIR resource stored in its own collection in **MongoDB**, linked by the active `encounter_id`.
3.  **Finalize Encounter**: At the end of the visit, the practitioner triggers the finalization process via `POST /api/encounters/:id/finalize`.
4.  **Bundle & Archive**: The backend service gathers all resources related to the encounter from MongoDB, assembles them into a single, signed FHIR `Bundle`, and uploads this bundle to **IPFS**.
5.  **Record Linkage**: The IPFS hash (CID) of the finalized bundle is saved back to the `Encounter` document in MongoDB, and its status is changed to `Finalized`.

The patient now has an immutable, verifiable, and shareable record of their visit, controlled via their DID.

### 3. Smart Contract Interaction

The Hedera smart contracts (`VerifiableCredentials.sol`) are used to anchor high-trust information, such as a practitioner\'s medical license. When a credential is issued, its hash is stored on-chain, allowing anyone to verify its validity and check its revocation status without needing to see the full credential content.

## 🛡️ Security and Compliance

This application is designed to handle highly sensitive health data, and as such, security and compliance are paramount. The following section outlines our approach to security and our plan to align with the principles of the Health Insurance Portability and Accountability Act (HIPAA) and the Business Associate Agreement (BAA).

### Key Concepts

*   **HIPAA (Health Insurance Portability and Accountability Act):** A US federal law that establishes a national standard for protecting sensitive patient health information from being disclosed without the patient\'s consent or knowledge.
*   **BAA (Business Associate Agreement):** A written agreement between a covered entity and a business associate that requires the business associate to create administrative, physical, and technical safeguards to ensure the confidentiality, integrity, and availability of protected health information (PHI).
*   **PHI (Protected Health Information):** PHI is any personally identifiable health information that is created, used, or disclosed during the course of care. This includes, but is not limited to, a patient\'s name, address, social security number, diagnoses, medical test results, and treatment information.

### Security Enhancement Plan

To ensure the confidentiality, integrity, and availability of PHI, the following multi-phase security enhancement plan has been implemented:

*   **Phase 1: Enhance Data Encryption**
    *   **At-Rest Encryption:** All PHI is encrypted at rest in the MongoDB database. The `fhir_patient` field in the `Patient` model is encrypted using AES-256-GCM. A blind index is used for the email address to allow for secure querying without storing the email address in plaintext.
    *   **TLS Encryption:** All network traffic is encrypted using TLS to prevent eavesdropping.

*   **Phase 2: Implement Step-Up Authentication**
    *   **High-Assurance Session:** A high-assurance session has been introduced for sensitive operations. This requires users to re-authenticate before performing actions such as granting access to medical records, issuing verifiable credentials, and updating practitioner license information.

*   **Phase 3: Improve Access Control and Auditing**
    *   **Smart Contract Permissions:** The smart contracts have been improved to include more granular permissions and events. The `HealthcareAccessControl.sol` contract now emits `PermissionGranted` and `PermissionRevoked` events.
    *   **Audit Logging:** A comprehensive, tamper-proof audit logging service has been implemented. All PHI-related events are logged to a separate, immutable log.

*   **Phase 4: Strengthen Smart Contract Security**
    *   **Re-entrancy Guard:** The smart contracts are protected against re-entrancy attacks using the OpenZeppelin `ReentrancyGuard` contract.
    *   **Ownership and Pausable:** the smart contracts now have a clear ownership model and can be paused by the owner in case of an emergency.
    *   **Canonical DID:** The DID document on Hedera is now canonical, meaning it is self-referential and can be easily resolved by DID resolvers.

### Smart Contract Security

To enhance the security and robustness of the smart contracts, several standard contracts from the OpenZeppelin Contracts library have been included. The library has been added as a git submodule in the `contracts/lib` directory.

*   **`Context.sol`**: This is a small, internal contract that provides the context of the current execution, specifically the `msg.sender` (the address of the account that called the function) and `msg.data` (the data that was sent with the call). Using the `_msgSender()` and `_msgData()` functions from this contract is a best practice for writing more flexible and secure code, especially for supporting meta-transactions in the future.

*   **`Ownable.sol`**: This contract provides a basic access control mechanism, where there is an account (an \"owner\") that can be granted exclusive access to specific functions. This is crucial for managing administrative tasks and ensuring that only authorized accounts can perform critical functions.

*   **`Pausable.sol`**: This contract allows for an emergency stop mechanism that can be triggered by the owner. This is a critical safety feature that allows you to temporarily halt the contract in case of a suspected vulnerability or attack, giving you time to investigate and address the issue.

*   **`ReentrancyGuard.sol`**: This contract helps prevent re-entrancy attacks, which are a common type of vulnerability in smart contracts. By adding the `nonReentrant` modifier to sensitive functions, we can protect them from this common attack vector.

**Disclaimer:** This project has undergone significant security enhancements to align with HIPAA and BAA guidelines. These efforts do not constitute a formal certification of HIPAA or BAA compliance. A comprehensive, third-party audit would be required to certify the application and its operational environment.

## 🚀 Quick Start

### Prerequisites

-   Docker and Docker Compose
-   Git

### 1. Clone & Configure

```bash
# Clone the repository
git clone <repository-url>
cd health_remastered

# Copy and edit the backend environment file
cp backend/env.example backend/.env
nano backend/.env
```

### 2. Start Services

```bash
docker-compose up -d
```

### 3. Access Services

-   **Backend API**: http://localhost:3000
-   **MongoDB Express**: http://localhost:8081
-   **IPFS Web UI**: http://localhost:8080

## 🌐 API Endpoints

### Authentication & Registration
-   `POST /api/auth/register` - Create a new user from a social login public key and get a session token.

### Encounters
-   `POST /api/encounters` - Create a new, active clinical encounter.
-   `POST /api/encounters/:id/finalize` - Finalize an encounter, bundling its data and archiving it to IPFS.

### Patients
-   `GET /api/patients/:id` - Get patient details by DID.

### Verifiable Credentials
-   `POST /api/credentials/issue` - Issue a new Verifiable Credential and anchor it on-chain.

## 🧪 Testing

### Health Check

```bash
curl http://localhost:3000/health
```

### 1. Register a New User

This simulates the call your frontend would make after getting a public key from Web3Auth.

```bash
curl -X POST http://localhost:3000/api/auth/register \
  -H "Content-Type: application/json" \
  -d 
{
    "name": "John Doe",
    "email": "john.doe@example.com",
    "public_key_hex": "6e85794657c6fa4c1518c6a92c145955b839a1823c69c856d042eb433a91d434"
  }
```

This will return the new user\'s profile, their `did:hedera`, and a JWT for authentication.

---

*The rest of the original README (Project Structure, Development, etc.) remains largely the same.*
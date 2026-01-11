# Decentralized Health Records System

**A patient-centric healthcare platform that gives you ownership of your data, powered by AI, Hedera Hashgraph, and a high-performance Rust backend.**

---

## 🎯 The Problem

In today's digital world, patient health data is fragmented across dozens of siloed systems. It's insecure, difficult to access, and almost impossible for patients to control. This lack of data sovereignty and interoperability hinders patient care and creates massive inefficiencies.

## ✨ Our Solution: WeCare

WeCare is a revolutionary dApp that puts patients back in control. By leveraging a hybrid Web3 architecture, we provide a seamless, secure, and intelligent platform for managing health records. Users get the simplicity of familiar logins while benefiting from the security and transparency of a decentralized backend.

## 🌟 Key Features

*   🤖 **AI-Powered Health Guidance:** Ask complex health-related questions in plain language and receive clear, context-aware answers powered by Google's Gemini Pro model, all through a secure backend API.
*   🔐 **Self-Sovereign Identity (SSI):** Say goodbye to multiple logins. With WeCare, your identity is a portable, secure Decentralized Identifier (DID) on the Hedera network (`did:hedera`), giving you full control.
*   🛡️ **Verifiable, Tamper-Proof Records:** Clinical records are cryptographically signed, encrypted, and stored on **IPFS**. Their final state is anchored to the Hedera ledger, creating an immutable and verifiable audit trail.
*   🔑 **Simple & Secure Access:** No complex seed phrases. Onboard in seconds using your existing Google, phone, or email accounts, linked securely to your decentralized identity.
*   ⚕️ **Built for Compliance:** The entire architecture is designed with the principles of **HIPAA** (data privacy) and **FHIR** (data interoperability) at its core, ensuring data is handled to the highest standard.

## 🛠️ Tech Stack Deep Dive

We chose a modern, robust, and high-performance stack to build a system that is both reliable and scalable.

| Category | Technology | Purpose |
| :--- | :--- | :--- |
| **Frontend** | **Flutterflow** | For rapid, beautiful, and cross-platform UI development. |
| **Backend** | **Rust (Axum Framework)** | For unparalleled performance, memory safety, and reliability in our core API. |
| **AI Model** | **Google Gemini Pro** | Providing state-of-the-art generative AI for health queries via a secure backend. |
| **Database** | **MongoDB** | Flexible, scalable storage for "hot" operational data like in-progress encounters. |
| **Distributed Ledger** | **Hedera Hashgraph** | The trust layer for our application, providing identity, auditability, and verification. |
| **Decentralized Storage**| **IPFS** | Content-addressed, tamper-proof storage for finalized, encrypted health records. |
| **Containerization** | **Docker & Docker Compose**| For consistent, reproducible deployments of our entire backend stack. |

## Architectural Highlights

### The Hedera Advantage

We chose Hedera as our DLT for three key reasons:
1.  **Performance & Low Fees:** With incredibly high throughput and fixed, low-cost transactions, Hedera is perfect for recording high-volume events like audit logs without breaking the bank.
2.  **Security (aBFT):** Asynchronous Byzantine Fault Tolerance provides the highest possible degree of security for a public ledger, ensuring the integrity of our identity layer and audit trails.
3.  **Sustainability:** Hedera's low carbon footprint makes it a responsible choice for a modern application.

**How We Use It:**
*   **Identity:** We create and manage W3C-compliant Decentralized Identifiers (`did:hedera`) for every user, giving them true self-sovereign identity.
*   **Auditing:** Every critical action is logged and anchored to the **Hedera Consensus Service (HCS)**, creating a tamper-proof, verifiable log for compliance.
*   **Verification:** High-trust information (like medical licenses) are issued as Verifiable Credentials, with their hash stored on Hedera smart contracts for public verification.

### Security & HIPAA/FHIR by Design

While not formally certified (a process outside the scope of a hackathon), the WeCare architecture is built to align with HIPAA's privacy and security principles:
*   **Encryption:** All Protected Health Information (PHI) is encrypted at rest (AES-256-GCM in the database and for IPFS files) and in transit (TLS).
*   **Access Control:** Granular permissions are managed by smart contracts, and sensitive operations require step-up authentication.
*   **Auditing:** The immutable audit trail on Hedera ensures all access and modifications to data are tracked.
*   **Interoperability:** By using the **FHIR** standard for all clinical data, we ensure our records are structured in a way that is universally understood by other healthcare systems.

## 🚀 Getting Started Locally

### Prerequisites
- Docker and Docker Compose
- Git

### 1. Clone & Configure
```bash
# Clone the repository
git clone <repository-url>
cd health_remastered

# Create the environment file from the example
cp .env.example .env

# Edit the file and add your secret keys (Hedera, Gemini, etc.)
nano .env
```

### 2. Start Services
This single command will build and start the entire backend stack (Rust, MongoDB, IPFS).
```bash
docker-compose up -d --build
```

### 3. Access Services
- **Backend API**: `https://localhost:3000`
- **IPFS Web UI**: `http://localhost:8080`

## 🌐 API Endpoints

A selection of key endpoints available.

*   `POST /api/auth/google` - Authenticate with a Google ID Token.
*   `POST /api/auth/phone/initiate` - Start phone-based OTP authentication.
*   `POST /api/auth/phone/verify` - Verify a phone OTP.
*   `POST /api/chat` - Submit a prompt to the Gemini AI assistant.
*   `POST /api/encounters` - Create a new, active clinical encounter.
*   `POST /api/encounters/:id/finalize` - Finalize an encounter, bundling its data and archiving it to IPFS.

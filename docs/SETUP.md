# Healthcare dApp Setup Guide

## Prerequisites

Before setting up the Healthcare dApp, ensure you have the following installed:

- **Docker** (version 20.10 or higher)
- **Docker Compose** (version 2.0 or higher)
- **Git** (for cloning the repository)
- **Node.js** (version 18 or higher) - for local development
- **Rust** (version 1.75 or higher) - for local development
- **MongoDB** (version 7.0 or higher) - for local development

## Quick Start with Docker

### 1. Clone the Repository
```bash
git clone <repository-url>
cd health_remastered
```

### 2. Environment Configuration
```bash
# Copy environment files
cp backend/env.example backend/.env
cp frontend/env.example frontend/.env

# Edit the environment files with your configuration
nano backend/.env
nano frontend/.env
```

### 3. Configure Hedera (Required)
Edit `backend/.env` and set your Hedera credentials:
```env
HEDERA_NETWORK=testnet
HEDERA_ACCOUNT_ID=0.0.123456
HEDERA_PRIVATE_KEY=your_private_key_here
JWT_SECRET=your_jwt_secret_here
```

### 4. Start the Application
```bash
# Start all services
docker-compose up -d

# View logs
docker-compose logs -f

# Check service status
docker-compose ps
```

### 5. Access the Application
- **Frontend**: http://localhost:3001
- **Backend API**: http://localhost:3000
- **MongoDB Express**: http://localhost:8081 (admin/admin)
- **IPFS Web UI**: http://localhost:8080

## Local Development Setup

### Backend Development

#### 1. Install Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

#### 2. Install Dependencies
```bash
cd backend
cargo build
```

#### 3. Set up Environment
```bash
cp env.example .env
# Edit .env with your configuration
```

#### 4. Run the Backend
```bash
cargo run
```

### Frontend Development

#### 1. Install Node.js Dependencies
```bash
cd frontend
npm install
```

#### 2. Set up Environment
```bash
cp env.example .env
# Edit .env with your configuration
```

#### 3. Start the Development Server
```bash
npm start
```

### Database Setup

#### 1. Start MongoDB
```bash
# Using Docker
docker run -d --name mongodb -p 27017:27017 mongo:7.0

# Or install locally
# Follow MongoDB installation guide for your OS
```

#### 2. Create Database and Collections
```bash
# Connect to MongoDB
mongosh

# Create database
use healthcare

# Create collections (optional - will be created automatically)
db.createCollection("patients")
db.createCollection("practitioners")
db.createCollection("prescriptions")
db.createCollection("access_controls")
db.createCollection("fhir_bundles")
```

### IPFS Setup

#### 1. Install IPFS
```bash
# Download IPFS
wget https://dist.ipfs.io/go-ipfs/v0.21.0/go-ipfs_v0.21.0_linux-amd64.tar.gz
tar -xzf go-ipfs_v0.21.0_linux-amd64.tar.gz
cd go-ipfs
sudo ./install.sh
```

#### 2. Initialize IPFS
```bash
ipfs init
ipfs daemon
```

## Hedera Configuration

### 1. Create Hedera Account
1. Visit [Hedera Portal](https://portal.hedera.com/)
2. Create a new account
3. Note your Account ID and Private Key
4. Fund your account with test HBAR (for testnet)

### 2. Deploy Smart Contracts
```bash
# Install Hedera CLI tools
npm install -g @hashgraph/cli

# Deploy contracts (requires Hedera account)
hedera-cli contract deploy HealthcareAccessControl.sol
hedera-cli contract deploy VerifiableCredentials.sol
```

### 3. Update Configuration
Update your `.env` file with the deployed contract addresses:
```env
HEDERA_ACCESS_CONTROL_CONTRACT=0.0.123456
HEDERA_CREDENTIALS_CONTRACT=0.0.123457
```

## Testing the Application

### 1. Health Check
```bash
curl http://localhost:3000/health
```

### 2. Create a Patient
```bash
curl -X POST http://localhost:3000/api/patients \
  -H "Content-Type: application/json" \
  -d '{
    "fhirPatient": {
      "resourceType": "Patient",
      "id": "patient-123",
      "identifier": [{"value": "123-45-6789"}],
      "name": [{"given": ["John"], "family": "Doe"}],
      "gender": "male",
      "birthDate": "1990-01-15",
      "address": [{"line": ["123 Main St"], "city": "Anytown"}],
      "telecom": [{"system": "phone", "value": "+1-555-123-4567"}]
    }
  }'
```

### 3. Create a Practitioner
```bash
curl -X POST http://localhost:3000/api/practitioners \
  -H "Content-Type: application/json" \
  -d '{
    "fhirPractitioner": {
      "resourceType": "Practitioner",
      "id": "practitioner-123",
      "identifier": [{"value": "1234567890"}],
      "name": [{"given": ["Dr. Jane"], "family": "Smith"}],
      "qualification": [{"code": {"text": "Doctor of Medicine"}}],
      "telecom": [{"system": "email", "value": "jane@hospital.com"}]
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
  }'
```

## Troubleshooting

### Common Issues

#### 1. MongoDB Connection Error
```bash
# Check if MongoDB is running
docker ps | grep mongodb

# Check MongoDB logs
docker logs healthcare_mongodb

# Restart MongoDB
docker-compose restart mongodb
```

#### 2. IPFS Connection Error
```bash
# Check if IPFS is running
docker ps | grep ipfs

# Check IPFS logs
docker logs healthcare_ipfs

# Restart IPFS
docker-compose restart ipfs
```

#### 3. Backend Compilation Error
```bash
# Clean and rebuild
cd backend
cargo clean
cargo build

# Check Rust version
rustc --version
```

#### 4. Frontend Build Error
```bash
# Clear npm cache
cd frontend
npm cache clean --force

# Delete node_modules and reinstall
rm -rf node_modules package-lock.json
npm install
```

### Logs and Debugging

#### View All Logs
```bash
docker-compose logs -f
```

#### View Specific Service Logs
```bash
docker-compose logs -f backend
docker-compose logs -f frontend
docker-compose logs -f mongodb
docker-compose logs -f ipfs
```

#### Debug Mode
```bash
# Set debug logging
export RUST_LOG=debug
docker-compose up -d
```

## Production Deployment

### 1. Environment Configuration
- Use production Hedera mainnet
- Set strong JWT secrets
- Configure SSL certificates
- Set up monitoring

### 2. Security Considerations
- Enable HTTPS
- Configure firewall rules
- Set up backup strategies
- Implement monitoring and alerting

### 3. Scaling
- Use load balancers
- Implement database clustering
- Set up IPFS pinning services
- Configure CDN for static assets

## Support

For issues and questions:
1. Check the troubleshooting section
2. Review the logs
3. Check the GitHub issues
4. Contact the development team

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

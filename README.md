# 🔐 Chronolock - Time-Locked NFT Platform

A revolutionary decentralized application built on the Internet Computer that enables users to create time-locked NFTs (Chronolocks) with encrypted metadata and media files. Chronolock combines blockchain technology with advanced cryptography to create digital time capsules that can only be unlocked at predetermined future dates.

**🚀 Key Features:**

- **Time-Locked NFTs**: Create NFTs that remain encrypted until a specific unlock time
- **Advanced Encryption**: Uses Internet Computer's VetKD (Verifiable Encrypted Threshold Key Derivation) for secure time-based decryption
- **ICRC-7 Compliant**: Full compatibility with Internet Computer NFT standards
- **Custom Token Economy**: Integrated CRNL token ledger with referral system
- **Rich Media Support**: Upload and encrypt various file types (images, videos, documents)
- **Multi-Recipient Support**: Share access with multiple recipients using their Internet Identity
- **Decentralized Storage**: All data stored on-chain with efficient chunked media upload system

## 🌟 Introduction

Chronolock revolutionizes the concept of digital time capsules by leveraging the Internet Computer's unique capabilities. Whether you want to create a birthday surprise, preserve memories for future generations, or implement time-delayed business processes, Chronolock provides a secure, decentralized platform for time-locked digital assets.

The platform solves the problem of creating truly immutable time-locked content without relying on centralized services or trusted third parties. Using cryptographic proofs and threshold encryption, Chronolock ensures that encrypted content can only be accessed after the predetermined unlock time, making it perfect for:

- **Personal Time Capsules**: Messages, photos, and documents for future viewing
- **Business Escrow**: Time-delayed asset releases and automated contracts
- **Educational Content**: Timed release of course materials and assignments
- **Digital Legacy**: Secure preservation of important documents and memories

## ✨ Features

### 🔒 **Time-Locked Encryption**

- Cryptographically secure time-based encryption using VetKD
- Content remains inaccessible until the specified unlock time
- Tamper-proof and immutable once created

### 🎨 **Rich Media Support**

- Support for images, videos, documents, and custom file types
- Efficient chunked upload system for large files
- Encrypted media storage with secure retrieval

### 👥 **Multi-Recipient Sharing**

- Share Chronolocks with multiple recipients
- Individual access control per recipient
- Support for Internet Identity principals

### 🪙 **Integrated Token Economy**

- CRNL token for platform interactions
- Referral system with token rewards
- ICRC-1 compliant token ledger

### � **ICRC-7 NFT Standard**

- Full compliance with Internet Computer NFT standards
- Standard transfer and ownership functions
- Rich metadata support

### 📊 **Analytics & Logging**

- Comprehensive activity logging
- Pagination support for large datasets
- Real-time statistics and monitoring

## 🛠️ Installation

### Prerequisites

Ensure the following tools are installed on your system:

- **Node.js** (v18 or higher)
- **Rust** (latest stable version)
- **DFX** (Internet Computer SDK) `>= 0.18`
- **pnpm** (package manager)

### Step-by-Step Setup

1. **Clone the repository**

   ```bash
   git clone https://github.com/crnlicp/Chronolock-React-Rust.git
   cd Chronolock-React-Rust
   ```

2. **Install dependencies**

   ```bash
   pnpm install
   ```

3. **Setup backend canisters**

   ```bash
   pnpm run setup:backend
   ```

   This script will:

   - Start a local DFX network
   - Create canister identities
   - Build Rust canisters
   - Generate Candid interfaces
   - Deploy all canisters

4. **Start the development environment**
   ```bash
   pnpm start
   ```
   This runs both the frontend (port 3000) and backend deployment watcher in parallel.

### Manual Setup (Alternative)

If you prefer manual setup:

```bash
# Start DFX network
dfx start --clean --background

# Create canisters
dfx canister create --all

# Build Rust canisters
cargo build --target wasm32-unknown-unknown --release

# Deploy canisters
dfx deploy

# Start frontend
cd src/frontend
pnpm dev
```

## 🚀 Usage

### For End Users

1. **Access the Application**

   - Visit the deployed frontend URL or run locally at `http://localhost:3000`
   - Connect using Internet Identity for authentication

2. **Create a Chronolock**

   - Navigate to the "Create" page
   - Set unlock time and recipients
   - Upload files and add metadata
   - Review and submit your time-locked NFT

3. **View Your Collection**
   - Access your Chronolocks in the "Collection" page
   - View locked and unlocked items
   - Transfer ownership to other users

### For Developers

#### Creating a Chronolock Programmatically

```javascript
import { chronolock_canister } from './declarations/chronolock_canister';

// Create encrypted metadata
const metadata = {
  unlock_time: Math.floor(Date.now() / 1000) + 3600, // 1 hour from now
  title: 'My Time Capsule',
  encrypted_metadata: encryptedData,
  user_keys: userKeyMap,
};

// Create the Chronolock
const result = await chronolock_canister.create_chronolock(
  JSON.stringify(metadata),
);
```

#### Checking Unlock Status

```javascript
// Get Chronolock details
const chronolock = await chronolock_canister.icrc7_token_metadata(tokenId);

// Check if unlocked
const now = Math.floor(Date.now() / 1000);
const metadata = JSON.parse(chronolock);
const isUnlocked = now >= metadata.unlock_time;
```

#### Retrieving Decryption Keys

```javascript
// Get time-based decryption key (after unlock time)
const keyResult = await chronolock_canister.get_time_decryption_key(
  tokenId,
  contextBytes,
);

if ('Ok' in keyResult) {
  const decryptionKey = keyResult.Ok.encrypted_key;
  // Use key to decrypt content
}
```

## 📚 Documentation

### API Reference

#### Chronolock Canister

- **`create_chronolock(metadata: text)`**: Create a new time-locked NFT
- **`get_time_decryption_key(token_id: text, context: blob)`**: Retrieve decryption key after unlock time
- **`icrc7_transfer(token_id: text, to: principal)`**: Transfer NFT ownership
- **`get_owner_chronolocks_paginated(owner: principal, offset: nat64, limit: nat64)`**: Get user's Chronolocks

#### CRNL Ledger Canister

- **`icrc1_transfer(args: TransferArgs)`**: Transfer CRNL tokens
- **`icrc1_balance_of(account: Account)`**: Check token balance
- **`claim_referral_reward(referrer_code: text)`**: Claim referral rewards

### Architecture Overview

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Frontend      │    │  Chronolock      │    │  CRNL Ledger    │
│   (React)       │◄──►│  Canister        │◄──►│  Canister       │
│                 │    │  (Rust)          │    │  (Rust)         │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                                │
                                ▼
                       ┌──────────────────┐
                       │  VetKD System    │
                       │  (Encryption)    │
                       └──────────────────┘
```

For detailed API documentation, refer to the generated Candid interfaces in `src/declarations/`.

## 🧪 Testing

### Run All Tests

```bash
# Run both backend and frontend tests
pnpm test
```

### Backend Tests

```bash
# Run Rust canister tests
pnpm run test:backend

# Or manually
cargo test --all
```

### Frontend Tests

```bash
# Run React component tests
pnpm run test:frontend

# Or manually in frontend directory
cd src/frontend
pnpm test
```

### Integration Tests

```bash
# Deploy to local network and run integration tests
dfx deploy
cargo test --test integration_tests
```

## 🗺️ Roadmap

Our development roadmap follows a structured 10-phase approach, with the first 5 phases now complete as of September 2025:

### Phase 01: Project Kickoff ✅

**Completed: January 2025**

- [x] Team formation and requirements gathering
- [x] Project scope and objectives definition
- [x] Success criteria establishment
- [x] Communication channels setup
- [x] Development environment configuration

### Phase 02: Architecture & Design ✅

**Completed: February 2025**

- [x] System architecture design (backend canisters, frontend structure)
- [x] Integration points planning
- [x] Wireframes and technical documentation
- [x] Core technology selection
- [x] Scalability and security considerations

### Phase 03: Core Backend Development ✅

**Completed: March 2025**

- [x] Main backend canisters developed in Rust
- [x] Essential business logic implementation
- [x] Data models and APIs creation
- [x] Smart contract functionality
- [x] Local testing environments setup

### Phase 04: Frontend MVP ✅

**Completed: August 2025**

- [x] Minimum viable product frontend (React + TypeScript)
- [x] Basic UI components integration
- [x] Backend canister connections
- [x] Core user flows enablement
- [x] Early feedback collection mechanisms

### Phase 05: Identity & Authentication ✅

**Completed: September 2025**

- [x] Robust identity management implementation
- [x] Authentication mechanisms deployment
- [x] Internet Identity integration
- [x] User data protection measures
- [x] Seamless onboarding experience

### Phase 06: Feature Expansion 🚧

**Target: Q4 2025**

- [ ] Advanced chronolock features
- [ ] Additional canister interactions
- [ ] Enhanced frontend with new pages
- [ ] Improved user experience design
- [ ] Responsive design optimization
- [ ] Mobile-friendly interface

### Phase 07: Testing & QA 📋

**Target: Q1 2026**

- [ ] Comprehensive unit testing
- [ ] Integration testing across all components
- [ ] End-to-end testing automation
- [ ] Bug fixes and performance optimization
- [ ] Reliability assurance
- [ ] External beta testing preparation

### Phase 08: Beta Launch 🔄

**Target: Q2 2026**

- [ ] Beta version release to select users
- [ ] User feedback collection and analysis
- [ ] System performance monitoring
- [ ] Feature iteration based on usage
- [ ] UI improvements from real-world feedback
- [ ] Community building initiatives

### Phase 09: Security Audit & Optimization 🔒

**Target: Q3 2026**

- [ ] Comprehensive security audits
- [ ] Smart contract vulnerability assessments
- [ ] Scalability optimizations
- [ ] Cost-efficiency improvements
- [ ] Robustness enhancements
- [ ] Documentation finalization

### Phase 10: Production Release 🚀

**Target: Q4 2026**

- [ ] Fully functional production deployment
- [ ] Mainnet launch
- [ ] User support systems
- [ ] Marketing campaign initiation
- [ ] Continuous monitoring setup
- [ ] Future update planning

## 🛠️ Technology Stack

- [Vite](https://vitejs.dev/): High-performance tooling for front-end web development
- [React](https://reactjs.org/): A component-based UI library
- [TypeScript](https://www.typescriptlang.org/): JavaScript extended with syntax for types
- [Rust CDK](https://docs.rs/ic-cdk/): The Canister Development Kit for Rust
- [VetKD](https://internetcomputer.org/docs/current/references/vetkeys-overview): Verifiable Encrypted Threshold Key Derivation
- [Internet Identity](https://identity.ic0.app/): Privacy-preserving authentication system

## 📄 License

This project is licensed under the MIT License - see the [LICENSE.md](LICENSE.md) file for details.

## 🤝 Contributing

We welcome contributions from the community! Please see [CONTRIBUTE.md](CONTRIBUTE.md) for guidelines on how to contribute to this project.

### Development Workflow

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## � Acknowledgements

- **DFINITY Foundation** - For the Internet Computer platform and VetKD technology
- **Internet Identity** - For providing secure, privacy-preserving authentication
- **Rust Community** - For the robust programming language and ecosystem
- **React Community** - For the powerful frontend framework and tooling

## 📚 References

- [Internet Computer docs](https://internetcomputer.org/docs/current/developer-docs/ic-overview)
- [VetKD Documentation](https://internetcomputer.org/docs/current/references/vetkeys-overview)
- [ICRC-7 NFT Standard](https://github.com/dfinity/ICRC/tree/main/ICRCs/ICRC-7)
- [ICRC-1 Token Standard](https://github.com/dfinity/ICRC-1)
- [Candid Interface Definition Language](https://internetcomputer.org/docs/current/references/candid-ref)
- [Vite developer docs](https://vitejs.dev/guide/)
- [React quick start guide](https://react.dev/learn)
- [Rust developer docs](https://internetcomputer.org/docs/current/developer-docs/backend/rust/)

## 🧪 Testing

- **Run frontend and backend tests**:

  ```sh
  npm run test
  ```

- **Run frontend only tests**:

  ```sh
  npm run test:frontend
  ```

- **Run backend only tests**:

  ```sh
  npm run test:backend
  ```

  ```sh
  cargo test --all
  ```

- **Run specific canister's tests**:
  ```sh
  cargo test --package [package_name]
  ```

## 🛡️ License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

---

**Built with ❤️ on the Internet Computer**

For support and questions, please open an issue or contact the development team.

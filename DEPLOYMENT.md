# 🚀 Chronolock Deployment Guide

This guide explains how to deploy Chronolock to both local development and IC mainnet environments.

## 🏗️ **Network-Aware Configuration**

Chronolock uses a smart configuration system that automatically adapts to your deployment target:

- **Local Development**: Includes `chainkey_testing_canister` for VetKD testing
- **IC Mainnet**: Excludes test canisters and uses production VetKD

## 📋 **Configuration Files**

- `dfx.json` - Active configuration (auto-generated, don't edit manually)
- `dfx.local.json` - Template for local development
- `dfx.ic.json` - Template for IC mainnet
- `generate_init_args.sh` - Smart setup script

## 🔧 **Setup Process**

### **Step 1: Run Setup Script**
```bash
pnpm run setup
# or directly:
./generate_init_args.sh
```

The script will ask you to choose:
1. **Local development** - Uses chainkey_testing_canister
2. **IC Mainnet** - Requires production VetKD canister ID

### **Step 2: Follow Network-Specific Instructions**

#### **For Local Development:**
```bash
# 1. Setup (already done above)
pnpm run setup  # Choose option 1

# 2. Start local replica
dfx start --clean --background

# 3. Create and deploy canisters
dfx canister create --all
dfx deploy

# 4. Start frontend
pnpm start
```

#### **For IC Mainnet:**
```bash
# 1. Setup (already done above)
pnpm run setup  # Choose option 2, enter production VetKD ID

# 2. Build frontend
pnpm run build

# 3. Deploy to IC mainnet
dfx deploy --network ic crnl_ledger_canister
dfx deploy --network ic chronolock_canister
dfx deploy --network ic frontend
```

## 🔄 **Switching Networks**

To switch between networks, simply run the setup script again:
```bash
./generate_init_args.sh
```

The script will:
- Update `dfx.json` with the correct canister configuration
- Generate appropriate initialization arguments
- Provide network-specific deployment instructions

## 🎯 **What Happens During Setup**

### **Local Development Mode:**
- ✅ Copies `dfx.local.json` → `dfx.json`
- ✅ Includes `chainkey_testing_canister` in configuration
- ✅ Uses chainkey_testing_canister ID as VetKD canister
- ✅ Sets network to "local"

### **IC Mainnet Mode:**
- ✅ Copies `dfx.ic.json` → `dfx.json`
- ✅ Excludes `chainkey_testing_canister` from configuration
- ✅ Uses production VetKD canister ID (user-provided)
- ✅ Sets network to "ic"

## ⚠️ **Important Notes**

### **Production VetKD Canister ID**
For IC mainnet deployment, you need the official VetKD canister ID from DFINITY:
- Check DFINITY documentation
- Contact DFINITY support
- **Do not use** the hardcoded test ID in production

### **Canister Dependencies**
- **Local**: Frontend depends on `chainkey_testing_canister`
- **IC**: Frontend excludes test canisters for production

### **Environment Variables**
DFX automatically sets environment variables based on the active `dfx.json`:
- `CANISTER_ID_CHAINKEY_TESTING_CANISTER` (local only)
- `CANISTER_ID_CHRONOLOCK_CANISTER`
- `CANISTER_ID_CRNL_LEDGER_CANISTER`
- `DFX_NETWORK`

## 🔍 **Troubleshooting**

### **"Could not retrieve chainkey_testing_canister canister ID"**
```bash
# Make sure you've created canisters first
dfx canister create --all
```

### **"VetKD canister ID is required for mainnet"**
- You must provide a valid production VetKD canister ID
- Contact DFINITY for the correct mainnet VetKD canister ID

### **Frontend can't find canister declarations**
```bash
# Regenerate declarations after network switch
dfx generate
```

## 📚 **File Structure**
```
Chronolock-React-Rust/
├── dfx.json              # Active config (auto-generated)
├── dfx.local.json        # Local development template
├── dfx.ic.json          # IC mainnet template
├── generate_init_args.sh # Smart setup script
└── src/
    ├── backend/
    │   ├── chainkey_testing_canister/  # Local only
    │   ├── chronolock_canister/        # Both networks
    │   └── crnl_ledger_canister/       # Both networks
    └── frontend/                       # Both networks
```

## ✅ **Deployment Checklist**

### **Local Development:**
- [ ] Run `pnpm run setup` (choose local)
- [ ] Start DFX: `dfx start --clean --background`
- [ ] Create canisters: `dfx canister create --all`
- [ ] Deploy: `dfx deploy`
- [ ] Start frontend: `pnpm start`

### **IC Mainnet:**
- [ ] Get production VetKD canister ID from DFINITY
- [ ] Run `pnpm run setup` (choose IC, enter VetKD ID)
- [ ] Build frontend: `pnpm run build`
- [ ] Deploy canisters: `dfx deploy --network ic`
- [ ] Verify deployment and test functionality

This automated approach ensures you never accidentally deploy test canisters to production! 🎉
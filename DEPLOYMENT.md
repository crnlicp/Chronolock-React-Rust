# 🚀 Chronolock Deployment Guide

This guide explains how to deploy Chronolock to both local development and IC mainnet environments.

## 🏗️ **Network-Aware Configuration**

Chronolock uses a smart configuration system that automatically adapts to your deployment target:

- **Local Development**: Uses management canister for vetKD (with `dfx_test_key`)
- **IC Mainnet**: Uses production vetKD system via management canister (with `test_key_1`)

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

1. **Local development** - Uses management canister with local vetKD (`dfx_test_key`)
2. **IC Mainnet** - Uses management canister with production vetKD (`test_key_1`)

### **Step 2: Follow Network-Specific Instructions**

#### **For Local Development:**

```bash
# 1. Setup (already done above)
pnpm run setup  # Choose option 1

# 2. Start frontend
pnpm run frontend
```

#### **For IC Mainnet:**

```bash
# 1. Setup (already done above)
pnpm run setup  # Choose option 2

# 2. Deploy to IC mainnet
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
- ✅ Configures chronolock_canister to use `dfx_test_key`
- ✅ All vetKD calls go directly to management canister
- ✅ Sets network to "local"

### **IC Mainnet Mode:**

- ✅ Copies `dfx.ic.json` → `dfx.json`
- ✅ Configures chronolock_canister to use `test_key_1`
- ✅ All vetKD calls go directly to management canister
- ✅ Sets network to "ic"

## ⚠️ **Important Notes**

### **VetKD Integration**

- **All Environments**: Uses management canister directly (no separate vetKD canister)
- **Local**: Automatically uses `dfx_test_key` for local testing
- **IC Mainnet**: Automatically uses `test_key_1` for production testing
- **Key Selection**: Network-aware key selection handled automatically in Rust code

### **Canister Dependencies**

- **All Networks**: Frontend only depends on core canisters (no vetKD canister dependency)
- **VetKD Calls**: All go through management canister for proper routing

### **Environment Variables**

DFX automatically sets environment variables based on the active `dfx.json`:

- `CANISTER_ID_CHRONOLOCK_CANISTER`
- `CANISTER_ID_CRNL_LEDGER_CANISTER`
- `CANISTER_ID_INTERNET_IDENTITY`
- `DFX_NETWORK`

## 🔍 **Troubleshooting**

### **"VetKD configuration error"**

```bash
# Ensure dfx.json is properly configured
./generate_init_args.sh
```

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

- [ ] Run `pnpm run setup` (choose IC)
- [ ] Build frontend: `pnpm run build`
- [ ] Deploy canisters: `dfx deploy --network ic`
- [ ] Verify deployment and test functionality

This automated approach ensures you never accidentally deploy test canisters to production! 🎉

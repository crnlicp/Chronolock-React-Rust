# 💰 Chronolock Cycles Analysis & Budget Planning

⚠️ **CRITICAL UPDATE - September 21, 2025**: VetKD operations require 30B cycles each (30x higher than initially estimated). This significantly impacts cycle consumption and runtime estimates.

This document provides a comprehensive analysis of cycle requirements for deploying Chronolock to the Internet Computer mainnet.

## 🎯 **Executive Summary**

⚠️ **CRITICAL**: VetKD operations cost 30B cycles each - this changes everything!

**10T cycles** for deploying and operating Chronolock canisters now provides:

- ✅ Complete deployment of all 3 canisters
- ⚠️ **LIMITED operational buffer**: 1-3 months runtime (was 6-12 months)
- ⚠️ **VetKD operations are expensive**: Each decryption costs 0.03T cycles
- ⚠️ **~333 total VetKD operations** possible with full 10T budget

**Key Impact**: Each time a user decrypts a Chronolock, it costs 30 billion cycles.

## 📊 **Estimated Cycle Costs**

### **Initial Deployment Costs**

| Canister                 | Estimated Size | Deployment Cost  | Initial Cycles |
| ------------------------ | -------------- | ---------------- | -------------- |
| **chronolock_canister**  | ~2-3MB         | ~200B cycles     | ~1T cycles     |
| **crnl_ledger_canister** | ~1-2MB         | ~150B cycles     | ~1T cycles     |
| **frontend**             | ~5-10MB        | ~500B cycles     | ~1T cycles     |
| **Total Initial**        |                | **~850B cycles** | **~3T cycles** |

### **Budget Overview**

- **Available**: ~10T cycles
- **Required for deployment**: ~3T cycles
- **Remaining for operations**: ~7T cycles
- **Safety margin**: Excellent (70% buffer)

## 🚀 **Recommended Deployment Strategy**

### **Phase 1: Initial Deployment (Use ~7T cycles)**

Deploy each canister with generous initial cycles to ensure smooth operation:

```bash
# Deploy CRNL Ledger Canister
dfx deploy --network ic --with-cycles 2000000000000 crnl_ledger_canister

# Deploy Chronolock Canister (main logic, needs more cycles)
dfx deploy --network ic --with-cycles 3000000000000 chronolock_canister

# Deploy Frontend Assets
dfx deploy --network ic --with-cycles 2000000000000 frontend
```

**Total Phase 1 Usage**: ~7T cycles  
**Remaining Budget**: ~3T cycles

### **Phase 2: Operational Top-up (Use remaining ~3T cycles)**

Add operational cycles for sustained usage:

```bash
# Top-up CRNL Ledger (efficient operations)
dfx canister deposit-cycles 1000000000000 crnl_ledger_canister --network ic

# Top-up Chronolock (heavy VetKD usage)
dfx canister deposit-cycles 1500000000000 chronolock_canister --network ic

# Top-up Frontend (static serving)
dfx canister deposit-cycles 500000000000 frontend --network ic
```

**Total Phase 2 Usage**: ~3T cycles  
**Total Budget Used**: 10T cycles (100% allocated)

## 💡 **Why Top-up is Essential**

### **Operational Cycle Consumption**

Your Chronolock canisters will consume cycles for:

#### **High-Cost Operations:**

- **VetKD calls**: ~30B cycles per encryption/decryption operation ⚠️ **CRITICAL COST**
- **Time-locked NFT creation**: High compute + multiple VetKD calls
- **Media file storage**: ~5M cycles per GB per year
- **Complex metadata operations**: Encryption, decryption, validation

#### **Standard Operations:**

- **ICRC-7 NFT transactions**: ~1M cycles per transaction
- **HTTP requests**: ~2M cycles per request
- **Compute operations**: ~1M cycles per million instructions
- **Storage operations**: Variable based on data size

#### **Chronolock-Specific Costs:**

- **Creating time-locked NFTs**: 30-80M cycles (includes 30B VetKD + storage)
- **Unlocking NFTs**: 30-50M cycles (30B VetKD decryption + processing)
- **Media file uploads**: 1-10M cycles per file
- **User key management**: 1-5M cycles per operation

## 📈 **Expected Runtime Analysis**

### **Conservative Usage Estimates**

⚠️ **UPDATED FOR 30B VetKD COSTS** - Previous estimates were 30x too low

With 10T cycles properly allocated:

| Usage Level  | NFTs/Day | VetKD Ops/Day | Expected Runtime | Primary Bottleneck |
| ------------ | -------- | ------------- | ---------------- | ------------------ |
| **Light**    | 2-5      | 4-10          | 3-6 months       | VetKD operations   |
| **Moderate** | 10-20    | 20-40         | 1-3 months       | VetKD operations   |
| **Heavy**    | 50+      | 100+          | 2-4 weeks        | VetKD operations   |

**⚠️ WARNING**: VetKD operations are now the primary cost driver, consuming 30B cycles each.

### **Per-Canister Runtime Expectations**

#### **chronolock_canister** (Most cycle-intensive)

- **With 4.5T cycles**: 1-3 months moderate usage (due to VetKD costs)
- **Main consumers**: VetKD calls (30B each), NFT storage, complex operations
- **Monitoring priority**: **CRITICAL** (check daily)
- **VetKD capacity**: ~150 operations per 4.5T cycles

#### **crnl_ledger_canister** (Efficient operations)

- **With 3T cycles**: 12+ months
- **Main consumers**: Token transfers, balance queries
- **Monitoring priority**: Medium (check monthly)

#### **frontend** (Static serving)

- **With 2.5T cycles**: 12+ months
- **Main consumers**: HTTP requests, asset serving
- **Monitoring priority**: Low (check monthly)

## ⚠️ **Cycle Management Best Practices**

### **1. Regular Monitoring**

Check canister status regularly to prevent freezing:

```bash
# Check all canister statuses
dfx canister status chronolock_canister --network ic
dfx canister status crnl_ledger_canister --network ic
dfx canister status frontend --network ic

# Quick status check (shows cycles balance)
dfx canister status --all --network ic
```

### **2. Alert Thresholds**

Set up monitoring alerts:

- **Critical**: Below 200B cycles (immediate top-up needed for VetKD ops)
- **Warning**: Below 1T cycles (plan top-up within week)
- **Info**: Below 2T cycles (plan top-up within month)

### **3. Emergency Top-up Commands**

Keep these commands ready for quick top-ups:

```bash
# Emergency top-up (1T cycles each)
dfx canister deposit-cycles 1000000000000 chronolock_canister --network ic
dfx canister deposit-cycles 1000000000000 crnl_ledger_canister --network ic
dfx canister deposit-cycles 1000000000000 frontend --network ic
```

## 🔧 **Optimization Strategies**

### **Cycle Efficiency Features in Chronolock**

Your codebase already includes several cycle-efficient patterns:

#### **✅ Efficient Storage**

- Uses `ic-stable-structures` for persistent storage
- Implements proper `Storable` traits with size bounds
- Memory management with `MemoryManager`

#### **✅ Optimized Operations**

- Batch operations where possible
- Efficient data structures (`StableBTreeMap`)
- Proper error handling to avoid wasted cycles

#### **✅ Smart Caching**

- VetKD public keys can be cached
- Metadata validation before expensive operations
- Efficient pagination for large datasets

### **Additional Optimization Opportunities**

⚠️ **Critical due to 30B VetKD costs:**

1. **Batch VetKD Operations**: **ESSENTIAL** - Group multiple encryption/decryption calls
2. **Cache VetKD Results**: Store derived keys temporarily to avoid re-derivation
3. **Optimize User Access Patterns**: Minimize redundant VetKD calls
4. **Lazy VetKD Loading**: Only derive keys when actually needed for decryption
5. **User Education**: Inform users about the cost of VetKD operations

## 💰 **Cost Breakdown Summary**

| Phase          | Purpose                   | Cycles Allocated | Cumulative Used |
| -------------- | ------------------------- | ---------------- | --------------- |
| **Deployment** | Initial canister creation | 7T               | 7T              |
| **Top-up**     | Operational buffer        | 3T               | 10T             |
| **Future**     | Additional top-ups needed | TBD              | 10T+            |

### **Expected Additional Costs**

⚠️ **UPDATED**: Much higher costs due to 30B cycle VetKD operations

After 1-3 months (much shorter runway), you may need:

- **Light usage**: 5-10T additional cycles per quarter
- **Moderate usage**: 15-30T additional cycles per quarter
- **Heavy usage**: 50-100T additional cycles per quarter

**Each VetKD operation costs 30B cycles = 0.03T cycles**

## 🎯 **Deployment Checklist**

### **Pre-Deployment**

- [ ] Confirm 10T cycles available in wallet
- [ ] Complete network configuration (`pnpm run setup`)
- [ ] Build frontend (`pnpm run build`)
- [ ] Verify production VetKD canister ID

### **Deployment Phase**

- [ ] Deploy with generous cycles (7T total)
- [ ] Verify all canisters deployed successfully
- [ ] Check initial cycle balances
- [ ] Test basic functionality

### **Post-Deployment**

- [ ] Top-up all canisters (3T total)
- [ ] Set up cycle monitoring schedule
- [ ] Document canister IDs and cycle balances
- [ ] Plan future cycle procurement

## 📊 **Monitoring Dashboard Template**

Track your cycle usage with this template:

```
Chronolock Cycle Status - [Date]
=====================================

chronolock_canister:    [X.XXT] cycles
crnl_ledger_canister:   [X.XXT] cycles
frontend:               [X.XXT] cycles
Total:                  [X.XXT] cycles

Days since deployment:  [XX]
Estimated runway:       [XX] months
Next check date:        [Date]

Actions needed:
[ ] None
[ ] Plan top-up (>1 month)
[ ] Schedule top-up (>1 week)
[ ] Emergency top-up (<1 week)
```

## 🎉 **Conclusion**

⚠️ **CRITICAL UPDATE**: VetKD operations cost 30B cycles each (30x previous estimate)

Your 10T cycle budget from DFINITY requires **careful management**:

- ⚠️ **Sufficient for deployment**: Still adequate for initial deployment
- ⚠️ **Limited operational buffer**: 1-3 months runtime (much shorter than originally estimated)
- ⚠️ **VetKD bottleneck**: Each decryption operation costs 0.03T cycles
- ⚠️ **Frequent top-ups needed**: Plan for quarterly cycle procurement

**Updated Recommendation**:

1. **Proceed with deployment** but implement strict VetKD usage monitoring
2. **Optimize VetKD calls immediately** - batch operations, cache results
3. **Plan for additional funding** - you'll need 20-50T cycles per year for moderate usage
4. **Monitor daily** - VetKD costs can exhaust cycles quickly

**VetKD Operation Capacity with 10T cycles**: ~333 operations total

---

_Last updated: September 21, 2025 - VetKD costs updated to 30B cycles_  
_Next review: After deployment + daily monitoring required_

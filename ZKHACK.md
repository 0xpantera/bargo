# ZK Hack Demo: Noir → Starknet Proof Verification with bargo

**Demo Objective:** Show complete end-to-end Zero-Knowledge proof verification pipeline from Noir circuit to live Starknet mainnet verification.

## 🚀 What We're Demonstrating

**bargo** is a developer-friendly CLI that consolidates the complex Noir ZK toolchain into simple, opinionated commands. Today we'll show the complete Cairo/Starknet integration:

- ✅ **Noir circuit** → **Cairo verifier contract** 
- ✅ **Optimized proof generation** (Starknet-native with ZK optimizations)
- ✅ **Seamless deployment workflow** (auto-save state between commands)
- ✅ **Live on-chain verification** on Starknet mainnet

## 📋 Prerequisites (Already Set Up)

```bash
# Tools installed
✅ nargo (Noir compiler)
✅ bb (Barretenberg prover) 
✅ garaga (Cairo verifier generator)
✅ bargo (our tool - consolidates everything)

# Environment ready
✅ Python virtual environment with garaga
✅ Starknet account with mainnet funding
✅ .secrets file with account credentials
```

## 🎯 Complete Demo Workflow

### Step 1: Start from Clean Slate
```bash
cd wkshp
../bargo/target/release/bargo clean
```
**What it does:** Removes all build artifacts to start fresh  
**Expected output:** `✅ Removed target/`

### Step 2: Build Noir Circuit
```bash
../bargo/target/release/bargo build --verbose
```
**What it does:** Compiles Noir circuit into bytecode + witness  
**Expected output:**
```
✅ Bytecode generated → target/bb/wkshp.json (967 B, ~100ms)
✅ Witness generated → target/bb/wkshp.gz (49 B, 0ms)
```

### Step 3: Generate Cairo Verifier (🔥 The Magic!)
```bash
source .venv/bin/activate
../bargo/target/release/bargo cairo gen --verbose
```
**What it does:** 
- Generates Starknet-optimized ZK proof using `ultra_starknet_zk_honk` + `--zk` flag
- Creates Cairo verifier contract for Starknet deployment
- Maximum gas optimization (Poseidon hash, ZK proofs)

**Expected output:**
```
✅ Starknet proof generated → target/starknet/proof (15.8 KB, ~170ms)
✅ Starknet VK generated → target/starknet/vk (1.7 KB, ~50ms)  
✅ Cairo verifier generated → contracts/Verifier.cairo (11.2 KB, ~2s)
```

### Step 4: Generate Calldata for Verification
```bash
../bargo/target/release/bargo cairo data --quiet
```
**What it does:** Converts proof into calldata format for on-chain verification  
**Expected output:** Long JSON array with ~2853 field elements (thousands of numbers)

### Step 5: Declare Contract on Starknet Mainnet
```bash
../bargo/target/release/bargo cairo declare --network mainnet --verbose
```
**What it does:** 
- Declares Cairo verifier contract on Starknet mainnet
- Auto-saves class hash for next step
- Uses real mainnet funds

**Expected output:**
```
✅ Class hash: 0x02755ac7ee11bbc9a675f01b77ba8b482450371b94d40e4132b4146c9a889dac
✅ Contract declared successfully (~20s)
```

### Step 6: Deploy Contract (Seamless UX!)
```bash
../bargo/target/release/bargo cairo deploy --verbose
```
**What it does:** 
- Automatically uses saved class hash (no copy/paste needed!)
- Deploys verifier contract instance
- Auto-saves contract address for verification

**Expected output:** Either success with contract address, or honest error reporting (no fake success!)

### Step 7: Verify Proof On-Chain (🎉 Grand Finale!)
```bash
../bargo/target/release/bargo cairo verify-onchain --verbose
```
**What it does:**
- Automatically uses saved contract address  
- Submits proof to live Starknet mainnet contract
- Real ZK verification happens on-chain

**Expected output:**
```
✅ Using saved contract address: 0x65bf3f2391439511353ca05dda89acaa82956ad7f871152f345b7917e0a2f34
Transaction hash: 0x66ea718e6a99b35877c5c7ed4e9e55aa8c2109923413c68b89d931329fb9f2c
Check it out on https://voyager.online/tx/0x66ea718e6a99b35877c5c7ed4e9e55aa8c2109923413c68b89d931329fb9f2c
✅ Proof verified on-chain successfully (~13s)
```

## 🎯 Key Demo Points to Highlight

### 1. **Developer Experience Revolution**
**Before bargo:**
```bash
# Complex, error-prone workflow
nargo execute
bb prove -s ultra_honk --oracle_hash starknet --zk -b target/foo.json -w target/foo.gz -o target/
bb write_vk -b target/foo.json -o target/ --oracle_hash starknet  
garaga gen --system ultra_starknet_zk_honk --vk target/vk --project-name foo
garaga calldata --system ultra_starknet_zk_honk --proof target/proof --vk target/vk --public-inputs target/public_inputs
garaga declare --project-path ./foo --network mainnet
# Copy class hash manually 😱
garaga deploy --class-hash 0x1234... 
# Copy contract address manually 😱
garaga verify-onchain --system ultra_starknet_zk_honk --contract-address 0xabcd... --network mainnet --vk target/vk --proof target/proof --public-inputs target/public_inputs
```

**After bargo:**
```bash
# Simple, opinionated workflow
bargo cairo gen
bargo cairo declare --network mainnet  
bargo cairo deploy
bargo cairo verify-onchain
```

### 2. **Automatic State Management**
- **Class hashes** automatically saved in `target/starknet/.bargo_class_hash`
- **Contract addresses** automatically saved in `target/starknet/.bargo_contract_address`  
- **No manual copying** between commands!

### 3. **Maximum Optimization**
- Uses `ultra_starknet_zk_honk` system (most gas-efficient)
- Proper `--zk` flag for ZK proof generation
- Starknet-native Poseidon hash instead of Keccak
- Reduces contract size and verification costs

### 4. **Honest Error Handling**
- **Before:** Fake success messages even when commands failed
- **After:** Real error reporting with helpful troubleshooting suggestions

### 5. **Production Ready**
- ✅ **Tested end-to-end** on Starknet mainnet
- ✅ **Real contract deployments** and proof verifications  
- ✅ **Live transaction hashes** you can verify on Voyager

## 🎤 Demo Script

1. **"Let me show you the problem"** - Show complex garaga/bb command sequences
2. **"Here's our solution"** - Show simple bargo commands
3. **"Watch it work live"** - Run the complete workflow
4. **"This is production-ready"** - Show real mainnet transactions on Voyager
5. **"Developer experience matters"** - Highlight auto-save, error handling, optimizations

## 🔗 Live Results to Show

- **Voyager Transaction:** https://voyager.online/tx/0x66ea718e6a99b35877c5c7ed4e9e55aa8c2109923413c68b89d931329fb9f2c
- **Contract Address:** `0x65bf3f2391439511353ca05dda89acaa82956ad7f871152f345b7917e0a2f34`
- **Class Hash:** `0x02755ac7ee11bbc9a675f01b77ba8b482450371b94d40e4132b4146c9a889dac`

## 💡 Demo Tips

- **Use `--verbose`** to show underlying commands being executed
- **Highlight the auto-save** features (no copy/paste needed)
- **Show error handling** if deployment fails (honest reporting)
- **Open Voyager links** to show real mainnet verification
- **Compare before/after** command complexity

## 🚀 Impact Statement

**"We've taken the complex, error-prone Noir → Starknet workflow and made it as simple as deploying to Ethereum. This unlocks ZK development for the masses and enables production-ready Starknet ZK applications."**

---

*Total demo time: ~3-5 minutes for complete end-to-end workflow*
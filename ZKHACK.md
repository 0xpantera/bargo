# ZK Hack Demo: Cross-Chain Noir ZK Verification with bargo

**Demo Objective:** Demonstrate a unified, developer-friendly CLI that enables seamless Zero-Knowledge proof verification across both **EVM** and **Starknet** ecosystems from a single Noir circuit.

## 🚀 What We're Demonstrating

**bargo** revolutionizes Noir ZK development by consolidating complex multi-tool workflows into simple, opinionated commands. Today we'll show **complete feature parity** between EVM and Starknet:

- ✅ **Single Noir circuit** → **Dual-chain verifier contracts** (Solidity + Cairo)
- ✅ **Chain-optimized proof generation** (Keccak for EVM, Poseidon for Starknet)
- ✅ **Complete project scaffolding** (Foundry + Scarb structures)
- ✅ **Seamless deployment workflows** with auto-state management
- ✅ **Live cross-chain verification** on both Ethereum and Starknet

## 📋 Prerequisites & Setup

### Required Tools Installation

```bash
# 1. Install Foundry (for EVM workflow)
curl -L https://foundry.paradigm.xyz | bash
foundryup

# 2. Install specific versions for Cairo workflow (CRITICAL!)
pip install garaga==0.18.1
noirup --version 1.0.0-beta.4
bbup --version 0.87.4-starknet.1

# 3. Verify installations
forge --version      # Should show foundry version
garaga --help        # Should show garaga CLI
nargo --version      # Should show 1.0.0-beta.4
bb --version         # Should show 0.87.4-starknet.1
```

### Environment Setup

**For EVM workflow**, create `.env` file:
```bash
# .env
SEPOLIA_RPC_URL="https://eth-sepolia.g.alchemy.com/v2/your_key"
SEPOLIA_PRIVATE_KEY=0x1234567890abcdef...
ETHERSCAN_API_KEY=your_etherscan_key
```

**For Cairo workflow**, create `.secrets` file:
```bash
# .secrets
SEPOLIA_RPC_URL="https://starknet-sepolia.g.alchemy.com/starknet/version/rpc/v0_8/your_key"
SEPOLIA_ACCOUNT_PRIVATE_KEY=0x1234567890abcdef...
SEPOLIA_ACCOUNT_ADDRESS=0x1234567890abcdef...

MAINNET_RPC_URL="https://starknet-mainnet.g.alchemy.com/starknet/version/rpc/v0_8/your_key"
MAINNET_ACCOUNT_PRIVATE_KEY=0x1234567890abcdef...
MAINNET_ACCOUNT_ADDRESS=0x1234567890abcdef...
```

## 🔧 Demo Setup

```bash
# Create a Noir demo project
nargo new demo && cd demo

# Clean any existing artifacts
../bargo/target/release/bargo clean

# Verify we have a basic Noir circuit
cat src/main.nr
```

**Expected:** Simple Noir circuit like:
```noir
fn main(x: Field, y: Field) -> Field {
    x + y
}
```

## 🌐 EVM Workflow Demo

### Step 1: Generate Complete Foundry Project
```bash
../bargo/target/release/bargo evm gen --verbose
```

**What happens:**
- Generates Keccak-optimized proof (Ethereum native)
- Creates complete Foundry project structure
- Produces Verifier.sol contract ready for deployment

**Expected output:**
```
🔧 Generating EVM verifier with Foundry integration...
✅ Keccak proof generated → target/bb/proof (13.8 KB, ~150ms)
✅ Keccak VK generated → target/bb/vk (1.2 KB, ~50ms)
✅ Foundry project created → contracts/
✅ Verifier.sol generated → contracts/src/Verifier.sol (25.3 KB, ~200ms)
✅ Ready for Ethereum deployment
```

### Step 2: Generate Calldata
```bash
../bargo/target/release/bargo evm calldata
```

**What happens:** Converts proof into ABI-encoded format for contract interaction

**Expected output:**
```
✅ ABI-encoded calldata generated for on-chain verification
0x1234567890abcdef... (long hex string)
```

### Step 3: Deploy to Ethereum (Optional - requires testnet setup)
```bash
# If environment is set up:
../bargo/target/release/bargo evm deploy --network sepolia
```

**Expected output:**
```
✅ Contract deployed to: 0x742d35Cc6634C0532925a3b8D9F9CCE8c8C8C82A
✅ Deployment transaction: 0x8f2a7b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0f1a2
```

## 🏛️ Cairo/Starknet Workflow Demo

### Step 1: Generate Cairo Verifier
```bash
# Activate Python environment if needed
source .venv/bin/activate

../bargo/target/release/bargo cairo gen --verbose
```

**What happens:**
- Generates Starknet-optimized ZK proof using `ultra_starknet_zk_honk`
- Creates Cairo verifier contract with Poseidon hash optimization
- Maximum gas efficiency for Starknet deployment

**Expected output:**
```
🔧 Generating Cairo verifier with maximum ZK optimization...
✅ Starknet proof generated → target/starknet/proof (15.8 KB, ~170ms)
✅ Starknet VK generated → target/starknet/vk (1.7 KB, ~50ms)
✅ Cairo verifier generated → contracts/Verifier.cairo (11.2 KB, ~2s)
✅ Optimized for maximum gas efficiency
```

### Step 2: Generate Calldata
```bash
../bargo/target/release/bargo cairo data
```

**What happens:** Converts proof into JSON format with field elements

**Expected output:**
```
✅ Calldata JSON generated (2853 field elements)
[0x1234, 0x5678, 0x9abc, ...] (long JSON array)
```

### Step 3: Declare Contract (Optional - requires Starknet account)
```bash
# If .secrets file is configured:
../bargo/target/release/bargo cairo declare --network sepolia
```

**Expected output:**
```
✅ Class hash: 0x02755ac7ee11bbc9a675f01b77ba8b482450371b94d40e4132b4146c9a889dac
✅ Contract declared successfully
```

### Step 4: Deploy Contract
```bash
../bargo/target/release/bargo cairo deploy
```

**What happens:** Automatically uses saved class hash, no manual copying needed!

**Expected output:**
```
✅ Using saved class hash: 0x02755ac7...
✅ Contract deployed: 0x65bf3f2391439511353ca05dda89acaa82956ad7f871152f345b7917e0a2f34
```

## 🌉 Cross-Chain Demonstration

### Show Feature Parity
```bash
# Generate verifiers for both chains from same circuit
../bargo/target/release/bargo evm gen      # ✅ Ethereum-ready
../bargo/target/release/bargo cairo gen    # ✅ Starknet-ready

# Generate calldata for both formats
../bargo/target/release/bargo evm calldata    # ✅ ABI-encoded
../bargo/target/release/bargo cairo data      # ✅ JSON format

# Deploy to both ecosystems (if environments configured)
../bargo/target/release/bargo evm deploy --network sepolia
../bargo/target/release/bargo cairo deploy --network sepolia
```

### Verify Project Structure
```bash
# Check generated artifacts
ls -la target/bb/          # EVM artifacts
ls -la target/starknet/    # Starknet artifacts
ls -la contracts/          # Generated contracts
```

**Expected structure:**
```
target/
├── bb/
│   ├── proof              # Keccak proof for EVM
│   └── vk                 # Keccak verification key
└── starknet/
    ├── proof              # Poseidon proof for Starknet
    ├── vk                 # Poseidon verification key
    └── .bargo_class_hash   # Auto-saved state

contracts/
├── foundry.toml           # Foundry configuration
├── src/
│   └── Verifier.sol       # Solidity verifier
└── Verifier.cairo         # Cairo verifier
```

## 🎯 Key Demo Points for Judges

### 1. Developer Experience Revolution

**Before bargo (EVM):**
```bash
# Complex, error-prone workflow
nargo execute
bb prove --oracle_hash keccak -b target/wkshp.json -w target/wkshp.gz -o target/
bb write_vk --oracle_hash keccak -b target/wkshp.json -o target/
bb write_solidity_verifier -k target/vk -o contracts/Verifier.sol
forge create --rpc-url $RPC_URL --private-key $PRIVATE_KEY Verifier.sol
# Manual contract interaction...
```

**Before bargo (Starknet):**
```bash
# Even more complex!
nargo execute
bb prove -s ultra_honk --oracle_hash starknet --zk -b target/wkshp.json -w target/wkshp.gz -o target/
bb write_vk --oracle_hash starknet -b target/wkshp.json -o target/
garaga gen --system ultra_starknet_zk_honk --vk target/vk --project-name wkshp
garaga calldata --system ultra_starknet_zk_honk --proof target/proof --vk target/vk
garaga declare --project-path ./wkshp --network sepolia
# Copy class hash manually 😱
garaga deploy --class-hash 0x1234...
# Copy contract address manually 😱
```

**After bargo:**
```bash
# Unified, simple workflow for both chains
bargo evm gen && bargo evm deploy --network sepolia
bargo cairo gen && bargo cairo deploy --network sepolia
```

### 2. Automatic State Management
- **EVM**: Automatic Foundry project setup and configuration
- **Starknet**: Auto-save class hashes and contract addresses
- **Both**: No manual copying between commands

### 3. Chain-Specific Optimizations
- **EVM**: Keccak hash (Ethereum native), standard proof format
- **Starknet**: Poseidon hash (Starknet native), ZK-optimized proofs
- **Both**: Optimized for their respective ecosystems

### 4. Production Ready
- ✅ **EVM**: Tested end-to-end with Foundry integration
- ✅ **Starknet**: Tested on mainnet with real deployments
- ✅ **Both**: Robust error handling and user feedback

## 📊 Technical Comparison

| Feature | EVM Implementation | Starknet Implementation |
|---------|-------------------|------------------------|
| **Hash Function** | Keccak (Ethereum native) | Poseidon (Starknet native) |
| **Proof System** | `ultra_honk` | `ultra_starknet_zk_honk` |
| **Project Structure** | Foundry + Solidity | Scarb + Cairo |
| **Calldata Format** | ABI-encoded hex | JSON field elements |
| **Deployment** | `forge create` | `starkli declare` + `deploy` |
| **State Management** | Environment variables | Auto-saved files |

## 🎤 Judge Evaluation Script

### Quick Demo (2-3 minutes)
```bash
# 1. Clean start
bargo clean

# 2. Show dual-chain generation
bargo evm gen      # ✅ Ethereum verifier ready
bargo cairo gen    # ✅ Starknet verifier ready

# 3. Show calldata generation
bargo evm calldata    # ✅ ABI format
bargo cairo data      # ✅ JSON format

# 4. Show project structure
ls -la target/ contracts/
```

### Full Demo (5-10 minutes)
Include actual deployment steps if environment is configured.

## 🚀 Impact Statement

**"We've solved the fragmentation problem in ZK development. Instead of learning different toolchains for EVM and Starknet, developers now have a unified CLI that provides identical developer experience across both ecosystems, each optimized for its target blockchain."**

## 🔗 Real Production Examples

- **Starknet Mainnet Contract**: `0x65bf3f2391439511353ca05dda89acaa82956ad7f871152f345b7917e0a2f34`
- **Successful Verification TX**: `0x66ea718e6a99b35877c5c7ed4e9e55aa8c2109923413c68b89d931329fb9f2c`
- **Voyager Link**: https://voyager.online/tx/0x66ea718e6a99b35877c5c7ed4e9e55aa8c2109923413c68b89d931329fb9f2c

## 💡 Troubleshooting for Judges

### Common Issues
1. **Version mismatches**: Ensure exact versions for Cairo workflow
2. **Environment setup**: Check `.env` and `.secrets` files
3. **Tool installation**: Verify `forge`, `garaga`, `nargo`, `bb` are available

### Quick Fixes
```bash
# Check tool versions
bargo doctor

# Reset environment
bargo clean && bargo rebuild

# Verbose output for debugging
bargo evm gen --verbose
bargo cairo gen --verbose
```

---

*Total evaluation time: 5-10 minutes for complete cross-chain demonstration*
*bargo: Making Zero-Knowledge development accessible across all blockchains*
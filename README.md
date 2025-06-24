<div align="center">
  <img src="./assets/bargo-logo.png" alt="bargo logo" width="400"/>
</div>

# bargo

A unified CLI tool for Noir zero-knowledge development that consolidates `nargo`, `bb`, and `garaga` workflows into a single, opinionated interface. Generate proofs, verification keys, and deploy verifier contracts to both EVM chains and Starknet with simple commands.

## What is bargo?

bargo is a Swiss Army knife for circuit proving, verification, smart contract generation, and deployment. It abstracts away the complexity of juggling multiple tools and provides a streamlined workflow for Noir developers targeting both Ethereum and Starknet ecosystems.

**Key Features:**
- **Unified Interface**: One tool for the entire ZK development lifecycle
- **Multi-Chain Support**: Generate verifiers for both EVM and Starknet
- **Isolated Workflows**: Separate target directories prevent cross-contamination
- **Smart Rebuilds**: Automatic detection of when rebuilds are needed
- **Rich Output**: Clear progress indicators and helpful error messages

## Requirements

### Core Dependencies
- **[nargo](https://noir-lang.org/docs/getting_started/installation/)** - Noir language toolchain
- **[bb](https://github.com/AztecProtocol/aztec-packages/tree/master/barretenberg)** - Barretenberg proving system

### EVM Workflow (Optional)
- **[Foundry](https://getfoundry.sh/)** - For Solidity contract deployment
- **Environment Variables**: `RPC_URL`, `PRIVATE_KEY` in `.env` file

### Starknet Workflow (Optional)  
- **[starkli](https://github.com/xJonathanLEI/starkli)** - Starknet CLI tool
- **[garaga](https://github.com/keep-starknet-strange/garaga)** - Cairo verifier generation
- **Python 3.10+** and **pipx** for garaga installation
- **Environment Variables**: Starknet network configuration

## Motivation

Currently, Noir developers must juggle multiple tools and remember complex command sequences:

```bash
# Current workflow (verbose and error-prone)
nargo check
nargo execute                  # produce bytecode + witness  
bb prove   -b target/foo.json -w target/foo.gz -o target/
bb write_vk -b target/foo.json -o target/
bb verify   -k target/vk -p target/proof

# Plus remembering different flags for Solidity generation
bb write_vk --oracle_hash keccak -b target/foo.json -o target/
bb write_solidity_verifier -k target/vk -o contracts/Verifier.sol

# And for Starknet verifier contracts:
bb prove --scheme ultra_honk --oracle_hash starknet --zk -b target/foo.json -w target/foo.gz -o target/
bb write_vk --oracle_hash starknet -b target/foo.json -o target/
garaga gen --system ultra_starknet_zk_honk --vk target/vk
garaga calldata --system ultra_starknet_zk_honk
```

**Problems with the current approach:**
- Commands overwrite each other's output files
- Different oracle hashes and flags for different targets
- Easy to forget required flags and parameters
- No organized artifact management

**bargo simplifies this to:**

```bash
# EVM Workflow
bargo build
bargo evm prove
bargo evm verify  
bargo evm gen
bargo evm calldata

# Starknet Workflow  
bargo build
bargo cairo prove
bargo cairo verify
bargo cairo gen
bargo cairo data
```

**How bargo improves on underlying tools:**
- **Organized Output**: Separate `target/evm/` and `target/starknet/` directories
- **Consistent Interface**: Same command patterns across different backends
- **Intelligent Defaults**: Automatically applies correct flags and parameters
- **Workflow Orchestration**: Chains related operations together
- **Error Prevention**: Validates prerequisites before running operations

## Commands

### Core Commands
- `bargo check` - Validate circuit syntax and dependencies
- `bargo build` - Generate bytecode and witness files
- `bargo clean` - Remove target directory and build artifacts
- `bargo rebuild` - Clean and rebuild from scratch
- `bargo doctor` - Check that all required tools are installed

### EVM Commands
- `bargo evm prove` - Generate proof and verification key with Keccak oracle
- `bargo evm verify` - Verify proof locally
- `bargo evm gen` - Generate Solidity verifier contract and Foundry project
- `bargo evm calldata` - Generate calldata for on-chain verification
- `bargo evm deploy` - Deploy verifier contract to EVM networks
- `bargo evm verify-onchain` - Verify proof on-chain

### Starknet Commands
- `bargo cairo prove` - Generate proof and verification key with Starknet oracle
- `bargo cairo verify` - Verify proof locally
- `bargo cairo gen` - Generate Cairo verifier contract using garaga
- `bargo cairo calldata` - Generate calldata for on-chain verification
- `bargo cairo declare` - Declare verifier contract on Starknet
- `bargo cairo deploy` - Deploy declared verifier contract
- `bargo cairo verify-onchain` - Verify proof on-chain

### Global Flags
- `--verbose` - Show underlying commands being executed
- `--dry-run` - Print commands without executing them
- `--pkg <name>` - Override package name (auto-detected from Nargo.toml)
- `--quiet` - Minimize output

## Installation

```bash
# Clone and build from source
git clone https://github.com/your-org/bargo
cd bargo
cargo install --path .

# Verify installation
bargo --help
bargo doctor  # Check dependencies
```

### EVM Setup (Optional)

```bash
# Install Foundry
curl -L https://foundry.paradigm.xyz | bash
foundryup

# Create .env file
echo "RPC_URL=https://eth-mainnet.g.alchemy.com/v2/your_key" >> .env
echo "PRIVATE_KEY=your_private_key" >> .env
```

### Starknet Setup (Optional)

**Requirements (read carefully to avoid 99% of issues!):**
- Garaga CLI Python package version 0.18.1 (install with `pip install garaga==0.18.1`)
- Noir 1.0.0-beta.4 (install with `noirup --version 1.0.0-beta.4` or `npm i @noir-lang/noir_js@1.0.0-beta.4`)
- Barretenberg 0.87.4-starknet.1 (install with `bbup --version 0.87.4-starknet.1` or `npm i @aztec/bb.js@0.87.4-starknet.1`)

**⚠️ Version Compatibility**: These may not be the latest versions of `bb` and `nargo`. You may need to switch between versions when generating Starknet artifacts vs EVM artifacts.

```bash
# Install specific versions
pip install garaga==0.18.1
noirup --version 1.0.0-beta.4
bbup --version 0.87.4-starknet.1

# Install starkli
curl https://get.starkli.sh | sh
starkliup

# Verify installations
garaga --help
nargo --version  # Should show 1.0.0-beta.4
bb --version     # Should show 0.87.4-starknet.1

# Create .secrets file for Starknet configuration
```
SEPOLIA_RPC_URL="https://free-rpc.nethermind.io/sepolia-juno"
SEPOLIA_ACCOUNT_PRIVATE_KEY=0x1
SEPOLIA_ACCOUNT_ADDRESS=0x2

MAINNET_RPC_URL="https://"
MAINNET_ACCOUNT_PRIVATE_KEY=0x3
MAINNET_ACCOUNT_ADDRESS=0x4
```

# Configure Starknet environment (see starkli documentation)
```

## Recommended Workflow

### EVM Development

```bash
# 1. Build your circuit
bargo build

# 2. Generate and verify proof
bargo evm prove
bargo evm verify

# 3. Generate Solidity verifier  
bargo evm gen

# 4. Deploy to testnet
bargo evm deploy --network sepolia

# 5. Generate calldata and verify on-chain
bargo evm calldata
bargo evm verify-onchain
```

### Starknet Development

```bash
# 1. Build your circuit  
bargo build

# 2. Generate and verify proof
bargo cairo prove
bargo cairo verify

# 3. Generate Cairo verifier
bargo cairo gen

# 4. Deploy to testnet
bargo cairo declare --network sepolia
bargo cairo deploy --network sepolia  

# 5. Generate calldata and verify on-chain
bargo cairo calldata
bargo cairo verify-onchain
```

### Cross-Chain Development

```bash
# Generate verifiers for both chains from same circuit
bargo build

# EVM verifier
bargo evm prove
bargo evm gen

# Starknet verifier  
bargo cairo prove
bargo cairo gen

# Artifacts are isolated in target/evm/ and target/starknet/
```

## Architecture

bargo organizes build artifacts in separate directories to prevent conflicts:

```
target/
├── bb/           # Core nargo build artifacts
│   ├── pkg.json  # Bytecode
│   └── pkg.gz    # Witness
├── evm/          # EVM-specific artifacts
│   ├── proof
│   ├── vk
│   ├── public_inputs
│   └── calldata.json
└── starknet/     # Starknet-specific artifacts
    ├── proof
    ├── vk  
    ├── public_inputs
    └── calldata.json

contracts/
├── evm/          # Foundry project with Solidity verifier
└── cairo/        # Cairo verifier project
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests: `cargo test`
5. Submit a pull request

### Development Setup

```bash
git clone https://github.com/your-org/bargo
cd bargo
cargo build
cargo test
```

## License

MIT License - see [LICENSE](LICENSE) file for details.
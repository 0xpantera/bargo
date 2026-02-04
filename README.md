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

### EVM Foundry Workflow (Optional, `evm-foundry` feature)
- **[Foundry](https://getfoundry.sh/)** - For Solidity contract deployment
- **Environment Variables**: `RPC_URL`, `PRIVATE_KEY` in `.env` file

### Starknet Workflow (Optional, `cairo` feature)
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
bb write_vk -b target/foo.json -o target/ -t evm
bb prove   -b target/foo.json -w target/foo.gz -o target/ -t evm -k target/vk
bb verify  -k target/vk -p target/proof -i target/public_inputs -t evm

# Plus remembering different flags for Solidity generation
bb write_vk -b target/foo.json -o target/ -t evm
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

# Starknet Workflow (requires `cairo` feature)  
bargo build
bargo cairo prove
bargo cairo verify
bargo cairo gen
bargo cairo calldata
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

### EVM Commands (Core)
- `bargo evm prove` - Generate proof and verification key with Keccak oracle
- `bargo evm verify` - Verify proof locally
- `bargo evm gen` - Generate Solidity verifier contract (Foundry project when enabled)
- `bargo evm calldata` - Generate calldata for on-chain verification

### EVM Commands (Foundry, `evm-foundry` feature)
- `bargo evm deploy` - Deploy verifier contract to EVM networks
- `bargo evm verify-onchain` - Verify proof on-chain

### Starknet Commands (`cairo` feature)
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

### Feature Flags

By default, `bargo` enables both the Cairo and Foundry feature sets. To disable them at compile time:

```toml
# Cargo.toml dependency example
bargo = { version = "0.2.0", default-features = false, features = ["cairo"] }
```

```bash
# Building locally
cargo build --no-default-features
cargo build --no-default-features --features cairo
cargo build --no-default-features --features evm-foundry
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
├── evm/          # Solidity verifier (Foundry project when enabled)
└── cairo/        # Cairo verifier project
```

## Errors

bargo provides rich error context to help you understand and fix issues quickly. All errors include:

- **Clear descriptions** of what went wrong
- **Contextual information** about the operation that failed
- **Actionable suggestions** for how to fix the problem
- **Error chains** that show the full path from root cause to symptom

### Error Categories

#### Project Configuration Errors
```
Error: Could not find Nargo.toml in current directory or any parent directory.
       Make sure you're running bargo from within a Noir project.
```

**Solution**: Navigate to your Noir project directory or create a new project with `nargo new <project_name>`.

#### Missing Dependencies
```
Error: Tool 'nargo' not found in PATH
       
Suggestions:
• Install nargo: curl -L https://raw.githubusercontent.com/noir-lang/noirup/main/install | bash
• Add nargo to your PATH
• Verify installation with `nargo --version`
```

**Solution**: Install the missing tool following the suggestions in the error message.

#### Missing Artifacts
```
Error: Required files are missing: target/bb/example.json, target/bb/example.gz

Suggestions:
• Run 'bargo build' to generate bytecode and witness files
• Ensure the previous workflow steps completed successfully
• Check that you're running from the correct directory
```

**Solution**: Run the suggested command to generate the missing files.

#### Tool Execution Failures
```
Error: Command execution failed: bb prove --scheme ultra_honk
   0: Command 'bb' failed with exit code 1
      Stdout: 
      Stderr: Error: Could not parse bytecode file
```

**Solution**: Check that your circuit compiles correctly with `bargo check` and that all input files are valid.

### Backend-Specific Errors

#### Cairo Backend Errors (requires `cairo` feature)
- **Deploy failures**: Issues with Starknet contract deployment
- **Class hash errors**: Problems with contract declaration
- **Garaga integration**: Tool-specific failures during contract generation

#### EVM Backend Errors  
- **Foundry integration**: Issues with Solidity compilation or deployment (requires `evm-foundry`)
- **Network errors**: Problems connecting to Ethereum networks
- **Contract compilation**: Solidity verifier generation failures

### Debugging Tips

1. **Use `--verbose` flag**: See exactly which commands are being executed
   ```bash
   bargo --verbose evm prove
   ```

2. **Check tool versions**: Ensure all dependencies are installed and compatible
   ```bash
   bargo doctor
   ```

3. **Use `--dry-run` flag**: See what commands would be executed without running them
   ```bash
   bargo --dry-run cairo deploy
   ```

4. **Clean and rebuild**: Start fresh if you encounter unexpected errors
   ```bash
   bargo clean
   bargo build
   ```

5. **Check environment configuration**: Verify `.env` and `.secrets` files are properly configured

### Common Issues

| Problem | Solution |
|---------|----------|
| "nargo not found" | Install Noir toolchain: `curl -L https://raw.githubusercontent.com/noir-lang/noirup/main/install | bash` |
| "bb not found" | Install Barretenberg: Follow Aztec installation docs |
| "garaga not found" | Install with pip: `pip install garaga==0.18.1` (only for `cairo`) |
| "forge not found" | Install Foundry: `curl -L https://foundry.paradigm.xyz \| bash && foundryup` (only for `evm-foundry`) |
| Version compatibility issues | Use `bargo doctor` to check versions and compatibility |
| Missing artifacts | Run prerequisite commands: `bargo build` → `bargo <backend> prove` |
| Network connection issues | Check RPC URLs and network configuration in `.env` |

### Getting Help

If you encounter an error not covered here:

1. Check the error message for specific suggestions
2. Run `bargo doctor` to verify your setup
3. Use `--verbose` to see detailed command execution
4. Search existing GitHub issues
5. Open a new issue with the full error output and your configuration

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

### Testing

bargo uses a comprehensive testing strategy with multiple test types:

#### Test Structure
- **Unit Tests**: Located in `crates/bargo-core/src/` alongside source code
- **Integration Tests**: Located in `tests/` directory with dedicated test files:
  - `tests/build_integration.rs` - Tests for `bargo build` workflow
  - `tests/cairo_integration.rs` - Tests for `cairo prove/gen` workflows
  - `tests/cli_smoke.rs` - Basic CLI command validation
  - `tests/auto_declare.rs` - Auto-declare functionality tests
  - `tests/error_context.rs` - Error handling and context tests

#### Integration Test Framework
Integration tests use `DryRunRunner` to verify command execution without running external tools:

```bash
# Run all tests
cargo test

# Run specific integration test suite
cargo test --test build_integration
cargo test --test cairo_integration

# Run individual test
cargo test --test build_integration test_build_command_dry_run
```

#### Golden File Snapshots
Integration tests compare generated directory structures against golden snapshots:

- **Fixtures**: `tests/fixtures/simple_circuit/` contains a minimal Noir project
- **Golden Snapshots**: `tests/goldens/simple_circuit_build/` contains expected build output
- **Cross-Platform**: Uses `path-slash` crate for consistent path handling across Windows/Unix

#### Refreshing Golden Snapshots
When build output format changes, update golden files:

1. Manually run a real build: `cd tests/fixtures/simple_circuit && nargo execute`
2. Copy generated `target/` directory to `tests/goldens/simple_circuit_build/`
3. Normalize paths using forward slashes for cross-platform compatibility
4. Commit updated golden files

#### Thread Safety
Integration tests use `ScopedDir` guards to prevent race conditions when running in parallel, ensuring each test operates in an isolated directory context.

## License

MIT License - see [LICENSE](LICENSE) file for details.

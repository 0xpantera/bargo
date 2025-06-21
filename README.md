# bargo

A developer-friendly CLI wrapper for Noir ZK development that consolidates `nargo` and `bb` workflows into a single, opinionated tool.

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
```

**bargo simplifies this to:**

```bash
bargo build         # ← check + execute
bargo prove         # ← prove + write_vk + verify (unless --skip-verify)
bargo solidity      # ← write_vk (keccak) + write_solidity_verifier
bargo verify        # ← explicit re-verification
bargo clean         # ← rm -rf target/
bargo rebuild       # ← clean + build in one step
```

## Command Specification

| Command | Underlying Tools | Default Behavior | Key Features |
|---------|------------------|------------------|--------------|
| `bargo check` | `nargo check` | Syntax & dependency validation | ✅ Error passthrough |
| `bargo build` | `nargo execute` | Generate bytecode + witness | 🔄 Smart rebuild detection |
| `bargo prove` | `bb prove` + `bb write_vk` + `bb verify` | End-to-end proving with verification | ⚡ `--skip-verify` flag available |
| `bargo verify` | `bb verify` | Re-verify existing proof | 📁 Auto-detect proof/vk paths |
| `bargo solidity` | `bb write_vk --oracle_hash keccak` + `bb write_solidity_verifier` | Generate Solidity verifier contract | 🎯 Optimized for Ethereum deployment |
| `bargo clean` | `rm -rf target/` | Remove all build artifacts | 🧹 Fresh start for debugging |
| `bargo rebuild` | `rm -rf target/` + `nargo execute` | Clean and rebuild from scratch | 🔄 Combined clean + build operation |

### Global Flags

- `-v, --verbose` → Show underlying commands being executed + set `RUST_LOG=info`
- `--dry-run` → Print commands without executing
- `--pkg <name>` → Override package name (auto-detected from `Nargo.toml`)
- `-q, --quiet` → Minimal output

## Features Checklist

### Core Commands
- [x] `bargo check` - nargo check wrapper
- [x] `bargo build` - nargo execute wrapper  
- [x] `bargo prove` - bb prove + write_vk + verify chain
- [x] `bargo verify` - bb verify wrapper
- [x] `bargo verifier` - Solidity verifier generation
- [x] `bargo clean` - target directory cleanup (with `--backend` support)
- [x] `bargo rebuild` - clean + build in one command (with `--backend` support)
- [x] `bargo doctor` - dependency verification tool

### Cairo Commands (requires garaga) - ✅ FULLY TESTED ON MAINNET
- [x] `bargo cairo gen` - generate optimized Cairo verifier contract for Starknet
- [x] `bargo cairo data` - generate calldata JSON for proof verification  
- [x] `bargo cairo declare` - declare verifier contract on Starknet (auto-saves class hash)
- [x] `bargo cairo deploy` - deploy declared verifier contract (auto-uses saved class hash)
- [x] `bargo cairo verify-onchain` - verify proof on-chain using deployed verifier (auto-uses saved contract address)

### CLI Infrastructure  
- [x] Clap-based command parsing
- [x] Global flags (`--verbose`, `--dry-run`, `--pkg`, `--quiet`)
- [x] Colored output and progress indicators
- [x] Error handling with context

### Path Intelligence
- [x] Auto-detect package name from `Nargo.toml`
- [x] Resolve target paths (`target/{pkg}.json`, `target/{pkg}.gz`)
- [x] Find project root (walk up directory tree for `Nargo.toml`)
- [x] Validate required files exist before running commands

### Smart Features
- [x] Smart rebuilds - Track file timestamps, auto-clean and rebuild when needed
- [x] Dependency-aware invalidation - Detect changes in `Nargo.toml` or source files
- [x] `bargo build` automatically handles stale artifacts
- [x] Multi-backend support - Separate `target/bb/` and `target/starknet/` directories
- [x] Backend-aware cleaning - Clean specific backends with `--backend` flag

### User Experience
- [x] Rich terminal output (emojis, colors, progress)
- [x] Verbose logging shows actual commands executed
- [x] Helpful error messages with suggested fixes
- [x] Integration tests with real Noir project
- [x] ASCII art headers - Aesthetic section separators for command output
- [x] File sizes & timing - Show file sizes and operation duration for all commands
- [x] Operation summaries - Professional summary showing what was accomplished

## Installation

```bash
# Clone and build
git clone <repo-url>
cd bargo
cargo build --release

# Add to PATH or use directly
./target/release/bargo --help
```

### Cairo/Starknet Support (Optional)

For Cairo verifier generation and Starknet deployment features, you'll also need garaga:

```bash
# Install garaga (requires Python 3.10+)
pipx install garaga

# Or with pip in a virtual environment
python -m venv .venv
source .venv/bin/activate  # On Windows: .venv\Scripts\activate
pip install garaga

# Verify installation
garaga --help
```

**Note**: All EVM/Solidity features work without garaga. Cairo features (`bargo cairo ...`) require garaga to be installed.

#### Starknet Environment Setup

Create a `.secrets` file in your project root for Starknet deployment:

```bash
# .secrets file format
SEPOLIA_RPC_URL="https://starknet-sepolia.g.alchemy.com/starknet/version/rpc/v0_8/your_key"
SEPOLIA_ACCOUNT_PRIVATE_KEY=0x...
SEPOLIA_ACCOUNT_ADDRESS=0x...

MAINNET_RPC_URL="https://starknet-mainnet.g.alchemy.com/starknet/version/rpc/v0_8/your_key"  
MAINNET_ACCOUNT_PRIVATE_KEY=0x...
MAINNET_ACCOUNT_ADDRESS=0x...
```

## Usage Examples

### Check Dependencies

```bash
# Verify all tools are installed
bargo doctor       # ✅ nargo: /usr/local/bin/nargo
                   # ✅ bb: /usr/local/bin/bb  
                   # ✅ garaga: /usr/local/bin/garaga
                   # 🎉 All required dependencies are available!
```

### Basic Development Workflow (EVM/Solidity)

```bash
# In a Noir project directory
bargo check        # ✓ All packages OK
bargo build        # ✓ Bytecode → target/bb/wkshp.json, Witness → target/bb/wkshp.gz  
bargo prove        # ✓ Proof generated → target/bb/proof (13.8 KB)
                   # ✓ VK saved → target/bb/vk
                   # ✅ Proof verified successfully
```

### EVM Verifier Generation

```bash
bargo verifier     # ✓ VK (keccak) → target/bb/vk
                   # ✓ Verifier contract → contracts/Verifier.sol
```

### Cairo/Starknet Workflow (requires garaga) - ✅ PRODUCTION READY

**Complete End-to-End Workflow:**

```bash
# 1. Generate optimized Cairo verifier (uses ultra_starknet_zk_honk + --zk flag)
bargo cairo gen    # ✓ Starknet proof → target/starknet/proof (15.8 KB)
                   # ✓ Starknet VK → target/starknet/vk (1.7 KB)  
                   # ✓ Cairo verifier → contracts/Verifier.cairo (11.2 KB)
                   # ✓ Optimized for maximum gas efficiency

# 2. Generate calldata for verification  
bargo cairo data   # ✓ Calldata JSON output (thousands of field elements)

# 3. Seamless deployment workflow (no manual copying needed!)
bargo cairo declare --network mainnet     # ✓ Contract declared → auto-saves class hash
bargo cairo deploy                        # ✓ Contract deployed → auto-uses saved class hash  
bargo cairo verify-onchain               # ✅ Proof verified on-chain → auto-uses saved address

# Real example from successful mainnet deployment:
# ✓ Class hash: 0x5ff378cb2f16804539ecb92e84f273aafbab57d450530e9fe8e87771705a673
# ✓ Contract: 0x65bf3f2391439511353ca05dda89acaa82956ad7f871152f345b7917e0a2f34  
# ✓ Verification TX: 0x59c37878e8851336a75e0d6eb0c49977de74a8264e52b75e8d2e449cf1365ba
```

**Key Features:**
- **Maximum Optimization**: Uses `ultra_starknet_zk_honk` system with proper `--zk` flag
- **Auto State Management**: Class hashes and contract addresses saved automatically
- **Network Selection**: `--network mainnet` or `--network sepolia` 
- **Error Handling**: Honest feedback - no fake success messages
- **Gas Efficient**: Starknet-optimized proofs reduce deployment costs

### Development Iteration

```bash
# Edit your circuit
vim src/main.nr

bargo build        # 🔄 Auto-detects changes, rebuilds automatically
bargo prove        # ✓ New proof with updated circuit
```

### Cross-Backend Management

```bash
# Clean specific backends
bargo clean --backend bb       # 🧹 Remove only EVM artifacts
bargo clean --backend starknet # 🧹 Remove only Cairo artifacts  
bargo clean                    # 🧹 Remove all artifacts (default)

# Backend-aware rebuild
bargo rebuild --backend bb     # 🔄 Clean + build EVM only
bargo rebuild                  # 🔄 Clean + build everything
```

### Debugging Workflow

```bash
bargo rebuild      # 🔄 Clean + build in one step
bargo prove --skip-verify  # ⚡ Skip verification for faster iteration
bargo verify       # ✅ Verify when ready

# Or step-by-step:
bargo clean        # 🧹 Removed target/
bargo build        # ✓ Fresh build
```

## Technical Implementation

### Architecture

```
bargo/
├── src/
│   ├── main.rs           # CLI entry point & command routing
│   ├── util.rs           # Path resolution & Nargo.toml parsing  
│   └── backends/
│       ├── mod.rs        # Common backend utilities
│       ├── nargo.rs      # nargo command wrappers
│       └── bb.rs         # bb command wrappers
└── tests/
    └── integration.rs    # End-to-end testing
```

### Key Design Decisions

- **No FFI**: Spawn existing binaries with `std::process::Command` for rapid prototyping
- **Path Convention**: Follow Noir's `target/{package_name}.{json,gz}` pattern
- **Opinionated Defaults**: `--oracle_hash keccak` for Solidity, auto-verify after proving
- **Error Transparency**: Pass through raw tool output while adding helpful context

### Dependencies

- `clap` (derive) - Declarative CLI parsing
- `color-eyre` - Beautiful error reporting with stack traces  
- `tracing` + `tracing-subscriber` - Structured logging for `--verbose` mode
- `serde` + `toml` - Parse `Nargo.toml` for package metadata

## Future Roadmap

### Smart Features
- [ ] **Parallel execution**: Run independent bb commands concurrently

### Advanced UX
- [ ] **Progress bars**: Show progress for long-running operations (using `indicatif`)
- [ ] **Tool version detection**: Show nargo/bb versions in verbose mode
- [ ] **Better dry-run visualization**: Enhanced workflow preview with dependencies
- [ ] **Auto-completion**: Shell completion for bash/zsh/fish
- [ ] **Configuration files**: `.bargorc` for project-specific defaults

### Integration Features  
- [ ] **Watch mode**: `bargo watch` - auto-rebuild on file changes
- [ ] **Benchmark tracking**: Track proof generation time across builds
- [ ] **Multi-package support**: Handle Noir workspaces with multiple packages
- [ ] **CI/CD integration**: GitHub Actions workflow templates

### Distribution & Adoption
- [ ] **Package for distribution**: Cargo install, Homebrew, pre-built binaries
- [ ] **Documentation**: Comprehensive guides and API documentation
- [ ] **Example projects**: Curated collection of bargo-ready Noir circuits
- [ ] **Tutorial content**: Blog posts, videos, and getting-started guides
- [ ] **Community building**: Discord, forum presence, and developer outreach

### Performance Optimizations
- [ ] **Feature-gated in-process nargo**: Pure Rust integration (larger binary)
- [ ] **Tiny FFI island**: Direct bb integration for hot paths
- [ ] **Caching**: Intelligent artifact caching between builds
- [ ] **Incremental compilation**: Only rebuild changed components

## Contributing

This project is designed for rapid iteration during hackathons and development sprints. The codebase prioritizes:

1. **Clarity over cleverness** - readable code that's easy to modify
2. **User experience** - developers should love using this tool  
3. **Reliability** - robust error handling and helpful messages
4. **Extensibility** - easy to add new commands and features

### Development Setup

```bash
# Test with the included example project
cd wkshp
../target/debug/bargo build
../target/debug/bargo prove
```

## Production-Ready Cairo/Starknet Integration

bargo has been **successfully tested end-to-end** on Starknet mainnet with real contract deployments and proof verifications. The Cairo integration provides:

### 🎯 **Optimization Levels**
- `ultra_keccak_honk` - Standard Keccak-based proofs  
- `ultra_starknet_honk` - Starknet-optimized with Poseidon hash
- `ultra_starknet_zk_honk` - **Maximum optimization** with ZK proofs (recommended)

### 🔄 **Seamless State Management**  
- Class hashes automatically saved in `target/starknet/.bargo_class_hash`
- Contract addresses automatically saved in `target/starknet/.bargo_contract_address`
- No manual copying of hashes between commands

### ⚡ **Gas Optimization**
- Starknet-native hash functions reduce verification costs
- ZK proofs minimize contract bytecode size
- Proper `bb prove -s ultra_honk --oracle_hash starknet --zk` flags

### 🛡️ **Reliable Error Handling**
- Honest failure reporting (no fake success messages)
- Helpful troubleshooting suggestions
- Network-aware error detection

## Target User Persona

- **Web3/crypto developer** building ZK applications on Ethereum and Starknet
- **Comfortable with Rust & command lines** but wants simplified ZK tooling
- **Appreciates lavish terminal UX**: colors, emojis, verbose explanations  
- **Values developer velocity** and reliable end-to-end workflows
- **Needs production-ready ZK verification** on both EVM and Starknet

---

*bargo: Because ZK development should be delightful, not a chore.*
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [v0.2.0]

### Added
- Runner abstraction with `CmdSpec`, `Runner` trait, `RealRunner`, and `DryRunRunner` implementations
- `run_capture()` method to Runner trait for stdout capture functionality
- Command history tracking in `DryRunRunner` with `history()` and `clear_history()` methods
- Unified `run_tool()` and `run_tool_capture()` helpers for all external command execution
- Auto-declare functionality for Cairo deploy command with `--auto-declare` and `--no-declare` flags
- Realistic fake output generation in `DryRunRunner::run_capture()` for better testing
- Enhanced command history tracking in `DryRunRunner` with captured output storage
- Backend trait methods now use `&mut self` for stateful operations (mutability upgrade)
- Comprehensive test coverage for auto-declare functionality (10 new tests)
- Comprehensive test coverage for runner abstraction (14 new unit tests)
- New `run_nargo_command` helper in `commands::common` for consolidated command execution
- Comprehensive smoke test coverage for all major commands (14 tests total)
- Test cases for check, clean, rebuild, cairo gen, and evm gen commands
- Test coverage for verbose, dry-run, and package flag combinations
- Comprehensive rustdoc documentation for new helper functions
- Integration test framework with golden file snapshots for `bargo build` and `cairo prove` workflows
- Cross-platform path handling in integration tests using `path-slash` crate
- Thread-safe test execution with `ScopedDir` guard to prevent parallel test race conditions
- Fixture-based testing with `simple_circuit` test project in `tests/fixtures/`
- Golden snapshot comparison for build artifacts in `tests/goldens/` directory
- `DryRunRunner`-based integration tests that verify command execution without running external tools

### Changed
- Command execution now uses runner abstraction for consistent dry-run and real execution modes
- All external tool commands (bb, garaga, forge, nargo) now use unified `run_tool()` interface
- Stdout capture operations (garaga calldata, foundry deploy) now use `run_tool_capture()`
- Config struct now includes `runner` field with `Arc<dyn Runner>` for command execution
- `DryRunRunner::run_capture()` now returns tool-specific fake output instead of generic placeholder
- `DryRunRunner::history()` now returns `Vec<(CmdSpec, Option<String>)>` to include captured output
- Migrated `run_nargo_command` to use runner abstraction instead of direct backend calls
- Cairo deploy command now auto-declares contracts by default (improves newcomer experience)
- Backend implementations now support stateful operations through mutable references
- Backend configuration now uses `configure()` method instead of down-casting for type-specific settings
- Improved dry-run mode handling for Cairo workflows
- Refactored `check` command to use consolidated helper pattern (reduced from 15 to 2 lines)
- Refactored `build` and `rebuild` commands to use consolidated helper pattern
- Improved quiet flag handling in verbose logging output

### Removed
- `cairo declare` command (use `cairo deploy --auto-declare` instead for automatic declaration and deployment)
- `as_any_mut()` method and RTTI down-casting from Backend trait (replaced with `configure()` method)
- All legacy backend helper functions (`backends::bb::run`, `backends::nargo::run`, etc.)
- Direct `std::process::Command` usage outside of `runner.rs` module
- Tool-specific command helpers (replaced with unified `run_tool()` interface)
- Enhanced global flag propagation consistency across all commands
- Deprecated helper functions: `run_bb_command()`, `run_garaga_command()`, `run_foundry_command()`
- Legacy `tests/integration.rs` single-file integration test (replaced with modular `tests/*_integration.rs` files)
- All commands now properly honor --pkg, --verbose, --dry-run, and --quiet flags

### Fixed
- Removed unused imports causing compiler warnings
- Fixed code to compile cleanly with `RUSTFLAGS="-D warnings"`
- Ensured dry-run mode properly bypasses filesystem operations and external commands
- Fixed test failures for cairo and evm commands by providing required package overrides

### Internal
- Consolidated argument building, logging, and dry-run handling patterns
- Eliminated code duplication across command modules
- Established consistent command execution pattern for all nargo-based commands
- Verified no unsafe blocks remain in codebase
- All acceptance criteria for Phase 3 satisfied

## [Unreleased]

### Added
- Compile-time feature gating for Cairo and Foundry functionality via Cargo features (`cairo`, `evm-foundry`)
- Consolidated architecture overview (`ARCHITECTURE_OVERVIEW.md`)
- EVM calldata generation now derives JSON from binary proof + public inputs

### Changed
- EVM BB CLI integration updated to use verifier target (`-t evm`) and VK-first prove flow
- `bargo evm gen` only initializes Foundry when `evm-foundry` is enabled
- `bargo doctor` checks feature-gated tools only when their features are enabled
- Tests for Cairo workflows are feature-gated; EVM deploy/on-chain commands are hidden when `evm-foundry` is off
- Architecture and reference docs updated for new BB CLI behavior and feature gating

### Removed
- `ARCHITECTURE_SUMMARY.txt` (merged into `ARCHITECTURE_OVERVIEW.md`)

### Fixed
- `bargo evm prove` now works with bb 3.x CLI (removal of `--output_format`)

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Runner abstraction with `CmdSpec`, `Runner` trait, `RealRunner`, and `DryRunRunner` implementations
- Auto-declare functionality for Cairo deploy command with `--auto-declare` and `--no-declare` flags
- Backend trait methods now use `&mut self` for stateful operations (mutability upgrade)
- Comprehensive test coverage for auto-declare functionality (10 new tests)
- Comprehensive test coverage for runner abstraction (10 new unit tests)
- New `run_nargo_command` helper in `commands::common` for consolidated command execution
- Comprehensive smoke test coverage for all major commands (14 tests total)
- Test cases for check, clean, rebuild, cairo gen, and evm gen commands
- Test coverage for verbose, dry-run, and package flag combinations
- Comprehensive rustdoc documentation for new helper functions

### Changed
- Command execution now uses runner abstraction for consistent dry-run and real execution modes
- Config struct now includes `runner` field with `Arc<dyn Runner>` for command execution
- Migrated `run_nargo_command` to use runner abstraction instead of direct backend calls
- Cairo deploy command now auto-declares contracts by default (improves newcomer experience)
- Cairo declare command is now deprecated (use `cairo deploy` with auto-declare instead)
- Backend implementations now support stateful operations through mutable references
- Improved dry-run mode handling for Cairo workflows
- Refactored `check` command to use consolidated helper pattern (reduced from 15 to 2 lines)
- Refactored `build` and `rebuild` commands to use consolidated helper pattern
- Improved quiet flag handling in verbose logging output
- Enhanced global flag propagation consistency across all commands
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
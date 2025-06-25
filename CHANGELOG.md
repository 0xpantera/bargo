# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- New `run_nargo_command` helper in `commands::common` for consolidated command execution
- Comprehensive smoke test coverage for all major commands (14 tests total)
- Test cases for check, clean, rebuild, cairo gen, and evm gen commands
- Test coverage for verbose, dry-run, and package flag combinations
- Comprehensive rustdoc documentation for new helper functions

### Changed
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
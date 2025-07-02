# Changelog

## [0.4.0] - 2025-01-XX
### Changed
- **BREAKING**: Rebranded project from RealMIR to Cliptions
- Updated package name from `realmir-core` to `cliptions-core`
- Updated library name from `realmir_core` to `cliptions_core`
- Updated all binary names with `cliptions_` prefix
- Updated social media handles: `realmir_testnet` → `cliptions_test`
- Updated repository URL to `grasslandnetwork/cliptions`
- Updated all documentation, code comments, and module descriptions
- Updated error types: `RealMirError` → `CliptionsError`
- Updated config structs: `RealMirConfig` → `CliptionsConfig`
- Updated Python bridge module name and all imports
- Updated hashtag references: `#RealMir` → `#Cliptions`

## [0.3.0] - 2025-04-13
### Added
- Comprehensive testing across diverse images with multiple target captions.

### Changed
- Improved CLIP scoring system: removed special character penalties in favor of a simpler baseline adjustment.
- Simplified text validation to only check for empty strings and maximum length.
- Updated tests to better match real-world CLIP model performance.

### Fixed
- Fixed dependency management with proper categorization in `requirements.txt`.
- Removed unused 'clip' import and updated `requirements.txt`.

### Security
- Improved CLIP scoring to prevent exploits and increase accuracy.

[0.3.0]: https://github.com/grasslandnetwork/cliptions/releases/tag/0.3.0 
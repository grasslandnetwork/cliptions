# Product Requirements Document: RealMIR → Cliptions Rebrand

## Introduction/Overview

This document outlines the complete rebranding of the RealMIR application to "Cliptions" within the existing codebase. The rebrand involves updating all technical components, naming conventions, and references throughout the codebase to reflect the new brand identity. The name "Cliptions" is phonetically easier than RealMIR, closely associated with the CLIP AI model being used, relates to "captions" that users create, and includes "clip" referencing short video files.

## Goals

1. **Complete Name Transition**: Replace all instances of "RealMIR", "realmir", "realMIR" with "Cliptions", "cliptions" throughout the codebase
2. **Update Social Media References**: Change Twitter/X handle references from "realmir_ai" → "cliptions" and "realmir_testnet" → "cliptions_test"
3. **Update Repository References**: Change GitHub repository references from "grasslandnetwork/realmir" to "grasslandnetwork/cliptions"
4. **Update Domain References**: Replace any domain references with "cliptions.com"
5. **Maintain Code Functionality**: Ensure all functionality remains intact after the rebrand
6. **Zero Downtime**: Complete rebrand without breaking existing functionality

## User Stories

1. **As a developer**, I want all code references to use "Cliptions" naming so that the codebase is consistent with the new brand
2. **As a developer**, I want file names and directory structures to reflect "Cliptions" so that the project structure is intuitive
3. **As a developer**, I want class names, function names, and variables to use "Cliptions" naming conventions so that code is self-documenting
4. **As a system administrator**, I want configuration files to reference "Cliptions" so that deployments use the correct branding
5. **As a maintainer**, I want documentation to reference "Cliptions" so that new contributors understand the current brand

## Functional Requirements

### 1. File and Directory Naming
1.1. **DO NOT** rename the project root directory - keep it as "realmir" to avoid Cursor IDE issues
1.2. Update any file names containing "realmir" to use "cliptions" (except the root directory)
1.3. Update subdirectory names containing "realmir" to use "cliptions" (except the root directory)

### 2. Code Content Updates
2.1. Replace all string literals containing "RealMIR", "realmir", "realMIR" with appropriate "Cliptions"/"cliptions" variants
2.2. Update class names from RealMIR-based naming to Cliptions-based naming
2.3. Update function names from realmir-based naming to cliptions-based naming
2.4. Update variable names from realmir-based naming to cliptions-based naming
2.5. Update constant definitions to use Cliptions naming

### 3. Configuration Files
3.1. Update package.json, Cargo.toml, requirements.txt to use "cliptions" as project name
3.2. Update any configuration files referencing "realmir" to use "cliptions"
3.3. **DO NOT** change environment variable names in configuration files (e.g., ./config/config.yaml) - keep existing environment variable names as-is

### 4. Documentation Updates
4.1. Update README.md to reference "Cliptions" throughout
4.2. Update CHANGELOG.md, CONTRIBUTING.md, and other documentation files
4.3. Update inline code comments referencing RealMIR
4.4. Update docstrings and documentation strings

### 5. External References
5.1. Update GitHub repository references from "grasslandnetwork/realmir" to "grasslandnetwork/cliptions"
5.2. Update Twitter handle references:
    - "realmir_ai" → "cliptions"
    - "realmir_testnet" → "cliptions_test"
5.3. Update domain references to "cliptions.com"

### 6. Asset and Media Files
6.1. Rename image files containing "realMIR" to use "Cliptions" naming
6.2. Update any text within image files that can be easily modified (if applicable)

### 7. Test Files
7.1. Update test file names containing "realmir" to use "cliptions"
7.2. Update test case names and descriptions to reference Cliptions
7.3. Update test data and fixtures to use Cliptions naming

### 8. Build and Deployment
8.1. Update build scripts to use Cliptions naming
8.2. Update any deployment configurations or Docker files
8.3. Update binary names in Cargo.toml and build outputs

## Non-Goals (Out of Scope)

1. **External Asset Creation**: Creating new logos, visual designs, or branding materials
2. **User Notification**: Communicating the rebrand to existing users
3. **External System Updates**: Updating external services, databases, or third-party integrations
4. **Domain Migration**: Actual domain name changes or DNS updates
5. **Social Media Account Management**: Creating or managing new social media accounts
6. **Marketing Material Updates**: Updating external marketing or promotional materials

## Design Considerations

- **Case Sensitivity**: Maintain appropriate casing conventions:
  - "Cliptions" for proper nouns and brand references
  - "cliptions" for technical identifiers, file names, and URLs
  - "CLIPTIONS" for constants and environment variables
- **Consistency**: Ensure naming conventions are consistent across all file types (Python, Rust, Markdown, JSON, etc.)
- **Backwards Compatibility**: Consider if any APIs or interfaces need to maintain backwards compatibility during transition

## Technical Considerations

1. **Multi-language Codebase**: The project contains both Python and Rust code - ensure updates are applied consistently across both languages
2. **Configuration Management**: Update YAML, JSON, and TOML configuration files appropriately
3. **Import Statements**: Update Python import statements that reference modules with "realmir" in the name
4. **Rust Module Structure**: Update Rust module declarations and use statements
5. **Binary Names**: Update Rust binary names in Cargo.toml to reflect new branding

## Success Metrics

1. **Completion Rate**: 100% of "realmir" references updated to "cliptions" equivalents
2. **Build Success**: All build processes (Cargo build, Python tests) complete successfully after rebrand
3. **Test Pass Rate**: All existing tests continue to pass after rebrand
4. **Consistency Check**: No mixed references between old and new naming exist in the codebase
5. **Documentation Accuracy**: All documentation accurately reflects the new Cliptions branding

## Implementation Priority

### Phase 1: Core Technical Components (Highest Priority)
- Update Cargo.toml and package configurations
- Update Rust module names and binary names
- Update Python module and package names
- Update configuration files

### Phase 2: Code Content
- Update class names, function names, and variables
- Update string literals and constants
- Update import statements and module references

### Phase 3: Documentation and Assets
- Update all documentation files
- Update comments and docstrings
- Rename asset files

### Phase 4: Verification
- Run comprehensive tests
- Verify build processes
- Check for any missed references

## Open Questions

1. Should we maintain any reference to "RealMIR" in historical documentation (like changelogs) or update everything?
2. Are there any external dependencies or libraries that reference "realmir" that need coordination?
3. Should Git commit messages or Git history be updated to reference the new name?

## Acceptance Criteria

- [ ] All file names containing "realmir" have been renamed to use "cliptions"
- [ ] All code references to RealMIR/realmir have been updated to Cliptions/cliptions
- [ ] All configuration files reference "cliptions" instead of "realmir"
- [ ] All documentation has been updated to use Cliptions branding
- [ ] All social media and external references have been updated
- [ ] Project builds successfully with new naming
- [ ] All tests pass with new naming
- [ ] No residual "realmir" references exist in the codebase (verified by comprehensive search) 
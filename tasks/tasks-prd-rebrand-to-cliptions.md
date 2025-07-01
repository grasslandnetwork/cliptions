# Task List: RealMIR → Cliptions Rebrand

Based on PRD: `prd-rebrand-to-cliptions.md`

## Relevant Files

- `Cargo.toml` - Update package name and binary names to use "cliptions" branding
- `requirements.txt` - Update Python package references if any contain "realmir"
- `README.md` - Update all brand references throughout the documentation
- `CHANGELOG.md` - Update brand references in changelog entries
- `CONTRIBUTING.md` - Update brand references in contribution guidelines
- `src/lib.rs` - Update module declarations and any realmir references
- `src/bin/*.rs` - Update binary names and internal references
- `src/*.rs` - Update all Rust source files for naming consistency
- `browser/*.py` - Update all Python browser automation files
- `core/*.py` - Update all core Python modules
- `tests/*.py` - Update Python test files
- `tests/*.rs` - Update Rust test files
- `media/realMIR_*` - Rename media assets to use Cliptions branding

### Notes

- **DO NOT** rename the project root directory to avoid Cursor IDE issues
- **DO NOT** change environment variable names in configuration files (e.g., ./config/llm.yaml)
- Maintain existing functionality while updating branding
- Use case-sensitive replacements: "Cliptions" for proper nouns, "cliptions" for technical identifiers
- Run tests after each major change to ensure functionality remains intact

## Tasks

- [ ] 1.0 Update Core Configuration Files
  - [x] 1.1 Update `Cargo.toml` package name from "realmir" to "cliptions"
  - [x] 1.2 Update binary names in `Cargo.toml` to use "cliptions" naming
  - [x] 1.3 Check and update `requirements.txt` for any realmir references
  - [x] 1.4 Update any configuration files that reference "realmir" (excluding environment variable names)
  - [x] 1.5 Test that the project still builds after configuration changes

- [ ] 2.0 Update Code Content and References
  - [ ] 2.1 Update all Rust source files (`src/*.rs`) for realmir → cliptions references
    - [ ] 2.1.1 Update string literals containing "RealMIR", "realmir", "realMIR"
    - [ ] 2.1.2 Update struct names, function names, and variable names
    - [ ] 2.1.3 Update module declarations and use statements
    - [ ] 2.1.4 Update constants and static variables
  - [ ] 2.2 Update all Python browser files (`browser/*.py`)
    - [ ] 2.2.1 Update class names and function names containing realmir
    - [ ] 2.2.2 Update string literals and constants
    - [ ] 2.2.3 Update import statements referencing realmir modules
    - [ ] 2.2.4 Update docstrings and comments
  - [ ] 2.3 Update all Python core files (`core/*.py`)
    - [ ] 2.3.1 Update class names and function names containing realmir
    - [ ] 2.3.2 Update string literals and constants
    - [ ] 2.3.3 Update import statements referencing realmir modules
    - [ ] 2.3.4 Update docstrings and comments
  - [ ] 2.4 Update social media handle references
    - [ ] 2.4.1 Change "realmir_ai" → "cliptions"
    - [ ] 2.4.2 Change "realmir_testnet" → "cliptions_test"
  - [ ] 2.5 Update GitHub repository references from "grasslandnetwork/realmir" to "grasslandnetwork/cliptions"

- [ ] 3.0 Update Documentation Files
  - [ ] 3.1 Update `README.md`
    - [ ] 3.1.1 Replace all "RealMIR" references with "Cliptions"
    - [ ] 3.1.2 Update project description and branding
    - [ ] 3.1.3 Update any repository links
    - [ ] 3.1.4 Update social media references
  - [ ] 3.2 Update `CHANGELOG.md`
    - [ ] 3.2.1 Update brand references in changelog entries
    - [ ] 3.2.2 Add entry for the rebrand itself
  - [ ] 3.3 Update `CONTRIBUTING.md`
    - [ ] 3.3.1 Replace RealMIR references with Cliptions
    - [ ] 3.3.2 Update any repository or project references
  - [ ] 3.4 Update `BROWSER_AUTOMATION_DEVELOPMENT.md` and other documentation files
  - [ ] 3.5 Update inline code comments throughout the codebase
  - [ ] 3.6 Update any notebook files (`*.ipynb`) for brand references

- [ ] 4.0 Update Asset and Media Files
  - [ ] 4.1 Rename `media/realMIR_logo.png` → `media/cliptions_logo.png`
  - [ ] 4.2 Rename `media/realMIR_profile_pic_with_name.png` → `media/cliptions_profile_pic_with_name.png`
  - [ ] 4.3 Rename `media/realMIR_profile_pic.png` → `media/cliptions_profile_pic.png`
  - [ ] 4.4 Update any code references to the renamed media files
  - [ ] 4.5 Check for any other asset files that may contain realmir references

- [ ] 5.0 Verify and Test Changes
  - [ ] 5.1 Run `cargo build` to ensure Rust code compiles successfully
  - [ ] 5.2 Run `cargo test` to ensure all Rust tests pass
  - [ ] 5.3 Run Python tests to ensure all Python functionality works
  - [ ] 5.4 Search codebase for any remaining "realmir" references using grep/ripgrep
  - [ ] 5.5 Verify that all renamed files are properly referenced
  - [ ] 5.6 Test basic application functionality to ensure no regressions
  - [ ] 5.7 Create a comprehensive list of all changes made for documentation 
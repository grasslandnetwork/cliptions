# Product Requirements Document: State Machine Refactor & MockEmbedder Removal

## 1. Introduction/Overview

This PRD outlines a comprehensive refactor of the Cliptions codebase to implement a proper typed state machine architecture and remove the MockEmbedder entirely from the application. The refactor follows the "Keep It Boring, Then Abstract" philosophy with a phased approach to ensure stability and maintainability.

**Problem**: The current codebase uses a simple `BlockStatus` enum for state management, which allows invalid state transitions and lacks compile-time guarantees. Additionally, the MockEmbedder is used throughout production code, creating inconsistencies and unreliable behavior.

**Goal**: Implement a robust typestate pattern for block lifecycle management and ensure all operations use real CLIP embeddings for consistent, production-ready behavior.

## 2. Goals

1. **Phase 1**: Implement comprehensive facade pattern to centralize logic and prepare for struct unification
2. **Phase 2**: Implement centralized path management system for configuration and data files
3. **Phase 3**: Replace MockEmbedder with real CLIP embeddings across all components
4. **Phase 4**: Unify `BlockData` and `Block<S>` into single typestate-enabled struct
5. **Phase 5**: Implement full typestate pattern with compile-time state transition enforcement
6. **Maintain Backward Compatibility**: Ensure JSON serialization/deserialization continues to work with existing data
7. **Improve Reliability**: Eliminate mock behavior from production code paths
8. **Enhance Developer Experience**: Provide clear, compile-time enforced state transitions

## 3. User Stories

- **As a Developer**, I want compile-time guarantees that block state transitions are valid, so that I can avoid runtime errors that could affect payouts
- **As a System Administrator**, I want all embedding operations to use real CLIP models, so that scoring is consistent and reliable
- **As a Maintainer**, I want centralized state logic through facades, so that I can modify state behavior without touching dozens of files
- **As a Developer**, I want tests to use real CLIP models, so that test results accurately reflect production behavior

## 4. Functional Requirements

### FR1: MockEmbedder Complete Removal
The application **must** remove all MockEmbedder usage:
1. Remove `MockEmbedder` struct and implementation from `src/embedder.rs`
2. Remove all `--use-mock` CLI flags and related logic
3. Update all tests to use real CLIP embeddings
4. Remove MockEmbedder imports and usage from all binaries
5. Application **must** panic if CLIP model loading fails (no fallback)

### FR2: Phase 1 - Comprehensive Facade Implementation
The application **must** implement facade patterns for all struct field access:
1. **BlockFacade**: Wrap `BlockData` with accessor methods for all fields
2. **ParticipantFacade**: Wrap `Participant` with accessor methods for all fields  
3. **GuessFacade**: Wrap `Guess` with accessor methods for all fields
4. **```Block<S>``` Facade**: Wrap typestate ``Block<S>`` structs with accessor methods
5. Replace **ALL** direct field access (`struct.field`) with facade methods (`struct.get_field()`)
6. Centralize all business logic in facade methods
7. Maintain backward compatibility with existing JSON serialization

### FR3: Phase 2 - Centralized Path Management
The application **must** implement centralized file path management:
1. **Add `dirs` crate**: Cross-platform access to user directories
2. **Create `PathManager`**: Centralized path management in `src/config.rs`
3. **Standardize file locations**: All files stored under `~/.cliptions/`
4. **Directory structure**: Organized by role (`data/`, `miner/`, `validator/`)
5. **Replace hardcoded paths**: All path access goes through `PathManager` facade
6. **Integration with facades**: Path management works with BlockFacade and other facades
7. **Error handling**: Clear messages when config files missing

### FR4: Phase 3 - Real CLIP Model Enforcement
The application **must** enforce real CLIP model usage:
1. Remove all conditional logic that chooses between Mock and CLIP embedders
2. Update `BlockProcessor` to only accept `ClipEmbedder`
3. Update all CLI binaries to only use `ClipEmbedder`
4. Ensure CLIP model download occurs at application startup
5. Application **must** fail fast if CLIP models are unavailable

### FR5: Phase 4 - Struct Unification
The application **must** unify `BlockData` and ``Block<S>`` into a single struct:
1. Create unified ``Block<S>`` struct combining fields from both structs
2. Migrate all `BlockData` usage to unified ``Block<S>`` 
3. Maintain facade interface compatibility during migration
4. Preserve all existing functionality from both structs
5. Update serialization to work with unified struct

### FR6: Phase 5 - Typestate Pattern Implementation
The application **must** implement the typestate pattern from `state_machine.rs`:
1. Replace enum-based state with typed state transitions in unified struct
2. Implement state transitions: `CommitmentsOpen → CommitmentsClosed → FrameCaptured → RevealsOpen → RevealsClosed → Payouts → Finished` (we do not use `Pending`)
3. Enforce valid transitions at compile time
4. Create DTO layer for JSON serialization/deserialization
5. Update Twitter API integration to work with typed states

### FR7: Test Suite Updates
The application **must** update all tests:
1. Remove tests that specifically test MockEmbedder behavior
2. Update remaining tests to use real CLIP embeddings
3. Ensure integration tests work with unified struct and state machine
4. Add tests for state transition validation
5. Add tests for facade layer functionality
6. Add tests for struct unification compatibility
7. Add tests for centralized path management

### FR8: CLI Command Updates
All CLI commands **must** be updated:
1. Remove `--use-mock` flags from all binaries
2. Update `BlockProcessor` instantiation to use new patterns
3. Ensure commands work with facade (Phase 1), path management (Phase 2), and all subsequent phases
4. Maintain existing command-line interfaces where possible

## 5. Non-Goals (Out of Scope)

- Changing the fundamental block lifecycle states or transitions
- Modifying Twitter API integration patterns
- Changing the underlying CLIP model or embedding algorithms
- Adding new block states or lifecycle phases
- Implementing new CLI commands or interfaces
- Modifying the payment or payout calculation logic
- Changes to the commitment/reveal cryptographic mechanisms

## 6. Facade Strategy & Design Considerations

### Comprehensive Field Access Control Strategy

Following the engineering principle: **"Never make fields pub, always use getters"**, this refactor implements a comprehensive facade pattern that eliminates all direct field access across the codebase.

#### Problem → Strategy Mapping
| Problem | Current State | Facade Solution |
|---------|---------------|-----------------|
| **Field is used everywhere** | `block.status`, `participant.verified` | `block.is_commitment_phase()`, `participant.is_verified()` |
| **Want to remove/rename field** | Breaking changes across dozens of files | Change only facade implementation |
| **Field is serialized/deserialized** | Direct struct serialization | DTO pattern with conversion logic |
| **Business logic tied to field** | Scattered validation, state checks | Centralized in facade methods |
| **Field will likely evolve** | `pub` fields, brittle dependencies | Private fields, stable accessor interface |

#### Facade Architecture Principles

1. **Complete Encapsulation**: All struct fields become private (`pub(crate)` at most)
2. **Single Access Point**: One facade method per field/concept
3. **Business Logic Centralization**: State transitions, validation in facades
4. **Backward Compatibility**: JSON serialization unchanged via DTO pattern
5. **Future-Proofing**: Can change internal representation without breaking consumers

## 7. Design Considerations

### Phase 1: Comprehensive Facade Patterns

#### BlockFacade - Centralized Block Logic
```rust
pub struct BlockFacade {
    inner: BlockData,
}

impl BlockFacade {
    // State queries (replaces direct status access)
    pub fn is_commitment_phase(&self) -> bool { /* ... */ }
    pub fn can_accept_participant(&self) -> bool { /* ... */ }
    pub fn is_complete(&self) -> bool { /* ... */ }
    
    // Field accessors (replaces direct field access)
    pub fn block_num(&self) -> &str { &self.inner.block_num }
    pub fn target_image_path(&self) -> &str { &self.inner.target_image_path }
    pub fn prize_pool(&self) -> f64 { self.inner.prize_pool }
    pub fn social_id(&self) -> &str { &self.inner.social_id }
    
    // Business logic (centralized)
    pub fn verified_participants(&self) -> Vec<ParticipantFacade> { /* ... */ }
    pub fn add_participant(&mut self, participant: Participant) -> Result<()> { /* ... */ }
    pub fn transition_to_reveals(&mut self) -> Result<()> { /* ... */ }
}
```

#### ParticipantFacade - Centralized Participant Logic
```rust
pub struct ParticipantFacade {
    inner: Participant,
}

impl ParticipantFacade {
    // Field accessors (replaces direct field access)
    pub fn social_id(&self) -> &str { &self.inner.social_id }
    pub fn username(&self) -> &str { &self.inner.username }
    pub fn guess(&self) -> GuessFacade { GuessFacade::new(&self.inner.guess) }
    pub fn commitment(&self) -> &str { &self.inner.commitment }
    pub fn wallet(&self) -> &str { &self.inner.wallet }
    pub fn score(&self) -> f64 { self.inner.score }
    
    // Business logic (centralized)
    pub fn is_verified(&self) -> bool { self.inner.verified }
    pub fn has_salt(&self) -> bool { self.inner.salt.is_some() }
    pub fn verify_commitment(&mut self, salt: &str) -> Result<bool> { /* ... */ }
}
```

#### GuessFacade - Centralized Guess Logic  
```rust
pub struct GuessFacade<'a> {
    inner: &'a Guess,
}

impl<'a> GuessFacade<'a> {
    // Field accessors (replaces direct field access)
    pub fn text(&self) -> &str { &self.inner.text }
    pub fn timestamp(&self) -> DateTime<Utc> { self.inner.timestamp }
    pub fn has_embedding(&self) -> bool { self.inner.embedding.is_some() }
    
    // Business logic (centralized)  
    pub fn get_embedding_array(&self) -> Option<Array1<f64>> { /* ... */ }
    pub fn similarity_to(&self, other: &GuessFacade) -> Result<f64> { /* ... */ }
}
```

### Phase 2: Centralized Path Management
```rust
// PathManager facade centralizes all file path logic
pub struct PathManager {
    base_path: PathBuf,
}

impl PathManager {
    pub fn new() -> Result<Self> {
        let home = dirs::home_dir().ok_or("Could not find home directory")?;
        let base_path = home.join(".cliptions");
        
        // Create directory structure
        std::fs::create_dir_all(&base_path.join("data"))?;
        std::fs::create_dir_all(&base_path.join("miner"))?;
        std::fs::create_dir_all(&base_path.join("validator"))?;
        
        Ok(Self { base_path })
    }
    
    // Centralized path accessors (facade pattern)
    pub fn get_config_path(&self) -> PathBuf { self.base_path.join("config.yaml") }
    pub fn get_blocks_path(&self) -> PathBuf { self.base_path.join("data/blocks.json") }
    pub fn get_miner_commitments_path(&self) -> PathBuf { self.base_path.join("miner/commitments.json") }
    // ... all other paths
}

// Replace hardcoded paths:
// OLD: "data/blocks.json"
// NEW: path_manager.get_blocks_path()
```

### Phase 3: CLIP-Only Architecture
```rust
// Remove this pattern:
if args.use_mock {
    let embedder = MockEmbedder::clip_like();
} else {
    let embedder = ClipEmbedder::new()?;
}

// Replace with:
let embedder = ClipEmbedder::new()
    .expect("CRITICAL: CLIP model required for operation");
```

### Phase 4: Struct Unification
```rust
// Unified struct combines best of both worlds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block<S = BlockStatus> {
    // From BlockData (business logic fields)
    pub(crate) block_version: i32,
    pub(crate) block_num: String,         // Unified on block_num
    pub(crate) prize_pool: f64,
    pub(crate) social_id: String,
    pub(crate) participants: Vec<Participant>,
    pub(crate) results: Vec<ScoringResult>,
    
    // From ``Block<S>`` (lifecycle fields)
    pub(crate) description: String,
    pub(crate) livestream_url: String,
    pub(crate) target_timestamp: DateTime<Utc>,
    pub(crate) target_frame_path: Option<PathBuf>,
    
    // Unified state handling
    pub(crate) state: S,                 // BlockStatus OR typestate marker
    
    // Common fields
    pub(crate) created_at: DateTime<Utc>,
    pub(crate) updated_at: DateTime<Utc>,
    pub(crate) commitment_deadline: Option<DateTime<Utc>>,
    pub(crate) reveals_deadline: Option<DateTime<Utc>>,
}

// Same facade interface works for unified struct!
impl<S> BlockFacade for Block< S > {
    fn block_id(&self) -> &str { &self.block_num }
    fn prize_pool(&self) -> f64 { self.prize_pool }
    // ... all existing facade methods unchanged
}
```

### Phase 5: Typestate Implementation
```rust
// We do not use `Pending`; we start at CommitmentsOpen
let block = Block::<CommitmentsOpen>::new(id, description, url, timestamp, commitment_deadline);
let block = block.close_commitments(&twitter_client).await?;
let block = block.capture_frame(frame_path)?;
```

### DTO Pattern for Serialization
```rust
#[derive(Serialize, Deserialize)]
pub struct BlockDataDTO {
    pub block_version: i32,
    pub status: String, // Serialized state name
    // ... other fields
}

impl From< Block< S > > for BlockDataDTO { /* ... */ }
impl TryFrom<BlockDataDTO> for Block<dyn StateMarker> { /* ... */ }
```

## 7. Technical Considerations

### Dependencies
- Real CLIP model files must be available in `models/` directory
- `twitter_api` crate integration must remain unchanged
- Serde serialization patterns must maintain backward compatibility

### Performance
- CLIP model loading may increase startup time
- Real embeddings will be slower than mock embeddings in tests
- Consider caching strategies for embedding operations

### Error Handling
- Application must fail fast if CLIP models unavailable
- Clear error messages for state transition violations
- Proper error propagation through facade layer

## 8. Success Metrics

### Phase 1 Success Criteria
- [ ] **All direct field access eliminated**: No `struct.field` patterns remain in codebase
- [ ] **BlockFacade implemented**: All `BlockData` access goes through facade methods
- [ ] **ParticipantFacade implemented**: All `Participant` access goes through facade methods  
- [ ] **GuessFacade implemented**: All `Guess` access goes through facade methods
- [ ] **Business logic centralized**: State transitions, validation in facade methods
- [ ] **Existing tests continue to pass**: No functional regressions
- [ ] **JSON serialization/deserialization maintains compatibility**: Backward compatibility preserved
- [ ] **No breaking changes to CLI interfaces**: External APIs unchanged

### Phase 2 Success Criteria
- [ ] **PathManager implemented**: Centralized path management in `src/config.rs`
- [ ] **Directory structure created**: `~/.cliptions/` with `data/`, `miner/`, `validator/` subdirs
- [ ] **All hardcoded paths eliminated**: Path access goes through PathManager facade
- [ ] **ConfigManager integration**: Uses PathManager for config file location
- [ ] **Error handling**: Clear messages for missing config files
- [ ] **Tests pass**: Path management functionality validated

### Phase 3 Success Criteria  
- [ ] Zero MockEmbedder references in production code
- [ ] All CLI commands use real CLIP embeddings
- [ ] Application panics gracefully if CLIP models unavailable
- [ ] Test suite uses real embeddings and passes

### Phase 4 Success Criteria
- [ ] **Unified ``Block<S>`` struct created**: Combines all fields from BlockData and ``Block<S>``
- [ ] **All BlockData usage migrated**: No references to old BlockData struct remain
- [ ] **Facade compatibility maintained**: All existing facade methods work with unified struct
- [ ] **Serialization preserved**: JSON compatibility maintained through DTO pattern
- [ ] **All tests pass**: No functional regressions from struct unification

### Phase 5 Success Criteria
- [ ] **All block operations use typestate pattern**: Compile-time state enforcement
- [ ] **Invalid state transitions caught at compile time**: Type system prevents errors
- [ ] **DTO layer successfully handles serialization**: Backward compatibility preserved
- [ ] **Twitter API integration works with typed states**: External APIs unchanged
- [ ] **Full test coverage for state transitions**: Comprehensive validation

## 9. Implementation Plan

### Phase 1: Comprehensive Facades (Week 1-2)
1. **Create facade modules**: 
   - `src/facades/block_facade.rs` - BlockData wrapper
   - `src/facades/participant_facade.rs` - Participant wrapper  
   - `src/facades/guess_facade.rs` - Guess wrapper
   - `src/facades/` - Facade modules directory (Rust 2021; no mod.rs)
2. **Implement comprehensive accessor methods**: All field access through getters
3. **Centralize business logic**: Move state transitions, validation to facades
4. **Update all consumers**: BlockProcessor, CLI binaries, tests use facades only
5. **Eliminate direct field access**: Audit codebase for `struct.field` patterns
6. **Ensure backward compatibility**: JSON serialization unchanged

### Phase 2: Centralized Path Management (Week 2)
1. **Add `dirs` crate**: Cross-platform directory access
2. **Create `PathManager`**: Centralized path facade in `src/config.rs`
3. **Implement path accessors**: All file paths through getter methods
4. **Create directory structure**: `~/.cliptions/` with organized subdirectories
5. **Replace hardcoded paths**: Audit and update all path usage
6. **Integrate with ConfigManager**: Use PathManager for config file location
7. **Add error handling**: Clear messages for missing files/directories

### Phase 3: MockEmbedder Removal (Week 3)
1. Remove MockEmbedder struct and implementation
2. Remove `--use-mock` flags from all CLI commands
3. Update all test files to use ClipEmbedder
4. Update BlockProcessor generic constraints
5. Add CLIP model validation at startup

### Phase 4: Struct Unification (Week 4)
1. **Design unified ``Block<S>`` struct**: Combine fields from both structs
2. **Create migration layer**: Conversion functions between old and new structs
3. **Update facade implementations**: Make facades work with unified struct
4. **Migrate consumers gradually**: BlockProcessor, CLI commands, tests
5. **Remove old BlockData struct**: Clean up after full migration
6. **Validate serialization compatibility**: Ensure JSON backward compatibility

### Phase 5: Typestate Implementation (Week 5)
1. Create DTO structs for serialization
2. Implement typestate transitions in unified ``Block<S>``
3. Update BlockProcessor to work with typed blocks
4. Update CLI commands for typestate pattern
5. Add comprehensive state transition tests

### Testing Strategy
- Run full test suite after each phase
- Validate JSON compatibility with sample data files
- Test CLI commands with real block data
- Performance testing with real CLIP models

## 10. Open Questions

1. **Model Distribution**: How should CLIP models be distributed with the application? Should they be bundled or downloaded on first run?

2. **Test Performance**: Real CLIP embeddings will slow down tests significantly. Should we implement test-specific optimizations (smaller models, caching)?

3. **Field Access Audit**: Should we implement a linter/macro to prevent future direct field access, or rely on code review?

4. **State Migration**: For existing BlockData in JSON files, should we implement automatic migration to typestate format, or require manual conversion?

5. **Error Recovery**: In Phase 3, if a block is found in an invalid state during deserialization, should we attempt recovery or fail fast?

6. **Backwards Compatibility Timeline**: How long should we maintain the ability to read old JSON formats before requiring migration?

---

**Priority**: High  
**Estimated Effort**: 5 weeks  
**Risk Level**: Medium (due to extensive refactoring, path management, and struct unification)  
**Dependencies**: CLIP model availability, Twitter API stability, `dirs` crate
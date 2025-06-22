# Browser Use - Roadmap: RealMir Modular Twitter Automation

**Goal:** Create a modular Twitter automation system for the RealMir prediction network, supporting both Validator and Miner workflows through separate, reusable components that implement a shared set of interfaces.

## Architecture Overview

Instead of a monolithic script, the system will be composed of specialized modules that conform to a strict **interface-based design**. `get_twitter_replies.py` serves as the first proof-of-concept and will be refactored to implement our formal `TwitterTask` interface. This ensures all modules are interchangeable, testable, and adhere to SOLID principles.

- **Single Responsibility**: Each module performs one job (e.g., collect commitments, post a reveal).
- **Open/Closed**: Extensible without modification
- **Dependency Inversion**: Use abstractions, not concrete implementations
- **Interface Segregation**: Clean, focused interfaces
- **Liskov Substitution**: Consistent behavior across implementations

## Phase 1: Core Infrastructure (Completed)

*   **Task 1.1:** ✅ **Base Twitter Reply Extraction** (`get_twitter_replies.py`)
    *   **Status:** Completed - Provides robust foundation for extracting replies from Twitter threads
    *   **Features:** Handles spam filtering, pagination, structured output via Pydantic models

*   **Task 1.2:** ✅ **Data Structure Alignment** (`rounds/guesses.json` with URLs)
    *   **Status:** Completed - Ground truth data includes commitment/reveal URLs for testing

*   **Task 1.3:** ✅ **Test Infrastructure** (`tests/test_twitter_data_extraction.py`)
    *   **Status:** Completed - Structural testing framework ready for modular components

## Phase 1.5: Rust-Driven Data Layer & Python/Rust Bridge (Revised)

*Goal: Establish Rust as the single source of truth for core data models while using Pydantic for interface validation in Python. Ensure consistency between Rust structs and Pydantic models via the Python/Rust bridge and dedicated integration tests.*

### **Task 1.5.1: Define Core Data Models in Rust**
*   **Action:** Create `src/models.rs` to house all core data structures.
*   **Action:** Define structs for `Round`, `Commitment`, `Participant`, etc., using `serde`.
*   **Status:** ✅ Completed - Created `src/models.rs` with `Commitment` and `Round` structs using serde and PyO3 derives

### **Task 1.5.2: Create Mirror Pydantic Models**
*   **Action:** Create `browser/data_models.py` for the Pydantic model definitions.
*   **Action:** Define Pydantic models that mirror the structure of the Rust structs.
*   **Status:** ✅ Completed - Created `browser/data_models.py` with corresponding Pydantic models for `Commitment` and `Round`

### **Task 1.5.3: Implement Schema Consistency Test**
*   **Action:** Create `tests/test_schema_consistency.py`.
*   **Action:** Write tests that pass data from each Pydantic model to the Rust core, ensuring they can be successfully deserialized into their corresponding Rust structs. This test will serve as our "consistency lock."
*   **Status:** ✅ Completed - Created comprehensive schema consistency tests with 3 test cases, all passing

### **Task 1.5.4: Implement Rust Data Access Layer**
*   **Action:** Create a Rust module (e.g., `src/data_access.rs`) to handle loading from and saving to `data/rounds.json`.
*   **Action:** Expose these data access functions to Python via `python_bridge.rs`.
*   **Status:** Not Started

### **Task 1.5.5: Refactor `collect_commitments.py`**
*   **Action:** Update `collect_commitments.py` to use the new Pydantic models for validation.
*   **Action:** After validation, the task will call the new Rust data access functions (via the bridge) to persist the collected commitments.
*   **Status:** Not Started

## Phase 2: Commitment Workflow

*This phase focuses on the initial round setup and the commitment interaction between Validator and Miner.*

### **Task 2.1:** Validator: Round Announcement
*   **Module:** `browser/validator/announce_round.py`
*   **Implements:** `TwitterPostingInterface`
*   **Purpose:** Post the initial round announcement tweet.
*   **Status:** ✅ Completed

### **Task 2.2:** Miner: Commitment Submission
*   **Module:** `browser/miner/submit_commitment.py`
*   **Implements:** `TwitterPostingInterface`
*   **Purpose:** Reply to an announcement with a commitment hash.
*   **Status:** ✅ Completed

### **Task 2.3:** Validator: Commitment Collection  
*   **Module:** `browser/validator/collect_commitments.py`
*   **Purpose:** Extract all miner commitments from the announcement tweet replies.
*   **Implements:** `TwitterExtractionInterface`
*   **Status:** ✅ Completed

### **Task 2.4:** Validator: Entry Fee Assignment
*   **Module:** `browser/validator/assign_entry_fees.py`  
*   **Implements:** `TwitterPostingInterface`
*   **Purpose:** Reply to each commitment with the TAO payment address.
*   **Status:** Not Started

### **Task 2.5:** Integration Test: Commitment Cycle
*   **Module:** `tests/test_commitment_workflow.py`
*   **Purpose:** Verify that a Validator can announce, a Miner can commit, and the Validator can collect the commitment and assign a fee address successfully.
*   **Status:** Not Started

## Phase 3: Reveal Workflow

*This phase handles the publication of the target frame and the subsequent reveal from the Miners.*

### **Task 3.1:** Validator: Target Frame Publication
*   **Module:** `browser-use/validator/publish_target_frame.py`
*   **Implements:** `TwitterPostingInterface`
*   **Purpose:** Post the target frame image as a reply to the announcement.
*   **Status:** Not Started

### **Task 3.2:** Miner: Reveal Submission  
*   **Module:** `browser-use/miner/submit_reveal.py`
*   **Implements:** `TwitterPostingInterface`
*   **Purpose:** Reply to the target frame with the plaintext prediction and salt.
*   **Status:** Not Started

### **Task 3.3:** Validator: Reveal Collection
*   **Module:** `browser-use/validator/collect_reveals.py`
*   **Purpose:** Extract all miner reveals from the target frame tweet replies.
*   **Implements:** `TwitterExtractionInterface`
*   **Status:** Not Started

### **Task 3.4:** Integration Test: Reveal Cycle
*   **Module:** `tests/test_reveal_workflow.py`
*   **Purpose:** Verify that a Validator can post a target, a Miner can reveal, and the Validator can collect the reveal.
*   **Status:** Not Started

## Phase 4: Scoring, Payouts, and Results

*This phase covers the final steps of the game: payment verification, scoring, and announcing the winners.*

### **Task 4.1 (Advanced):** Validator: Payment Verification
*   **Module:** `browser-use/validator/verify_payments.py`
*   **Purpose:** Check the TAO network to verify which miners have paid their entry fees.
*   **Integration:** TAO network APIs.
*   **Status:** Not Started

### **Task 4.2 (Advanced):** Validator: Scoring & Payouts
*   **Modules:** `score_predictions.py`, `distribute_payouts.py`
*   **Purpose:** Integrate with the CLIP embedder and TAO network to calculate winners and distribute rewards.
*   **Status:** Not Started

### **Task 4.3:** Validator: Results Publication
*   **Module:** `browser-use/validator/publish_results.py`
*   **Implements:** `TwitterPostingInterface`
*   **Purpose:** Post the final results tweet, listing winners and payouts.
*   **Status:** Not Started

## Phase 5: Orchestration & Monitoring

*This phase involves creating higher-level orchestrators to automate the full game cycle and tools for monitoring.*

### **Task 5.1:** Validator Orchestrator
*   **Module:** `browser-use/validator/orchestrator.py`
*   **Purpose:** Coordinate the full validator workflow for a complete round.
*   **Status:** Not Started

### **Task 5.2:** Miner Orchestrator
*   **Module:** `browser-use/miner/orchestrator.py` 
*   **Purpose:** Coordinate the full miner participation in a round.
*   **Status:** Not Started

### **Task 5.3:** Miner: Round Monitoring
*   **Module:** `browser-use/miner/monitor_rounds.py`
*   **Purpose:** Watch for new round announcements to enable fully automated participation.
*   **Status:** Not Started

## Design Principles & Standards

### **Core Interfaces** (e.g., in `browser/core/interfaces.py`)
- **`TwitterTask` (ABC):** An abstract base class defining the contract for any automated Twitter action.
  - `async def execute(self, **kwargs) -> BaseModel:`: Standard execution method.
  - `setup_agent(...)`: Configures the `browser-use` agent.
  - `validate_output(...)`: Ensures the result conforms to a Pydantic model.
- **`TwitterExtractionInterface` (Inherits `TwitterTask`):** Specialized for data collection.
- **`TwitterPostingInterface` (Inherits `TwitterTask`):** Specialized for creating content.

### **Module Implementation**
Each module will be a concrete implementation of a core interface. This replaces the "Module Template" concept with a formal, enforceable contract. Common functionality (config loading, browser context) will be handled in a base class that implements the `TwitterTask` interface.

### **Shared Infrastructure**
- **Core Interfaces**: `TwitterTask`, `TwitterExtractionInterface`, `TwitterPostingInterface`.
- **Abstract Base Classes**: `BaseTwitterTask` to provide shared setup and execution logic.
- **Shared Pydantic Models**: `CommitmentData`, `RevealData`, `RoundConfig`, `PayoutInfo` for type-safe data transfer.
- **Configuration Management**: Centralized config loading and validation
- **Error Handling**: Standardized exception hierarchy
- **Testing Utilities**: Mock generators, test data factories

## Next Steps

**Immediate Priority:** 
1. **Implement Phase 1.5:** Complete the Data Layer Refactoring to establish a single source of truth.
2. **Continue with Phase 2:** Once the DAL is in place, proceed with the Commitment Workflow, starting with `Task 2.4: Validator: Entry Fee Assignment`.

**Recommended Approach:**
1.  ✅ Create `browser/core/interfaces.py` and define the abstract base classes.
2.  ✅ Create `browser/core/base_task.py` for shared logic (config, browser setup).
3.  ✅ Implement the **Commitment Workflow (Phase 2)** Round Announcement module.
4.  **Implement the Data Layer Refactoring (Phase 1.5)** as outlined above.
5.  Refactor existing modules like `collect_commitments` to use the new data layer.
6.  Continue with remaining Phase 2 modules (`assign_entry_fees`).
7.  Build the `test_commitment_workflow.py` to test the interaction between the Phase 2 modules.
8.  Proceed to the Reveal Workflow, implementing and testing the modules for that stage.
9.  Finally, build the orchestrators to tie the full vertical slices together.

This modular approach ensures each component can be developed, tested, and deployed independently while maintaining consistency across the entire system.

## Summary of Changes from Original Plan

**Key Architectural Shifts:**
- **From Single Script to Modular System**: Instead of one monolithic `twitter_data_fetcher.py`, we now have specialized modules for each game phase
- **From Data Extraction to Full Game Automation**: Expanded scope from just collecting existing data to facilitating the entire Validator/Miner workflow  
- **From Ad-Hoc Testing to Systematic Architecture**: Following SOLID principles with proper abstractions and interfaces
- **From Manual Process to Automated Orchestration**: Full automation of round management, fee collection, and payout distribution

**Immediate Benefits:**
- **True Modularity**: Modules are truly interchangeable, not just similar in pattern.
- **Testability**: Easy to mock dependencies and test vertical slices of gameplay.
- **Maintainability**: Clear contracts make the system easier to understand and debug.
- **Scalability**: New features can be added by creating new classes that conform to existing interfaces.

## Test Coverage Analysis

### Current Test Status
- **Rust Tests**: 45 total (33 unit + 12 integration) - All passing ✅
- **Python Tests**: 84 total (69 passing + 15 failing)
- **Schema Consistency Tests**: 3 (bridging Rust-Python gap) - All passing ✅

### Critical Test Gaps Identified

#### **🚨 Missing Rust Functionality (High Priority)**

**💰 Payout & Economics (0/11 features)**
- Prize pool distribution, player rankings, platform fees
- Multi-player scenarios, payout validation
- **Impact**: Core business logic missing from Rust

**🔑 Configuration Management (0/9 features)**  
- Config loading, API key validation, spending limits
- Cost tracking integration
- **Impact**: No configuration system in Rust

**🐦 Social Integration (0/9 features)**
- Announcement formatting, URL parsing, hashtag handling
- Twitter workflow automation
- **Impact**: No social media capabilities in Rust

#### **⚠️ Medium Priority Gaps**

**🖼️ Enhanced Embedder Features (Missing 5/13 features)**
- CLI interface, byte/PIL image handling
- Semantic similarity scoring

**✅ Verification Edge Cases (Missing 6/10 features)**
- Empty rounds, file errors, mixed commitments
- Missing data handling

### Feature Coverage Matrix

| **Feature Category** | **Rust Coverage** | **Python Coverage** | **Priority** |
|---------------------|-------------------|---------------------|--------------|
| **Core Cryptography** | 🟢 Excellent (9/9) | 🟡 Good (4/9) | ✅ Well covered |
| **Data Models** | 🟢 Excellent (3/3) | 🟢 Excellent (3/3) | ✅ Well covered |
| **Embeddings/CLIP** | 🟡 Good (7/13) | 🟢 Excellent (13/13) | 🟡 Medium gap |
| **Scoring** | 🟢 Good (8/14) | 🟢 Excellent (14/14) | 🟡 Medium gap |
| **Round Management** | 🟢 Good (6/9) | 🟡 Good (5/9) | 🟡 Medium gap |
| **Payouts/Economics** | 🔴 **Missing (0/11)** | 🟢 Excellent (11/11) | 🚨 **Critical** |
| **Configuration** | 🔴 **Missing (0/9)** | 🟡 Partial (9/9) | 🚨 **Critical** |
| **Social Integration** | 🔴 **Missing (0/9)** | 🟡 Partial (9/9) | 🚨 **Critical** |
| **Verification** | 🟡 Limited (4/10) | 🟢 Excellent (10/10) | 🟡 Medium gap |

### Recommended Test Development Plan

**Phase 1: Critical Rust Features (30 tests)**
1. Implement payout/economics module with comprehensive tests (11 tests)
2. Add configuration management system with validation (9 tests)
3. Create social integration framework with URL/hashtag handling (9 tests)

**Phase 2: Enhanced Coverage (10 tests)**
4. Expand embedder capabilities (CLI, bytes, PIL support) (5 tests)
5. Add verification edge cases (empty rounds, file errors) (6 tests)

**Phase 3: Python Gap Filling (8 tests)**
6. Add missing Python commitment features (batch, deterministic) (5 tests)
7. Add missing Python round management features (3 tests)

**Total: ~48 additional tests needed** to achieve comprehensive parity

### Architecture Implications

The test gap analysis reveals that **Rust currently handles the core cryptographic and data processing** excellently, but is missing the **business logic, configuration, and social features** that exist in Python. This suggests:

1. **Rust Core**: Focus on performance-critical operations (crypto, scoring, data processing)
2. **Python Layer**: Handle business logic, configuration, and social integration
3. **Bridge**: Ensure seamless data flow between layers via schema consistency tests

This polyglot architecture leverages each language's strengths while maintaining type safety and performance where needed.
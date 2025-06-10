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

## Phase 2: Validator Modules (High Priority)

### **Task 2.1:** Round Announcement Module
*   **Module:** `browser-use/validator/announce_round.py`
*   **Purpose:** Post round announcement tweets with target time and stream URL
*   **Inputs:** Round config (target time, stream URL, entry fee, etc.)
*   **Outputs:** Tweet URL, round ID for tracking
*   **Status:** Not Started

### **Task 2.2:** Commitment Collection Module  
*   **Module:** `browser-use/validator/collect_commitments.py`
*   **Purpose:** Extract miner commitments from announcement tweet replies
*   **Implements:** `TwitterExtractionInterface`
*   **Inputs:** Announcement tweet URL
*   **Outputs:** Structured list of commitments (username, hash, payout address)
*   **Status:** Not Started

### **Task 2.3:** Entry Fee Assignment Module
*   **Module:** `browser-use/validator/assign_entry_fees.py`  
*   **Purpose:** Reply to each commitment with TAO payment address
*   **Inputs:** Commitment list, TAO addresses pool
*   **Outputs:** Payment tracking data
*   **Status:** Not Started

### **Task 2.4:** Target Frame Publishing Module
*   **Module:** `browser-use/validator/publish_target_frame.py`
*   **Purpose:** Post target frame image at specified time
*   **Inputs:** Target time, captured frame, announcement tweet URL
*   **Outputs:** Target frame tweet URL
*   **Status:** Not Started

### **Task 2.5:** Reveal Collection Module
*   **Module:** `browser-use/validator/collect_reveals.py`
*   **Purpose:** Extract miner reveals from target frame tweet replies  
*   **Implements:** `TwitterExtractionInterface`
*   **Inputs:** Target frame tweet URL
*   **Outputs:** Structured list of reveals (username, plaintext, salt)
*   **Status:** Not Started

### **Task 2.6:** Results Publication Module
*   **Module:** `browser-use/validator/publish_results.py`
*   **Purpose:** Post final results with winners and payouts
*   **Inputs:** Winner calculations, payout amounts
*   **Outputs:** Results tweet URL
*   **Status:** Not Started

## Phase 3: Miner Modules (Medium Priority)

### **Task 3.1:** Commitment Submission Module
*   **Module:** `browser-use/miner/submit_commitment.py`
*   **Purpose:** Reply to announcement with commitment hash and payout address
*   **Inputs:** Announcement URL, prediction, salt, wallet address
*   **Outputs:** Commitment tweet URL
*   **Status:** Not Started

### **Task 3.2:** Reveal Submission Module  
*   **Module:** `browser-use/miner/submit_reveal.py`
*   **Purpose:** Reply to target frame with plaintext prediction and salt
*   **Inputs:** Target frame URL, prediction, salt
*   **Outputs:** Reveal tweet URL
*   **Status:** Not Started

### **Task 3.3:** Round Monitoring Module
*   **Module:** `browser-use/miner/monitor_rounds.py`
*   **Purpose:** Watch for new round announcements and phase transitions
*   **Inputs:** Validator Twitter account, polling interval
*   **Outputs:** Round state updates, notification callbacks
*   **Status:** Not Started

## Phase 4: Integration & Orchestration (Lower Priority)

### **Task 4.1:** Validator Orchestrator
*   **Module:** `browser-use/validator/orchestrator.py`
*   **Purpose:** Coordinate full validator workflow for a complete round
*   **Dependencies:** All validator modules (2.1-2.6)
*   **Features:** State management, error recovery, timing coordination
*   **Status:** Not Started

### **Task 4.2:** Miner Orchestrator
*   **Module:** `browser-use/miner/orchestrator.py` 
*   **Purpose:** Coordinate full miner participation in a round
*   **Dependencies:** All miner modules (3.1-3.3)
*   **Features:** Automatic participation, strategy plugins
*   **Status:** Not Started

### **Task 4.3:** Cross-Module Testing
*   **Module:** `tests/test_integration.py`
*   **Purpose:** End-to-end testing of validator/miner interactions
*   **Dependencies:** All modules
*   **Features:** Mock round simulation, timing validation
*   **Status:** Not Started

## Phase 5: Advanced Features (Future)

### **Task 5.1:** Payment Verification Module
*   **Module:** `browser-use/validator/verify_payments.py`
*   **Purpose:** Check TAO blockchain for entry fee payments
*   **Integration:** TAO network APIs, payment tracking
*   **Status:** Not Started

### **Task 5.2:** CLIP Scoring Integration  
*   **Module:** `browser-use/validator/score_predictions.py`
*   **Purpose:** Calculate similarity scores using CLIP embedder
*   **Dependencies:** `clip_embedder.py`, target frame, reveals
*   **Status:** Not Started

### **Task 5.3:** Payout Distribution Module
*   **Module:** `browser-use/validator/distribute_payouts.py`
*   **Purpose:** Execute cryptocurrency payouts to winners
*   **Integration:** TAO network, wallet management
*   **Status:** Not Started

## Design Principles & Standards

### **Core Interfaces** (e.g., in `browser-use/core/interfaces.py`)
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

**Immediate Priority:** Define and implement the core interfaces and base classes (`TwitterTask`, `BaseTwitterTask`).

**Recommended Approach:**
1.  Create `browser-use/core/interfaces.py` and define the abstract base classes.
2.  Create `browser-use/core/base_task.py` for shared logic (config, browser setup).
3.  Refactor `get_twitter_replies.py` to be the first concrete implementation of the `TwitterExtractionInterface`.
4.  Implement **Task 2.2** (Commitment Collection Module) as a new implementation of the same interface.
5.  Continue building other modules by implementing the appropriate interfaces.
6.  Create orchestrators that operate on the `TwitterTask` abstraction, not concrete classes.

This modular approach ensures each component can be developed, tested, and deployed independently while maintaining consistency across the entire system.

## Summary of Changes from Original Plan

**Key Architectural Shifts:**
- **From Single Script to Modular System**: Instead of one monolithic `twitter_data_fetcher.py`, we now have specialized modules for each game phase
- **From Data Extraction to Full Game Automation**: Expanded scope from just collecting existing data to facilitating the entire Validator/Miner workflow  
- **From Ad-Hoc Testing to Systematic Architecture**: Following SOLID principles with proper abstractions and interfaces
- **From Manual Process to Automated Orchestration**: Full automation of round management, fee collection, and payout distribution

**Immediate Benefits:**
- **True Modularity**: Modules are truly interchangeable, not just similar in pattern.
- **Testability**: Easy to mock dependencies by creating a test implementation of an interface.
- **Maintainability**: Clear contracts make the system easier to understand and debug.
- **Scalability**: New features can be added by creating new classes that conform to existing interfaces.
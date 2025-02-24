# realmir

A decentralized prediction game where players compete to guess how an AI Agent will caption a specific future frame in a live video stream.

### Index
- [Gameplay](#gameplay)
- [Key Features](#key-features)
- [Example Round](#example-round)
- [Score and Payout Calculation](#score-and-payout-calculation)
  - [Ranking Process](#ranking-process)
  - [Payout Distribution](#payout-distribution)
  - [Basic Scoring](#basic-scoring-no-ties)
  - [Handling Ties](#handling-ties)
- [CLIP Embedder](#clip-embedder)

### Gameplay
1. A target timestamp in the video stream is announced (e.g. "20240223 13:30:57 EST")
2. Players predict what an AI Agent (using CLIP) will say this exact frame shows
3. When that moment arrives and the frame is revealed, each prediction is compared using CLIP
4. Players are ranked by how well their predictions matched CLIP's understanding
5. The prize pool is distributed based on rankings, with better predictions earning larger shares

### Key Features
- **Timestamp Predictions**: Guess how an AI Agent will interpret a specific future video frame
- **AI-Powered**: Uses OpenAI's CLIP model for objective scoring
- **Web3 Integration**: Decentralized gameplay and prize distribution
- **Crypto Rewards**: Prize pools paid out based on prediction accuracy
- **Transparent**: All calculations and rankings are verifiable

### Example Round
1. Target: #targetframe20250223_133057EST from live stream of a cat sanctuary
2. Players submit predictions like:
   - "Cat shelter with caretakers"
   - "People caring for cats indoors"
   - "Pet store with animals"
3. When target timestamp arrives, CLIP calculates similarity scores
4. Players are ranked by score
5. Prize pool is distributed according to rankings

## Score and Payout Calculation

The system calculates payouts based on similarity rankings between guesses and the target image.

### Ranking Process
1. Calculate CLIP embeddings for target image and each guess
2. Calculate cosine similarity between target and each guess
3. Rank guesses by similarity (highest to lowest)

### Payout Distribution
The payout system uses a position-based scoring method that:
- Distributes the entire prize pool
- Rewards higher ranks with larger shares
- Handles ties fairly

#### Basic Scoring (No Ties)
For n players, each position's score is calculated as:
```
position_score = (n - position) / sum(1..n)
```

Example for 3 players:
- Denominator = 1 + 2 + 3 = 6
- 1st place: 3/6 = 0.50 (50% of pool)
- 2nd place: 2/6 ≈ 0.33 (33% of pool)
- 3rd place: 1/6 ≈ 0.17 (17% of pool)

#### Handling Ties
When multiple guesses have equal similarity scores:
1. Group tied positions together
2. Calculate combined points for tied positions
3. Split points equally among tied guesses

Example with 5 players and ties:
```
Similarities:
Player1: 0.9 (tied for 1st/2nd)
Player2: 0.9 (tied for 1st/2nd)
Player3: 0.7
Player4: 0.5 (tied for 4th/5th)
Player5: 0.5 (tied for 4th/5th)

Groups:
[Player1, Player2]   - Split points for 1st/2nd
[Player3]           - Gets points for 3rd
[Player4, Player5]   - Split points for 4th/5th
```

### Usage
```bash
python3 calculate_scores_payout.py <target_image_path> <guess1> <guess2> [guess3 ...]
```

Output shows:
- Similarity scores for each guess
- Payout amounts (normalized to prize pool)
- Total payout verification

## CLIP Embedder

The CLIP embedder generates embeddings for images and text using OpenAI's CLIP model. It can be used from the command line and accepts input via stdin.

### Installation

```bash
pip install torch transformers Pillow numpy
```

### First Run Behavior
On first run, the script will download the CLIP model (approximately 605MB). This will:
- Take about 15-20 seconds
- Show download progress bars
- Cache the model locally for future use

Subsequent runs will use the cached model and be much faster (1-2 seconds).

### Windows Setup Notes
When running on Windows, you might see a warning about symlinks in the Hugging Face cache system. You have two options:

1. **Enable Developer Mode (Recommended)**
   - Open Windows Settings
   - Navigate to Privacy & Security > For Developers
   - Enable "Developer Mode"

2. **Run as Administrator**
   - Run Python/command prompt as administrator

Alternatively, you can suppress the warning by setting an environment variable:
```bash
set HF_HUB_DISABLE_SYMLINKS_WARNING=1
```

The model will still work without these changes, but caching might be less efficient.

### Usage

#### Generate Image Embeddings
```bash
echo '{"image": "'$(cat image.jpg | base64)'"}' | python clip_embedder.py --mode image
```

#### Generate Text Embeddings
```bash
echo '{"text": "a photo of a dog"}' | python clip_embedder.py --mode text
```

### Output Format
The script outputs JSON to stdout with the following structure:
```json
{
    "embedding": [0.1, 0.2, ...],  // 512-dimensional embedding vector
    "shape": [512]                 // shape of the embedding
}
```

### Cross-Platform Behavior
Important note about CLIP embeddings:
- Different platforms or hardware may produce slightly different embedding vectors for the same input
- This is normal and expected behavior
- While the exact numbers might differ, the semantic relationships between embeddings remain consistent
- Similarity scores between related concepts will maintain their relative ordering
- For example, if a cat image is more similar to "cat" than "dog" on one machine, this relationship will hold on other machines

### Testing
To run the tests, you'll need:
1. All dependencies installed
2. Run from the project root directory:

```bash
python -m unittest discover tests
```

The test suite verifies:

#### Core Functionality
- Image embedding generation
  - From file path
  - From PIL Image object
  - From bytes
- Text embedding generation
  - Single text input
  - Batch text input
- Embedding properties
  - 512 dimensions
  - Normalized vectors
- Cross-platform consistency
  - Soft verification of embedding reproducibility
  - Semantic similarity score validation
  - Tolerance ranges for expected scores

#### CLI Interface
- Image input processing via stdin
- Text input processing via stdin
- JSON output format
- Error handling
  - Invalid JSON input
  - Missing required fields
  - Invalid mode arguments

### Error Handling
Errors are written to stderr with descriptive messages for:
- Invalid JSON input
- Missing required fields
- Image processing errors
- Model inference errors

## Contributing

### Git Workflow
Development follows the [git flow](https://datasift.github.io/gitflow/IntroducingGitFlow.html) methodology.

We recommend using [gitflow-avh](https://github.com/petervanderdoes/gitflow-avh/wiki) with the following settings:

```
Branch name for production releases: master 
Branch name for "next release" development: develop 
Feature branch prefix: feature/ 
Bugfix branch prefix: bugfix/ 
Release branch prefix: release/ 
Hotfix branch prefix: hotfix/ 
Support branch prefix: support/ 
Version tag prefix:
```

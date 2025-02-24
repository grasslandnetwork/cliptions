# realmir

Predict how an AI Agent will caption upcoming frames from live video streams. Players compete for cryptocurrency rewards based on prediction accuracy.

### Index
- [Gameplay](#gameplay)
- [Key Rules](#key-rules)
- [Key Features](#key-features)
- [Example Round](#example-round)
- [Commitment Hash Generation](#commitment-hash-generation)
  - [Installation](#installation)
  - [Usage](#usage)
  - [Common Errors](#common-errors)
- [Score and Payout Calculation](#score-and-payout-calculation)
  - [Ranking Process](#ranking-process)
  - [Payout Distribution](#payout-distribution)
  - [Basic Scoring](#basic-scoring-no-ties)
  - [Handling Ties](#handling-ties)
- [CLIP Embedder](#clip-embedder)
- [Contributing](#contributing)

### Gameplay
1. The Agent announces a new round on Twitter with a target frame
2. Players submit predictions in two steps:
   - First, reply to announcement with a commitment hash of their prediction
   - Later, reveal their actual prediction and salt to verify the commitment
3. When that moment arrives and the frame is revealed, each prediction is compared using CLIP
4. Players are ranked by how well their predictions matched CLIP's understanding
5. The prize pool is distributed based on rankings, with better predictions earning larger shares

### Key Rules
- **Submission Deadline:** All predictions must be submitted **before** the target timestamp. Late submissions are disqualified.
- **Commitment Format:** Predictions must be submitted as **hash commitments** along with the player's wallet address.
- **Reveal Phase:** After the target frame is posted, players must publicly reveal their plaintext guess and salt.
- **No Edited Tweets:** Edited commitment tweets are **disqualified** and **no refunds** are issued for fees paid.

### Key Features
- **Timestamp Predictions:** Guess how an AI Agent will interpret a specific future video frame.
- **AI-Powered:** Uses OpenAI's CLIP model for objective scoring.
- **Web3 Integration:** Decentralized gameplay and prize distribution.
- **Crypto Rewards:** Prize pools paid out based on prediction accuracy.
- **Transparent:** All calculations and rankings are verifiable.

### Example Round
1. Agent tweets "@round2 #targetframe20250223_133057EST from live stream of a cat sanctuary"
2. Players participate via Twitter:
   - Submit commitment hash as reply to announcement
   - After frame reveal, reply with prediction and salt
   - Example prediction: "Cat shelter with caretakers"
3. After target frame is revealed and players share their predictions, CLIP calculates similarity scores
4. Players are ranked by score
5. Prize pool is distributed according to rankings

## Commitment Hash Generation
Players must generate a **hash commitment** for their prediction.

### Installation
```bash
pip install -r requirements.txt
```

### Usage
Run the script using the following format:
```bash
python3 generate_commitment.py "Your predicted caption here" --salt "your-salt-value-here"
```

**Example:**  
```bash
python3 generate_commitment.py "Cat sanctuary with woman wearing snoopy sweater" --salt "random_secret_123"
```

This will output:
```
Commitment: f7b7889b520e01e8a2e915e8e9124bc299c6f584c0e0dd255a7e38fe8ec35747
```

### Common Errors
- **Missing Quotes:** Always use **quotes** around both the **caption** and **salt**.
- **Argument Order:** The **caption** must come **first**, followed by the `--salt` argument.
- **PowerShell Users:** Ensure you use **double quotes** to avoid argument parsing issues.

## Score and Payout Calculation
The system calculates payouts based on similarity rankings between guesses and the target image.

### Ranking Process
1. Calculate CLIP embeddings for the target image and each guess.
2. Calculate cosine similarity between the target and each guess.
3. Rank guesses by similarity (highest to lowest).

### Payout Distribution
The payout system uses a position-based scoring method that:
- Distributes the entire prize pool.
- Rewards higher ranks with larger shares.
- Handles ties fairly.

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
1. Group tied positions together.
2. Calculate combined points for tied positions.
3. Split points equally among tied guesses.

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

## CLIP Embedder
The CLIP embedder generates embeddings for images and text using OpenAI's CLIP model. It can be used from the command line and accepts input via stdin.

### Installation
```bash
pip install torch transformers Pillow numpy
```

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
```json
{
    "embedding": [0.1, 0.2, ...],
    "shape": [512]
}
```

## Contributing

### Development Setup
1. Clone the repository.
2. Create a new branch for your feature or bugfix.
3. Install dependencies using:
```bash
pip install -r requirements.txt
```
4. Set up pre-commit hooks.

### Running Tests
```bash
python -m unittest discover tests
```

### Pull Request Process
1. Create a new branch for your feature or bugfix.
2. Make your changes.
3. Run tests to ensure everything works.
4. Commit your changes.
5. Push your branch to GitHub.
6. Create a pull request.
7. Wait for review and merge.

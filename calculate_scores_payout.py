import sys
from clip_embedder import ClipEmbedder
import numpy as np

def calculate_rankings(target_image_path, guesses):
    """Calculate rankings for guesses based on similarity to target image.
    
    Args:
        target_image_path: Path to the target image
        guesses: List of text guesses to rank
    
    Returns:
        List of tuples (guess, similarity) sorted by similarity (highest to lowest)
    """
    embedder = ClipEmbedder()
    
    # Get target image embedding
    image_embedding = embedder.get_image_embedding(target_image_path)
    
    # Calculate similarity for each guess
    similarities = []
    for guess in guesses:
        text_embedding = embedder.get_text_embedding(guess)
        similarity = float(np.dot(image_embedding, text_embedding))
        similarities.append((guess, similarity))
    
    # Sort by similarity score (highest to lowest)
    return sorted(similarities, key=lambda x: x[1], reverse=True)

def calculate_payouts(ranked_results, prize_pool=1.0):
    """Calculate payouts based on rankings.
    
    The payout calculation uses a position-based scoring system where:
    - Scores are based only on position (1st, 2nd, etc), not similarity values
    - Scores sum to 1.0 to distribute full prize pool
    - Higher positions get proportionally higher scores
    
    For n players, the denominator is sum(1..n) = n(n+1)/2
    Each position's score is: (n-position)/denominator
    
    Example for 2 players:
    - Denominator = sum(1,2) = 3
    - 1st place score = 2/3 ≈ 0.67 -> gets 67% of pool
    - 2nd place score = 1/3 ≈ 0.33 -> gets 33% of pool
    
    Example for 3 players:
    - Denominator = sum(1,2,3) = 6
    - 1st place score = 3/6 = 0.50 -> gets 50% of pool
    - 2nd place score = 2/6 ≈ 0.33 -> gets 33% of pool
    - 3rd place score = 1/6 ≈ 0.17 -> gets 17% of pool
    
    Args:
        ranked_results: List of (guess, similarity) tuples sorted by similarity
        prize_pool: Total amount to distribute (default: 1.0)
    
    Returns:
        List of payouts corresponding to ranked_results
    """
    total_guesses = len(ranked_results)
    
    # Calculate denominator: sum(1..n) where n is total_guesses
    denominator = sum(range(1, total_guesses + 1))
    
    # Calculate normalized scores (sum to 1.0)
    scores = []
    for i in range(total_guesses):
        # For each position i (0-based):
        # - (total_guesses - i) is the "points" for this position
        # - Dividing by denominator normalizes scores to sum to 1.0
        score = (total_guesses - i) / denominator
        scores.append(score)
    
    # Calculate payouts by multiplying each score by prize pool
    return [score * prize_pool for score in scores]

def display_results(ranked_results, payouts, prize_pool):
    """Display rankings and payouts in a formatted way."""
    print("\nRankings and Payouts:")
    print("-" * 50)
    for i, ((guess, similarity), payout) in enumerate(zip(ranked_results, payouts), 1):
        print(f"{i}. \"{guess}\"")
        print(f"   Similarity score: {similarity:.4f}")
        print(f"   Payout: ${payout:.2f}")
        print()
    
    print(f"Total prize pool: ${prize_pool:.2f}")
    print(f"Total payout: ${sum(payouts):.2f}")

if __name__ == "__main__":
    if len(sys.argv) < 3:
        print("Usage: python3 calculate_guess_ranking.py <target_image_path> <guess1> <guess2> [guess3 ...]")
        sys.exit(1)
        
    target_path = sys.argv[1]
    guesses = sys.argv[2:]
    
    # Calculate rankings
    ranked_results = calculate_rankings(target_path, guesses)
    
    # Calculate payouts
    prize_pool = 1.0
    payouts = calculate_payouts(ranked_results, prize_pool)
    
    # Display results
    display_results(ranked_results, payouts, prize_pool) 
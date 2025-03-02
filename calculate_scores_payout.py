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

def calculate_payouts(ranked_results, prize_pool):
    """Calculate payouts based on rankings.
    
    The payout calculation uses a position-based scoring system where:
    - Scores are based only on position (1st, 2nd, etc), not similarity values
    - Equal similarity scores get equal payouts (ties split the combined payout)
    - Scores sum to 1.0 to distribute full prize pool
    - Higher positions get proportionally higher scores
    
    For n players without ties:
    - Denominator = sum(1..n)
    - Each position's score = (n-position)/denominator
    
    For players with ties:
    - Players with equal similarity scores split their combined payout
    
    Args:
        ranked_results: List of (guess, similarity) tuples sorted by similarity
        prize_pool: Total amount to distribute
    
    Returns:
        List of payouts corresponding to ranked_results
    """
    total_guesses = len(ranked_results)
    denominator = sum(range(1, total_guesses + 1))
    
    # Group positions by similarity score
    groups = []
    current_group = []
    current_similarity = None
    
    for guess, similarity in ranked_results:
        if similarity != current_similarity:
            if current_group:
                groups.append(current_group)
            current_group = [(guess, similarity)]
            current_similarity = similarity
        else:
            current_group.append((guess, similarity))
    
    if current_group:
        groups.append(current_group)
    
    # Calculate payouts
    payouts = []
    position = 0
    
    for group in groups:
        # Calculate total points for this group's positions
        group_size = len(group)
        group_points = sum(total_guesses - (position + i) for i in range(group_size))
        
        # Split points equally among tied positions
        points_per_position = group_points / group_size
        score = points_per_position / denominator
        
        # Add same payout for each tied position
        payouts.extend([score * prize_pool] * group_size)
        position += group_size
    
    return payouts

def display_results(ranked_results, payouts, prize_pool):
    """Display rankings and payouts in a formatted way."""
    print("\nRankings and Payouts:")
    print("-" * 50)
    for i, ((guess, similarity), payout) in enumerate(zip(ranked_results, payouts), 1):
        print(f"{i}. \"{guess}\"")
        print(f"   Similarity score: {similarity:.4f}")
        print(f"   Payout: {payout:.9f}")
        print()
    
    print(f"Total prize pool: {prize_pool:.9f}")
    print(f"Total payout: {sum(payouts):.9f}")

if __name__ == "__main__":
    if len(sys.argv) < 4:
        print("Usage: python3 calculate_guess_ranking.py <target_image_path> <prize_pool> <guess1> <guess2> [guess3 ...]")
        print("Note: prize_pool can be a small decimal value (up to 9 decimal places)")
        sys.exit(1)
        
    target_path = sys.argv[1]
    prize_pool = float(sys.argv[2])
    guesses = sys.argv[3:]
    
    # Calculate rankings
    ranked_results = calculate_rankings(target_path, guesses)
    
    # Calculate payouts
    payouts = calculate_payouts(ranked_results, prize_pool)
    
    # Display results
    display_results(ranked_results, payouts, prize_pool) 
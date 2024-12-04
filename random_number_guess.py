import random
import time
from dataclasses import dataclass
from typing import List, Dict

@dataclass
class Player:
    """Represents a player in the guessing game with their guess, fee, and potential winnings."""
    id: str
    guess: int
    fee_paid: float
    score: float = 0.0
    payout: float = 0.0

class GuessingGame:
    """Manages a number guessing game where players pay to guess a random number and win proportional payouts."""
    
    def __init__(self, fee: float = 10.0, platform_fee_percent: float = 0.2):
        """Initialize a new game with specified entry fee and platform fee percentage.
        
        Args:
            fee: Cost to enter the game (default: $10.0)
            platform_fee_percent: Percentage of fee kept by platform (default: 20%)
        """
        self.fee = fee
        self.platform_fee_percent = platform_fee_percent
        self.players: Dict[str, Player] = {}
        self.target_number: int = None
        self.prize_pool: float = 0.0
        self.platform_fees: float = 0.0
        
    def add_player(self, player_id: str, guess: int) -> None:
        """Register a new player with their guess and collect their entry fee.
        
        Args:
            player_id: Unique identifier for the player
            guess: Player's guess between 0 and 100
            
        Raises:
            ValueError: If guess is outside valid range (0-100)
        """
        if not (0 <= guess <= 100):  # Assuming valid range is 0-100
            raise ValueError("Guess must be between 0 and 100")
            
        self.players[player_id] = Player(player_id, guess, self.fee)
        platform_fee = self.fee * self.platform_fee_percent
        self.platform_fees += platform_fee
        self.prize_pool += self.fee - platform_fee

    def calculate_scores(self) -> None:
        """Calculate scores for all players based on how close their guesses were to target number.
        Uses formula: score = 1 / (1 + |guess - target|)
        Higher scores are awarded for closer guesses.
        """
        if not self.target_number:
            self.target_number = random.randint(0, 100)
        
        for player in self.players.values():
            player.score = 1 / (1 + abs(player.guess - self.target_number))

    def distribute_prizes(self) -> None:
        """Distribute the prize pool among players proportionally based on their scores.
        A player's payout = (player's score / total scores) * prize pool
        """
        total_score = sum(player.score for player in self.players.values())
        
        for player in self.players.values():
            player.payout = (player.score / total_score) * self.prize_pool

    def run_game(self) -> None:
        """Execute the game sequence: calculate scores, distribute prizes, and display results.
        
        Raises:
            ValueError: If fewer than 2 players have registered
        """
        if len(self.players) < 2:
            raise ValueError("Need at least 2 players to start the game")
            
        self.calculate_scores()
        self.distribute_prizes()
        
        print(f"\nTarget number was: {self.target_number}")
        print("\nResults:")
        for player in sorted(self.players.values(), key=lambda x: x.payout, reverse=True):
            print(f"Player {player.id}: Guess={player.guess}, Score={player.score:.2f}, Payout=${player.payout:.2f}")

# Example usage
if __name__ == "__main__":
    game = GuessingGame(fee=10.0)
    
    # Simulate some players
    game.add_player("Alice", 48)
    game.add_player("Bob", 52)
    game.add_player("Charlie", 45)
    game.add_player("David", 60)
    game.add_player("Eve", 70)
    
    game.run_game()
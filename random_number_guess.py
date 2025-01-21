import random
import time
from dataclasses import dataclass
from typing import List, Dict, Optional
from abc import ABC, abstractmethod

@dataclass
class Player:
    """Represents a player in the guessing game with their guess, fee, and potential winnings."""
    id: str
    guess: int
    fee_paid: float
    score: float = 0.0
    payout: float = 0.0

class IPlayerManager(ABC):
    @abstractmethod
    def add_player(self, player_id: str, guess: int) -> None:
        pass

class IScoringStrategy(ABC):
    @abstractmethod
    def calculate_score(self, guess: int, target: int) -> float:
        pass

class IPrizeDistributor(ABC):
    @abstractmethod
    def distribute_prizes(self, players: Dict[str, Player], prize_pool: float) -> None:
        pass

class IGameOutput(ABC):
    @abstractmethod
    def display_results(self, target: int, players: Dict[str, Player]) -> None:
        pass

class DefaultScoringStrategy(IScoringStrategy):
    def calculate_score(self, guess: int, target: int) -> float:
        return 1 / (1 + abs(guess - target))

class ConsoleGameOutput(IGameOutput):
    def display_results(self, target: int, players: Dict[str, Player]) -> None:
        print(f"\nTarget number was: {target}")
        print("\nResults:")
        for player in sorted(players.values(), key=lambda x: x.payout, reverse=True):
            print(f"Player {player.id}: Guess={player.guess}, Score={player.score:.2f}, Payout=${player.payout:.2f}")

class PlayerRegistry:
    """Handles player management and fee collection"""
    def __init__(self, entry_fee: float, platform_fee_percent: float):
        self.entry_fee = entry_fee
        self.platform_fee_percent = platform_fee_percent
        self.players: Dict[str, Player] = {}
        self.prize_pool: float = 0.0
        self.platform_fees: float = 0.0

    def add_player(self, player_id: str, guess: int) -> None:
        if not (0 <= guess <= 100):
            raise ValueError("Guess must be between 0 and 100")
            
        self.players[player_id] = Player(player_id, guess, self.entry_fee)
        self._process_fee()

    def _process_fee(self) -> None:
        platform_fee = self.entry_fee * self.platform_fee_percent
        self.platform_fees += platform_fee
        self.prize_pool += self.entry_fee - platform_fee

    @property
    def player_count(self) -> int:
        return len(self.players)

class RandomNumberGenerator:
    """Responsible for generating target numbers"""
    def __init__(self, min_value: int = 0, max_value: int = 100):
        self.min_value = min_value
        self.max_value = max_value

    def generate(self) -> int:
        return random.randint(self.min_value, self.max_value)

class PrizeDistributor:
    """Handles prize distribution logic"""
    def distribute_prizes(self, players: Dict[str, Player], prize_pool: float) -> None:
        total_score = sum(player.score for player in players.values())
        
        if total_score == 0:
            return

        for player in players.values():
            player.payout = (player.score / total_score) * prize_pool

class GameEngine:
    """Coordinates game flow and manages game state"""
    def __init__(self,
                 player_registry: PlayerRegistry,
                 number_generator: RandomNumberGenerator,
                 scoring_strategy: IScoringStrategy,
                 prize_distributor: PrizeDistributor,
                 output_handler: IGameOutput):
        self.player_registry = player_registry
        self.number_generator = number_generator
        self.scoring_strategy = scoring_strategy
        self.prize_distributor = prize_distributor
        self.output_handler = output_handler
        self.target_number: Optional[int] = None

    def run_game(self) -> None:
        if self.player_registry.player_count < 2:
            raise ValueError("Need at least 2 players to start the game")

        self._generate_target()
        self._calculate_scores()
        self._distribute_prizes()
        self._display_results()

    def _generate_target(self) -> None:
        self.target_number = self.number_generator.generate()

    def _calculate_scores(self) -> None:
        for player in self.player_registry.players.values():
            player.score = self.scoring_strategy.calculate_score(
                player.guess,
                self.target_number
            )

    def _distribute_prizes(self) -> None:
        self.prize_distributor.distribute_prizes(
            self.player_registry.players,
            self.player_registry.prize_pool
        )

    def _display_results(self) -> None:
        self.output_handler.display_results(
            self.target_number,
            self.player_registry.players
        )

# Example alternative scoring strategies (OCP)
class LinearScoringStrategy(IScoringStrategy):
    """Linear scoring based on distance from target"""
    def calculate_score(self, guess: int, target: int) -> float:
        max_distance = 100
        distance = abs(guess - target)
        return 1 - (distance / max_distance)

class ExponentialScoringStrategy(IScoringStrategy):
    """Exponential scoring that heavily rewards very close guesses"""
    def calculate_score(self, guess: int, target: int) -> float:
        distance = abs(guess - target)
        return 2 ** (-distance/10)

# Example alternative output handlers (OCP)
class JsonGameOutput(IGameOutput):
    """Outputs game results in JSON format"""
    def display_results(self, target: int, players: Dict[str, Player]) -> None:
        import json
        results = {
            "target": target,
            "players": [
                {
                    "id": p.id,
                    "guess": p.guess,
                    "score": round(p.score, 2),
                    "payout": round(p.payout, 2)
                }
                for p in players.values()
            ]
        }
        print(json.dumps(results, indent=2))

# Factory for creating preconfigured game instances
class GuessingGameFactory:
    @staticmethod
    def create_default_game(entry_fee: float = 10.0, platform_fee_percent: float = 0.2) -> GameEngine:
        player_registry = PlayerRegistry(entry_fee, platform_fee_percent)
        number_generator = RandomNumberGenerator()
        scoring_strategy = DefaultScoringStrategy()
        prize_distributor = PrizeDistributor()
        output_handler = ConsoleGameOutput()

        return GameEngine(
            player_registry,
            number_generator,
            scoring_strategy,
            prize_distributor,
            output_handler
        )

# Example usage
if __name__ == "__main__":
    # Create game with default configuration
    # game = GuessingGameFactory.create_default_game()
    
    # Or create game with custom components
    custom_game = GameEngine(
        PlayerRegistry(entry_fee=20.0, platform_fee_percent=0.1),
        RandomNumberGenerator(0, 100),
        ExponentialScoringStrategy(),
        PrizeDistributor(),
        JsonGameOutput()
    )

    custom_game.player_registry.add_player("Alice", 48)
    custom_game.player_registry.add_player("Bob", 52)
    custom_game.run_game()
from abc import ABC, abstractmethod
from typing import Union, List
import numpy as np
from PIL import Image
from pathlib import Path

class IEmbedder(ABC):
    """Interface for embedding models like CLIP.
    
    This interface defines the contract for any embedding model
    used to convert images and text to feature vectors.
    """
    
    @abstractmethod
    def get_image_embedding(self, image: Union[str, Path, Image.Image, bytes]) -> np.ndarray:
        """Generate embedding for an image.
        
        Args:
            image: Path to image file, PIL Image object, or bytes
            
        Returns:
            np.ndarray: Normalized image embedding vector
        """
        pass
    
    @abstractmethod
    def get_text_embedding(self, text: Union[str, List[str]]) -> np.ndarray:
        """Generate embedding for text.
        
        Args:
            text: String or list of strings to embed
            
        Returns:
            np.ndarray: Normalized text embedding vector(s)
        """
        pass

class IScoreValidator(ABC):
    """Interface for score validators.
    
    This interface defines the contract for any validator used
    to validate guesses and calculate their scores.
    """
    
    @abstractmethod
    def validate_guess(self, guess: str) -> bool:
        """Check if a guess meets the validity criteria.
        
        Args:
            guess: The text guess to validate
            
        Returns:
            bool: True if the guess is valid, False otherwise
        """
        pass
    
    @abstractmethod
    def calculate_adjusted_score(self, image_features: np.ndarray, guess: str) -> float:
        """Calculate the score for a guess relative to image features.
        
        Args:
            image_features: The embedding vector for the target image
            guess: The text guess to score
            
        Returns:
            float: The calculated score, adjusted as needed
        """
        pass 
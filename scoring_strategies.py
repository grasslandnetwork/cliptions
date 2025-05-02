from abc import ABC, abstractmethod
import numpy as np

class IScoringStrategy(ABC):
    """Interface for scoring strategies.
    
    This interface defines the contract for any scoring algorithm
    used to calculate similarity between image and text.
    """
    
    @abstractmethod
    def calculate_score(self, image_features: np.ndarray, text_features: np.ndarray, **kwargs) -> float:
        """Calculate the similarity score between image and text features.
        
        Args:
            image_features: The embedding vector for the image
            text_features: The embedding vector for the text
            **kwargs: Additional parameters that may be needed by the strategy
            
        Returns:
            float: The calculated similarity score
        """
        pass


class RawSimilarityStrategy(IScoringStrategy):
    """Simple raw cosine similarity scoring strategy.
    
    This strategy calculates the raw dot product between normalized vectors,
    which is equivalent to cosine similarity for normalized vectors.
    """
    
    def calculate_score(self, image_features: np.ndarray, text_features: np.ndarray, **kwargs) -> float:
        """Calculate raw cosine similarity.
        
        Args:
            image_features: The embedding vector for the image
            text_features: The embedding vector for the text
            
        Returns:
            float: The raw similarity score
        """
        # Ensure features are 1D
        if image_features.ndim > 1:
            image_features = image_features.flatten()
        
        # Calculate raw similarity using dot product
        return float(np.dot(text_features, image_features))


class BaselineAdjustedStrategy(IScoringStrategy):
    """Baseline-adjusted similarity scoring strategy.
    
    This strategy adjusts the raw similarity by comparing it to a baseline,
    which helps differentiate between meaningful and random matches.
    """
    
    def calculate_score(self, image_features: np.ndarray, text_features: np.ndarray, **kwargs) -> float:
        """Calculate baseline-adjusted similarity.
        
        Args:
            image_features: The embedding vector for the image
            text_features: The embedding vector for the text
            **kwargs: Must include 'baseline_features' for the baseline embedding
            
        Returns:
            float: The adjusted similarity score
        """
        # Verify baseline features are provided
        if 'baseline_features' not in kwargs:
            raise ValueError("baseline_features is required for BaselineAdjustedStrategy")
            
        baseline_features = kwargs['baseline_features']
        
        # Ensure features are 1D
        if image_features.ndim > 1:
            image_features = image_features.flatten()
        
        # Calculate raw similarity
        raw_score = np.dot(text_features, image_features)
        
        # Calculate baseline similarity
        baseline_score = np.dot(baseline_features, image_features)
        
        # Adjust score relative to baseline
        adjusted_score = (raw_score - baseline_score) / (1 - baseline_score)
        
        return float(max(0.0, adjusted_score)) 
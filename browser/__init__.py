"""
Browser-use Twitter automation package for RealMir

This package provides modular Twitter automation components for the RealMir
prediction network, supporting both Validator and Miner workflows.
"""

from .core.interfaces import TwitterTask, TwitterExtractionInterface, TwitterPostingInterface
from .core.base_task import BaseTwitterTask

__version__ = "0.1.0"
__all__ = [
    "TwitterTask",
    "TwitterExtractionInterface", 
    "TwitterPostingInterface",
    "BaseTwitterTask"
] 
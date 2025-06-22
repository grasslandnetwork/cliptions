"""
JSON Data Access Implementation for RealMir system.

This module provides a concrete implementation of the DataAccessInterface
that reads and writes data to/from the data/rounds.json file.
"""

import json
import os
import asyncio
from pathlib import Path
from typing import Any, Dict, List, Optional
from datetime import datetime

from ..browser.core.interfaces import DataAccessInterface
from .models import RoundsData, Round, Participant, CommitmentData


class JsonDataAccess(DataAccessInterface):
    """
    JSON file-based implementation of the DataAccessInterface.
    
    This class provides thread-safe read/write operations to the rounds.json file,
    with proper error handling and data validation.
    """
    
    def __init__(self, data_file_path: Optional[str] = None):
        """
        Initialize the JSON data access with the specified file path.
        
        Args:
            data_file_path: Path to the JSON data file (defaults to data/rounds.json)
        """
        if data_file_path is None:
            # Default to data/rounds.json relative to project root
            project_root = Path(__file__).parent.parent
            data_file_path = project_root / "data" / "rounds.json"
        
        self.data_file_path = Path(data_file_path)
        self._lock = asyncio.Lock()
        
        # Ensure the data directory exists
        self.data_file_path.parent.mkdir(parents=True, exist_ok=True)
        
        # Initialize empty file if it doesn't exist
        if not self.data_file_path.exists():
            self._write_data({})
    
    async def _read_data(self) -> Dict[str, Any]:
        """Read data from the JSON file with proper error handling."""
        try:
            with open(self.data_file_path, 'r', encoding='utf-8') as f:
                data = json.load(f)
            return data
        except FileNotFoundError:
            # Return empty data if file doesn't exist
            return {}
        except json.JSONDecodeError as e:
            raise ValueError(f"Invalid JSON in data file: {e}")
        except Exception as e:
            raise RuntimeError(f"Error reading data file: {e}")
    
    def _write_data(self, data: Dict[str, Any]) -> None:
        """Write data to the JSON file with proper formatting."""
        try:
            with open(self.data_file_path, 'w', encoding='utf-8') as f:
                json.dump(data, f, indent=4, ensure_ascii=False, default=self._json_serializer)
        except Exception as e:
            raise RuntimeError(f"Error writing data file: {e}")
    
    def _json_serializer(self, obj):
        """Custom JSON serializer for datetime objects."""
        if isinstance(obj, datetime):
            return obj.isoformat()
        raise TypeError(f"Object of type {type(obj)} is not JSON serializable")
    
    async def get_round(self, round_id: str) -> Optional[Dict[str, Any]]:
        """
        Retrieve a specific round by ID.
        
        Args:
            round_id: The unique identifier of the round
            
        Returns:
            Round data dictionary or None if not found
        """
        async with self._lock:
            data = await self._read_data()
            return data.get(round_id)
    
    async def save_round(self, round_id: str, round_data: Dict[str, Any]) -> bool:
        """
        Save or update a round's data.
        
        Args:
            round_id: The unique identifier of the round
            round_data: Complete round data dictionary
            
        Returns:
            True if save was successful, False otherwise
        """
        try:
            async with self._lock:
                data = await self._read_data()
                data[round_id] = round_data
                self._write_data(data)
                return True
        except Exception as e:
            print(f"Error saving round {round_id}: {e}")
            return False
    
    async def list_rounds(self) -> List[str]:
        """
        List all available round IDs.
        
        Returns:
            List of round identifiers
        """
        async with self._lock:
            data = await self._read_data()
            return list(data.keys())
    
    async def save_commitments(self, round_id: str, commitments: List[Dict[str, Any]]) -> bool:
        """
        Save collected commitments for a round.
        
        Args:
            round_id: The round identifier
            commitments: List of commitment data dictionaries
            
        Returns:
            True if save was successful, False otherwise
        """
        try:
            async with self._lock:
                data = await self._read_data()
                
                # Ensure round exists
                if round_id not in data:
                    data[round_id] = {"participants": []}
                
                # Update the collected_commitments field
                data[round_id]["collected_commitments"] = {
                    "success": True,
                    "commitments": commitments,
                    "total_commitments_found": len(commitments),
                    "timestamp": datetime.now().isoformat()
                }
                
                # Also update or create participant entries
                for commitment in commitments:
                    await self._update_participant_from_commitment(data[round_id], commitment)
                
                self._write_data(data)
                return True
        except Exception as e:
            print(f"Error saving commitments for round {round_id}: {e}")
            return False
    
    async def _update_participant_from_commitment(self, round_data: Dict[str, Any], commitment: Dict[str, Any]) -> None:
        """Update or create participant entry from commitment data."""
        participants = round_data.setdefault("participants", [])
        username = commitment.get("username", "").lstrip("@")
        
        # Find existing participant or create new one
        participant = None
        for p in participants:
            if p.get("username") == username:
                participant = p
                break
        
        if participant is None:
            participant = {
                "username": username,
                "wallet": commitment.get("wallet_address", ""),
                "commitment": commitment.get("commitment_hash", ""),
                "commitment_url": commitment.get("tweet_url", ""),
                "valid": True,
                "entry_fee_assigned": False
            }
            participants.append(participant)
        else:
            # Update existing participant
            participant["commitment"] = commitment.get("commitment_hash", participant.get("commitment", ""))
            participant["commitment_url"] = commitment.get("tweet_url", participant.get("commitment_url", ""))
            participant["wallet"] = commitment.get("wallet_address", participant.get("wallet", ""))
    
    async def get_commitments(self, round_id: str) -> List[Dict[str, Any]]:
        """
        Get all commitments for a specific round.
        
        Args:
            round_id: The round identifier
            
        Returns:
            List of commitment data dictionaries
        """
        async with self._lock:
            data = await self._read_data()
            round_data = data.get(round_id, {})
            collected_commitments = round_data.get("collected_commitments", {})
            return collected_commitments.get("commitments", [])
    
    async def update_participant(self, round_id: str, username: str, updates: Dict[str, Any]) -> bool:
        """
        Update participant data within a round.
        
        Args:
            round_id: The round identifier
            username: The participant's username
            updates: Dictionary of fields to update
            
        Returns:
            True if update was successful, False otherwise
        """
        try:
            async with self._lock:
                data = await self._read_data()
                
                if round_id not in data:
                    return False
                
                participants = data[round_id].get("participants", [])
                username_clean = username.lstrip("@")
                
                # Find the participant
                for participant in participants:
                    if participant.get("username") == username_clean:
                        # Update the participant data
                        participant.update(updates)
                        self._write_data(data)
                        return True
                
                return False  # Participant not found
        except Exception as e:
            print(f"Error updating participant {username} in round {round_id}: {e}")
            return False
    
    async def get_all_rounds(self) -> Dict[str, Any]:
        """
        Get all rounds data.
        
        Returns:
            Dictionary containing all rounds data
        """
        async with self._lock:
            return await self._read_data()
    
    async def create_round(self, round_data: Dict[str, Any]) -> bool:
        """
        Create a new round.
        
        Args:
            round_data: Complete round data dictionary including round_id
            
        Returns:
            True if creation was successful, False otherwise
        """
        round_id = round_data.get("round_id")
        if not round_id:
            return False
        
        return await self.save_round(round_id, round_data)
    
    async def get_participants(self, round_id: str) -> List[Dict[str, Any]]:
        """
        Get all participants for a specific round.
        
        Args:
            round_id: The round identifier
            
        Returns:
            List of participant data dictionaries
        """
        round_data = await self.get_round(round_id)
        if not round_data:
            return []
        
        return round_data.get("participants", [])
    
    async def add_participant(self, round_id: str, participant_data: Dict[str, Any]) -> bool:
        """
        Add a new participant to a round.
        
        Args:
            round_id: The round identifier
            participant_data: Participant data dictionary
            
        Returns:
            True if addition was successful, False otherwise
        """
        try:
            async with self._lock:
                data = await self._read_data()
                
                if round_id not in data:
                    return False
                
                participants = data[round_id].setdefault("participants", [])
                participants.append(participant_data)
                
                self._write_data(data)
                return True
        except Exception as e:
            print(f"Error adding participant to round {round_id}: {e}")
            return False
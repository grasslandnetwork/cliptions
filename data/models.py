"""
Comprehensive data models for the RealMir prediction system.

This module defines all Pydantic models used throughout the system,
providing type safety and validation for round data, participants, payouts, etc.
"""

from datetime import datetime
from typing import List, Optional, Dict, Any
from pydantic import BaseModel, Field, validator


class Payout(BaseModel):
    """Represents a payout to a participant"""
    amount: float = Field(..., description="Payout amount in TAO")
    currency: str = Field(default="TAO", description="Payment currency")
    url: Optional[str] = Field(None, description="URL of the payout announcement tweet")
    timestamp: Optional[datetime] = Field(None, description="When the payout was distributed")


class RawReply(BaseModel):
    """Raw reply data from Twitter extraction"""
    url: str = Field(..., description="URL of the reply tweet")
    author: str = Field(..., description="Author of the reply")
    text_preview: str = Field(..., description="Preview of the reply text")
    was_spam_flagged: bool = Field(default=False, description="Whether this reply was flagged as spam")


class RawCommitmentReplies(BaseModel):
    """Raw commitment reply data from Twitter extraction"""
    original_tweet_url: str = Field(..., description="URL of the original announcement tweet")
    total_replies_found: int = Field(..., description="Total number of replies found")
    replies: List[RawReply] = Field(default_factory=list, description="List of raw reply data")


class CommitmentData(BaseModel):
    """Processed commitment data from a participant"""
    username: str = Field(..., description="Twitter username of the participant")
    commitment_hash: str = Field(..., description="SHA-256 commitment hash")
    wallet_address: str = Field(..., description="Wallet address for payouts")
    tweet_url: str = Field(..., description="URL of the commitment tweet")
    timestamp: datetime = Field(..., description="When the commitment was submitted")


class CollectedCommitments(BaseModel):
    """Result of commitment collection process"""
    success: bool = Field(..., description="Whether collection was successful")
    commitments: List[CommitmentData] = Field(default_factory=list, description="Collected commitments")
    announcement_url: str = Field(..., description="URL of the announcement tweet")
    total_commitments_found: int = Field(..., description="Number of commitments found")
    error_message: Optional[str] = Field(None, description="Error message if collection failed")


class Participant(BaseModel):
    """Represents a round participant with all their data"""
    username: str = Field(..., description="Twitter username")
    wallet: str = Field(..., description="Wallet address")
    commitment: str = Field(..., description="Commitment hash")
    commitment_url: str = Field(..., description="URL of commitment tweet")
    reveal: Optional[str] = Field(None, description="Revealed prediction text")
    reveal_url: Optional[str] = Field(None, description="URL of reveal tweet")
    salt: Optional[str] = Field(None, description="Salt used for commitment")
    valid: bool = Field(default=True, description="Whether participant's data is valid")
    score: Optional[float] = Field(None, description="Prediction score (0-1)")
    payout: Optional[Payout] = Field(None, description="Payout information")
    entry_fee_assigned: bool = Field(default=False, description="Whether entry fee has been assigned")
    entry_fee_url: Optional[str] = Field(None, description="URL of entry fee assignment tweet")


class Round(BaseModel):
    """Complete round data structure"""
    round_id: str = Field(..., description="Unique round identifier")
    participants: List[Participant] = Field(default_factory=list, description="Round participants")
    target_image: Optional[str] = Field(None, description="Path to target image")
    target_time: Optional[str] = Field(None, description="Target timestamp")
    round_commitment_url: Optional[str] = Field(None, description="URL of round announcement tweet")
    round_reveal_url: Optional[str] = Field(None, description="URL of target frame publication tweet")
    total_payout: float = Field(default=0.0, description="Total payout amount")
    prize_pool: float = Field(default=0.0, description="Total prize pool")
    
    # Optional processed data
    raw_commitment_replies: Optional[RawCommitmentReplies] = Field(None, description="Raw commitment reply data")
    collected_commitments: Optional[CollectedCommitments] = Field(None, description="Processed commitment data")
    
    # Round configuration
    entry_fee: float = Field(default=0.001, description="Entry fee in TAO")
    commitment_deadline: Optional[datetime] = Field(None, description="Commitment deadline")
    reveal_deadline: Optional[datetime] = Field(None, description="Reveal deadline")
    livestream_url: Optional[str] = Field(None, description="URL of the livestream being predicted")
    
    # Round status
    status: str = Field(default="active", description="Round status: active, completed, cancelled")
    created_at: datetime = Field(default_factory=datetime.now, description="When round was created")
    completed_at: Optional[datetime] = Field(None, description="When round was completed")


class RoundsData(BaseModel):
    """Container for all rounds data"""
    rounds: Dict[str, Round] = Field(default_factory=dict, description="Dictionary of rounds by round_id")
    
    def get_round(self, round_id: str) -> Optional[Round]:
        """Get a specific round by ID"""
        return self.rounds.get(round_id)
    
    def add_round(self, round: Round) -> None:
        """Add a new round"""
        self.rounds[round.round_id] = round
    
    def list_rounds(self) -> List[str]:
        """List all round IDs"""
        return list(self.rounds.keys())
    
    def get_active_rounds(self) -> List[Round]:
        """Get all active rounds"""
        return [round for round in self.rounds.values() if round.status == "active"]


# Specific data models for different operations

class RoundAnnouncementData(BaseModel):
    """Data for announcing a new round"""
    round_id: str = Field(..., description="Unique round identifier")
    entry_fee: float = Field(default=0.001, description="Entry fee in TAO")
    commitment_deadline: datetime = Field(..., description="Commitment deadline")
    reveal_deadline: datetime = Field(..., description="Reveal deadline")
    livestream_url: str = Field(..., description="URL of the livestream to predict")
    instructions: str = Field(default="", description="Additional instructions")
    hashtags: List[str] = Field(default_factory=lambda: ["#realmir", "$TAO"], description="Hashtags to include")


class CommitmentSubmissionData(BaseModel):
    """Data for submitting a commitment"""
    prediction: str = Field(..., description="The plaintext prediction")
    salt: str = Field(..., description="Salt for commitment hash")
    wallet_address: str = Field(..., description="Miner's wallet address")
    reply_to_url: str = Field(..., description="URL to reply to")
    commitment_hash: Optional[str] = Field(None, description="Pre-computed hash")


class EntryFeeAssignmentData(BaseModel):
    """Data for assigning entry fees to participants"""
    participant_username: str = Field(..., description="Username of the participant")
    participant_wallet: str = Field(..., description="Participant's wallet address")
    commitment_url: str = Field(..., description="URL of their commitment tweet")
    entry_fee_amount: float = Field(..., description="Entry fee amount in TAO")
    payment_address: str = Field(..., description="Address where participant should send payment")
    payment_deadline: datetime = Field(..., description="Payment deadline")


# Result models for task outputs

class TaskResult(BaseModel):
    """Base result model for all tasks"""
    success: bool = Field(..., description="Whether the task succeeded")
    timestamp: datetime = Field(default_factory=datetime.now, description="When the task completed")
    error_message: Optional[str] = Field(None, description="Error message if task failed")


class AnnouncementResult(TaskResult):
    """Result from round announcement task"""
    tweet_url: Optional[str] = Field(None, description="URL of the announcement tweet")
    tweet_id: Optional[str] = Field(None, description="ID of the announcement tweet")
    round_id: str = Field(..., description="ID of the announced round")


class CommitmentSubmissionResult(TaskResult):
    """Result from commitment submission task"""
    tweet_url: Optional[str] = Field(None, description="URL of the commitment tweet")
    tweet_id: Optional[str] = Field(None, description="ID of the commitment tweet")
    commitment_hash: str = Field(..., description="The submitted commitment hash")
    wallet_address: str = Field(..., description="The submitted wallet address")


class CommitmentCollectionResult(TaskResult):
    """Result from commitment collection task"""
    commitments: List[CommitmentData] = Field(default_factory=list, description="Collected commitments")
    announcement_url: str = Field(..., description="URL of the announcement tweet")
    total_commitments_found: int = Field(default=0, description="Number of commitments found")


class EntryFeeAssignmentResult(TaskResult):
    """Result from entry fee assignment task"""
    assignments: List[Dict[str, Any]] = Field(default_factory=list, description="List of fee assignments made")
    total_assignments: int = Field(default=0, description="Total number of assignments made")
    round_id: str = Field(..., description="Round ID for the assignments")
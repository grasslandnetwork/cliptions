"""
Validator modules for Cliptions Twitter automation.

This package contains modules that implement the Validator side of the Cliptions prediction game:
- Block announcements
- Commitment collection  
- Entry fee assignment
- Target frame publication
- Reveal collection
- Payment verification
- Results publication
"""

# Note: Expose BlockAnnouncementTask for easy import
from .announce_block import BlockAnnouncementTask

__all__ = ['BlockAnnouncementTask'] 
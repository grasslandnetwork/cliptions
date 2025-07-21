import hashlib
import argparse
import sys

def generate_commitment(message: str, salt: str) -> str:
    """Generate a commitment hash from a message and salt.
    
    Args:
        message: The plaintext message to commit
        salt: A salt value required to prevent message revelation
        
    Returns:
        str: The hex digest of the commitment
    """
    if not salt:
        raise ValueError("Salt is required for generating commitments")
        
    data = message.encode() + salt.encode()
    return hashlib.sha256(data).hexdigest()

def main():
    parser = argparse.ArgumentParser(description='Generate a commitment hash for a message.')
    parser.add_argument('message', type=str, help='The plaintext message to commit')
    parser.add_argument('--salt', type=str, required=True,
                      help='Salt value required to prevent message revelation')
    
    args = parser.parse_args()
    
    try:
        commitment = generate_commitment(args.message, args.salt)
        print(commitment)
    except ValueError as e:
        print(f"Error: {str(e)}", file=sys.stderr)
        sys.exit(1)

if __name__ == "__main__":
    main()
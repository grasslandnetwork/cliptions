import hashlib
import argparse
import sys

def pedersen_commit(guess: str, secret: str) -> str:
    """Generate a commitment hash from a guess and secret.
    
    Args:
        guess: The plaintext guess to commit
        secret: A secret value to prevent guess revelation
        
    Returns:
        str: The hex digest of the commitment
    """
    data = guess.encode() + secret.encode()
    return hashlib.sha256(data).hexdigest()

def main():
    parser = argparse.ArgumentParser(description='Generate a commitment hash for a guess.')
    parser.add_argument('guess', type=str, help='The plaintext guess to commit')
    parser.add_argument('--secret', type=str, default='random_secret_123',
                      help='Secret value to prevent guess revelation (default: random_secret_123)')
    
    args = parser.parse_args()
    
    commitment = pedersen_commit(args.guess, args.secret)
    print(commitment)

if __name__ == "__main__":
    main()
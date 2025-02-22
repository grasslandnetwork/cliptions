import sys
import os

# Add parent directory to Python path to find clip_embedder
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from clip_embedder import ClipEmbedder
import hashlib

def generate_reference_hash():
    """Generate reference hash for example.jpg embedding."""
    embedder = ClipEmbedder()
    
    # Use correct path from tests directory
    image_path = os.path.join("fixtures", "example.jpg")
    
    embedding = embedder.get_image_embedding(image_path)
    embedding_bytes = embedding.tobytes()
    hash_value = hashlib.sha256(embedding_bytes).hexdigest()
    print(f"Reference hash: {hash_value}")

if __name__ == "__main__":
    generate_reference_hash() 
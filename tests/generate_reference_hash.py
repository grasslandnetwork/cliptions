from clip_embedder import ClipEmbedder
import hashlib

def generate_reference_hash():
    """Generate reference hash for example.jpg embedding."""
    embedder = ClipEmbedder()
    embedding = embedder.get_image_embedding("example.jpg")
    embedding_bytes = embedding.tobytes()
    hash_value = hashlib.sha256(embedding_bytes).hexdigest()
    print(f"Reference hash: {hash_value}")

if __name__ == "__main__":
    generate_reference_hash() 
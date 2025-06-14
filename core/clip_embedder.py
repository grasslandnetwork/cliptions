import torch
from transformers import CLIPProcessor, CLIPModel
from PIL import Image
from typing import Union, List
import numpy as np
from pathlib import Path
import sys
import json
import io
import base64
import argparse
from .interfaces import IEmbedder

class ClipEmbedder(IEmbedder):
    """Generates CLIP embeddings for images and text using the CLIP model."""
    
    def __init__(self, model_name: str = "openai/clip-vit-base-patch32"):
        """Initialize the CLIP model and processor.
        
        Args:
            model_name: The name of the CLIP model to use from HuggingFace
        """
        self.device = "cuda" if torch.cuda.is_available() else "cpu"
        self.model = CLIPModel.from_pretrained(model_name).to(self.device)
        self.processor = CLIPProcessor.from_pretrained(model_name)

    def get_image_embedding(self, image: Union[str, Path, Image.Image, bytes]) -> np.ndarray:
        """Generate CLIP embedding for an image.
        
        Args:
            image: Path to image file, PIL Image object, or bytes
            
        Returns:
            np.ndarray: Normalized image embedding vector
        """
        # Handle different input types
        if isinstance(image, (str, Path)):
            image = Image.open(image)
        elif isinstance(image, bytes):
            image = Image.open(io.BytesIO(image))
            
        # Prepare image for model
        inputs = self.processor(images=image, return_tensors="pt")
        pixel_values = inputs['pixel_values'].to(self.device)
        
        # Generate embedding
        with torch.no_grad():
            image_features = self.model.get_image_features(pixel_values)
            
        # Normalize and convert to numpy
        image_embedding = image_features.cpu().numpy()
        image_embedding = image_embedding / np.linalg.norm(image_embedding)
        return image_embedding[0]

    def get_text_embedding(self, text: Union[str, List[str]]) -> np.ndarray:
        """Generate CLIP embedding for text.
        
        Args:
            text: String or list of strings to embed
            
        Returns:
            np.ndarray: Normalized text embedding vector(s)
        """
        # Prepare text for model
        inputs = self.processor(text=text, return_tensors="pt", padding=True)
        input_ids = inputs['input_ids'].to(self.device)
        attention_mask = inputs['attention_mask'].to(self.device)
        
        # Generate embedding
        with torch.no_grad():
            text_features = self.model.get_text_features(
                input_ids=input_ids,
                attention_mask=attention_mask
            )
            
        # Normalize and convert to numpy
        text_embedding = text_features.cpu().numpy()
        text_embedding = text_embedding / np.linalg.norm(text_embedding, axis=1, keepdims=True)
        return text_embedding[0] if isinstance(text, str) else text_embedding

    def compute_similarity(self, 
                         image: Union[str, Path, Image.Image], 
                         text: Union[str, List[str]]) -> float:
        """Compute similarity score between image and text.
        
        Args:
            image: Path to image file or PIL Image object
            text: String or list of strings to compare
            
        Returns:
            float: Similarity score between 0 and 1
        """
        image_embedding = self.get_image_embedding(image)
        text_embedding = self.get_text_embedding(text)
        
        # Compute cosine similarity
        similarity = np.dot(image_embedding, text_embedding.T)
        return float(similarity)

def main():
    parser = argparse.ArgumentParser(description='Generate CLIP embeddings for images or text.')
    parser.add_argument('--mode', choices=['image', 'text'], required=True,
                      help='Whether to generate embedding for image or text')
    
    args = parser.parse_args()
    
    # Initialize embedder
    embedder = ClipEmbedder()
    
    try:
        if args.mode == 'image':
            # Read base64 encoded image from stdin
            input_data = json.loads(sys.stdin.read())
            image_bytes = base64.b64decode(input_data['image'])
            
            # Generate embedding
            embedding = embedder.get_image_embedding(image_bytes)
            
        else:  # text mode
            # Read text from stdin
            input_data = json.loads(sys.stdin.read())
            text = input_data['text']
            
            # Generate embedding
            embedding = embedder.get_text_embedding(text)
        
        # Output embedding as JSON
        result = {
            'embedding': embedding.tolist(),
            'shape': embedding.shape
        }
        json.dump(result, sys.stdout)
        
    except json.JSONDecodeError:
        print("Error: Invalid JSON input", file=sys.stderr)
        sys.exit(1)
    except KeyError as e:
        print(f"Error: Missing required field {e}", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"Error: {str(e)}", file=sys.stderr)
        sys.exit(1)

if __name__ == "__main__":
    """
    Usage examples:
    
    1. For images:
    echo '{"image": "'$(cat image.jpg | base64)'"}' | python clip_embedder.py --mode image
    
    2. For text:
    echo '{"text": "a photo of a dog"}' | python clip_embedder.py --mode text
    """
    main() 
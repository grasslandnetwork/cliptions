import unittest
from clip_embedder import ClipEmbedder
from PIL import Image
import numpy as np
import base64
import json
import io
import os

class TestClipEmbedder(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        """Initialize the CLIP embedder once for all tests."""
        cls.embedder = ClipEmbedder()
        cls.test_image_path = "example.jpg"
        
        # Ensure test image exists
        if not os.path.exists(cls.test_image_path):
            raise FileNotFoundError(f"Test image {cls.test_image_path} not found")

    def test_image_embedding_from_path(self):
        """Test generating embedding from image path."""
        embedding = self.embedder.get_image_embedding(self.test_image_path)
        
        self.assertIsInstance(embedding, np.ndarray)
        self.assertEqual(embedding.shape, (512,))  # CLIP's standard embedding size
        self.assertAlmostEqual(np.linalg.norm(embedding), 1.0, places=6)  # Should be normalized

    def test_image_embedding_from_pil(self):
        """Test generating embedding from PIL Image."""
        image = Image.open(self.test_image_path)
        embedding = self.embedder.get_image_embedding(image)
        
        self.assertIsInstance(embedding, np.ndarray)
        self.assertEqual(embedding.shape, (512,))
        self.assertAlmostEqual(np.linalg.norm(embedding), 1.0, places=6)

    def test_image_embedding_from_bytes(self):
        """Test generating embedding from bytes."""
        with open(self.test_image_path, 'rb') as f:
            image_bytes = f.read()
        embedding = self.embedder.get_image_embedding(image_bytes)
        
        self.assertIsInstance(embedding, np.ndarray)
        self.assertEqual(embedding.shape, (512,))
        self.assertAlmostEqual(np.linalg.norm(embedding), 1.0, places=6)

    def test_text_embedding_single(self):
        """Test generating embedding from single text string."""
        text = "a photo of a dog"
        embedding = self.embedder.get_text_embedding(text)
        
        self.assertIsInstance(embedding, np.ndarray)
        self.assertEqual(embedding.shape, (512,))
        self.assertAlmostEqual(np.linalg.norm(embedding), 1.0, places=6)

    def test_text_embedding_batch(self):
        """Test generating embeddings from multiple texts."""
        texts = ["a photo of a dog", "a photo of a cat"]
        embeddings = self.embedder.get_text_embedding(texts)
        
        self.assertIsInstance(embeddings, np.ndarray)
        self.assertEqual(embeddings.shape, (2, 512))
        for embedding in embeddings:
            self.assertAlmostEqual(np.linalg.norm(embedding), 1.0, places=6)

    def test_compute_similarity(self):
        """Test computing similarity between image and text."""
        similarity = self.embedder.compute_similarity(
            self.test_image_path,
            "a photo of a dog"
        )
        
        self.assertIsInstance(similarity, float)
        self.assertGreaterEqual(similarity, -1.0)
        self.assertLessEqual(similarity, 1.0)

    def test_cli_image_input(self):
        """Test CLI image input processing."""
        # Prepare base64 encoded image
        with open(self.test_image_path, 'rb') as f:
            image_base64 = base64.b64encode(f.read()).decode()
        
        # Create input JSON
        input_data = {'image': image_base64}
        
        # Convert to bytes for stdin simulation
        input_bytes = json.dumps(input_data).encode()
        
        # Create file-like object for stdin
        stdin = io.BytesIO(input_bytes)
        
        # TODO: Test main() function with stdin input
        # This would require mocking sys.stdin and sys.stdout

    def test_cli_text_input(self):
        """Test CLI text input processing."""
        # Create input JSON
        input_data = {'text': 'a photo of a dog'}
        
        # Convert to bytes for stdin simulation
        input_bytes = json.dumps(input_data).encode()
        
        # Create file-like object for stdin
        stdin = io.BytesIO(input_bytes)
        
        # TODO: Test main() function with stdin input
        # This would require mocking sys.stdin and sys.stdout

if __name__ == '__main__':
    unittest.main() 
import unittest
from core.clip_embedder import ClipEmbedder
from PIL import Image
import numpy as np
import base64
import json
import io
import os
from unittest.mock import patch
import sys
import hashlib
import warnings

class TestClipEmbedder(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        """Initialize the CLIP embedder once for all tests."""
        cls.embedder = ClipEmbedder()
        cls.test_image_path = os.path.join("tests", "fixtures", "example.jpg")
        
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
        input_str = json.dumps(input_data)
        
        # Mock stdin and stdout
        with patch('sys.stdin', io.StringIO(input_str)), \
             patch('sys.stdout', new_callable=io.StringIO) as mock_stdout, \
             patch('sys.argv', ['clip_embedder.py', '--mode', 'image']):
            
            from core.clip_embedder import main
            main()
            
            # Parse output
            output = json.loads(mock_stdout.getvalue())
            
            # Verify output structure
            self.assertIn('embedding', output)
            self.assertIn('shape', output)
            
            # Verify embedding properties
            embedding = np.array(output['embedding'])
            self.assertEqual(output['shape'], [512])
            self.assertEqual(embedding.shape, (512,))
            self.assertAlmostEqual(np.linalg.norm(embedding), 1.0, places=6)

    def test_cli_text_input(self):
        """Test CLI text input processing."""
        # Create input JSON
        input_data = {'text': 'a photo of a dog'}
        input_str = json.dumps(input_data)
        
        # Mock stdin and stdout
        with patch('sys.stdin', io.StringIO(input_str)), \
             patch('sys.stdout', new_callable=io.StringIO) as mock_stdout, \
             patch('sys.argv', ['clip_embedder.py', '--mode', 'text']):
            
            from core.clip_embedder import main
            main()
            
            # Parse output
            output = json.loads(mock_stdout.getvalue())
            
            # Verify output structure
            self.assertIn('embedding', output)
            self.assertIn('shape', output)
            
            # Verify embedding properties
            embedding = np.array(output['embedding'])
            self.assertEqual(output['shape'], [512])
            self.assertEqual(embedding.shape, (512,))
            self.assertAlmostEqual(np.linalg.norm(embedding), 1.0, places=6)

    def test_cli_invalid_json(self):
        """Test CLI handling of invalid JSON input."""
        invalid_input = "not valid json"
        
        with patch('sys.stdin', io.StringIO(invalid_input)), \
             patch('sys.stderr', new_callable=io.StringIO) as mock_stderr, \
             patch('sys.argv', ['clip_embedder.py', '--mode', 'text']):
            
            from core.clip_embedder import main
            with self.assertRaises(SystemExit):
                main()
            
            self.assertIn("Invalid JSON input", mock_stderr.getvalue())

    def test_cli_missing_field(self):
        """Test CLI handling of missing required field."""
        input_data = {'wrong_field': 'some value'}
        input_str = json.dumps(input_data)
        
        with patch('sys.stdin', io.StringIO(input_str)), \
             patch('sys.stderr', new_callable=io.StringIO) as mock_stderr, \
             patch('sys.argv', ['clip_embedder.py', '--mode', 'text']):
            
            from core.clip_embedder import main
            with self.assertRaises(SystemExit):
                main()
            
            self.assertIn("Missing required field", mock_stderr.getvalue())

    def test_cli_invalid_mode(self):
        """Test CLI handling of invalid mode argument."""
        input_data = {'text': 'some text'}
        input_str = json.dumps(input_data)
        
        with patch('sys.stdin', io.StringIO(input_str)), \
             patch('sys.stderr', new_callable=io.StringIO) as mock_stderr, \
             patch('sys.argv', ['clip_embedder.py', '--mode', 'invalid_mode']):
            
            from core.clip_embedder import main
            with self.assertRaises(SystemExit):
                main()

    def test_deterministic_embedding(self):
        """Test that example.jpg produces consistent embedding vectors.
        
        Note: This test is informative rather than enforcing. Different platforms
        or hardware may produce slightly different embeddings while still being
        functionally equivalent. A warning will be issued if embeddings differ,
        but the test will not fail.
        """
        # Get embedding from example.jpg
        embedding = self.embedder.get_image_embedding(self.test_image_path)
        
        # Convert embedding to bytes (ensure consistent byte representation)
        embedding_bytes = embedding.tobytes()
        
        # Generate hash
        current_hash = hashlib.sha256(embedding_bytes).hexdigest()
        
        # Known hash from reference embedding
        expected_hash = "e1e0d1e0969ed2544eff4c80a4d52c7501d7a6efcbf398d3e8a6509d5a4e1de1"
        
        # Compare hashes, but only warn if different
        if current_hash != expected_hash:
            warnings.warn(
                f"\nEmbedding hash differs from reference value.\n"
                f"Expected: {expected_hash}\n"
                f"Got:      {current_hash}\n"
                f"This is okay if similarity scores remain consistent.\n"
                f"Different platforms may produce slightly different embeddings."
            )

    def test_semantic_similarity_scores(self):
        """Test that example.jpg produces semantically meaningful similarity scores.
        
        The test image shows the interior of a cat rescue center, featuring:
        - Multiple cats
        - Cat toys and beds
        - Women caring for the cats
        
        This test verifies that the embedding maintains expected semantic
        relationships by checking the relative ordering of similarity scores.
        
        Note on similarity calculation:
        The cosine similarity between vectors a and b is normally:
            cos(θ) = (a · b) / (|a| * |b|)
        where |a| and |b| are the vector magnitudes.
        
        Since both embeddings are normalized to length 1, this simplifies to:
            cos(θ) = a · b
        """
        # Get embedding from example.jpg
        embedding = self.embedder.get_image_embedding(self.test_image_path)
        
        # Test shape
        self.assertEqual(embedding.shape, (512,))
        
        # Test normalization
        self.assertAlmostEqual(np.linalg.norm(embedding), 1.0, places=6)
        
        # Test prompts in expected order of similarity (most similar to least similar)
        test_prompts = [
            "a photo of cats and cat toys indoors",              # Should match best
            "a photo of a cat shelter with cats and caretakers",  # Should match well
            "a photo of a dog kennel",                           # Should match less
            "a photo of an outdoor landscape"                    # Should match least
        ]
        
        # Get similarity scores
        scores = []
        for prompt in test_prompts:
            text_embedding = self.embedder.get_text_embedding(prompt)
            # Verify text embedding is also normalized
            self.assertAlmostEqual(np.linalg.norm(text_embedding), 1.0, places=6)
            # Calculate cosine similarity
            score = float(np.dot(embedding, text_embedding))
            scores.append(score)
        
        # Verify relative ordering of similarities matches expected order
        sorted_scores = sorted(scores, reverse=True)
        self.assertEqual(
            scores, 
            sorted_scores,
            "Semantic relationships don't match expected ordering.\n"
            f"Expected decreasing similarity for:\n"
            f"{test_prompts}\n"
            f"Got scores: {scores}"
        )

if __name__ == '__main__':
    unittest.main() 
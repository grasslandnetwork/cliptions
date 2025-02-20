# realmir

## CLIP Embedder

The CLIP embedder generates embeddings for images and text using OpenAI's CLIP model. It can be used from the command line and accepts input via stdin.

### Installation

```bash
pip install torch transformers Pillow numpy
```

### First Run Behavior
On first run, the script will download the CLIP model (approximately 605MB). This will:
- Take about 15-20 seconds
- Show download progress bars
- Cache the model locally for future use

Subsequent runs will use the cached model and be much faster (1-2 seconds).

### Windows Setup Notes
When running on Windows, you might see a warning about symlinks in the Hugging Face cache system. You have two options:

1. **Enable Developer Mode (Recommended)**
   - Open Windows Settings
   - Navigate to Privacy & Security > For Developers
   - Enable "Developer Mode"

2. **Run as Administrator**
   - Run Python/command prompt as administrator

Alternatively, you can suppress the warning by setting an environment variable:
```bash
set HF_HUB_DISABLE_SYMLINKS_WARNING=1
```

The model will still work without these changes, but caching might be less efficient.

### Usage

#### Generate Image Embeddings
```bash
echo '{"image": "'$(cat image.jpg | base64)'"}' | python clip_embedder.py --mode image
```

#### Generate Text Embeddings
```bash
echo '{"text": "a photo of a dog"}' | python clip_embedder.py --mode text
```

### Output Format
The script outputs JSON to stdout with the following structure:
```json
{
    "embedding": [0.1, 0.2, ...],  // embedding vector
    "shape": [512]                 // shape of the embedding
}
```

### Error Handling
Errors are written to stderr with descriptive messages for:
- Invalid JSON input
- Missing required fields
- Image processing errors
- Model inference errors

### Testing
To run the tests, you'll need:
1. The test image file (`example.jpg`) in your project directory
2. All dependencies installed

Run the tests with:
```bash
python -m unittest test_clip_embedder.py
```

The test suite verifies:

#### Core Functionality
- Image embedding generation
  - From file path
  - From PIL Image object
  - From bytes
- Text embedding generation
  - Single text input
  - Batch text input
- Embedding properties
  - 512 dimensions
  - Normalized vectors
- Similarity computation between images and text

#### CLI Interface
- Image input processing via stdin
- Text input processing via stdin
- JSON output format
- Error handling
  - Invalid JSON input
  - Missing required fields
  - Invalid mode arguments

## Contributing

### Git Workflow
Development follows the [git flow](https://datasift.github.io/gitflow/IntroducingGitFlow.html) methodology.

We recommend using [gitflow-avh](https://github.com/petervanderdoes/gitflow-avh/wiki) with the following settings:

```
Branch name for production releases: master 
Branch name for "next release" development: develop 
Feature branch prefix: feature/ 
Bugfix branch prefix: bugfix/ 
Release branch prefix: release/ 
Hotfix branch prefix: hotfix/ 
Support branch prefix: support/ 
Version tag prefix:
```

# RealMir Steganography & Proof of Work System

This document describes the enhanced commitment system with CLIP vector steganography and proof of work for spam prevention, as discussed in the team Slack thread.

## Overview

The enhanced system prevents spam and requires actual work from both miners and validators by:

1. **CLIP Vector Commitments**: Miners must provide CLIP vectors of their predictions
2. **Steganography**: CLIP vectors are embedded in images using LSB encoding
3. **Proof of Work**: Both commitment and reveal phases require computational work
4. **Enhanced Verification**: Validators can verify both text predictions and CLIP vectors

## Architecture

### Commitment Phase
```
hash = sha256(plaintext_prediction || salt || clip_vector)
```

### Reveal Phase
- Post plaintext prediction + salt (as before)
- Post image with embedded CLIP vector using steganography
- Optional: Include proof of work nonce for extra verification

## CLI Tools

### 1. Generate Enhanced Commitment

Generate a commitment with CLIP vector and proof of work:

```bash
cargo run --bin generate_enhanced_commitment -- \
  --prediction "The cat will be orange" \
  --round-id "round_001" \
  --difficulty 4 \
  --output-json commitment.json
```

Options:
- `--prediction`: Your prediction text
- `--round-id`: Round identifier  
- `--salt`: Optional salt (generated if not provided)
- `--difficulty`: Proof of work difficulty (1-10, default: 4)
- `--mock`: Use mock embedder for testing
- `--output-json`: Save commitment data to JSON file
- `--verbose`: Show detailed output
- `--skip-pow`: Skip proof of work (for testing)

### 2. Encode Vector in Image

Embed a CLIP vector into an image using steganography:

```bash
cargo run --bin encode_vector -- \
  --input source_image.png \
  --output encoded_image.png \
  --text "The cat will be orange" \
  --salt "your_salt_here" \
  --round-id "round_001" \
  --create-test-image
```

Options:
- `--input`: Source image path
- `--output`: Output image path
- `--text`: Text to generate CLIP vector for
- `--salt`: Salt used in commitment
- `--round-id`: Round identifier
- `--mock`: Use mock embedder for testing
- `--bits-per-channel`: Steganography bits per channel (1-3 recommended)
- `--create-test-image`: Create test image if input doesn't exist
- `--image-size`: Test image dimensions (default: 512)

### 3. Decode Vector from Image

Extract and verify CLIP vector from an image:

```bash
cargo run --bin decode_vector -- \
  --input encoded_image.png \
  --text "The cat will be orange" \
  --salt "your_salt_here" \
  --commitment "commitment_hash_here" \
  --output-json extracted_data.json \
  --verbose
```

Options:
- `--input`: Image containing embedded vector
- `--text`: Text to verify against (optional)
- `--salt`: Salt for commitment verification (optional)
- `--commitment`: Commitment hash to verify against (optional)
- `--mock`: Use mock embedder for testing
- `--bits-per-channel`: Steganography bits per channel (must match encoding)
- `--output-json`: Save extracted data to JSON
- `--verbose`: Show detailed vector information

## Workflow Example

### Step 1: Generate Enhanced Commitment

```bash
# Generate commitment with proof of work
cargo run --bin generate_enhanced_commitment -- \
  --prediction "A fluffy orange cat" \
  --round-id "round_123" \
  --difficulty 4 \
  --output-json my_commitment.json

# Output:
# Commitment Hash: a1b2c3d4e5f6...
# Salt: f7e8d9c0b1a2...
```

### Step 2: Post Commitment to Twitter

```
Commit: a1b2c3d4e5f6...
Wallet: 0xYourWalletAddress
```

### Step 3: Prepare Reveal Image

```bash
# Embed CLIP vector in image
cargo run --bin encode_vector -- \
  --input my_photo.jpg \
  --output reveal_image.png \
  --text "A fluffy orange cat" \
  --salt "f7e8d9c0b1a2..." \
  --round-id "round_123" \
  --create-test-image
```

### Step 4: Post Reveal to Twitter

```
Prediction: A fluffy orange cat
Salt: f7e8d9c0b1a2...
[Attach reveal_image.png]
```

### Step 5: Validators Verify

```bash
# Validators can extract and verify
cargo run --bin decode_vector -- \
  --input reveal_image.png \
  --text "A fluffy orange cat" \
  --salt "f7e8d9c0b1a2..." \
  --commitment "a1b2c3d4e5f6..." \
  --verbose
```

## Technical Details

### Steganography

- Uses LSB (Least Significant Bit) encoding
- Embeds data in RGB channels of PNG images
- Configurable bits per channel (1-3 recommended for stealth)
- Includes metadata: version, dimension, salt, round ID
- Magic header "RMCLIP" for data identification

### Proof of Work

- Hashcash-style proof of work using SHA-256
- Configurable difficulty (number of leading zeros)
- Includes challenge string with prediction, salt, and round ID
- Prevents spam by requiring computational work
- Timeout protection (30 seconds default)

### CLIP Vector Format

- 512 or 768 dimensional vectors (depending on model)
- f64 precision for accuracy
- Normalized vectors for consistency
- Supports both real CLIP models and mock embedders for testing

## Security Considerations

### Attack Vectors & Mitigations

1. **Copy-paste vectors**: Mitigated by including salt in commitment
2. **Fake vectors**: Prevented by commitment scheme tying everything together
3. **Spam attacks**: Mitigated by proof of work requirement
4. **Vector reuse**: Prevented by per-round salts

### Recommended Settings

- **Difficulty**: 4-6 for production (balances security vs. UX)
- **Bits per channel**: 2 (good balance of capacity vs. detectability)
- **Image size**: 512x512 minimum for 512-dimensional vectors
- **Salt length**: 32 bytes (default)

## Integration with Existing System

The enhanced system is backward compatible:

```rust
// Basic commitment (existing)
let basic_commitment = generator.generate("prediction", "salt")?;

// Enhanced commitment (new)
let enhanced_commitment = enhanced_generator.generate_enhanced(
    "prediction", 
    "salt", 
    &clip_vector, 
    "round_id"
)?;
```

## Development and Testing

### Mock Mode

Use `--mock` flag to use deterministic mock embedders for testing:

```bash
cargo run --bin generate_enhanced_commitment -- \
  --prediction "Test prediction" \
  --round-id "test" \
  --mock \
  --skip-pow
```

### Test Suite

Run comprehensive tests:

```bash
cargo test steganography
cargo test proof_of_work  
cargo test commitment::enhanced
```

## Performance Considerations

### Proof of Work Times

- Difficulty 1: ~10ms
- Difficulty 4: ~1-10 seconds
- Difficulty 6: ~1-5 minutes
- Difficulty 8: ~10-30 minutes

### Image Processing

- Encoding: ~100-500ms for 512x512 image
- Decoding: ~50-200ms for 512x512 image
- CLIP vector generation: ~1-3 seconds

### Storage Requirements

- 512-dimensional vector: ~4KB
- Metadata: ~200 bytes
- Total overhead: ~4.2KB minimum
- 512x512 image capacity (2 bits/channel): ~96KB

## Troubleshooting

### Common Issues

1. **"Insufficient capacity"**: Use larger image or fewer bits per channel
2. **"No embedded data found"**: Check bits per channel setting matches encoding
3. **"CLIP model not found"**: Use `--mock` flag or install CLIP model
4. **"Proof of work timeout"**: Reduce difficulty or increase timeout

### Debugging

Enable verbose output for detailed information:

```bash
cargo run --bin decode_vector -- --input image.png --verbose
```

## Contributing

When adding new features:

1. Update relevant error types in `src/error.rs`
2. Add comprehensive tests
3. Update CLI help text
4. Consider backward compatibility
5. Update this documentation

## License

This implementation is part of the RealMir project and follows the same license terms.
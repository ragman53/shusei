# OCR Models

This directory contains bundled ONNX models for the NDLOCR-Lite OCR engine.

## Models

The OCR pipeline uses PaddleOCR v5 models converted to ONNX format:

| Model | File | Size | Purpose |
|-------|------|------|---------|
| Text Detection | `models/text_detection.onnx` | 84 MB | Detects text regions in images |
| Text Recognition | `models/text_recognition.onnx` | 81 MB | Recognizes text in detected regions |
| Character Dictionary | `models/dict.txt` | 73 KB | Character mapping for recognition |

## Source

Models are from the [PaddleOCR ONNX collection](https://huggingface.co/monkt/paddleocr-onnx) by PaddlePaddle Team.

- **Detection**: `detection/v5/det.onnx`
- **Recognition**: `languages/chinese/rec.onnx` (supports Chinese and Japanese)
- **Dictionary**: `languages/chinese/dict.txt`

## License

Apache 2.0 - Commercial use allowed.

## Download Instructions

If models are missing, download them manually:

```bash
# Detection model (84 MB)
curl -L -o assets/ocr/models/text_detection.onnx \
  "https://huggingface.co/monkt/paddleocr-onnx/resolve/main/detection/v5/det.onnx"

# Recognition model (81 MB)
curl -L -o assets/ocr/models/text_recognition.onnx \
  "https://huggingface.co/monkt/paddleocr-onnx/resolve/main/languages/chinese/rec.onnx"

# Dictionary (73 KB)
curl -L -o assets/ocr/models/dict.txt \
  "https://huggingface.co/monkt/paddleocr-onnx/resolve/main/languages/chinese/dict.txt"
```

## Supported Languages

- Japanese (Hiragana, Katakana, Kanji)
- Chinese (Simplified, Traditional)
- English (basic support)

## Usage

The models are automatically loaded by `NdlocrEngine::initialize()` when the application starts.

```rust
let mut engine = NdlocrEngine::new("assets/ocr/models", "ja");
engine.initialize().await?;
let result = engine.process_image(&image_bytes).await?;
```

## Performance

- Detection: ~200-500ms per page (CPU)
- Recognition: ~500-1000ms per page (CPU)
- Total: < 2 seconds per page on mid-range devices

## Model Specifications

- **ONNX Opset**: 11
- **Precision**: FP32
- **Detection Input**: `[1, 3, H, W]` (dynamic size)
- **Recognition Input**: `[1, 3, 32, W]` (height fixed at 32px)

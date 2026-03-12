# PaddleOCR ONNX Models

Bundled OCR models for the Shusei reading app.

## Model Source

- **Repository**: [monkt/paddleocr-onnx](https://huggingface.co/monkt/paddleocr-onnx)
- **Original Framework**: PaddlePaddle PP-OCRv5
- **License**: Apache 2.0
- **Format**: ONNX (Opset 11, FP32)

## Included Models

### Detection Model
- **File**: `text_detection.onnx` (84 MB)
- **Source**: `detection/v5/det.onnx`
- **Input**: `[batch, 3, height, width]` - dynamic size
- **Output**: Text bounding boxes
- **Purpose**: Detects text regions in images

### Recognition Model
- **File**: `text_recognition.onnx` (81 MB)
- **Source**: `languages/chinese/rec.onnx`
- **Input**: `[batch, 3, 32, width]` - height fixed at 32px
- **Output**: CTC logits → decoded with dictionary
- **Purpose**: Recognizes text in detected regions
- **Languages**: Chinese (Simplified, Traditional), Japanese (Hiragana, Katakana, Kanji)

### Dictionary
- **File**: `dict.txt` (73 KB)
- **Source**: `languages/chinese/dict.txt`
- **Purpose**: Character mapping for recognition decoding

## Download Instructions

If models are missing, download manually:

```bash
# Detection model
curl -L -o assets/ocr/models/text_detection.onnx \
  "https://huggingface.co/monkt/paddleocr-onnx/resolve/main/detection/v5/det.onnx"

# Recognition model (Chinese/Japanese)
curl -L -o assets/ocr/models/text_recognition.onnx \
  "https://huggingface.co/monkt/paddleocr-onnx/resolve/main/languages/chinese/rec.onnx"

# Dictionary
curl -L -o assets/ocr/models/dict.txt \
  "https://huggingface.co/monkt/paddleocr-onnx/resolve/main/languages/chinese/dict.txt"
```

## Model Selection

For other languages, download from the same repository:

| Language | Recognition Path |
|----------|------------------|
| English | `languages/english/rec.onnx` |
| Latin (FR/DE/ES/IT/PT + 27 more) | `languages/latin/rec.onnx` |
| Korean | `languages/korean/rec.onnx` |
| Thai | `languages/thai/rec.onnx` |
| Greek | `languages/greek/rec.onnx` |

Always use `detection/v5/det.onnx` with v5 recognition models.

## Performance

- **Inference Time**: ~500ms-2s per page (CPU)
- **Accuracy**: 85%+ on clear text
- **Memory**: ~200MB during inference

## Usage

```rust
use shusei::core::ocr::{NdlocrEngine, get_model_path};

// Initialize engine with bundled models
let mut engine = NdlocrEngine::new(get_model_path("detection"), "ja");
engine.initialize().await?;

// Process image
let result = engine.process_image(&image_bytes).await?;
println!("OCR Result: {}", result.markdown);
```

## Notes

- Models are NOT tracked in Git (large binary files)
- Download on first build or use git-lfs
- Verify file integrity before use

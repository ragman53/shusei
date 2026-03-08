//! NDLOCR-Lite ONNX tract compatibility test
//!
//! This test verifies that tract-onnx can load and run NDLOCR-Lite ONNX models.
//!
//! ## Models Required
//!
//! The NDLOCR-Lite models can be obtained from:
//! - Main repository: https://github.com/ndl-lab/ndlocr_ocr
//! - Lite models are typically smaller optimized versions for mobile/embedded use
//!
//! Required model files:
//! - `text_detection.onnx` - Text detection model (typically ~2-5MB)
//! - `text_recognition.onnx` - Text recognition model (typically ~5-10MB)  
//! - `direction_classifier.onnx` - Text direction classifier (typically ~1-2MB)
//!
//! ## Running the Test
//!
//! 1. Download the NDLOCR-Lite ONNX models
//! 2. Place them in `assets/models/ndlocr/` directory
//! 3. Run: `cargo test ndlocr_tract_test --features ndlocr-test`
//!
//! ## Model Input/Output Specifications (Expected)
//!
//! ### text_detection.onnx
//! - Input: Image tensor [1, 3, H, W] (typically 720x1280 or similar)
//! - Output: Text region probability map
//!
//! ### text_recognition.onnx
//! - Input: Cropped text image [1, 3, 32, W] (height typically 32)
//! - Output: Character sequence probabilities
//!
//! ### direction_classifier.onnx
//! - Input: Cropped text region [1, 3, H, W]
//! - Output: Direction classification (0, 90, 180, 270 degrees)

use std::path::PathBuf;

/// Model directory path - adjust as needed for your setup
fn get_model_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("models")
        .join("ndlocr")
}

/// Check if all required model files exist
fn models_exist() -> bool {
    let model_dir = get_model_dir();
    let detection = model_dir.join("text_detection.onnx");
    let recognition = model_dir.join("text_recognition.onnx");
    let direction = model_dir.join("direction_classifier.onnx");
    
    detection.exists() && recognition.exists() && direction.exists()
}

/// Get list of missing model files
fn get_missing_models() -> Vec<String> {
    let model_dir = get_model_dir();
    let mut missing = Vec::new();
    
    let models = [
        ("text_detection.onnx", model_dir.join("text_detection.onnx")),
        ("text_recognition.onnx", model_dir.join("text_recognition.onnx")),
        ("direction_classifier.onnx", model_dir.join("direction_classifier.onnx")),
    ];
    
    for (name, path) in models {
        if !path.exists() {
            missing.push(name.to_string());
        }
    }
    
    missing
}

#[cfg(feature = "ndlocr-test")]
mod tract_tests {
    use super::*;
    use tract_onnx::prelude::*;

    /// Load and analyze an ONNX model file
    fn load_onnx_model(model_path: &PathBuf) -> Result<TypedModel, String> {
        tract_onnx::onnx()
            .model_for_path(model_path)
            .map_err(|e| format!("Failed to load model: {:?}", e))?
            .into_optimized()
            .map_err(|e| format!("Failed to optimize model: {:?}", e))?
            .into_run_config()
            .map_err(|e| format!("Failed to configure model: {:?}", e))
    }

    /// Test text_detection.onnx compatibility
    #[test]
    fn test_text_detection_model() {
        let model_dir = get_model_dir();
        let model_path = model_dir.join("text_detection.onnx");
        
        if !model_path.exists() {
            eprintln!("SKIP: text_detection.onnx not found at {:?}", model_path);
            return;
        }
        
        println!("Testing text_detection.onnx compatibility...");
        
        // Attempt to load model
        match load_onnx_model(&model_path) {
            Ok(model) => {
                println!("✓ Model loaded successfully");
                
                // Print input specifications
                println!("\nModel Inputs:");
                for (idx, input) in model.inputs.iter().enumerate() {
                    println!("  [{}] {:?}: {:?}", idx, input.name, input.fact);
                }
                
                // Print output specifications
                println!("\nModel Outputs:");
                for (idx, output) in model.outputs.iter().enumerate() {
                    println!("  [{}] {:?}", idx, output.fact);
                }
                
                // Print symbol table for shapes
                println!("\nSymbol Table:");
                for (sym, value) in model.symbol_table.iter() {
                    println!("  {:?} = {:?}", sym, value);
                }
                
                println!("✓ text_detection.onnx is compatible with tract");
            }
            Err(e) => {
                panic!("✗ Failed to load text_detection.onnx: {}", e);
            }
        }
    }

    /// Test text_recognition.onnx compatibility
    #[test]
    fn test_text_recognition_model() {
        let model_dir = get_model_dir();
        let model_path = model_dir.join("text_recognition.onnx");
        
        if !model_path.exists() {
            eprintln!("SKIP: text_recognition.onnx not found at {:?}", model_path);
            return;
        }
        
        println!("Testing text_recognition.onnx compatibility...");
        
        match load_onnx_model(&model_path) {
            Ok(model) => {
                println!("✓ Model loaded successfully");
                
                println!("\nModel Inputs:");
                for (idx, input) in model.inputs.iter().enumerate() {
                    println!("  [{}] {:?}: {:?}", idx, input.name, input.fact);
                }
                
                println!("\nModel Outputs:");
                for (idx, output) in model.outputs.iter().enumerate() {
                    println!("  [{}] {:?}", idx, output.fact);
                }
                
                println!("\nSymbol Table:");
                for (sym, value) in model.symbol_table.iter() {
                    println!("  {:?} = {:?}", sym, value);
                }
                
                println!("✓ text_recognition.onnx is compatible with tract");
            }
            Err(e) => {
                panic!("✗ Failed to load text_recognition.onnx: {}", e);
            }
        }
    }

    /// Test direction_classifier.onnx compatibility
    #[test]
    fn test_direction_classifier_model() {
        let model_dir = get_model_dir();
        let model_path = model_dir.join("direction_classifier.onnx");
        
        if !model_path.exists() {
            eprintln!("SKIP: direction_classifier.onnx not found at {:?}", model_path);
            return;
        }
        
        println!("Testing direction_classifier.onnx compatibility...");
        
        match load_onnx_model(&model_path) {
            Ok(model) => {
                println!("✓ Model loaded successfully");
                
                println!("\nModel Inputs:");
                for (idx, input) in model.inputs.iter().enumerate() {
                    println!("  [{}] {:?}: {:?}", idx, input.name, input.fact);
                }
                
                println!("\nModel Outputs:");
                for (idx, output) in model.outputs.iter().enumerate() {
                    println!("  [{}] {:?}", idx, output.fact);
                }
                
                println!("\nSymbol Table:");
                for (sym, value) in model.symbol_table.iter() {
                    println!("  {:?} = {:?}", sym, value);
                }
                
                println!("✓ direction_classifier.onnx is compatible with tract");
            }
            Err(e) => {
                panic!("✗ Failed to load direction_classifier.onnx: {}", e);
            }
        }
    }

    /// Test inference with dummy input (if models exist)
    #[test]
    fn test_inference_with_dummy_input() {
        let model_dir = get_model_dir();
        let model_path = model_dir.join("text_recognition.onnx");
        
        if !model_path.exists() {
            eprintln!("SKIP: text_recognition.onnx not found for inference test");
            return;
        }
        
        println!("Testing inference with dummy input...");
        
        // Load model
        let model = tract_onnx::onnx()
            .model_for_path(&model_path)
            .expect("Failed to load model")
            .into_optimized()
            .expect("Failed to optimize model")
            .into_run_config()
            .expect("Failed to configure model");
        
        // Create dummy input (adjust shape based on actual model requirements)
        // This is a placeholder - actual shape needs to match model input
        let input_shape: TVec<usize> = tvec!(1, 3, 32, 100); // Typical for recognition
        
        // Create dummy input tensor (zeros)
        let dummy_input: Tensor = tract_ndarray::ArrayD::zeros(&input_shape).into();
        
        println!("  Input shape: {:?}", input_shape);
        
        // Run inference
        let result = model.run(tvec!(dummy_input.into()));
        
        match result {
            Ok(outputs) => {
                println!("  ✓ Inference successful");
                for (idx, output) in outputs.iter().enumerate() {
                    println!("  Output [{}]: shape {:?}", idx, output.shape());
                }
            }
            Err(e) => {
                panic!("  ✗ Inference failed: {:?}", e);
            }
        }
    }
}

#[test]
fn test_models_existence_check() {
    println!("\n=== NDLOCR-Lite Model Existence Check ===");
    println!("Model directory: {:?}", get_model_dir());
    
    if models_exist() {
        println!("✓ All required NDLOCR-Lite models are present");
    } else {
        let missing = get_missing_models();
        println!("✗ Missing models: {:?}", missing);
        println!("\nTo obtain the models:");
        println!("1. Visit: https://github.com/ndl-lab/ndlocr_ocr");
        println!("2. Download the NDLOCR-Lite ONNX models");
        println!("3. Place them in: {:?}", get_model_dir());
    }
}

#[test]
fn test_tract_onnx_available() {
    println!("\n=== tract-onnx Availability Check ===");
    println!("tract-onnx version: {}", env!("CARGO_PKG_VERSION"));
    println!("✓ tract-onnx crate is available");
    
    // Verify tract types are accessible
    use tract_onnx::prelude::*;
    println!("✓ tract-onnx prelude types are accessible");
}
//! Tract ONNX utilities - shared helpers for tract inference

use anyhow::{Context, Result};
use ndarray::{Array4, Array2};
use tract_onnx::prelude::*;

/// Type alias for tract ONNX model
pub type TractModel = SimplePlan<TypedFact, Box<dyn TypedOp>, Graph<TypedFact, Box<dyn TypedOp>>>;

/// Load an ONNX model from file into a tract session
pub fn load_model(model_path: &std::path::Path) -> Result<TractModel> {
    let model = tract_onnx::onnx()
        .model_for_path(model_path)
        .with_context(|| format!("Failed to load ONNX model from {:?}", model_path))?
        .into_optimized()?
        .into_runnable()?;
    
    Ok(model)
}

/// Convert Array4<f32> to tract Tensor
pub fn array4_to_tensor(array: &Array4<f32>) -> Result<Tensor> {
    let shape: Vec<usize> = array.shape().to_vec();
    let data = array.as_slice().context("Failed to get array slice")?;
    
    // Create tensor with proper shape
    let tensor = Tensor::from_shape(&shape, data)
        .with_context(|| format!("Failed to create tensor from shape {:?}", shape))?;
    
    Ok(tensor)
}

/// Convert Array2<f32> to tract Tensor
pub fn array2_to_tensor(array: &Array2<f32>) -> Result<Tensor> {
    let shape: Vec<usize> = array.shape().to_vec();
    let data = array.as_slice().context("Failed to get array slice")?;
    
    let tensor = Tensor::from_shape(&shape, data)
        .with_context(|| format!("Failed to create tensor from shape {:?}", shape))?;
    
    Ok(tensor)
}

/// Run inference with a tract model and return output tensor
pub fn run_inference(
    model: &TractModel,
    input: &Tensor,
) -> Result<Tensor> {
    let outputs = model.run(tvec![input.clone().into()])
        .with_context(|| "Failed to run tract inference")?;
    
    let output = outputs.into_iter()
        .next()
        .with_context(|| "No output from inference")?;
    
    // Convert TValue to Tensor
    Ok(output.into_tensor())
}

/// Extract f32 data from tract Tensor
pub fn extract_tensor_data(tensor: &Tensor) -> Result<(Vec<i64>, Vec<f32>)> {
    let shape: Vec<i64> = tensor.shape().iter().map(|&d| d as i64).collect();
    
    // Convert tensor to array and extract data
    let array = tensor.to_array_view::<f32>()
        .context("Failed to convert tensor to array view")?;
    
    let data = array.as_slice().unwrap_or(&[]).to_vec();
    
    Ok((shape, data))
}

/// Create a tensor from raw f32 data with dynamic shape
pub fn create_tensor(shape: &[usize], data: &[f32]) -> Result<Tensor> {
    Tensor::from_shape(shape, data)
        .with_context(|| format!("Failed to create tensor with shape {:?}", shape))
}

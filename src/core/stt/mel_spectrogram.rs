//! Mel-spectrogram computation for Moonshine STT
//!
//! This module implements audio preprocessing for the Moonshine speech-to-text engine.
//! Moonshine expects log-mel spectrogram input with these specifications:
//!
//! | Parameter | Value |
//! |-----------|-------|
//! | Sample Rate | 16000 Hz |
//! | Window Size | 25ms (400 samples) |
//! | Hop Length | 10ms (160 samples) |
//! | Mel Bins | 80 |
//! | FFT Size | 400 |

use crate::core::error::{SttError, Result};
use ndarray::{Array2, Axis};
use std::f32::consts::PI;

/// Audio preprocessing parameters for Moonshine
pub struct AudioPreprocessor {
    /// Sample rate (16000 Hz)
    pub sample_rate: u32,
    /// Window size in samples (25ms = 400 samples at 16kHz)
    pub window_size: usize,
    /// Hop length in samples (10ms = 160 samples at 16kHz)
    pub hop_length: usize,
    /// Number of mel bins (80)
    pub n_mels: usize,
    /// FFT size (400)
    pub n_fft: usize,
    /// Mel filterbank matrix
    mel_filterbank: Array2<f32>,
    /// Window function (Hann window)
    window: Vec<f32>,
}

impl AudioPreprocessor {
    /// Create a new audio preprocessor with Moonshine default parameters
    pub fn new() -> Self {
        let sample_rate = 16000;
        let window_size = 400; // 25ms
        let hop_length = 160; // 10ms
        let n_mels = 80;
        let n_fft = 400;
        
        let mel_filterbank = create_mel_filterbank(sample_rate, n_fft, n_mels);
        let window = create_hann_window(window_size);
        
        Self {
            sample_rate,
            window_size,
            hop_length,
            n_mels,
            n_fft,
            mel_filterbank,
            window,
        }
    }
    
    /// Create a preprocessor with custom parameters
    pub fn with_params(
        sample_rate: u32,
        window_size: usize,
        hop_length: usize,
        n_mels: usize,
        n_fft: usize,
    ) -> Self {
        let mel_filterbank = create_mel_filterbank(sample_rate, n_fft, n_mels);
        let window = create_hann_window(window_size);
        
        Self {
            sample_rate,
            window_size,
            hop_length,
            n_mels,
            n_fft,
            mel_filterbank,
            window,
        }
    }
    
    /// Preprocess audio samples into log-mel spectrogram
    ///
    /// # Arguments
    /// * `audio` - Raw audio samples (16kHz mono PCM as f32)
    ///
    /// # Returns
    /// * `Ok(Array2<f32>)` - Log-mel spectrogram [time_frames, n_mels]
    /// * `Err(SttError)` - If preprocessing fails
    ///
    /// # Output Shape
    /// The output spectrogram has shape [time_frames, 80] where:
    /// - time_frames = (audio_len - window_size) / hop_length + 1
    /// - 80 = number of mel bins
    pub fn preprocess(&self, audio: &[f32]) -> Result<Array2<f32>> {
        if audio.is_empty() {
            return Err(SttError::Preprocessing("Empty audio input".into()).into());
        }
        
        if audio.len() < self.window_size {
            return Err(SttError::Preprocessing(format!(
                "Audio too short: {} samples (minimum: {})",
                audio.len(),
                self.window_size
            )).into());
        }
        
        log::debug!(
            "Preprocessing audio: {} samples -> spectrogram",
            audio.len()
        );
        
        // Step 1: Compute STFT (Short-Time Fourier Transform)
        let stft = self.compute_stft(audio);
        
        // Step 2: Compute power spectrogram (magnitude squared)
        let power_spectrogram = stft.mapv(|x| x * x);
        
        // Step 3: Apply mel filterbank
        let mel_spectrogram = power_spectrogram.dot(&self.mel_filterbank);
        
        // Step 4: Apply log compression with epsilon to avoid log(0)
        let log_mel_spectrogram = mel_spectrogram.mapv(|x| (x + 1e-10).ln());
        
        log::debug!(
            "Preprocessing complete: {:?} -> {:?}",
            stft.dim(),
            log_mel_spectrogram.dim()
        );
        
        Ok(log_mel_spectrogram)
    }
    
    /// Compute Short-Time Fourier Transform
    fn compute_stft(&self, audio: &[f32]) -> Array2<f32> {
        let num_frames = (audio.len() - self.window_size) / self.hop_length + 1;
        let mut stft = Array2::zeros((num_frames, self.n_fft / 2 + 1));
        
        for frame_idx in 0..num_frames {
            let start = frame_idx * self.hop_length;
            let end = start + self.window_size;
            
            if end > audio.len() {
                break;
            }
            
            // Apply window function
            let mut frame: Vec<f32> = audio[start..end]
                .iter()
                .zip(self.window.iter())
                .map(|(&sample, &window)| sample * window)
                .collect();
            
            // Zero-pad to n_fft if necessary
            if frame.len() < self.n_fft {
                frame.resize(self.n_fft, 0.0);
            }
            
            // Compute FFT and take magnitude
            let fft_result = self.fft(&frame);
            let magnitude = fft_result
                .into_iter()
                .take(self.n_fft / 2 + 1)
                .map(|(re, im)| (re * re + im * im).sqrt())
                .collect::<Vec<f32>>();
            
            for (bin_idx, &mag) in magnitude.iter().enumerate() {
                stft[[frame_idx, bin_idx]] = mag;
            }
        }
        
        stft
    }
    
    /// Compute FFT (Fast Fourier Transform)
    /// Returns vector of (real, imaginary) pairs
    fn fft(&self, samples: &[f32]) -> Vec<(f32, f32)> {
        let n = samples.len();
        if n <= 1 {
            return vec![(samples[0], 0.0)];
        }
        
        // Cooley-Tukey FFT algorithm (radix-2)
        // For simplicity, using DFT for non-power-of-2 sizes
        if n.is_power_of_two() {
            self.fft_radix2(samples)
        } else {
            self.dft(samples)
        }
    }
    
    /// Radix-2 FFT (for power-of-2 sizes)
    fn fft_radix2(&self, samples: &[f32]) -> Vec<(f32, f32)> {
        let n = samples.len();
        if n == 1 {
            return vec![(samples[0], 0.0)];
        }
        
        // Split into even and odd
        let even: Vec<f32> = samples.iter().step_by(2).copied().collect();
        let odd: Vec<f32> = samples.iter().skip(1).step_by(2).copied().collect();
        
        // Recursive FFT
        let even_fft = self.fft_radix2(&even);
        let odd_fft = self.fft_radix2(&odd);
        
        // Combine
        let mut result = vec![(0.0f32, 0.0f32); n];
        for k in 0..n / 2 {
            let angle = -2.0 * PI * k as f32 / n as f32;
            let twiddle = (angle.cos(), angle.sin());
            
            let even_re = even_fft[k].0;
            let even_im = even_fft[k].1;
            let odd_re = odd_fft[k].0;
            let odd_im = odd_fft[k].1;
            
            let twiddle_odd_re = twiddle.0 * odd_re - twiddle.1 * odd_im;
            let twiddle_odd_im = twiddle.0 * odd_im + twiddle.1 * odd_re;
            
            result[k] = (even_re + twiddle_odd_re, even_im + twiddle_odd_im);
            result[k + n / 2] = (even_re - twiddle_odd_re, even_im - twiddle_odd_im);
        }
        
        result
    }
    
    /// DFT (Discrete Fourier Transform) for non-power-of-2 sizes
    fn dft(&self, samples: &[f32]) -> Vec<(f32, f32)> {
        let n = samples.len();
        let mut result = Vec::with_capacity(n);
        
        for k in 0..n {
            let mut re = 0.0f32;
            let mut im = 0.0f32;
            
            for t in 0..n {
                let angle = -2.0 * PI * k as f32 * t as f32 / n as f32;
                re += samples[t] * angle.cos();
                im += samples[t] * angle.sin();
            }
            
            result.push((re, im));
        }
        
        result
    }
}

impl Default for AudioPreprocessor {
    fn default() -> Self {
        Self::new()
    }
}

/// Create mel filterbank matrix
fn create_mel_filterbank(sample_rate: u32, n_fft: usize, n_mels: usize) -> Array2<f32> {
    let nyquist = sample_rate as f32 / 2.0;
    let mel_min = hz_to_mel(0.0);
    let mel_max = hz_to_mel(nyquist);
    
    // Create mel points (evenly spaced in mel scale)
    let mel_points: Vec<f32> = (0..n_mels + 2)
        .map(|i| mel_min + (mel_max - mel_min) * i as f32 / (n_mels + 1) as f32)
        .collect();
    
    // Convert to Hz
    let hz_points: Vec<f32> = mel_points.iter().map(|&mel| mel_to_hz(mel)).collect();
    
    // Convert to FFT bin indices
    let bin_points: Vec<usize> = hz_points
        .iter()
        .map(|&hz| ((hz / nyquist) * (n_fft / 2) as f32).round() as usize)
        .collect();
    
    // Create filterbank matrix
    let mut filterbank = Array2::zeros((n_fft / 2 + 1, n_mels));
    
    for i in 0..n_mels {
        let start = bin_points[i];
        let center = bin_points[i + 1];
        let end = bin_points[i + 2];
        
        // Rising edge
        for j in start..center.min(n_fft / 2) {
            filterbank[[j, i]] = (j - start) as f32 / ((center - start) as f32 + 1e-10);
        }
        
        // Falling edge
        for j in center..end.min(n_fft / 2 + 1) {
            filterbank[[j, i]] = (end - j) as f32 / ((end - center) as f32 + 1e-10);
        }
    }
    
    filterbank
}

/// Create Hann window
fn create_hann_window(size: usize) -> Vec<f32> {
    (0..size)
        .map(|n| 0.5 * (1.0 - (2.0 * PI * n as f32 / size as f32).cos()))
        .collect()
}

/// Convert Hz to mel scale
fn hz_to_mel(hz: f32) -> f32 {
    2595.0 * (1.0 + hz / 700.0).log10()
}

/// Convert mel scale to Hz
fn mel_to_hz(mel: f32) -> f32 {
    700.0 * (10.0_f32.powf(mel / 2595.0) - 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_audio_preprocessor_new() {
        let preprocessor = AudioPreprocessor::new();
        assert_eq!(preprocessor.sample_rate, 16000);
        assert_eq!(preprocessor.window_size, 400);
        assert_eq!(preprocessor.hop_length, 160);
        assert_eq!(preprocessor.n_mels, 80);
        assert_eq!(preprocessor.n_fft, 400);
    }
    
    #[test]
    fn test_hann_window() {
        let window = create_hann_window(400);
        assert_eq!(window.len(), 400);
        // Hann window starts and ends at 0
        assert!((window[0] - 0.0).abs() < 1e-6);
        assert!((window[399] - 0.0).abs() < 1e-6);
        // Middle of window is near 1
        assert!((window[200] - 1.0).abs() < 0.01);
    }
    
    #[test]
    fn test_hz_to_mel() {
        // 1000 Hz should be approximately 1000 mel
        let mel = hz_to_mel(1000.0);
        assert!((mel - 1000.0).abs() < 100.0);
    }
    
    #[test]
    fn test_mel_to_hz() {
        // Round-trip conversion
        let hz = 1000.0;
        let mel = hz_to_mel(hz);
        let hz_back = mel_to_hz(mel);
        assert!((hz - hz_back).abs() < 1.0);
    }
    
    #[test]
    fn test_empty_audio() {
        let preprocessor = AudioPreprocessor::new();
        let result = preprocessor.preprocess(&[]);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_short_audio() {
        let preprocessor = AudioPreprocessor::new();
        let audio = vec![0.0f32; 100]; // Too short
        let result = preprocessor.preprocess(&audio);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_preprocess_output_shape() {
        let preprocessor = AudioPreprocessor::new();
        // 1 second of audio at 16kHz = 16000 samples
        let audio = vec![0.0f32; 16000];
        let result = preprocessor.preprocess(&audio).unwrap();
        
        // Expected time frames: (16000 - 400) / 160 + 1 = 98
        let expected_frames = (16000 - 400) / 160 + 1;
        assert_eq!(result.dim().0, expected_frames);
        assert_eq!(result.dim().1, 80); // n_mels
    }
    
    #[test]
    fn test_fft_radix2() {
        let preprocessor = AudioPreprocessor::new();
        // Simple test: DC signal (all ones)
        let samples = vec![1.0f32; 8];
        let fft = preprocessor.fft_radix2(&samples);
        
        // DC component should be 8, all others should be 0
        assert!((fft[0].0 - 8.0).abs() < 1e-5);
        assert!((fft[0].1 - 0.0).abs() < 1e-5);
        for i in 1..8 {
            assert!((fft[i].0 - 0.0).abs() < 1e-5);
        }
    }
    
    #[test]
    fn test_dft() {
        let preprocessor = AudioPreprocessor::new();
        // Test with non-power-of-2 size
        let samples = vec![1.0f32; 5];
        let fft = preprocessor.dft(&samples);
        
        // DC component should be 5
        assert!((fft[0].0 - 5.0).abs() < 1e-5);
    }
}

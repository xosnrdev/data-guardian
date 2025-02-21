//! Data compression utilities for persisting usage data
//!
//! This module provides compression and decompression functionality
//! specifically optimized for storing process usage data efficiently.
//!
//! # Example
//! ```
//! use data_guardian::compression::{compress_usage_data, decompress_usage_data};
//! use std::collections::HashMap;
//!
//! let mut data = HashMap::new();
//! data.insert("chrome".to_string(), 1024u64);
//!
//! let compressed = compress_usage_data(&data).unwrap();
//! let decompressed = decompress_usage_data(&compressed).unwrap();
//! assert_eq!(data, decompressed);
//! ```

use std::collections::HashMap;
use std::io::{self, Read};

use flate2::{Compression, GzBuilder};
use thiserror::Error;

/// Configuration for compression operations
#[derive(Debug, Clone, Copy)]
pub struct CompressionConfig {
    /// The compression level to use (0-9)
    pub level: u32,
    /// Initial capacity for the output buffer as a multiplier of input size
    pub capacity_multiplier: f32,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            level: 9,                 // Best compression
            capacity_multiplier: 0.5, // Assume 50% compression ratio
        }
    }
}

/// Errors that can occur during compression operations
#[derive(Error, Debug)]
pub enum CompressionError {
    #[error("Failed to serialize data: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("IO error during compression: {0}")]
    Io(#[from] io::Error),
    #[error("Invalid compression level: {0}")]
    InvalidLevel(u32),
}

/// Compress process usage data using gzip with custom configuration
///
/// # Arguments
/// * `data` - The usage data to compress
/// * `config` - Configuration for compression parameters
///
/// # Returns
/// * `Ok(Vec<u8>)` - The compressed data
/// * `Err(CompressionError)` - If compression fails
///
/// # Example
/// ```
/// use data_guardian::compression::{compress_usage_data_with_config, CompressionConfig};
/// use std::collections::HashMap;
///
/// let mut data = HashMap::new();
/// data.insert("chrome".to_string(), 1024u64);
///
/// let config = CompressionConfig {
///     level: 6,
///     capacity_multiplier: 0.7,
/// };
///
/// let compressed = compress_usage_data_with_config(&data, config).unwrap();
/// ```
pub fn compress_usage_data_with_config(
    data: &HashMap<String, u64>,
    config: CompressionConfig,
) -> Result<Vec<u8>, CompressionError> {
    if config.level > 9 {
        return Err(CompressionError::InvalidLevel(config.level));
    }

    let estimated_capacity = (data.len() as f32 * config.capacity_multiplier) as usize;
    let mut encoder = GzBuilder::new().comment("DataGuardian usage data").write(
        Vec::with_capacity(estimated_capacity.max(64)),
        Compression::new(config.level),
    );

    serde_json::to_writer(&mut encoder, data)?;
    Ok(encoder.finish()?)
}

/// Compress process usage data using gzip with default configuration
///
/// This is a convenience wrapper around `compress_usage_data_with_config`
/// that uses the default compression configuration.
pub fn compress_usage_data(data: &HashMap<String, u64>) -> Result<Vec<u8>, CompressionError> {
    compress_usage_data_with_config(data, CompressionConfig::default())
}

/// Decompress process usage data from gzip format
///
/// # Arguments
/// * `data` - The compressed data to decompress
///
/// # Returns
/// * `Ok(HashMap<String, u64>)` - The decompressed usage data
/// * `Err(CompressionError)` - If decompression fails
pub fn decompress_usage_data(data: &[u8]) -> Result<HashMap<String, u64>, CompressionError> {
    let mut decoder = flate2::read::GzDecoder::new(data);
    let mut decompressed = Vec::with_capacity(data.len() * 2);
    decoder.read_to_end(&mut decompressed)?;
    Ok(serde_json::from_slice(&decompressed)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper function to create test data
    fn create_test_data(size: usize) -> HashMap<String, u64> {
        let mut data = HashMap::with_capacity(size);
        for i in 0..size {
            data.insert(format!("process_{}", i), i as u64);
        }
        data
    }

    #[test]
    fn test_compression_roundtrip() {
        let data = create_test_data(2);
        let compressed = compress_usage_data(&data).unwrap();
        let decompressed = decompress_usage_data(&compressed).unwrap();
        assert_eq!(data, decompressed);
    }

    #[test]
    fn test_compression_efficiency() {
        let data = create_test_data(1000);
        let compressed = compress_usage_data(&data).unwrap();
        let json_size = serde_json::to_vec(&data).unwrap().len();

        assert!(
            compressed.len() < json_size,
            "Compressed size {} should be less than JSON size {}",
            compressed.len(),
            json_size
        );
    }

    #[test]
    fn test_compression_empty_data() {
        let data = HashMap::new();
        let compressed = compress_usage_data(&data).unwrap();
        let decompressed = decompress_usage_data(&compressed).unwrap();
        assert_eq!(data, decompressed);
    }

    #[test]
    fn test_invalid_data() {
        let result = decompress_usage_data(b"invalid data");
        assert!(result.is_err());
    }

    #[test]
    fn test_custom_compression_config() {
        let data = create_test_data(100);

        // Test different compression levels
        for level in 0..=9 {
            let config = CompressionConfig {
                level,
                ..Default::default()
            };
            let compressed = compress_usage_data_with_config(&data, config).unwrap();
            let decompressed = decompress_usage_data(&compressed).unwrap();
            assert_eq!(data, decompressed);
        }

        // Test invalid compression level
        let config = CompressionConfig {
            level: 10,
            ..Default::default()
        };
        let result = compress_usage_data_with_config(&data, config);
        assert!(matches!(result, Err(CompressionError::InvalidLevel(10))));
    }

    #[test]
    fn test_compression_deterministic() {
        let data = create_test_data(100);
        let config = CompressionConfig::default();

        // Multiple compressions of the same data should yield the same result
        let compressed1 = compress_usage_data_with_config(&data, config).unwrap();
        let compressed2 = compress_usage_data_with_config(&data, config).unwrap();
        assert_eq!(compressed1, compressed2);
    }

    #[test]
    fn test_large_data_compression() {
        let data = create_test_data(10000);
        let compressed = compress_usage_data(&data).unwrap();
        let decompressed = decompress_usage_data(&compressed).unwrap();
        assert_eq!(data, decompressed);
    }
}

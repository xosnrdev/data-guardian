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

/// Errors that can occur during compression operations
#[derive(Error, Debug)]
pub enum CompressionError {
    #[error("Failed to serialize data: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("IO error during compression: {0}")]
    Io(#[from] io::Error),
}

/// Compress process usage data using gzip
pub fn compress_usage_data(data: &HashMap<String, u64>) -> Result<Vec<u8>, CompressionError> {
    let mut encoder = GzBuilder::new()
        .comment("DataGuardian usage data")
        .write(Vec::with_capacity(data.len() * 32), Compression::best());

    serde_json::to_writer(&mut encoder, data)?;
    Ok(encoder.finish()?)
}

/// Decompress process usage data from gzip format
pub fn decompress_usage_data(data: &[u8]) -> Result<HashMap<String, u64>, CompressionError> {
    let mut decoder = flate2::read::GzDecoder::new(data);
    let mut decompressed = Vec::with_capacity(data.len() * 2);
    decoder.read_to_end(&mut decompressed)?;
    Ok(serde_json::from_slice(&decompressed)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression_roundtrip() {
        let mut data = HashMap::with_capacity(2);
        data.insert("chrome".to_string(), 1024);
        data.insert("firefox".to_string(), 2048);

        let compressed = compress_usage_data(&data).unwrap();
        let decompressed = decompress_usage_data(&compressed).unwrap();

        assert_eq!(data, decompressed);
    }

    #[test]
    fn test_compression_efficiency() {
        let mut data = HashMap::with_capacity(1000);
        for i in 0..1000 {
            data.insert(format!("process_{}", i), i as u64);
        }

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
}

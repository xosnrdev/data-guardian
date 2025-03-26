use std::collections::HashMap;
use std::io::{self, Read};

use flate2::{Compression, GzBuilder};
use thiserror::Error;

#[derive(Debug, Clone, Copy)]
pub struct CompressionConfig {
    pub level: u32,
    pub capacity_multiplier: f32,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            level: 9,
            capacity_multiplier: 0.5,
        }
    }
}

#[derive(Error, Debug)]
pub enum CompressionError {
    #[error("Failed to serialize data: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("IO error during compression: {0}")]
    Io(#[from] io::Error),
    #[error("Invalid compression level: {0}")]
    InvalidLevel(u32),
}

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

pub fn compress_usage_data(data: &HashMap<String, u64>) -> Result<Vec<u8>, CompressionError> {
    compress_usage_data_with_config(data, CompressionConfig::default())
}

pub fn decompress_usage_data(data: &[u8]) -> Result<HashMap<String, u64>, CompressionError> {
    let mut decoder = flate2::read::GzDecoder::new(data);
    let mut decompressed = Vec::with_capacity(data.len() * 2);
    decoder.read_to_end(&mut decompressed)?;
    Ok(serde_json::from_slice(&decompressed)?)
}

#[cfg(test)]
mod tests {
    use super::*;

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

        for level in 0..=9 {
            let config = CompressionConfig {
                level,
                ..Default::default()
            };
            let compressed = compress_usage_data_with_config(&data, config).unwrap();
            let decompressed = decompress_usage_data(&compressed).unwrap();
            assert_eq!(data, decompressed);
        }

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

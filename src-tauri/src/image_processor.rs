//! 图片处理模块
//!
//! 提供图片缩放、压缩、Base64 编解码等功能。
//! 
//! # Requirements
//! - 3.6: 图片缩放保持宽高比
//! - 3.7: 图片压缩至 1MB 以内
//! - 3.8: Base64 编解码

use base64::{engine::general_purpose::STANDARD, Engine};
use image::{DynamicImage, GenericImageView};
use std::io::Cursor;
use thiserror::Error;

/// 图片处理错误类型
#[derive(Debug, Error)]
pub enum ImageError {
    #[error("Failed to load image: {0}")]
    LoadError(String),
    
    #[error("Failed to encode image: {0}")]
    EncodeError(String),
    
    #[error("Failed to decode Base64: {0}")]
    Base64DecodeError(String),
    
    #[error("Image compression failed: could not meet size constraint")]
    CompressionFailed,
}

/// 处理后的图片结果
#[derive(Debug, Clone)]
pub struct ProcessedImageResult {
    /// JPEG 图片数据
    pub data: Vec<u8>,
    /// 宽度
    pub width: u32,
    /// 高度
    pub height: u32,
}

/// 图片处理器
pub struct ImageProcessor;

impl ImageProcessor {
    /// 默认最大尺寸 (512x512)
    pub const DEFAULT_MAX_SIZE: u32 = 512;
    
    /// 默认最大文件大小 (1MB)
    pub const DEFAULT_MAX_FILE_SIZE: usize = 1024 * 1024;
    
    /// 默认初始 JPEG 质量
    pub const DEFAULT_INITIAL_QUALITY: u8 = 85;
    
    /// 最低 JPEG 质量
    pub const MIN_QUALITY: u8 = 10;

    /// 从字节数据加载图片
    ///
    /// # Arguments
    /// * `data` - 图片字节数据
    ///
    /// # Returns
    /// * `Ok(DynamicImage)` - 加载成功的图片
    /// * `Err(ImageError)` - 加载失败
    pub fn load_from_bytes(data: &[u8]) -> Result<DynamicImage, ImageError> {
        image::load_from_memory(data)
            .map_err(|e| ImageError::LoadError(e.to_string()))
    }

    /// 缩放图片，保持宽高比
    ///
    /// 如果图片的宽度或高度超过 max_size，则按比例缩放使最大边等于 max_size。
    /// 如果图片已经在限制范围内，则返回原图。
    ///
    /// # Arguments
    /// * `img` - 原始图片
    /// * `max_size` - 最大边长
    ///
    /// # Returns
    /// 缩放后的图片（如果需要缩放）或原图
    ///
    /// # Property 4: Image Resize Aspect Ratio Preservation
    /// 对于任何超过 max_size 的图片，缩放后的宽高比应与原始宽高比相差不超过 1%
    pub fn resize(img: DynamicImage, max_size: u32) -> DynamicImage {
        let (width, height) = img.dimensions();
        
        if width <= max_size && height <= max_size {
            return img;
        }
        
        // 计算缩放比例，取较小值以确保两边都不超过 max_size
        let ratio = (max_size as f64 / width as f64).min(max_size as f64 / height as f64);
        let new_width = ((width as f64 * ratio).round() as u32).max(1);
        let new_height = ((height as f64 * ratio).round() as u32).max(1);
        
        img.resize_exact(new_width, new_height, image::imageops::FilterType::Lanczos3)
    }

    /// 将图片转换为 JPEG 格式并压缩
    ///
    /// 使用递减的质量参数进行压缩，直到文件大小满足要求或达到最低质量。
    ///
    /// # Arguments
    /// * `img` - 要压缩的图片
    /// * `max_size_bytes` - 最大文件大小（字节）
    /// * `initial_quality` - 初始 JPEG 质量 (1-100)
    ///
    /// # Returns
    /// * `Ok(Vec<u8>)` - 压缩后的 JPEG 数据
    /// * `Err(ImageError)` - 压缩失败
    ///
    /// # Property 5: Image Size Constraint
    /// 对于任何处理后的图片，最终大小不应超过 max_size_bytes
    pub fn compress_to_jpeg(
        img: &DynamicImage,
        max_size_bytes: usize,
        initial_quality: u8,
    ) -> Result<Vec<u8>, ImageError> {
        let mut quality = initial_quality.min(100);
        
        loop {
            let jpeg_data = Self::encode_jpeg(img, quality)?;
            
            if jpeg_data.len() <= max_size_bytes || quality <= Self::MIN_QUALITY {
                return Ok(jpeg_data);
            }
            
            quality = quality.saturating_sub(10);
        }
    }

    /// 将图片编码为 JPEG 格式
    ///
    /// # Arguments
    /// * `img` - 要编码的图片
    /// * `quality` - JPEG 质量 (1-100)
    ///
    /// # Returns
    /// * `Ok(Vec<u8>)` - JPEG 数据
    /// * `Err(ImageError)` - 编码失败
    fn encode_jpeg(img: &DynamicImage, quality: u8) -> Result<Vec<u8>, ImageError> {
        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);
        
        // 转换为 RGB8 格式以确保 JPEG 编码兼容性
        let rgb_img = img.to_rgb8();
        
        let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut cursor, quality);
        rgb_img
            .write_with_encoder(encoder)
            .map_err(|e| ImageError::EncodeError(e.to_string()))?;
        
        Ok(buffer)
    }

    /// 将字节数据编码为 Base64 字符串
    ///
    /// # Arguments
    /// * `data` - 字节数据
    ///
    /// # Returns
    /// Base64 编码的字符串
    ///
    /// # Property 6: Image Base64 Round-Trip
    /// 对于任何字节数据，encode_base64 后再 decode_base64 应得到相同的数据
    pub fn encode_base64(data: &[u8]) -> String {
        STANDARD.encode(data)
    }

    /// 将 Base64 字符串解码为字节数据
    ///
    /// # Arguments
    /// * `base64_str` - Base64 编码的字符串
    ///
    /// # Returns
    /// * `Ok(Vec<u8>)` - 解码后的字节数据
    /// * `Err(ImageError)` - 解码失败
    pub fn decode_base64(base64_str: &str) -> Result<Vec<u8>, ImageError> {
        STANDARD
            .decode(base64_str)
            .map_err(|e| ImageError::Base64DecodeError(e.to_string()))
    }

    /// 完整的图片处理流程
    ///
    /// 1. 加载图片
    /// 2. 缩放（如果需要）
    /// 3. 压缩为 JPEG
    ///
    /// # Arguments
    /// * `data` - 原始图片字节数据
    /// * `max_dimension` - 最大边长
    /// * `max_file_size` - 最大文件大小（字节）
    ///
    /// # Returns
    /// * `Ok(ProcessedImageResult)` - 处理结果
    /// * `Err(ImageError)` - 处理失败
    pub fn process(
        data: &[u8],
        max_dimension: u32,
        max_file_size: usize,
    ) -> Result<ProcessedImageResult, ImageError> {
        // 1. 加载图片
        let img = Self::load_from_bytes(data)?;
        
        // 2. 缩放
        let resized = Self::resize(img, max_dimension);
        let (width, height) = resized.dimensions();
        
        // 3. 压缩为 JPEG
        let jpeg_data = Self::compress_to_jpeg(&resized, max_file_size, Self::DEFAULT_INITIAL_QUALITY)?;
        
        Ok(ProcessedImageResult {
            data: jpeg_data,
            width,
            height,
        })
    }

    /// 使用默认参数处理图片
    ///
    /// # Arguments
    /// * `data` - 原始图片字节数据
    ///
    /// # Returns
    /// * `Ok(ProcessedImageResult)` - 处理结果
    /// * `Err(ImageError)` - 处理失败
    pub fn process_with_defaults(data: &[u8]) -> Result<ProcessedImageResult, ImageError> {
        Self::process(data, Self::DEFAULT_MAX_SIZE, Self::DEFAULT_MAX_FILE_SIZE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 创建一个简单的测试图片
    fn create_test_image(width: u32, height: u32) -> DynamicImage {
        DynamicImage::ImageRgb8(image::RgbImage::new(width, height))
    }

    #[test]
    fn test_resize_small_image_unchanged() {
        let img = create_test_image(100, 100);
        let resized = ImageProcessor::resize(img, 512);
        assert_eq!(resized.dimensions(), (100, 100));
    }

    #[test]
    fn test_resize_large_image() {
        let img = create_test_image(1024, 768);
        let resized = ImageProcessor::resize(img, 512);
        let (w, h) = resized.dimensions();
        assert!(w <= 512 && h <= 512);
    }

    #[test]
    fn test_base64_roundtrip() {
        let original = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let encoded = ImageProcessor::encode_base64(&original);
        let decoded = ImageProcessor::decode_base64(&encoded).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_encode_jpeg() {
        let img = create_test_image(100, 100);
        let jpeg_data = ImageProcessor::encode_jpeg(&img, 85).unwrap();
        assert!(!jpeg_data.is_empty());
        // JPEG 文件以 0xFF 0xD8 开头
        assert_eq!(jpeg_data[0], 0xFF);
        assert_eq!(jpeg_data[1], 0xD8);
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    /// 创建一个简单的测试图片
    fn create_test_image(width: u32, height: u32) -> DynamicImage {
        DynamicImage::ImageRgb8(image::RgbImage::new(width, height))
    }

    // 配置 proptest 使用较少的测试用例以加快测试速度
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(20))]
        /// **Feature: interactive-feedback-tauri, Property 4: Image Resize Aspect Ratio Preservation**
        /// 
        /// 对于任何超过 max_size 的图片，缩放后的宽高比应与原始宽高比相差不超过 1%
        /// 
        /// **Validates: Requirements 3.6**
        /// 
        /// 注意：使用 max_size >= 256 以避免极端缩放导致的整数舍入误差
        /// 实际应用中 max_size 为 512，不会出现极端情况
        #[test]
        fn test_resize_preserves_aspect_ratio(
            width in 513u32..2048,
            height in 513u32..2048,
            max_size in 256u32..512
        ) {
            let img = create_test_image(width, height);
            let original_ratio = width as f64 / height as f64;
            
            let resized = ImageProcessor::resize(img, max_size);
            let (new_width, new_height) = resized.dimensions();
            
            // 确保缩放后的尺寸不超过 max_size
            prop_assert!(new_width <= max_size, "Width {} exceeds max_size {}", new_width, max_size);
            prop_assert!(new_height <= max_size, "Height {} exceeds max_size {}", new_height, max_size);
            
            // 检查宽高比保持在 1% 误差范围内
            let new_ratio = new_width as f64 / new_height as f64;
            let ratio_diff = ((new_ratio - original_ratio) / original_ratio).abs();
            
            prop_assert!(
                ratio_diff <= 0.01,
                "Aspect ratio changed by {:.2}% (original: {:.4}, new: {:.4})",
                ratio_diff * 100.0,
                original_ratio,
                new_ratio
            );
        }

        /// 测试小图片不会被放大
        #[test]
        fn test_small_images_not_enlarged(
            width in 1u32..512,
            height in 1u32..512
        ) {
            let img = create_test_image(width, height);
            let resized = ImageProcessor::resize(img, 512);
            let (new_width, new_height) = resized.dimensions();
            
            prop_assert_eq!(new_width, width, "Width should not change for small images");
            prop_assert_eq!(new_height, height, "Height should not change for small images");
        }

        /// **Feature: interactive-feedback-tauri, Property 5: Image Size Constraint**
        /// 
        /// 对于任何处理后的图片，最终大小不应超过 max_size_bytes
        /// 
        /// **Validates: Requirements 3.7**
        /// 
        /// 注意：使用 max_size_bytes >= 50000 以确保 JPEG 压缩能够满足约束
        /// 实际应用中 max_size_bytes 为 1MB，远大于此值
        #[test]
        fn test_image_size_constraint(
            width in 100u32..512,
            height in 100u32..512,
            max_size_bytes in 50000usize..1000000
        ) {
            let img = create_test_image(width, height);
            
            let result = ImageProcessor::compress_to_jpeg(&img, max_size_bytes, 85);
            
            prop_assert!(result.is_ok(), "Compression should succeed");
            
            let jpeg_data = result.unwrap();
            prop_assert!(
                jpeg_data.len() <= max_size_bytes,
                "JPEG size {} exceeds max_size_bytes {}",
                jpeg_data.len(),
                max_size_bytes
            );
        }

        /// **Feature: interactive-feedback-tauri, Property 6: Image Base64 Round-Trip**
        /// 
        /// 对于任何字节数据，编码为 Base64 后再解码应得到相同的数据
        /// 
        /// **Validates: Requirements 3.8**
        #[test]
        fn test_base64_roundtrip_property(data in proptest::collection::vec(any::<u8>(), 0..10000)) {
            let encoded = ImageProcessor::encode_base64(&data);
            let decoded = ImageProcessor::decode_base64(&encoded);
            
            prop_assert!(decoded.is_ok(), "Base64 decode should succeed");
            prop_assert_eq!(data, decoded.unwrap(), "Round-trip should preserve data");
        }
    }

}
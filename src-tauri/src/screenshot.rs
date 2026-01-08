//! 截图功能模块
//! 
//! Requirements: 8.1, 8.2, 8.3, 8.4, 8.5
//! 
//! 提供屏幕捕获和区域截图功能

use crate::image_processor::ImageProcessor;
use image::{DynamicImage, ImageEncoder, RgbaImage};
use xcap::Monitor;

/// 截图错误类型
#[derive(Debug, thiserror::Error)]
pub enum ScreenshotError {
    #[error("Failed to get monitors: {0}")]
    MonitorError(String),
    
    #[error("Failed to capture screen: {0}")]
    CaptureError(String),
    
    #[error("Failed to process image: {0}")]
    ProcessError(String),
    
    #[error("Invalid region: {0}")]
    InvalidRegion(String),
}

/// 截图区域
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScreenshotRegion {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// 截图结果
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScreenshotResult {
    pub data: String,      // Base64 编码的图片数据
    pub mime_type: String,
    pub width: u32,
    pub height: u32,
    pub size: usize,
}

/// 显示器信息
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MonitorInfo {
    pub id: u32,
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub is_primary: bool,
}

/// 截图管理器
pub struct ScreenshotManager;

impl ScreenshotManager {
    /// 获取所有显示器信息
    /// 
    /// Requirement 8.2: 支持多显示器
    pub fn get_monitors() -> Result<Vec<MonitorInfo>, ScreenshotError> {
        let monitors = Monitor::all()
            .map_err(|e| ScreenshotError::MonitorError(e.to_string()))?;
        
        let mut result = Vec::new();
        for (i, monitor) in monitors.iter().enumerate() {
            // xcap 0.8 的方法返回 Result，需要处理错误
            let name = monitor.name()
                .map_err(|e| ScreenshotError::MonitorError(e.to_string()))?;
            let x = monitor.x()
                .map_err(|e| ScreenshotError::MonitorError(e.to_string()))?;
            let y = monitor.y()
                .map_err(|e| ScreenshotError::MonitorError(e.to_string()))?;
            let width = monitor.width()
                .map_err(|e| ScreenshotError::MonitorError(e.to_string()))?;
            let height = monitor.height()
                .map_err(|e| ScreenshotError::MonitorError(e.to_string()))?;
            let is_primary = monitor.is_primary()
                .map_err(|e| ScreenshotError::MonitorError(e.to_string()))?;
            
            result.push(MonitorInfo {
                id: i as u32,
                name,
                x,
                y,
                width,
                height,
                is_primary,
            });
        }
        
        Ok(result)
    }
    
    /// 捕获整个屏幕
    /// 
    /// Requirement 8.4: 实现屏幕捕获
    pub fn capture_full_screen(monitor_id: Option<u32>) -> Result<ScreenshotResult, ScreenshotError> {
        let monitors = Monitor::all()
            .map_err(|e| ScreenshotError::MonitorError(e.to_string()))?;
        
        // 选择显示器
        let monitor = if let Some(id) = monitor_id {
            monitors.get(id as usize)
                .ok_or_else(|| ScreenshotError::MonitorError(format!("Monitor {} not found", id)))?
        } else {
            // 默认使用主显示器
            let primary = monitors.iter()
                .find(|m| m.is_primary().unwrap_or(false));
            
            primary.or_else(|| monitors.first())
                .ok_or_else(|| ScreenshotError::MonitorError("No monitors found".to_string()))?
        };
        
        // 捕获屏幕
        let image = monitor.capture_image()
            .map_err(|e| ScreenshotError::CaptureError(e.to_string()))?;
        
        Self::process_captured_image(image)
    }
    
    /// 捕获指定区域
    /// 
    /// Requirement 8.2, 8.3: 矩形选择和实时预览
    pub fn capture_region(region: ScreenshotRegion) -> Result<ScreenshotResult, ScreenshotError> {
        // 验证区域
        if region.width == 0 || region.height == 0 {
            return Err(ScreenshotError::InvalidRegion("Width and height must be greater than 0".to_string()));
        }
        
        let monitors = Monitor::all()
            .map_err(|e| ScreenshotError::MonitorError(e.to_string()))?;
        
        // 找到包含该区域的显示器
        let monitor = monitors.iter()
            .find(|m| {
                let mx = m.x().unwrap_or(0);
                let my = m.y().unwrap_or(0);
                let mw = m.width().unwrap_or(0) as i32;
                let mh = m.height().unwrap_or(0) as i32;
                
                region.x >= mx && region.x < mx + mw &&
                region.y >= my && region.y < my + mh
            })
            .or_else(|| monitors.iter().find(|m| m.is_primary().unwrap_or(false)))
            .or_else(|| monitors.first())
            .ok_or_else(|| ScreenshotError::MonitorError("No monitors found".to_string()))?;
        
        // 捕获整个屏幕
        let full_image = monitor.capture_image()
            .map_err(|e| ScreenshotError::CaptureError(e.to_string()))?;
        
        // 计算相对于显示器的坐标
        let monitor_x = monitor.x().unwrap_or(0);
        let monitor_y = monitor.y().unwrap_or(0);
        let rel_x = (region.x - monitor_x).max(0) as u32;
        let rel_y = (region.y - monitor_y).max(0) as u32;
        
        // 裁剪区域
        let cropped = Self::crop_image(&full_image, rel_x, rel_y, region.width, region.height)?;
        
        Self::process_captured_image(cropped)
    }
    
    /// 裁剪图片
    fn crop_image(
        image: &RgbaImage,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> Result<RgbaImage, ScreenshotError> {
        let img_width = image.width();
        let img_height = image.height();
        
        // 确保裁剪区域在图片范围内
        let actual_x = x.min(img_width.saturating_sub(1));
        let actual_y = y.min(img_height.saturating_sub(1));
        let actual_width = width.min(img_width.saturating_sub(actual_x));
        let actual_height = height.min(img_height.saturating_sub(actual_y));
        
        if actual_width == 0 || actual_height == 0 {
            return Err(ScreenshotError::InvalidRegion("Crop region is outside image bounds".to_string()));
        }
        
        let dynamic_image = DynamicImage::ImageRgba8(image.clone());
        let cropped = dynamic_image.crop_imm(actual_x, actual_y, actual_width, actual_height);
        
        Ok(cropped.to_rgba8())
    }
    
    /// 处理捕获的图片
    fn process_captured_image(image: RgbaImage) -> Result<ScreenshotResult, ScreenshotError> {
        let width = image.width();
        let height = image.height();
        
        // 转换为 PNG 格式
        let mut buffer = Vec::new();
        let encoder = image::codecs::png::PngEncoder::new(&mut buffer);
        encoder.write_image(
            image.as_raw(),
            width,
            height,
            image::ExtendedColorType::Rgba8,
        ).map_err(|e| ScreenshotError::ProcessError(e.to_string()))?;
        
        // Base64 编码
        let base64_data = ImageProcessor::encode_base64(&buffer);
        
        Ok(ScreenshotResult {
            data: base64_data,
            mime_type: "image/png".to_string(),
            width,
            height,
            size: buffer.len(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_get_monitors() {
        // 这个测试在 CI 环境可能会失败，因为没有显示器
        let result = ScreenshotManager::get_monitors();
        // 只验证不会 panic
        let _ = result;
    }
    
    #[test]
    fn test_invalid_region() {
        let region = ScreenshotRegion {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        };
        
        let result = ScreenshotManager::capture_region(region);
        assert!(result.is_err());
    }
}

//! 音频通知模块
//! 
//! 实现音频播放功能，支持自定义音频文件
//! Requirements: 12.1, 12.3
//! 
//! - 12.1: 反馈窗口打开时播放通知音
//! - 12.3: 支持选择自定义音频文件

use rodio::{Decoder, OutputStream, Sink};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, Cursor};
use std::path::Path;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use thiserror::Error;

/// 内置音频文件
const SOUND_NOTIFICATION: &[u8] = include_bytes!("../assets/sounds/notification.wav");
const SOUND_100W: &[u8] = include_bytes!("../assets/sounds/100w.mp3");
const SOUND_GANMA: &[u8] = include_bytes!("../assets/sounds/ganma.mp3");
const SOUND_GAOWAN: &[u8] = include_bytes!("../assets/sounds/gaowan.mp3");
const SOUND_JI: &[u8] = include_bytes!("../assets/sounds/ji.mp3");
const SOUND_DENG: &[u8] = include_bytes!("../assets/sounds/deng.mp3");

/// 内置音频信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuiltinSound {
    pub id: String,
    pub name: String,
    pub description: String,
}

/// 获取内置音频列表
pub fn get_builtin_sounds() -> Vec<BuiltinSound> {
    vec![
        BuiltinSound {
            id: "notification".to_string(),
            name: "默认提示音".to_string(),
            description: "经典提示音".to_string(),
        },
        BuiltinSound {
            id: "100w".to_string(),
            name: "支付宝到账一百万".to_string(),
            description: "土豪专属".to_string(),
        },
        BuiltinSound {
            id: "ganma".to_string(),
            name: "干嘛".to_string(),
            description: "蔡徐坤".to_string(),
        },
        BuiltinSound {
            id: "gaowan".to_string(),
            name: "搞完".to_string(),
            description: "完成提示".to_string(),
        },
        BuiltinSound {
            id: "ji".to_string(),
            name: "鸡".to_string(),
            description: "只因你太美".to_string(),
        },
        BuiltinSound {
            id: "deng".to_string(),
            name: "等".to_string(),
            description: "微信提示音".to_string(),
        },
    ]
}

/// 根据 ID 获取内置音频数据
fn get_builtin_sound_data(id: &str) -> Option<&'static [u8]> {
    match id {
        "notification" => Some(SOUND_NOTIFICATION),
        "100w" => Some(SOUND_100W),
        "ganma" => Some(SOUND_GANMA),
        "gaowan" => Some(SOUND_GAOWAN),
        "ji" => Some(SOUND_JI),
        "deng" => Some(SOUND_DENG),
        _ => None,
    }
}

/// 音频错误类型
#[derive(Error, Debug)]
pub enum AudioError {
    #[error("音频文件未找到: {0}")]
    FileNotFound(String),
    
    #[error("无法打开音频文件: {0}")]
    FileOpenError(String),
    
    #[error("无法解码音频文件: {0}")]
    DecodeError(String),
    
    #[error("无法获取音频输出设备: {0}")]
    OutputDeviceError(String),
    
    #[error("音频播放失败: {0}")]
    PlaybackError(String),
    
    #[error("不支持的音频格式: {0}")]
    UnsupportedFormat(String),
}

/// 音频通知器
/// 
/// 负责播放通知音，支持自定义音频文件
pub struct AudioNotifier;

impl AudioNotifier {
    /// 播放通知音
    /// 
    /// # Arguments
    /// * `sound_path` - 可选的自定义音频文件路径，如果为 None 则使用默认音频
    /// 
    /// # Returns
    /// * `Ok(())` - 播放成功（异步播放，立即返回）
    /// * `Err(AudioError)` - 播放失败
    /// 
    /// # Requirements
    /// - 12.1: WHEN the Feedback_Window opens THEN the Audio_Notifier SHALL play a notification sound
    /// - 12.3: WHEN in the settings page THEN the Config_Manager SHALL allow selecting a custom audio file
    pub fn play_notification(sound_path: Option<&str>) -> Result<(), AudioError> {
        // 验证音频文件是否存在（内置音频或自定义文件）
        if let Some(path) = sound_path {
            if !path.is_empty() {
                // 内置音频不需要检查文件是否存在
                if !path.starts_with("builtin:") && !Path::new(path).exists() {
                    return Err(AudioError::FileNotFound(path.to_string()));
                }
            }
        }
        
        // 克隆路径用于线程
        let path_owned = sound_path.map(|s| s.to_string());
        
        // 使用通道来传递错误（如果需要同步等待）
        let (tx, rx) = mpsc::channel();
        
        // 在新线程中播放音频，避免阻塞主线程
        thread::spawn(move || {
            let result = Self::play_sound_blocking(path_owned.as_deref());
            let _ = tx.send(result);
        });
        
        // 等待一小段时间检查是否有立即错误
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(result) => result,
            Err(_) => {
                // 超时意味着音频正在播放中，这是正常的
                Ok(())
            }
        }
    }
    
    /// 异步播放通知音（不等待完成）
    /// 
    /// 这个方法会立即返回，音频在后台播放
    /// 如果播放失败，错误会被记录到日志
    /// 
    /// # Requirements
    /// - 12.4: IF audio playback fails THEN the Audio_Notifier SHALL silently continue without interrupting the workflow
    pub fn play_notification_async(sound_path: Option<&str>) {
        let path_owned = sound_path.map(|s| s.to_string());
        
        thread::spawn(move || {
            if let Err(e) = Self::play_sound_blocking(path_owned.as_deref()) {
                log::warn!("音频播放失败（静默继续）: {}", e);
            }
        });
    }
    
    /// 阻塞式播放音频
    fn play_sound_blocking(sound_path: Option<&str>) -> Result<(), AudioError> {
        // 获取音频输出流
        let (_stream, stream_handle) = OutputStream::try_default()
            .map_err(|e| AudioError::OutputDeviceError(e.to_string()))?;
        
        // 创建 Sink
        let sink = Sink::try_new(&stream_handle)
            .map_err(|e| AudioError::PlaybackError(e.to_string()))?;
        
        // 根据是否有自定义路径选择音频源
        match sound_path {
            Some(path) if !path.is_empty() => {
                // 检查是否是内置音频 ID（以 "builtin:" 开头）
                if let Some(builtin_id) = path.strip_prefix("builtin:") {
                    Self::play_builtin_sound(&sink, builtin_id)?;
                } else {
                    // 使用自定义音频文件
                    Self::play_custom_sound(&sink, path)?;
                }
            }
            _ => {
                // 使用默认音频
                Self::play_builtin_sound(&sink, "notification")?;
            }
        }
        
        // 等待播放完成
        sink.sleep_until_end();
        
        Ok(())
    }
    
    /// 播放自定义音频文件
    fn play_custom_sound(sink: &Sink, path: &str) -> Result<(), AudioError> {
        let file = File::open(path)
            .map_err(|e| AudioError::FileOpenError(format!("{}: {}", path, e)))?;
        
        let source = Decoder::new(BufReader::new(file))
            .map_err(|e| AudioError::DecodeError(format!("{}: {}", path, e)))?;
        
        sink.append(source);
        
        log::info!("播放自定义音频: {}", path);
        Ok(())
    }
    
    /// 播放内置音频
    fn play_builtin_sound(sink: &Sink, id: &str) -> Result<(), AudioError> {
        let sound_data = get_builtin_sound_data(id)
            .ok_or_else(|| AudioError::FileNotFound(format!("内置音频不存在: {}", id)))?;
        
        let cursor = Cursor::new(sound_data);
        
        let source = Decoder::new(cursor)
            .map_err(|e| AudioError::DecodeError(format!("内置音频 {}: {}", id, e)))?;
        
        sink.append(source);
        
        log::info!("播放内置音频: {}", id);
        Ok(())
    }
    
    /// 验证音频文件是否有效
    /// 
    /// 检查文件是否存在且格式受支持
    /// 支持内置音频（以 "builtin:" 开头）和自定义文件路径
    /// 
    /// # Arguments
    /// * `path` - 音频文件路径或内置音频 ID（如 "builtin:100w"）
    /// 
    /// # Returns
    /// * `Ok(())` - 文件有效
    /// * `Err(AudioError)` - 文件无效
    pub fn validate_audio_file(path: &str) -> Result<(), AudioError> {
        // 检查是否是内置音频
        if let Some(builtin_id) = path.strip_prefix("builtin:") {
            if get_builtin_sound_data(builtin_id).is_some() {
                return Ok(());
            } else {
                return Err(AudioError::FileNotFound(format!("内置音频不存在: {}", builtin_id)));
            }
        }
        
        let path = Path::new(path);
        
        // 检查文件是否存在
        if !path.exists() {
            return Err(AudioError::FileNotFound(path.display().to_string()));
        }
        
        // 检查文件扩展名
        let extension = path.extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase());
        
        let supported_formats = ["wav", "mp3", "ogg", "flac"];
        
        match extension {
            Some(ext) if supported_formats.contains(&ext.as_str()) => {
                // 尝试打开并解码文件以验证格式
                let file = File::open(path)
                    .map_err(|e| AudioError::FileOpenError(e.to_string()))?;
                
                Decoder::new(BufReader::new(file))
                    .map_err(|e| AudioError::DecodeError(e.to_string()))?;
                
                Ok(())
            }
            Some(ext) => Err(AudioError::UnsupportedFormat(ext)),
            None => Err(AudioError::UnsupportedFormat("未知".to_string())),
        }
    }
    
    /// 获取支持的音频格式列表
    pub fn supported_formats() -> Vec<&'static str> {
        vec!["wav", "mp3", "ogg", "flac"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_supported_formats() {
        let formats = AudioNotifier::supported_formats();
        assert!(formats.contains(&"wav"));
        assert!(formats.contains(&"mp3"));
        assert!(formats.contains(&"ogg"));
        assert!(formats.contains(&"flac"));
    }
    
    #[test]
    fn test_validate_nonexistent_file() {
        let result = AudioNotifier::validate_audio_file("/nonexistent/path/audio.wav");
        assert!(matches!(result, Err(AudioError::FileNotFound(_))));
    }
    
    #[test]
    fn test_validate_unsupported_format() {
        // 创建临时文件
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("test_audio.xyz");
        std::fs::write(&temp_file, b"dummy content").unwrap();
        
        let result = AudioNotifier::validate_audio_file(temp_file.to_str().unwrap());
        assert!(matches!(result, Err(AudioError::UnsupportedFormat(_))));
        
        // 清理
        let _ = std::fs::remove_file(temp_file);
    }
}

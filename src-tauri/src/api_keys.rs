//! API 密钥安全存储模块
//! 
//! 使用配置文件存储 + Base64 混淆
//! Requirements: 7.5, 14.5

use base64::{Engine as _, engine::general_purpose::STANDARD};
use thiserror::Error;

/// API 密钥管理错误
#[derive(Error, Debug)]
pub enum ApiKeyError {
    #[error("Invalid provider: {0}")]
    InvalidProvider(String),
    #[error("Encoding error: {0}")]
    EncodingError(String),
}

/// 支持的 AI 提供商
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApiProvider {
    OpenAI,
    Gemini,
    DeepSeek,
    Volcengine,
}

impl ApiProvider {
    /// 从字符串解析提供商
    pub fn from_str(s: &str) -> Result<Self, ApiKeyError> {
        match s.to_lowercase().as_str() {
            "openai" => Ok(Self::OpenAI),
            "gemini" => Ok(Self::Gemini),
            "deepseek" => Ok(Self::DeepSeek),
            "volcengine" => Ok(Self::Volcengine),
            _ => Err(ApiKeyError::InvalidProvider(s.to_string())),
        }
    }

    /// 获取所有支持的提供商
    pub fn all() -> &'static [ApiProvider] {
        &[
            Self::OpenAI,
            Self::Gemini,
            Self::DeepSeek,
            Self::Volcengine,
        ]
    }
    
    /// 获取提供商名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::OpenAI => "openai",
            Self::Gemini => "gemini",
            Self::DeepSeek => "deepseek",
            Self::Volcengine => "volcengine",
        }
    }
}

/// 简单的混淆加密（Base64 + 反转 + 前缀）
fn encode_key(key: &str) -> String {
    let reversed: String = key.chars().rev().collect();
    let encoded = STANDARD.encode(reversed.as_bytes());
    format!("ENC:{}", encoded)
}

/// 解密
fn decode_key(encoded: &str) -> Result<String, ApiKeyError> {
    if !encoded.starts_with("ENC:") {
        // 兼容未加密的旧数据
        return Ok(encoded.to_string());
    }
    
    let data = &encoded[4..];
    let decoded = STANDARD.decode(data)
        .map_err(|e| ApiKeyError::EncodingError(e.to_string()))?;
    let reversed = String::from_utf8(decoded)
        .map_err(|e| ApiKeyError::EncodingError(e.to_string()))?;
    Ok(reversed.chars().rev().collect())
}

/// API 密钥管理器
/// 
/// 使用配置文件存储加密后的 API 密钥
pub struct ApiKeyManager;

impl ApiKeyManager {
    /// 混淆 API 密钥（用于存储）
    pub fn obfuscate(key: &str) -> String {
        if key.is_empty() {
            return String::new();
        }
        encode_key(key)
    }
    
    /// 解混淆 API 密钥（用于使用）
    pub fn deobfuscate(encoded: &str) -> Result<String, ApiKeyError> {
        if encoded.is_empty() {
            return Ok(String::new());
        }
        decode_key(encoded)
    }
    
    /// 获取掩码版本（用于 UI 显示）
    pub fn mask_key(key: &str) -> Option<String> {
        if key.is_empty() {
            return None;
        }
        
        if key.len() > 8 {
            let prefix = &key[..4];
            let suffix = &key[key.len() - 4..];
            Some(format!("{}****...****{}", prefix, suffix))
        } else if !key.is_empty() {
            Some("****".to_string())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_obfuscate_deobfuscate() {
        let key = "sk-test-api-key-12345";
        let obfuscated = ApiKeyManager::obfuscate(key);
        assert!(obfuscated.starts_with("ENC:"));
        
        let deobfuscated = ApiKeyManager::deobfuscate(&obfuscated).unwrap();
        assert_eq!(deobfuscated, key);
    }
    
    #[test]
    fn test_empty_key() {
        let obfuscated = ApiKeyManager::obfuscate("");
        assert_eq!(obfuscated, "");
        
        let deobfuscated = ApiKeyManager::deobfuscate("").unwrap();
        assert_eq!(deobfuscated, "");
    }
    
    #[test]
    fn test_mask_key() {
        let key = "sk-1234567890abcdef";
        let masked = ApiKeyManager::mask_key(key);
        assert_eq!(masked, Some("sk-1****...****cdef".to_string()));
    }
}

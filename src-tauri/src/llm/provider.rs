//! OpenAI 兼容 API Provider
//! 
//! 支持 OpenAI、Gemini、DeepSeek、火山引擎等提供商

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// LLM 配置
#[derive(Debug, Clone)]
pub struct LlmConfig {
    /// API 密钥
    pub api_key: String,
    /// API 基础 URL
    pub base_url: String,
    /// 模型名称
    pub model: String,
    /// 请求超时（秒）
    pub timeout_secs: u64,
}

impl LlmConfig {
    /// 创建 OpenAI 配置
    pub fn openai(api_key: String) -> Self {
        Self {
            api_key,
            base_url: "https://api.openai.com/v1".to_string(),
            model: "gpt-4o-mini".to_string(),
            timeout_secs: 60,
        }
    }

    /// 创建 Gemini 配置
    pub fn gemini(api_key: String) -> Self {
        Self {
            api_key,
            base_url: "https://generativelanguage.googleapis.com/v1beta/openai".to_string(),
            model: "gemini-2.0-flash-lite".to_string(),
            timeout_secs: 60,
        }
    }

    /// 创建 DeepSeek 配置
    pub fn deepseek(api_key: String) -> Self {
        Self {
            api_key,
            base_url: "https://api.deepseek.com/v1".to_string(),
            model: "deepseek-chat".to_string(),
            timeout_secs: 60,
        }
    }

    /// 创建火山引擎配置
    pub fn volcengine(api_key: String) -> Self {
        Self {
            api_key,
            base_url: "https://ark.cn-beijing.volces.com/api/v3".to_string(),
            model: "doubao-seed-1-6-lite-251015".to_string(),
            timeout_secs: 60,
        }
    }

    /// 根据提供商名称创建配置
    pub fn from_provider(provider: &str, api_key: String) -> Option<Self> {
        match provider.to_lowercase().as_str() {
            "openai" => Some(Self::openai(api_key)),
            "gemini" => Some(Self::gemini(api_key)),
            "deepseek" => Some(Self::deepseek(api_key)),
            "volcengine" => Some(Self::volcengine(api_key)),
            _ => None,
        }
    }
}

/// 聊天消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

impl ChatMessage {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".to_string(),
            content: content.into(),
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
        }
    }
}

/// 聊天请求
#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
}

/// 聊天响应
#[derive(Debug, Deserialize)]
pub struct ChatResponse {
    pub id: Option<String>,
    pub choices: Vec<ChatChoice>,
    pub usage: Option<Usage>,
}

#[derive(Debug, Deserialize)]
pub struct ChatChoice {
    pub index: u32,
    pub message: ChatMessage,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// API 错误响应
#[derive(Debug, Deserialize)]
struct ApiError {
    error: ApiErrorDetail,
}

#[derive(Debug, Deserialize)]
struct ApiErrorDetail {
    message: String,
    #[serde(rename = "type")]
    error_type: Option<String>,
    code: Option<String>,
}

/// LLM Provider
pub struct LlmProvider {
    config: LlmConfig,
    client: Client,
}

impl LlmProvider {
    /// 创建新的 Provider
    pub fn new(config: LlmConfig) -> Result<Self, String> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        Ok(Self { config, client })
    }

    /// 发送聊天请求
    pub async fn chat(&self, messages: Vec<ChatMessage>) -> Result<String, String> {
        let url = format!("{}/chat/completions", self.config.base_url);

        let request = ChatRequest {
            model: self.config.model.clone(),
            messages,
            temperature: Some(0.7),
            max_tokens: Some(2048),
        };

        log::info!("[LLM] Sending request to: {}", url);
        log::info!("[LLM] Model: {}, Timeout: {}s", self.config.model, self.config.timeout_secs);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                log::error!("[LLM] Request error: {:?}", e);
                if e.is_timeout() {
                    format!("请求超时({}秒)，请稍后重试", self.config.timeout_secs)
                } else if e.is_connect() {
                    format!("无法连接到 API 服务器: {}", e)
                } else {
                    format!("请求失败: {}", e)
                }
            })?;

        let status = response.status();
        let body = response.text().await.map_err(|e| format!("读取响应失败: {}", e))?;

        if !status.is_success() {
            // 尝试解析错误响应
            if let Ok(api_error) = serde_json::from_str::<ApiError>(&body) {
                return Err(format!("API 错误: {}", api_error.error.message));
            }
            return Err(format!("HTTP 错误 {}: {}", status.as_u16(), body));
        }

        let chat_response: ChatResponse =
            serde_json::from_str(&body).map_err(|e| format!("解析响应失败: {}", e))?;

        chat_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| "API 返回空响应".to_string())
    }

    /// 测试 API 连接
    pub async fn test_connection(&self) -> Result<String, String> {
        let messages = vec![
            ChatMessage::system("你是一个助手。"),
            ChatMessage::user("请回复 OK"),
        ];

        self.chat(messages).await
    }

    /// 优化文本
    pub async fn optimize_text(&self, text: &str, system_prompt: &str) -> Result<String, String> {
        let messages = vec![
            ChatMessage::system(system_prompt),
            ChatMessage::user(text),
        ];

        self.chat(messages).await
    }

    /// 使用完整提示词优化文本（提示词中已包含待处理的文本）
    pub async fn optimize_text_with_prompt(&self, full_prompt: &str) -> Result<String, String> {
        let messages = vec![
            ChatMessage::user(full_prompt),
        ];

        self.chat(messages).await
    }
}

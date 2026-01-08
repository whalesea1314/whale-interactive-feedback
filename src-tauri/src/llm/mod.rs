//! LLM 模块
//! 
//! 提供 OpenAI 兼容 API 的统一接口，支持多个 AI 提供商

mod provider;
mod prompts;

pub use provider::{LlmProvider, LlmConfig, ChatMessage, ChatResponse};
pub use prompts::{get_optimization_prompt, OptimizationType};

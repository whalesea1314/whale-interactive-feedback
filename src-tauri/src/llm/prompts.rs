//! 优化提示词模块 - 仅用于 MCP 工具的回退处理

use serde::{Deserialize, Serialize};

/// MCP 工具优化模式（仅作为回退使用，主要优化类型在 types.rs 的 OptimizationTypeConfig 中配置）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OptimizationType {
    /// 一键优化（MCP 工具使用）
    Optimize,
    /// 自定义增强（MCP 工具使用）
    Reinforce,
}

impl OptimizationType {
    /// 从字符串解析
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "optimize" => Some(Self::Optimize),
            "reinforce" | "enhance" => Some(Self::Reinforce),
            _ => None,
        }
    }
}

/// 获取 MCP 工具的优化提示词
pub fn get_optimization_prompt(opt_type: OptimizationType, custom_prompt: Option<&str>) -> String {
    match opt_type {
        OptimizationType::Optimize => OPTIMIZE_PROMPT.to_string(),
        OptimizationType::Reinforce => {
            if let Some(prompt) = custom_prompt {
                format!("{}\n\n用户自定义指令：{}", REINFORCE_BASE_PROMPT, prompt)
            } else {
                OPTIMIZE_PROMPT.to_string()
            }
        }
    }
}

const OPTIMIZE_PROMPT: &str = r#"你是一个专业的提示词优化助手。请将用户的口语化输入转换为结构化、清晰的指令。

要求：
1. 理解用户的真实意图
2. 消除歧义和模糊表达
3. 使用清晰、具体的语言
4. 保持指令的可执行性
5. 适当补充必要的上下文

请直接输出优化后的文本，不要添加任何解释或说明。"#;

const REINFORCE_BASE_PROMPT: &str = r#"你是一个专业的文本处理助手。请根据用户的自定义指令处理文本。

请直接输出处理后的文本，不要添加任何解释或说明。"#;

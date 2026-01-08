//! MCP (Model Context Protocol) 服务器模块

use std::sync::Arc;
use rmcp::{
    ServerHandler, ServiceExt,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{ServerCapabilities, ServerInfo, ListToolsResult, PaginatedRequestParam, Tool},
    service::RequestContext,
    schemars, tool, tool_router, RoleServer, ErrorData as McpError,
};
use serde::{Deserialize, Serialize};

use crate::popup::{PopupRequest, launch_popup_and_wait, cleanup_request_file};

/// MCP 工具调用参数 - interactive_feedback
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct InteractiveFeedbackParams {
    #[schemars(description = "Summary of the changes or work done by the AI that needs user review")]
    pub message: String,
    
    #[serde(default)]
    #[schemars(description = "Full detailed content (optional, shown in expandable section)")]
    pub full_response: Option<String>,
    
    #[serde(default)]
    #[schemars(description = "List of predefined options for the user to choose from")]
    pub predefined_options: Option<Vec<String>>,
}

/// MCP 工具调用参数 - optimize_user_input
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct OptimizeUserInputParams {
    #[schemars(description = "需要优化的用户输入文本")]
    pub text: String,
    
    #[schemars(description = "优化模式: 'optimize' 进行标准优化, 'enhance' 使用自定义增强指令")]
    pub mode: Option<String>,
    
    #[schemars(description = "自定义增强指令，仅在 mode 为 'enhance' 时使用")]
    pub custom_prompt: Option<String>,
}

/// 优化结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizeResult {
    pub optimized_text: String,
    pub success: bool,
    pub error: Option<String>,
}

/// MCP 服务器
#[derive(Debug, Clone)]
pub struct McpServer {
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl McpServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    /// whale_interactive_feedback 工具 - 启动 GUI 弹窗收集用户反馈
    #[tool(
        name = "whale_interactive_feedback",
        description = "Request interactive feedback from the user. Opens a popup for the user to review AI's work and provide feedback, select options, or attach images."
    )]
    async fn interactive_feedback(
        &self,
        Parameters(params): Parameters<InteractiveFeedbackParams>,
    ) -> String {
        log::info!("interactive_feedback called with message: {}", params.message);
        
        // 创建 popup 请求
        let request = PopupRequest::new(
            Some(params.message.clone()),
            params.full_response.clone(),
            params.predefined_options.clone(),
        );
        let request_id = request.id.clone();
        
        // 启动 GUI 并等待响应
        match launch_popup_and_wait(&request).await {
            Ok(response) => {
                // 清理请求文件
                if let Err(e) = cleanup_request_file(&request_id).await {
                    log::warn!("Failed to cleanup request file: {}", e);
                }
                
                if response.cancelled {
                    return "[User cancelled or provided no feedback]".to_string();
                }
                
                // 格式化结果
                let mut parts = Vec::new();
                
                if !response.selected_options.is_empty() {
                    parts.push(format!("**Selected Options:** {}", response.selected_options.join(", ")));
                }
                
                if let Some(ref feedback) = response.user_input {
                    if !feedback.is_empty() {
                        parts.push(format!("**User Feedback:**\n{}", feedback));
                    }
                }
                
                if !response.images.is_empty() {
                    parts.push(format!("**Attached Images:** {} image(s)", response.images.len()));
                }
                
                if !response.file_references.is_empty() {
                    let file_list: Vec<String> = response.file_references.iter()
                        .map(|f| {
                            let icon = if f.is_directory { "📁" } else { "📄" };
                            format!("{} {}", icon, f.path)
                        })
                        .collect();
                    parts.push(format!("**Attached Files:**\n{}", file_list.join("\n")));
                }
                
                if parts.is_empty() {
                    "No feedback provided by user.".to_string()
                } else {
                    parts.join("\n\n")
                }
            }
            Err(e) => {
                let _ = cleanup_request_file(&request_id).await;
                log::error!("Failed to get feedback: {}", e);
                format!("Error: Failed to get user feedback - {}", e)
            }
        }
    }

    /// whale_optimize_user_input 工具
    #[tool(
        name = "whale_optimize_user_input",
        description = "使用 AI 优化用户输入文本，将口语化输入转换为结构化指令。"
    )]
    async fn optimize_user_input(
        &self,
        Parameters(params): Parameters<OptimizeUserInputParams>,
    ) -> String {
        if params.text.trim().is_empty() {
            return "Error: 'text' 参数不能为空".to_string();
        }
        
        let mode = params.mode.as_deref().unwrap_or("optimize");
        
        if mode != "optimize" && mode != "enhance" {
            return "Error: 'mode' 参数必须是 'optimize' 或 'enhance'".to_string();
        }
        
        if mode == "enhance" && params.custom_prompt.is_none() {
            return "Error: 当 mode 为 'enhance' 时，必须提供 'custom_prompt' 参数".to_string();
        }
        
        log::info!("optimize_user_input 工具被调用，模式: {}", mode);
        
        // 直接从配置文件加载配置
        let config = match crate::config::load_config_direct().await {
            Ok(c) => c,
            Err(e) => return format!("Error: 加载配置失败: {}", e),
        };
        
        // 获取已配置的提供商和 API 密钥
        let (provider_name, obfuscated_key) = if let Some(ref key) = config.api_keys.openai {
            if !key.is_empty() { ("openai", key.clone()) } else { ("", String::new()) }
        } else if let Some(ref key) = config.api_keys.gemini {
            if !key.is_empty() { ("gemini", key.clone()) } else { ("", String::new()) }
        } else if let Some(ref key) = config.api_keys.deepseek {
            if !key.is_empty() { ("deepseek", key.clone()) } else { ("", String::new()) }
        } else if let Some(ref key) = config.api_keys.volcengine {
            if !key.is_empty() { ("volcengine", key.clone()) } else { ("", String::new()) }
        } else {
            return "Error: 未配置任何 API 密钥，请先在设置中配置".to_string();
        };
        
        if provider_name.is_empty() {
            return "Error: 未配置任何 API 密钥，请先在设置中配置".to_string();
        }
        
        // 解混淆 API 密钥
        let api_key = match crate::api_keys::ApiKeyManager::deobfuscate(&obfuscated_key) {
            Ok(key) => key,
            Err(e) => return format!("Error: 解密 API 密钥失败: {}", e),
        };
        
        // 创建 LLM 配置
        let config = match crate::llm::LlmConfig::from_provider(provider_name, api_key) {
            Some(c) => c,
            None => return format!("Error: 不支持的提供商: {}", provider_name),
        };
        
        // 创建 Provider
        let llm = match crate::llm::LlmProvider::new(config) {
            Ok(l) => l,
            Err(e) => return format!("Error: 创建 LLM Provider 失败: {}", e),
        };
        
        // 获取优化类型
        let opt_type = if mode == "enhance" {
            crate::llm::OptimizationType::Reinforce
        } else {
            crate::llm::OptimizationType::Optimize
        };
        
        // 获取提示词
        let system_prompt = crate::llm::get_optimization_prompt(opt_type, params.custom_prompt.as_deref());
        
        // 调用 LLM
        match llm.optimize_text(&params.text, &system_prompt).await {
            Ok(result) => result,
            Err(e) => format!("Error: 优化失败: {}", e),
        }
    }
}

impl Default for McpServer {
    fn default() -> Self {
        Self::new()
    }
}

/// 移除 JSON Schema 中的 $schema 字段，解决 Kiro 不支持 draft/2020-12 的问题
fn remove_schema_field(tool: &Tool) -> Tool {
    let mut new_schema = tool.input_schema.as_ref().clone();
    new_schema.remove("$schema");
    
    Tool {
        name: tool.name.clone(),
        title: tool.title.clone(),
        description: tool.description.clone(),
        input_schema: Arc::new(new_schema),
        output_schema: tool.output_schema.clone(),
        annotations: tool.annotations.clone(),
        icons: tool.icons.clone(),
        meta: tool.meta.clone(),
    }
}

impl ServerHandler for McpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "Whale Interactive Feedback MCP 服务器 - 通过 GUI 弹窗收集用户反馈".into()
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
    
    fn list_tools(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<ListToolsResult, McpError>> + Send + '_ {
        async move {
            // 获取原始工具列表
            let tools = self.tool_router.list_all();
            
            // 移除每个工具 schema 中的 $schema 字段
            let fixed_tools: Vec<Tool> = tools.iter().map(remove_schema_field).collect();
            
            Ok(ListToolsResult {
                tools: fixed_tools,
                next_cursor: None,
                meta: Default::default(),
            })
        }
    }
    
    fn call_tool(
        &self,
        request: rmcp::model::CallToolRequestParam,
        context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<rmcp::model::CallToolResult, McpError>> + Send + '_ {
        use rmcp::handler::server::tool::ToolCallContext;
        let tool_context = ToolCallContext::new(self, request, context);
        self.tool_router.call(tool_context)
    }
}

/// 运行 MCP 服务器
pub async fn run_mcp_server() -> anyhow::Result<()> {
    log::info!("启动 MCP 服务器...");
    
    let server = McpServer::new();
    let transport = rmcp::transport::io::stdio();
    let server_handle = server.serve(transport).await?;
    
    log::info!("MCP 服务器已启动，等待连接...");
    
    server_handle.waiting().await?;
    
    log::info!("MCP 服务器已关闭");
    Ok(())
}

// 保留旧的导出以兼容
pub use crate::popup::PopupResponse;

/// 验证 interactive_feedback 参数
pub fn validate_interactive_feedback_params(params: &InteractiveFeedbackParams) -> Result<(), String> {
    if params.message.trim().is_empty() {
        return Err("'message' 参数不能为空".to_string());
    }
    
    if let Some(ref options) = params.predefined_options {
        if options.iter().any(|opt| opt.trim().is_empty()) {
            return Err("predefined_options 中不能包含空字符串".to_string());
        }
    }
    
    Ok(())
}

/// 验证 optimize_user_input 参数
pub fn validate_optimize_user_input_params(params: &OptimizeUserInputParams) -> Result<(), String> {
    if params.text.trim().is_empty() {
        return Err("'text' 参数不能为空".to_string());
    }
    
    if let Some(ref mode) = params.mode {
        if mode != "optimize" && mode != "enhance" {
            return Err("'mode' 参数必须是 'optimize' 或 'enhance'".to_string());
        }
        
        if mode == "enhance" && params.custom_prompt.is_none() {
            return Err("当 mode 为 'enhance' 时，必须提供 'custom_prompt' 参数".to_string());
        }
    }
    
    Ok(())
}

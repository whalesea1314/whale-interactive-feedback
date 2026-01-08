//! Popup window creation and management for MCP interactions
//!
//! This module handles creating and managing the popup window for MCP interactions.
//! 
//! ## 工作原理
//! 1. MCP Server 调用 `launch_popup_and_wait()` 启动 GUI 子进程
//! 2. 使用同步阻塞方式等待子进程结束（类似 Python subprocess.run()）
//! 3. GUI 进程写入响应文件后退出
//! 4. MCP Server 读取响应文件并返回结果
//!
//! ## 休眠处理
//! 同步阻塞方式在休眠时进程被挂起，恢复后继续等待，更简单可靠

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;
use uuid::Uuid;

/// MCP request file prefix
pub const MCP_REQUEST_FILE_PREFIX: &str = "whale_mcp_request_";
/// MCP response file prefix  
pub const MCP_RESPONSE_FILE_PREFIX: &str = "whale_mcp_response_";

/// Popup request sent to the GUI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopupRequest {
    pub id: String,
    pub message: Option<String>,
    pub full_response: Option<String>,
    pub predefined_options: Option<Vec<String>>,
    pub created_at: String,
}

impl PopupRequest {
    pub fn new(message: Option<String>, full_response: Option<String>, predefined_options: Option<Vec<String>>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            message,
            full_response,
            predefined_options,
            created_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}

/// Response from the popup GUI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopupResponse {
    pub request_id: String,
    pub user_input: Option<String>,
    pub selected_options: Vec<String>,
    pub images: Vec<ImageData>,
    #[serde(default)]
    pub file_references: Vec<FileReferenceData>,
    pub cancelled: bool,
}

/// Image data in response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageData {
    pub data: String,  // base64 encoded
    pub mime_type: String,
}

/// File reference data in response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileReferenceData {
    pub display_name: String,
    pub path: String,
    pub is_directory: bool,
}

/// Create a temporary request file for the popup
pub async fn create_request_file(request: &PopupRequest) -> Result<PathBuf> {
    let temp_dir = std::env::temp_dir();
    let file_name = format!("{}{}.json", MCP_REQUEST_FILE_PREFIX, request.id);
    let file_path = temp_dir.join(file_name);
    
    let content = serde_json::to_string_pretty(request)?;
    tokio::fs::write(&file_path, content).await?;
    
    log::info!("Created MCP request file: {:?}", file_path);
    Ok(file_path)
}

/// Get the response file path for a request
pub fn get_response_file_path(request_id: &str) -> PathBuf {
    let temp_dir = std::env::temp_dir();
    let file_name = format!("{}{}.json", MCP_RESPONSE_FILE_PREFIX, request_id);
    temp_dir.join(file_name)
}

/// Find the UI executable path
/// 优先查找与 mcp-server 同目录的 app，支持环境变量 WHALE_UI_PATH 覆盖
pub fn find_ui_executable() -> Result<PathBuf> {
    let current_exe = std::env::current_exe()?;
    let exe_dir = current_exe.parent().ok_or_else(|| anyhow!("Cannot get exe directory"))?;
    
    log::info!("[find_ui_executable] 当前可执行文件: {:?}", current_exe);
    log::info!("[find_ui_executable] 可执行文件目录: {:?}", exe_dir);
    
    // 1. 首先检查环境变量（用于自定义路径）
    if let Ok(env_path) = std::env::var("WHALE_UI_PATH") {
        let path = PathBuf::from(&env_path);
        if path.exists() {
            log::info!("[find_ui_executable] 使用环境变量路径: {:?}", path);
            return Ok(path);
        } else {
            log::warn!("[find_ui_executable] 环境变量 WHALE_UI_PATH 指定的路径不存在: {:?}", path);
        }
    }
    
    // 2. 优先查找同目录的 app（最常见场景）
    let same_dir_app = exe_dir.join("app");
    if same_dir_app.exists() {
        log::info!("[find_ui_executable] 找到同目录 app: {:?}", same_dir_app);
        return Ok(same_dir_app);
    }
    
    // 3. 收集其他候选路径
    let mut candidates: Vec<PathBuf> = Vec::new();
    
    #[cfg(target_os = "macos")]
    {
        // macOS 候选路径
        candidates.push(exe_dir.join("WhaleInteractiveFeedback.app/Contents/MacOS/WhaleInteractiveFeedback"));
        candidates.push(exe_dir.join("whale-interactive-feedback"));
        // 开发模式路径
        candidates.push(exe_dir.join("../../../target/release/app"));
        candidates.push(exe_dir.join("../../../target/debug/app"));
    }
    
    #[cfg(target_os = "windows")]
    {
        candidates.push(exe_dir.join("WhaleInteractiveFeedback.exe"));
        candidates.push(exe_dir.join("app.exe"));
        candidates.push(exe_dir.join("whale-interactive-feedback.exe"));
    }
    
    #[cfg(target_os = "linux")]
    {
        candidates.push(exe_dir.join("whale-interactive-feedback"));
        candidates.push(exe_dir.join("app"));
    }
    
    // 检查每个候选路径
    for candidate in &candidates {
        let exists = candidate.exists();
        log::debug!("[find_ui_executable] 检查路径: {:?} -> {}", candidate, if exists { "存在" } else { "不存在" });
        if exists {
            log::info!("[find_ui_executable] 找到 UI 可执行文件: {:?}", candidate);
            return Ok(candidate.clone());
        }
    }
    
    // 记录所有尝试的路径
    log::error!("[find_ui_executable] 未找到 UI 可执行文件，尝试过的路径:");
    for candidate in &candidates {
        log::error!("  - {:?}", candidate);
    }
    
    Err(anyhow!("UI executable not found. Set WHALE_UI_PATH environment variable or ensure the app is built."))
}

/// Launch the popup UI with an MCP request
pub async fn launch_popup(request: &PopupRequest) -> Result<tokio::process::Child> {
    let request_file = create_request_file(request).await?;
    let ui_exe = find_ui_executable()?;
    
    log::info!("[launch_popup] ========================================");
    log::info!("[launch_popup] 启动 GUI 弹窗");
    log::info!("[launch_popup] 请求 ID: {}", request.id);
    log::info!("[launch_popup] 请求文件: {:?}", request_file);
    log::info!("[launch_popup] UI 可执行文件: {:?}", ui_exe);
    log::info!("[launch_popup] 消息: {:?}", request.message.as_ref().map(|s| {
        // 安全截断，避免在中文字符边界处 panic
        if s.chars().count() > 50 {
            format!("{}...", s.chars().take(50).collect::<String>())
        } else {
            s.clone()
        }
    }));
    log::info!("[launch_popup] 预定义选项: {:?}", request.predefined_options);
    log::info!("[launch_popup] ========================================");
    
    let mut command = tokio::process::Command::new(&ui_exe);
    command
        .arg("--mcp-request")
        .arg(&request_file)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())  // 捕获输出用于调试
        .stderr(std::process::Stdio::piped());
    
    log::info!("[launch_popup] 执行命令: {:?} --mcp-request {:?}", ui_exe, request_file);
    
    match command.spawn() {
        Ok(child) => {
            log::info!("[launch_popup] GUI 进程已启动，PID: {:?}", child.id());
            Ok(child)
        }
        Err(e) => {
            log::error!("[launch_popup] 启动 GUI 进程失败: {}", e);
            Err(anyhow!("Failed to launch UI: {}", e))
        }
    }
}

/// 读取响应文件
async fn read_response_file(request_id: &str) -> Result<PopupResponse> {
    let response_path = get_response_file_path(request_id);
    
    // 等待文件写入完成（短暂延迟）
    tokio::time::sleep(Duration::from_millis(50)).await;
    
    log::info!("[read_response_file] 读取响应文件: {:?}", response_path);
    
    let content = tokio::fs::read_to_string(&response_path).await
        .map_err(|e| anyhow!("Failed to read response file: {}", e))?;
    
    log::debug!("[read_response_file] 响应内容: {}", content);
    
    let response: PopupResponse = serde_json::from_str(&content)
        .map_err(|e| anyhow!("Failed to parse response JSON: {}", e))?;
    
    // Clean up response file
    if let Err(e) = tokio::fs::remove_file(&response_path).await {
        log::warn!("[read_response_file] 删除响应文件失败: {}", e);
    }
    
    log::info!("[read_response_file] 成功读取响应，request_id: {}", response.request_id);
    Ok(response)
}

/// Launch popup and wait for user response
/// 使用同步阻塞方式等待子进程，类似 Python 的 subprocess.run()
/// 这种方式更简单可靠，休眠恢复后能正常继续
pub async fn launch_popup_and_wait(request: &PopupRequest) -> Result<PopupResponse> {
    let request_id = request.id.clone();
    let response_path = get_response_file_path(&request_id);
    
    log::info!("[launch_popup_and_wait] ========================================");
    log::info!("[launch_popup_and_wait] 开始处理 MCP 请求: {}", request_id);
    log::info!("[launch_popup_and_wait] 响应文件路径: {:?}", response_path);
    
    let start_time = std::time::Instant::now();
    
    // 启动 GUI 进程
    let mut child = launch_popup(request).await?;
    
    log::info!("[launch_popup_and_wait] 等待用户响应（同步阻塞模式）...");
    
    // 同步阻塞等待子进程结束
    // 这种方式类似 Python 的 subprocess.run()，更简单可靠
    // 休眠时进程被挂起，恢复后继续等待
    let exit_status = child.wait().await
        .map_err(|e| anyhow!("等待 GUI 进程失败: {}", e))?;
    
    log::info!("[launch_popup_and_wait] GUI 进程退出，状态: {:?}, 耗时: {:?}", 
              exit_status, start_time.elapsed());
    
    // 等待文件写入完成
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // 检查响应文件
    if response_path.exists() {
        read_response_file(&request_id).await
    } else {
        log::warn!("[launch_popup_and_wait] 进程退出但未找到响应文件，返回取消状态");
        Ok(PopupResponse {
            request_id: request_id.to_string(),
            user_input: None,
            selected_options: vec![],
            images: vec![],
            file_references: vec![],
            cancelled: true,
        })
    }
}

/// Clean up request file after response
pub async fn cleanup_request_file(request_id: &str) -> Result<()> {
    let temp_dir = std::env::temp_dir();
    let file_name = format!("{}{}.json", MCP_REQUEST_FILE_PREFIX, request_id);
    let file_path = temp_dir.join(file_name);
    
    if file_path.exists() {
        tokio::fs::remove_file(&file_path).await?;
        log::info!("[cleanup_request_file] 已清理请求文件: {:?}", file_path);
    } else {
        log::debug!("[cleanup_request_file] 请求文件不存在，无需清理: {:?}", file_path);
    }
    
    Ok(())
}

/// 检查 GUI 是否可用（用于诊断）
pub fn check_ui_availability() -> Result<PathBuf> {
    let exe_path = find_ui_executable()?;
    log::info!("[check_ui_availability] UI 可执行文件可用: {:?}", exe_path);
    Ok(exe_path)
}

use crate::api_keys::{ApiKeyManager, ApiProvider};
use crate::config;
use crate::types::{AppConfig, CannedResponse, FeedbackData, ProcessedImage};
use tauri::{AppHandle, Manager};

/// 获取配置
#[tauri::command]
pub async fn get_config(app_handle: AppHandle) -> Result<AppConfig, String> {
    config::load_config(&app_handle)
        .await
        .map_err(|e| e.to_string())
}

/// 保存配置
#[tauri::command]
pub async fn save_config(app_handle: AppHandle, config: AppConfig) -> Result<(), String> {
    config::save_config(&app_handle, &config)
        .await
        .map_err(|e| e.to_string())
}

/// 提交反馈
/// 
/// 接收前端提交的反馈数据，进行序列化处理并返回结构化的 JSON 响应。
/// 该命令用于将用户反馈数据传递给 MCP 服务器。
/// 
/// # Arguments
/// * `feedback` - 反馈数据结构，包含文本、图片和文件引用
/// 
/// # Returns
/// * `Ok(String)` - 序列化后的 JSON 字符串
/// * `Err(String)` - 错误信息
#[tauri::command]
pub async fn submit_feedback(feedback: FeedbackData) -> Result<String, String> {
    log::info!("Received feedback with {} content items", feedback.content.len());
    
    // 验证反馈数据
    if feedback.content.is_empty() {
        log::warn!("Empty feedback submitted");
        // 返回默认的无反馈消息
        let empty_feedback = FeedbackData {
            content: vec![crate::types::FeedbackContent::Text {
                text: "[User provided no feedback]".to_string(),
            }],
        };
        return serde_json::to_string(&empty_feedback)
            .map_err(|e| format!("Failed to serialize empty feedback: {}", e));
    }
    
    // 记录反馈内容摘要
    for (i, content) in feedback.content.iter().enumerate() {
        match content {
            crate::types::FeedbackContent::Text { text } => {
                log::debug!("Content[{}]: Text ({} chars)", i, text.len());
            }
            crate::types::FeedbackContent::Image { mime_type, data } => {
                log::debug!("Content[{}]: Image ({}, {} bytes base64)", i, mime_type, data.len());
            }
            crate::types::FeedbackContent::FileReference { display_name, path } => {
                log::debug!("Content[{}]: FileReference ({} -> {})", i, display_name, path);
            }
        }
    }
    
    // 序列化反馈数据为 JSON
    let json_response = serde_json::to_string(&feedback)
        .map_err(|e| format!("Failed to serialize feedback: {}", e))?;
    
    log::info!("Feedback serialized successfully ({} bytes)", json_response.len());
    
    Ok(json_response)
}

/// 处理图片
#[tauri::command]
pub async fn process_image(image_data: Vec<u8>) -> Result<ProcessedImage, String> {
    use crate::image_processor::ImageProcessor;

    // 使用 ImageProcessor 处理图片
    let result = ImageProcessor::process_with_defaults(&image_data)
        .map_err(|e| e.to_string())?;

    // Base64 编码
    let base64_data = ImageProcessor::encode_base64(&result.data);

    Ok(ProcessedImage {
        data: base64_data,
        mime_type: "image/jpeg".to_string(),
        width: result.width,
        height: result.height,
        size: result.data.len(),
    })
}

/// 播放通知音
/// 
/// Requirements: 12.1, 12.3
/// - 12.1: WHEN the Feedback_Window opens THEN the Audio_Notifier SHALL play a notification sound
/// - 12.3: WHEN in the settings page THEN the Config_Manager SHALL allow selecting a custom audio file
/// - 12.4: IF audio playback fails THEN the Audio_Notifier SHALL silently continue without interrupting the workflow
/// 
/// # Arguments
/// * `sound_path` - 可选的自定义音频文件路径，如果为 None 则使用默认音频
/// 
/// # Returns
/// * `Ok(())` - 播放成功（异步播放，立即返回）
/// * `Err(String)` - 播放失败（仅在严重错误时返回）
#[tauri::command]
pub async fn play_notification_sound(sound_path: Option<String>) -> Result<(), String> {
    use crate::audio::AudioNotifier;
    
    log::info!("播放通知音: {:?}", sound_path);
    
    // 使用异步播放，不阻塞主线程
    // 如果播放失败，会静默继续（Requirement 12.4）
    AudioNotifier::play_notification_async(sound_path.as_deref());
    
    Ok(())
}

/// 验证音频文件
/// 
/// 检查音频文件是否存在且格式受支持
/// 
/// # Arguments
/// * `path` - 音频文件路径
/// 
/// # Returns
/// * `Ok(())` - 文件有效
/// * `Err(String)` - 文件无效，包含错误信息
#[tauri::command]
pub async fn validate_audio_file(path: String) -> Result<(), String> {
    use crate::audio::AudioNotifier;
    
    AudioNotifier::validate_audio_file(&path)
        .map_err(|e| e.to_string())
}

/// 获取支持的音频格式
/// 
/// # Returns
/// * 支持的音频格式扩展名列表
#[tauri::command]
pub fn get_supported_audio_formats() -> Vec<String> {
    use crate::audio::AudioNotifier;
    
    AudioNotifier::supported_formats()
        .into_iter()
        .map(|s| s.to_string())
        .collect()
}

/// 获取内置音频列表
/// 
/// # Returns
/// * 内置音频信息列表
#[tauri::command]
pub fn get_builtin_sounds() -> Vec<crate::audio::BuiltinSound> {
    crate::audio::get_builtin_sounds()
}

/// 获取常用语列表
#[tauri::command]
pub async fn get_canned_responses(app_handle: AppHandle) -> Result<Vec<CannedResponse>, String> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|_| "Failed to get app data directory")?;
    
    let path = app_data_dir.join("canned_responses.json");
    
    if path.exists() {
        let content = tokio::fs::read_to_string(&path)
            .await
            .map_err(|e| e.to_string())?;
        let responses: Vec<CannedResponse> = serde_json::from_str(&content)
            .map_err(|e| e.to_string())?;
        Ok(responses)
    } else {
        Ok(Vec::new())
    }
}

/// 保存常用语列表
#[tauri::command]
pub async fn save_canned_responses(
    app_handle: AppHandle,
    responses: Vec<CannedResponse>,
) -> Result<(), String> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|_| "Failed to get app data directory")?;
    
    // 确保目录存在
    tokio::fs::create_dir_all(&app_data_dir)
        .await
        .map_err(|e| e.to_string())?;
    
    let path = app_data_dir.join("canned_responses.json");
    let json = serde_json::to_string_pretty(&responses)
        .map_err(|e| e.to_string())?;
    
    tokio::fs::write(&path, json)
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(())
}


// ============================================================================
// API 密钥管理命令
// Requirements: 7.5, 14.5
// 使用配置文件存储 + 混淆加密
// ============================================================================

/// 保存 API 密钥到配置文件
/// 
/// # Arguments
/// * `app_handle` - Tauri 应用句柄
/// * `provider` - AI 提供商名称 (openai, gemini, deepseek, volcengine)
/// * `api_key` - API 密钥
/// 
/// # Returns
/// * `Ok(())` - 保存成功
/// * `Err(String)` - 错误信息
#[tauri::command]
pub async fn save_api_key(app_handle: AppHandle, provider: String, api_key: String) -> Result<(), String> {
    let provider_enum = ApiProvider::from_str(&provider)
        .map_err(|e| e.to_string())?;
    
    // 混淆 API 密钥
    let obfuscated = ApiKeyManager::obfuscate(&api_key);
    
    // 加载当前配置
    let mut current_config = config::load_config(&app_handle).await
        .map_err(|e| e.to_string())?;
    
    // 更新对应的 API 密钥
    match provider_enum {
        ApiProvider::OpenAI => current_config.api_keys.openai = Some(obfuscated),
        ApiProvider::Gemini => current_config.api_keys.gemini = Some(obfuscated),
        ApiProvider::DeepSeek => current_config.api_keys.deepseek = Some(obfuscated),
        ApiProvider::Volcengine => current_config.api_keys.volcengine = Some(obfuscated),
    }
    
    // 保存配置
    config::save_config(&app_handle, &current_config).await
        .map_err(|e| e.to_string())?;
    
    log::info!("Saved API key for provider: {}", provider);
    Ok(())
}

/// 获取 API 密钥
/// 
/// # Arguments
/// * `app_handle` - Tauri 应用句柄
/// * `provider` - AI 提供商名称
/// 
/// # Returns
/// * `Ok(Some(key))` - 找到密钥（已解密）
/// * `Ok(None)` - 未找到密钥
/// * `Err(String)` - 错误信息
#[tauri::command]
pub async fn get_api_key(app_handle: AppHandle, provider: String) -> Result<Option<String>, String> {
    let provider_enum = ApiProvider::from_str(&provider)
        .map_err(|e| e.to_string())?;
    
    let current_config = config::load_config(&app_handle).await
        .map_err(|e| e.to_string())?;
    
    let obfuscated = match provider_enum {
        ApiProvider::OpenAI => current_config.api_keys.openai,
        ApiProvider::Gemini => current_config.api_keys.gemini,
        ApiProvider::DeepSeek => current_config.api_keys.deepseek,
        ApiProvider::Volcengine => current_config.api_keys.volcengine,
    };
    
    match obfuscated {
        Some(ref s) if !s.is_empty() => {
            let key = ApiKeyManager::deobfuscate(s)
                .map_err(|e| e.to_string())?;
            Ok(Some(key))
        }
        _ => Ok(None),
    }
}

/// 删除 API 密钥
/// 
/// # Arguments
/// * `app_handle` - Tauri 应用句柄
/// * `provider` - AI 提供商名称
/// 
/// # Returns
/// * `Ok(())` - 删除成功
/// * `Err(String)` - 错误信息
#[tauri::command]
pub async fn delete_api_key(app_handle: AppHandle, provider: String) -> Result<(), String> {
    let provider_enum = ApiProvider::from_str(&provider)
        .map_err(|e| e.to_string())?;
    
    let mut current_config = config::load_config(&app_handle).await
        .map_err(|e| e.to_string())?;
    
    match provider_enum {
        ApiProvider::OpenAI => current_config.api_keys.openai = None,
        ApiProvider::Gemini => current_config.api_keys.gemini = None,
        ApiProvider::DeepSeek => current_config.api_keys.deepseek = None,
        ApiProvider::Volcengine => current_config.api_keys.volcengine = None,
    }
    
    config::save_config(&app_handle, &current_config).await
        .map_err(|e| e.to_string())?;
    
    log::info!("Deleted API key for provider: {}", provider);
    Ok(())
}

/// 检查是否存在 API 密钥
/// 
/// # Arguments
/// * `app_handle` - Tauri 应用句柄
/// * `provider` - AI 提供商名称
/// 
/// # Returns
/// * `Ok(true)` - 密钥存在
/// * `Ok(false)` - 密钥不存在
#[tauri::command]
pub async fn has_api_key(app_handle: AppHandle, provider: String) -> Result<bool, String> {
    let key = get_api_key(app_handle, provider).await?;
    Ok(key.is_some())
}

/// 获取已配置的提供商列表
/// 
/// # Arguments
/// * `app_handle` - Tauri 应用句柄
/// 
/// # Returns
/// * 已配置 API 密钥的提供商名称列表
#[tauri::command]
pub async fn get_configured_providers(app_handle: AppHandle) -> Vec<String> {
    let config = match config::load_config(&app_handle).await {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    
    let mut providers = Vec::new();
    
    if config.api_keys.openai.as_ref().map(|s| !s.is_empty()).unwrap_or(false) {
        providers.push("openai".to_string());
    }
    if config.api_keys.gemini.as_ref().map(|s| !s.is_empty()).unwrap_or(false) {
        providers.push("gemini".to_string());
    }
    if config.api_keys.deepseek.as_ref().map(|s| !s.is_empty()).unwrap_or(false) {
        providers.push("deepseek".to_string());
    }
    if config.api_keys.volcengine.as_ref().map(|s| !s.is_empty()).unwrap_or(false) {
        providers.push("volcengine".to_string());
    }
    
    providers
}

/// 获取 API 密钥的掩码版本（用于 UI 显示）
/// 
/// # Arguments
/// * `app_handle` - Tauri 应用句柄
/// * `provider` - AI 提供商名称
/// 
/// # Returns
/// * 掩码后的密钥字符串，如 "sk-****...****"
#[tauri::command]
pub async fn get_masked_api_key(app_handle: AppHandle, provider: String) -> Result<Option<String>, String> {
    let key = get_api_key(app_handle, provider).await?;
    Ok(key.and_then(|k| ApiKeyManager::mask_key(&k)))
}

/// 设置 API 提供商优先级顺序
/// 
/// # Arguments
/// * `app_handle` - Tauri 应用句柄
/// * `order` - 提供商 ID 列表，按优先级排序（第一个优先级最高）
#[tauri::command]
pub async fn set_provider_order(app_handle: AppHandle, order: Vec<String>) -> Result<(), String> {
    let mut current_config = config::load_config(&app_handle).await
        .map_err(|e| e.to_string())?;
    
    current_config.provider_order = order.clone();
    
    config::save_config(&app_handle, &current_config).await
        .map_err(|e| e.to_string())?;
    
    log::info!("Set provider order: {:?}", order);
    Ok(())
}

/// 获取 API 提供商优先级顺序
/// 
/// # Arguments
/// * `app_handle` - Tauri 应用句柄
/// 
/// # Returns
/// * 提供商 ID 列表，按优先级排序
#[tauri::command]
pub async fn get_provider_order(app_handle: AppHandle) -> Result<Vec<String>, String> {
    let current_config = config::load_config(&app_handle).await
        .map_err(|e| e.to_string())?;
    
    Ok(current_config.provider_order)
}

/// 设置 API 测试状态
/// 
/// # Arguments
/// * `app_handle` - Tauri 应用句柄
/// * `provider` - AI 提供商名称
/// * `tested` - 是否测试通过
#[tauri::command]
pub async fn set_api_test_status(app_handle: AppHandle, provider: String, tested: bool) -> Result<(), String> {
    let provider_enum = ApiProvider::from_str(&provider)
        .map_err(|e| e.to_string())?;
    
    let mut current_config = config::load_config(&app_handle).await
        .map_err(|e| e.to_string())?;
    
    match provider_enum {
        ApiProvider::OpenAI => current_config.api_test_status.openai = tested,
        ApiProvider::Gemini => current_config.api_test_status.gemini = tested,
        ApiProvider::DeepSeek => current_config.api_test_status.deepseek = tested,
        ApiProvider::Volcengine => current_config.api_test_status.volcengine = tested,
    }
    
    config::save_config(&app_handle, &current_config).await
        .map_err(|e| e.to_string())?;
    
    log::info!("Set API test status for {}: {}", provider, tested);
    Ok(())
}

/// 获取 API 测试状态
/// 
/// # Arguments
/// * `app_handle` - Tauri 应用句柄
/// * `provider` - AI 提供商名称
#[tauri::command]
pub async fn get_api_test_status(app_handle: AppHandle, provider: String) -> Result<bool, String> {
    let provider_enum = ApiProvider::from_str(&provider)
        .map_err(|e| e.to_string())?;
    
    let current_config = config::load_config(&app_handle).await
        .map_err(|e| e.to_string())?;
    
    let tested = match provider_enum {
        ApiProvider::OpenAI => current_config.api_test_status.openai,
        ApiProvider::Gemini => current_config.api_test_status.gemini,
        ApiProvider::DeepSeek => current_config.api_test_status.deepseek,
        ApiProvider::Volcengine => current_config.api_test_status.volcengine,
    };
    
    Ok(tested)
}


// ============================================================================
// 截图功能命令
// Requirements: 8.1, 8.2, 8.3, 8.4, 8.5
// ============================================================================

use crate::screenshot::{ScreenshotManager, ScreenshotRegion, ScreenshotResult, MonitorInfo};

/// 获取所有显示器信息
/// 
/// Requirement 8.2: 支持多显示器截图
/// 
/// # Returns
/// * 显示器信息列表
#[tauri::command]
pub async fn get_monitors() -> Result<Vec<MonitorInfo>, String> {
    ScreenshotManager::get_monitors()
        .map_err(|e| e.to_string())
}

/// 捕获整个屏幕
/// 
/// Requirement 8.4: 实现屏幕捕获
/// 
/// # Arguments
/// * `monitor_id` - 可选的显示器 ID，默认使用主显示器
/// 
/// # Returns
/// * 截图结果，包含 Base64 编码的图片数据
#[tauri::command]
pub async fn capture_full_screen(monitor_id: Option<u32>) -> Result<ScreenshotResult, String> {
    ScreenshotManager::capture_full_screen(monitor_id)
        .map_err(|e| e.to_string())
}

/// 捕获指定区域
/// 
/// Requirement 8.2, 8.3: 矩形选择截图
/// 
/// # Arguments
/// * `region` - 截图区域（x, y, width, height）
/// 
/// # Returns
/// * 截图结果，包含 Base64 编码的图片数据
#[tauri::command]
pub async fn capture_region(region: ScreenshotRegion) -> Result<ScreenshotResult, String> {
    ScreenshotManager::capture_region(region)
        .map_err(|e| e.to_string())
}

/// 隐藏窗口后截取全屏
/// 
/// 用于实现全屏截图功能：
/// 1. 隐藏应用窗口
/// 2. 等待窗口完全隐藏
/// 3. 截取全屏
/// 4. 返回截图数据（窗口保持隐藏，由前端控制恢复）
/// 
/// # Arguments
/// * `window` - Tauri 窗口句柄
/// * `monitor_id` - 可选的显示器 ID
/// 
/// # Returns
/// * 截图结果，包含 Base64 编码的图片数据
#[tauri::command]
pub async fn capture_screen_hidden(
    window: tauri::Window,
    monitor_id: Option<u32>,
) -> Result<ScreenshotResult, String> {
    // 隐藏窗口
    window.hide().map_err(|e| format!("Failed to hide window: {}", e))?;
    
    // 等待窗口完全隐藏
    tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
    
    // 截取全屏
    let result = ScreenshotManager::capture_full_screen(monitor_id)
        .map_err(|e| e.to_string());
    
    // 注意：窗口保持隐藏状态，由前端在选区完成后调用 show_window 恢复
    result
}

/// 显示窗口
/// 
/// 用于截图选区完成后恢复窗口显示
/// 
/// # Arguments
/// * `window` - Tauri 窗口句柄
#[tauri::command]
pub async fn show_window(window: tauri::Window) -> Result<(), String> {
    window.show().map_err(|e| format!("Failed to show window: {}", e))?;
    window.set_focus().map_err(|e| format!("Failed to focus window: {}", e))?;
    Ok(())
}

/// 裁剪图片
/// 
/// 根据选区坐标裁剪 Base64 图片
/// 
/// # Arguments
/// * `image_data` - Base64 编码的图片数据
/// * `region` - 裁剪区域
/// 
/// # Returns
/// * 裁剪后的截图结果
#[tauri::command]
pub async fn crop_screenshot(
    image_data: String,
    region: ScreenshotRegion,
) -> Result<ScreenshotResult, String> {
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    use image::{DynamicImage, ImageEncoder};
    
    // 解码 Base64
    let bytes = STANDARD.decode(&image_data)
        .map_err(|e| format!("Failed to decode base64: {}", e))?;
    
    // 加载图片
    let img = image::load_from_memory(&bytes)
        .map_err(|e| format!("Failed to load image: {}", e))?;
    
    // 裁剪
    let cropped = img.crop_imm(
        region.x.max(0) as u32,
        region.y.max(0) as u32,
        region.width,
        region.height,
    );
    
    // 编码为 PNG
    let mut buffer = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut buffer);
    encoder.write_image(
        cropped.to_rgba8().as_raw(),
        cropped.width(),
        cropped.height(),
        image::ExtendedColorType::Rgba8,
    ).map_err(|e| format!("Failed to encode image: {}", e))?;
    
    // Base64 编码
    let base64_data = STANDARD.encode(&buffer);
    
    Ok(ScreenshotResult {
        data: base64_data,
        mime_type: "image/png".to_string(),
        width: cropped.width(),
        height: cropped.height(),
        size: buffer.len(),
    })
}

// ============================================================================
// 窗口控制命令
// ============================================================================

/// 设置窗口始终在最上层并激活
#[tauri::command]
pub async fn set_window_always_on_top(window: tauri::WebviewWindow, on_top: bool) -> Result<(), String> {
    window.set_always_on_top(on_top)
        .map_err(|e| format!("Failed to set always on top: {}", e))?;
    
    if on_top {
        window.set_focus()
            .map_err(|e| format!("Failed to set focus: {}", e))?;
    }
    
    Ok(())
}


// ============================================================================
// MCP 相关命令
// ============================================================================

use crate::popup::{PopupRequest, PopupResponse};

/// CLI 参数结构
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct CliArgs {
    /// MCP 请求文件路径
    pub mcp_request_file: Option<String>,
    /// 是否为 MCP 模式
    pub mcp_mode: bool,
}

impl CliArgs {
    /// 解析命令行参数
    pub fn parse() -> Self {
        let args: Vec<String> = std::env::args().collect();
        let mut cli_args = CliArgs::default();
        
        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "--mcp-request" | "-r" => {
                    if i + 1 < args.len() {
                        cli_args.mcp_request_file = Some(args[i + 1].clone());
                        cli_args.mcp_mode = true;
                        i += 1;
                    }
                }
                "--mcp" | "-m" => {
                    cli_args.mcp_mode = true;
                }
                _ => {}
            }
            i += 1;
        }
        
        cli_args
    }
}

/// 获取 CLI 参数
#[tauri::command]
pub fn get_cli_args() -> Result<CliArgs, String> {
    Ok(CliArgs::parse())
}

/// 读取 MCP 请求文件
#[tauri::command]
pub async fn read_mcp_request(file_path: String) -> Result<PopupRequest, String> {
    let content = tokio::fs::read_to_string(&file_path)
        .await
        .map_err(|e| format!("Failed to read MCP request file: {}", e))?;
    
    let request: PopupRequest = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse MCP request: {}", e))?;
    
    log::info!("Loaded MCP request: id={}, message={:?}", request.id, request.message);
    Ok(request)
}

/// 写入 MCP 响应文件
#[tauri::command]
pub async fn write_response_file(
    file_path: String,
    response: PopupResponse,
) -> Result<(), String> {
    let content = serde_json::to_string_pretty(&response)
        .map_err(|e| format!("Failed to serialize response: {}", e))?;
    
    tokio::fs::write(&file_path, content)
        .await
        .map_err(|e| format!("Failed to write response file: {}", e))?;
    
    log::info!("Wrote MCP response to: {}", file_path);
    Ok(())
}

/// 退出应用
/// 确保在 MCP 模式下正确退出进程
#[tauri::command]
pub fn exit_app(app_handle: tauri::AppHandle) -> Result<(), String> {
    log::info!("[exit_app] 正在退出应用...");
    
    // 使用 app_handle.exit() 确保进程完全退出
    app_handle.exit(0);
    
    Ok(())
}


// ============================================================================
// LLM 文本优化命令
// ============================================================================

use crate::llm::{LlmProvider, LlmConfig, get_optimization_prompt, OptimizationType};

/// 从配置中获取指定提供商的 API 密钥
async fn get_api_key_from_config(app_handle: &AppHandle, provider: &str) -> Result<String, String> {
    let current_config = config::load_config(app_handle).await
        .map_err(|e| e.to_string())?;
    
    let provider_enum = ApiProvider::from_str(provider)
        .map_err(|e| e.to_string())?;
    
    let obfuscated = match provider_enum {
        ApiProvider::OpenAI => current_config.api_keys.openai,
        ApiProvider::Gemini => current_config.api_keys.gemini,
        ApiProvider::DeepSeek => current_config.api_keys.deepseek,
        ApiProvider::Volcengine => current_config.api_keys.volcengine,
    };
    
    match obfuscated {
        Some(ref s) if !s.is_empty() => {
            ApiKeyManager::deobfuscate(s).map_err(|e| e.to_string())
        }
        _ => Err(format!("未配置 {} 的 API 密钥", provider)),
    }
}

/// 从配置中获取第一个已配置的提供商（按优先级顺序）
async fn get_first_configured_provider(app_handle: &AppHandle) -> Result<(String, String), String> {
    log::info!("[优化] 获取已配置的提供商...");
    let current_config = config::load_config(app_handle).await
        .map_err(|e| e.to_string())?;
    
    log::info!("[优化] provider_order: {:?}", current_config.provider_order);
    log::info!("[优化] api_keys - openai: {}, gemini: {}, deepseek: {}, volcengine: {}", 
        current_config.api_keys.openai.is_some(),
        current_config.api_keys.gemini.is_some(),
        current_config.api_keys.deepseek.is_some(),
        current_config.api_keys.volcengine.is_some()
    );
    
    // 获取 API 密钥的辅助函数
    let get_key = |provider: &str| -> Option<String> {
        let obfuscated = match provider {
            "openai" => current_config.api_keys.openai.as_ref(),
            "gemini" => current_config.api_keys.gemini.as_ref(),
            "deepseek" => current_config.api_keys.deepseek.as_ref(),
            "volcengine" => current_config.api_keys.volcengine.as_ref(),
            _ => None,
        };
        obfuscated.and_then(|s| {
            if s.is_empty() {
                log::info!("[优化] {} 密钥为空", provider);
                None
            } else {
                match ApiKeyManager::deobfuscate(s) {
                    Ok(key) => {
                        log::info!("[优化] {} 密钥解密成功", provider);
                        Some(key)
                    }
                    Err(e) => {
                        log::error!("[优化] {} 密钥解密失败: {}", provider, e);
                        None
                    }
                }
            }
        })
    };
    
    // 优先按 provider_order 顺序查找
    for provider in &current_config.provider_order {
        if let Some(api_key) = get_key(provider) {
            log::info!("[优化] 使用提供商: {}", provider);
            return Ok((provider.clone(), api_key));
        }
    }
    
    // 如果 provider_order 为空或没有找到，使用默认顺序
    let default_order = ["openai", "gemini", "deepseek", "volcengine"];
    for provider in default_order {
        if let Some(api_key) = get_key(provider) {
            log::info!("[优化] 使用提供商: {}", provider);
            return Ok((provider.to_string(), api_key));
        }
    }
    
    log::error!("[优化] 未找到任何已配置的 API 密钥");
    Err("未配置任何 API 密钥，请先在设置中配置".to_string())
}

/// 优化文本
/// 
/// 使用配置的 AI 提供商优化文本
/// 
/// # Arguments
/// * `app_handle` - Tauri 应用句柄
/// * `text` - 要优化的文本
/// * `optimization_type` - 优化类型 ID（从配置的 optimization_types 中匹配）
/// 
/// # Returns
/// * 优化后的文本
#[tauri::command]
pub async fn optimize_text(
    app_handle: AppHandle,
    text: String,
    optimization_type: String,
) -> Result<String, String> {
    log::info!("[优化] 开始文本优化，类型: {}", optimization_type);
    
    // 从配置中查找优化类型
    let prompt_template = {
        let config = crate::config::load_config(&app_handle).await
            .map_err(|e| e.to_string())?;
        
        // 在配置的优化类型中查找匹配的 ID
        let found = config.optimization_types.iter()
            .find(|t| t.id == optimization_type && t.enabled);
        
        match found {
            Some(t) => {
                log::info!("[优化] 找到优化类型: id={}, label={}", t.id, t.label);
                t.prompt.clone()
            }
            None => {
                log::error!("[优化] 未找到优化类型: {}", optimization_type);
                log::info!("[优化] 可用的优化类型: {:?}", 
                    config.optimization_types.iter()
                        .map(|t| format!("{}(enabled={})", t.id, t.enabled))
                        .collect::<Vec<_>>()
                );
                return Err(format!("未找到优化类型: {}", optimization_type));
            }
        }
    };
    
    log::debug!("[优化] 提示词模板前100字符: {}", &prompt_template.chars().take(100).collect::<String>());
    
    // 获取第一个已配置的提供商
    let (provider_name, api_key) = get_first_configured_provider(&app_handle).await?;
    log::info!("[优化] 创建 LLM 配置...");
    
    // 创建 LLM 配置
    let config = LlmConfig::from_provider(&provider_name, api_key)
        .ok_or_else(|| format!("不支持的提供商: {}", provider_name))?;
    log::info!("[优化] LLM 配置创建成功: model={}, base_url={}", config.model, config.base_url);
    
    // 创建 Provider
    let llm = LlmProvider::new(config)?;
    log::info!("[优化] LLM Provider 创建成功，开始调用 API...");
    
    // 系统提示词作为 system 角色，用户输入作为 user 角色
    log::info!("[优化] 系统提示词长度: {} 字符, 用户输入长度: {} 字符", prompt_template.len(), text.len());
    
    match llm.optimize_text(&text, &prompt_template).await {
        Ok(result) => {
            log::info!("[优化] API 调用成功，结果长度: {} 字符", result.len());
            Ok(result)
        }
        Err(e) => {
            log::error!("[优化] API 调用失败: {}", e);
            Err(e)
        }
    }
}

/// 使用指定提供商优化文本
/// 
/// # Arguments
/// * `app_handle` - Tauri 应用句柄
/// * `text` - 要优化的文本
/// * `provider` - AI 提供商名称
/// * `mode` - 优化模式 (optimize, reinforce)
/// * `custom_prompt` - 自定义提示词（reinforce 模式使用）
/// 
/// # Returns
/// * 优化后的文本
#[tauri::command]
pub async fn optimize_text_with_provider(
    app_handle: AppHandle,
    text: String,
    provider: String,
    mode: String,
    custom_prompt: Option<String>,
) -> Result<String, String> {
    // 解析优化类型
    let opt_type = OptimizationType::from_str(&mode)
        .ok_or_else(|| format!("无效的优化模式: {}", mode))?;
    
    // 获取 API 密钥
    let api_key = get_api_key_from_config(&app_handle, &provider).await?;
    
    // 创建 LLM 配置
    let config = LlmConfig::from_provider(&provider, api_key)
        .ok_or_else(|| format!("不支持的提供商: {}", provider))?;
    
    // 创建 Provider
    let llm = LlmProvider::new(config)?;
    
    // 获取提示词
    let system_prompt = get_optimization_prompt(opt_type, custom_prompt.as_deref());
    
    // 调用 LLM
    llm.optimize_text(&text, &system_prompt).await
}

/// 测试 API 连接
/// 
/// # Arguments
/// * `app_handle` - Tauri 应用句柄
/// * `provider` - AI 提供商名称
/// 
/// # Returns
/// * 测试结果消息
#[tauri::command]
pub async fn test_api_connection(app_handle: AppHandle, provider: String) -> Result<String, String> {
    // 获取 API 密钥
    let api_key = get_api_key_from_config(&app_handle, &provider).await?;
    
    // 创建 LLM 配置
    let config = LlmConfig::from_provider(&provider, api_key)
        .ok_or_else(|| format!("不支持的提供商: {}", provider))?;
    
    // 创建 Provider
    let llm = LlmProvider::new(config)?;
    
    // 测试连接
    llm.test_connection().await?;
    
    Ok(format!("{} API 连接成功", provider))
}

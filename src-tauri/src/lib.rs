mod api_keys;
mod audio;
mod config;
mod commands;
mod image_processor;
pub mod llm;
pub mod mcp_server;
pub mod popup;
mod screenshot;
mod types;

use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

pub use api_keys::{ApiKeyManager, ApiKeyError, ApiProvider};
pub use audio::{AudioNotifier, AudioError};
pub use config::load_config_direct;
pub use image_processor::ImageProcessor;
pub use mcp_server::{
    McpServer, InteractiveFeedbackParams, OptimizeUserInputParams,
    OptimizeResult, PopupResponse,
    run_mcp_server,
    validate_interactive_feedback_params, validate_optimize_user_input_params,
};
pub use popup::PopupRequest;
pub use screenshot::{ScreenshotManager, ScreenshotRegion, ScreenshotResult, MonitorInfo};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_screenshots::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::save_config,
            commands::submit_feedback,
            commands::process_image,
            commands::play_notification_sound,
            commands::validate_audio_file,
            commands::get_supported_audio_formats,
            commands::get_builtin_sounds,
            commands::get_canned_responses,
            commands::save_canned_responses,
            // API 密钥管理命令
            commands::save_api_key,
            commands::get_api_key,
            commands::delete_api_key,
            commands::has_api_key,
            commands::get_configured_providers,
            commands::get_masked_api_key,
            commands::set_api_test_status,
            commands::get_api_test_status,
            commands::set_provider_order,
            commands::get_provider_order,
            // 截图功能命令
            commands::get_monitors,
            commands::capture_full_screen,
            commands::capture_region,
            commands::capture_screen_hidden,
            commands::show_window,
            commands::crop_screenshot,
            // 窗口控制命令
            commands::set_window_always_on_top,
            // MCP 相关命令
            commands::get_cli_args,
            commands::read_mcp_request,
            commands::write_response_file,
            commands::exit_app,
            // LLM 文本优化命令
            commands::optimize_text,
            commands::optimize_text_with_provider,
            commands::test_api_connection,
        ])
        // 注意：不要添加自定义 on_webview_event 处理器
        // Tauri 内部会自动处理 DragDrop 事件并发送到前端
        // 自定义处理器会干扰默认行为
        .setup(|app| {
            // 初始化日志
            env_logger::init();
            log::info!("Interactive Feedback MCP started");
            
            // 动态获取版本号
            let version = app.config().version.clone().unwrap_or_else(|| "0.0.0".to_string());
            let title = format!("Interactive Feedback (v{})", version);
            
            // 手动创建窗口，使用 Tauri 原生拖拽以获取完整文件路径
            let _window = WebviewWindowBuilder::new(
                app,
                "main",
                WebviewUrl::App("index.html".into())
            )
            .title(&title)
            .inner_size(1024.0, 800.0)
            .min_inner_size(400.0, 300.0)
            .resizable(true)
            .center()
            .focused(true)
            .visible(true)
            // 不禁用拖拽处理器，使用 Tauri 原生拖拽以获取完整文件路径
            // .disable_drag_drop_handler()
            .build()?;
            
            log::info!("[Setup] 窗口已创建 ({}), 使用 Tauri 原生拖拽", title);
            
            // 初始化配置
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = config::init_config(&app_handle).await {
                    log::error!("Failed to initialize config: {}", e);
                }
            });
            
            // MCP 模式下强制激活窗口
            let app_handle_window = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                // 短暂延迟确保窗口初始化完成
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                
                // 检测 MCP 模式
                let args: Vec<String> = std::env::args().collect();
                let is_mcp_mode = args.iter().any(|arg| arg == "--mcp-request" || arg == "-r");
                
                if is_mcp_mode {
                    log::info!("[MCP] 检测到 MCP 模式，强制激活窗口");
                    
                    // macOS: 使用 NSApplication 激活应用
                    #[cfg(target_os = "macos")]
                    {
                        use cocoa::appkit::{NSApp, NSApplication, NSApplicationActivationPolicy};
                        use objc::runtime::YES;
                        unsafe {
                            let app = NSApp();
                            app.setActivationPolicy_(NSApplicationActivationPolicy::NSApplicationActivationPolicyRegular);
                            app.activateIgnoringOtherApps_(YES);
                        }
                        log::info!("[MCP] macOS NSApplication 已激活");
                    }
                    
                    if let Some(window) = app_handle_window.get_webview_window("main") {
                        let _ = window.unminimize();
                        let _ = window.show();
                        let _ = window.set_focus();
                        let _ = window.set_always_on_top(true);
                        log::info!("[MCP] 窗口已激活并置顶");
                        
                        // 短暂延迟后取消置顶
                        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                        let _ = window.set_always_on_top(false);
                        log::info!("[MCP] 窗口置顶已取消");
                    } else {
                        log::warn!("[MCP] 未找到主窗口");
                    }
                }
            });
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

use crate::types::AppConfig;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Manager};
use thiserror::Error;
use tokio::sync::RwLock;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Failed to get app data directory")]
    NoAppDataDir,
    #[error("Configuration file corrupted, reset to defaults")]
    Corrupted,
}

/// ConfigManager - 配置管理器
/// 负责配置的加载、保存和持久化
/// Requirements: 14.1, 14.2, 14.3, 14.4
pub struct ConfigManager {
    config_path: PathBuf,
    config: Arc<RwLock<AppConfig>>,
}

impl ConfigManager {
    /// 创建新的 ConfigManager 实例
    pub fn new(config_path: PathBuf) -> Self {
        Self {
            config_path,
            config: Arc::new(RwLock::new(AppConfig::default())),
        }
    }

    /// 从 AppHandle 创建 ConfigManager
    pub fn from_app_handle(app_handle: &AppHandle) -> Result<Self, ConfigError> {
        let config_path = get_config_path(app_handle)?;
        Ok(Self::new(config_path))
    }

    /// 获取配置文件路径
    pub fn config_path(&self) -> &PathBuf {
        &self.config_path
    }

    /// 加载配置 (Requirements: 14.2, 14.3, 14.4)
    pub async fn load(&self) -> Result<AppConfig, ConfigError> {
        // 确保目录存在
        if let Some(parent) = self.config_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        if self.config_path.exists() {
            let content = tokio::fs::read_to_string(&self.config_path).await?;
            
            // 尝试解析配置，如果失败则重置为默认值 (Requirement 14.4)
            match serde_json::from_str::<AppConfig>(&content) {
                Ok(loaded_config) => {
                    let mut config = self.config.write().await;
                    *config = loaded_config.clone();
                    log::info!("Config loaded from {:?}", self.config_path);
                    Ok(loaded_config)
                }
                Err(e) => {
                    log::warn!("Config file corrupted: {}, resetting to defaults", e);
                    let default_config = AppConfig::default();
                    self.save_internal(&default_config).await?;
                    let mut config = self.config.write().await;
                    *config = default_config.clone();
                    Err(ConfigError::Corrupted)
                }
            }
        } else {
            // 配置文件不存在，创建默认配置 (Requirement 14.3)
            let default_config = AppConfig::default();
            self.save_internal(&default_config).await?;
            let mut config = self.config.write().await;
            *config = default_config.clone();
            log::info!("Created default config at {:?}", self.config_path);
            Ok(default_config)
        }
    }

    /// 保存配置 (Requirement 14.1)
    pub async fn save(&self, new_config: &AppConfig) -> Result<(), ConfigError> {
        self.save_internal(new_config).await?;
        let mut config = self.config.write().await;
        *config = new_config.clone();
        Ok(())
    }

    /// 内部保存方法
    async fn save_internal(&self, config: &AppConfig) -> Result<(), ConfigError> {
        // 确保目录存在
        if let Some(parent) = self.config_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let json = serde_json::to_string_pretty(config)?;
        tokio::fs::write(&self.config_path, json).await?;
        log::info!("Config saved to {:?}", self.config_path);
        Ok(())
    }

    /// 获取当前配置的克隆
    pub async fn get(&self) -> AppConfig {
        self.config.read().await.clone()
    }

    /// 更新配置的特定字段
    pub async fn update<F>(&self, updater: F) -> Result<(), ConfigError>
    where
        F: FnOnce(&mut AppConfig),
    {
        let mut config = self.config.write().await;
        updater(&mut config);
        self.save_internal(&config).await?;
        Ok(())
    }
}

/// 获取配置文件路径
pub fn get_config_path(app_handle: &AppHandle) -> Result<PathBuf, ConfigError> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|_| ConfigError::NoAppDataDir)?;
    
    Ok(app_data_dir.join("config.json"))
}

/// 初始化配置 (Requirements: 14.2, 14.3)
pub async fn init_config(app_handle: &AppHandle) -> Result<(), ConfigError> {
    let config_path = get_config_path(app_handle)?;
    
    // 确保目录存在
    if let Some(parent) = config_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    
    // 如果配置文件不存在，创建默认配置
    if !config_path.exists() {
        let default_config = AppConfig::default();
        let json = serde_json::to_string_pretty(&default_config)?;
        tokio::fs::write(&config_path, json).await?;
        log::info!("Created default config at {:?}", config_path);
    } else {
        // 验证现有配置文件是否有效
        let content = tokio::fs::read_to_string(&config_path).await?;
        if serde_json::from_str::<AppConfig>(&content).is_err() {
            // 配置文件损坏，重置为默认值 (Requirement 14.4)
            log::warn!("Config file corrupted, resetting to defaults");
            let default_config = AppConfig::default();
            let json = serde_json::to_string_pretty(&default_config)?;
            tokio::fs::write(&config_path, json).await?;
        }
    }
    
    Ok(())
}

/// 加载配置 (Requirements: 14.2, 14.3, 14.4)
pub async fn load_config(app_handle: &AppHandle) -> Result<AppConfig, ConfigError> {
    let config_path = get_config_path(app_handle)?;
    
    if config_path.exists() {
        let content = tokio::fs::read_to_string(&config_path).await?;
        
        // 尝试解析，失败则返回默认配置 (Requirement 14.4)
        match serde_json::from_str::<AppConfig>(&content) {
            Ok(config) => Ok(config),
            Err(e) => {
                log::warn!("Failed to parse config: {}, using defaults", e);
                let default_config = AppConfig::default();
                // 重置损坏的配置文件
                let json = serde_json::to_string_pretty(&default_config)?;
                tokio::fs::write(&config_path, json).await?;
                Ok(default_config)
            }
        }
    } else {
        // 返回默认配置 (Requirement 14.3)
        Ok(AppConfig::default())
    }
}

/// 保存配置 (Requirement 14.1)
pub async fn save_config(app_handle: &AppHandle, config: &AppConfig) -> Result<(), ConfigError> {
    let config_path = get_config_path(app_handle)?;
    
    // 确保目录存在
    if let Some(parent) = config_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    
    let json = serde_json::to_string_pretty(config)?;
    tokio::fs::write(&config_path, json).await?;
    
    log::info!("Config saved to {:?}", config_path);
    Ok(())
}

/// 获取默认配置文件路径（不依赖 AppHandle，用于 MCP server）
pub fn get_default_config_path() -> Result<PathBuf, ConfigError> {
    let app_data_dir = dirs::data_dir()
        .ok_or(ConfigError::NoAppDataDir)?
        .join("com.whale-interactive-feedback.app");
    
    Ok(app_data_dir.join("config.json"))
}

/// 直接从文件加载配置（不依赖 AppHandle，用于 MCP server）
pub async fn load_config_direct() -> Result<AppConfig, ConfigError> {
    let config_path = get_default_config_path()?;
    
    if config_path.exists() {
        let content = tokio::fs::read_to_string(&config_path).await?;
        
        match serde_json::from_str::<AppConfig>(&content) {
            Ok(config) => Ok(config),
            Err(e) => {
                log::warn!("Failed to parse config: {}, using defaults", e);
                Ok(AppConfig::default())
            }
        }
    } else {
        Ok(AppConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{DisplayMode, Layout, Theme};
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_config_manager_create_default() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.json");
        
        let manager = ConfigManager::new(config_path.clone());
        let config = manager.load().await.unwrap();
        
        // 验证默认配置
        assert_eq!(config.theme, Theme::Dark);
        assert_eq!(config.layout, Layout::Vertical);
        assert_eq!(config.display_mode, DisplayMode::Simple);
        assert!(config.audio_enabled);
        assert!(!config.window_pinned);
        
        // 验证文件已创建
        assert!(config_path.exists());
    }

    #[tokio::test]
    async fn test_config_manager_save_and_load() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.json");
        
        let manager = ConfigManager::new(config_path);
        
        // 创建自定义配置
        let mut custom_config = AppConfig::default();
        custom_config.theme = Theme::Light;
        custom_config.layout = Layout::Horizontal;
        custom_config.splitter_position = 75.0;
        
        // 保存配置
        manager.save(&custom_config).await.unwrap();
        
        // 重新加载并验证
        let loaded_config = manager.load().await.unwrap();
        assert_eq!(loaded_config.theme, Theme::Light);
        assert_eq!(loaded_config.layout, Layout::Horizontal);
        assert!((loaded_config.splitter_position - 75.0).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_config_manager_corrupted_file() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.json");
        
        // 写入损坏的 JSON
        tokio::fs::write(&config_path, "{ invalid json }").await.unwrap();
        
        let manager = ConfigManager::new(config_path.clone());
        
        // 加载应该返回 Corrupted 错误并重置为默认值
        let result = manager.load().await;
        assert!(matches!(result, Err(ConfigError::Corrupted)));
        
        // 再次加载应该成功（已重置为默认值）
        let config = manager.load().await.unwrap();
        assert_eq!(config.theme, Theme::Dark);
    }

    #[tokio::test]
    async fn test_config_manager_update() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.json");
        
        let manager = ConfigManager::new(config_path);
        manager.load().await.unwrap();
        
        // 更新配置
        manager.update(|config| {
            config.theme = Theme::Light;
            config.audio_enabled = false;
        }).await.unwrap();
        
        // 验证更新
        let config = manager.get().await;
        assert_eq!(config.theme, Theme::Light);
        assert!(!config.audio_enabled);
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use crate::types::{ApiKeys, DisplayMode, Layout, Theme};
    use proptest::prelude::*;

    // 生成随机字符串的策略
    fn arbitrary_string() -> impl Strategy<Value = String> {
        prop::string::string_regex("[a-zA-Z0-9_-]{0,50}").unwrap()
    }

    // 生成可选字符串的策略
    fn arbitrary_optional_string() -> impl Strategy<Value = Option<String>> {
        prop::option::of(arbitrary_string())
    }

    // 生成 ApiKeys 的策略
    fn arbitrary_api_keys() -> impl Strategy<Value = ApiKeys> {
        (
            arbitrary_optional_string(),
            arbitrary_optional_string(),
            arbitrary_optional_string(),
            arbitrary_optional_string(),
        ).prop_map(|(openai, gemini, deepseek, volcengine)| {
            ApiKeys {
                openai,
                gemini,
                deepseek,
                volcengine,
            }
        })
    }

    // 生成 AppConfig 的策略
    fn arbitrary_config() -> impl Strategy<Value = AppConfig> {
        (
            prop_oneof![Just(Theme::Dark), Just(Theme::Light)],
            prop_oneof![Just(Layout::Vertical), Just(Layout::Horizontal)],
            prop_oneof![Just(DisplayMode::Simple), Just(DisplayMode::Full)],
            any::<bool>(),
            arbitrary_optional_string(),
            any::<bool>(),
            any::<bool>(),
            0.0f64..=100.0f64,
            arbitrary_api_keys(),
            arbitrary_string(),
            arbitrary_string(),
            arbitrary_string(),
        ).prop_map(|(
            theme,
            layout,
            display_mode,
            audio_enabled,
            audio_file,
            window_pinned,
            auto_minimize,
            splitter_position,
            api_keys,
            selected_provider,
            optimize_prompt,
            enhance_prompt,
        )| {
            AppConfig {
                theme,
                layout,
                display_mode,
                audio_enabled,
                audio_file,
                window_pinned,
                auto_minimize,
                splitter_position,
                api_keys,
                selected_provider,
                optimize_prompt,
                enhance_prompt,
            }
        })
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// **Feature: interactive-feedback-tauri, Property 11: Configuration Persistence Round-Trip**
        /// **Validates: Requirements 14.1, 14.2**
        /// 
        /// *For any* valid configuration, saving to file and loading back SHALL produce 
        /// an equivalent configuration.
        #[test]
        fn test_config_json_roundtrip(config in arbitrary_config()) {
            // Serialize to JSON
            let json = serde_json::to_string(&config).unwrap();
            
            // Deserialize back
            let deserialized: AppConfig = serde_json::from_str(&json).unwrap();

            // Verify all fields match
            prop_assert_eq!(config.theme, deserialized.theme);
            prop_assert_eq!(config.layout, deserialized.layout);
            prop_assert_eq!(config.display_mode, deserialized.display_mode);
            prop_assert_eq!(config.audio_enabled, deserialized.audio_enabled);
            prop_assert_eq!(config.audio_file, deserialized.audio_file);
            prop_assert_eq!(config.window_pinned, deserialized.window_pinned);
            prop_assert_eq!(config.auto_minimize, deserialized.auto_minimize);
            prop_assert!((config.splitter_position - deserialized.splitter_position).abs() < 0.0001);
            prop_assert_eq!(config.api_keys.openai, deserialized.api_keys.openai);
            prop_assert_eq!(config.api_keys.gemini, deserialized.api_keys.gemini);
            prop_assert_eq!(config.api_keys.deepseek, deserialized.api_keys.deepseek);
            prop_assert_eq!(config.api_keys.volcengine, deserialized.api_keys.volcengine);
            prop_assert_eq!(config.selected_provider, deserialized.selected_provider);
            prop_assert_eq!(config.optimize_prompt, deserialized.optimize_prompt);
            prop_assert_eq!(config.enhance_prompt, deserialized.enhance_prompt);
        }

        /// Property test: Configuration file persistence round-trip
        /// Tests that saving to file and loading back produces equivalent config
        #[test]
        fn test_config_file_roundtrip(config in arbitrary_config()) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let temp_dir = tempfile::tempdir().unwrap();
                let config_path = temp_dir.path().join("config.json");
                
                let manager = ConfigManager::new(config_path);
                
                // Save config
                manager.save(&config).await.unwrap();
                
                // Load config back
                let loaded = manager.load().await.unwrap();
                
                // Verify all fields match
                assert_eq!(config.theme, loaded.theme);
                assert_eq!(config.layout, loaded.layout);
                assert_eq!(config.display_mode, loaded.display_mode);
                assert_eq!(config.audio_enabled, loaded.audio_enabled);
                assert_eq!(config.audio_file, loaded.audio_file);
                assert_eq!(config.window_pinned, loaded.window_pinned);
                assert_eq!(config.auto_minimize, loaded.auto_minimize);
                assert!((config.splitter_position - loaded.splitter_position).abs() < 0.0001);
                assert_eq!(config.api_keys.openai, loaded.api_keys.openai);
                assert_eq!(config.api_keys.gemini, loaded.api_keys.gemini);
                assert_eq!(config.api_keys.deepseek, loaded.api_keys.deepseek);
                assert_eq!(config.api_keys.volcengine, loaded.api_keys.volcengine);
                assert_eq!(config.selected_provider, loaded.selected_provider);
                assert_eq!(config.optimize_prompt, loaded.optimize_prompt);
                assert_eq!(config.enhance_prompt, loaded.enhance_prompt);
            });
        }
    }
}

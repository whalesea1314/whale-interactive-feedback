use serde::{Deserialize, Serialize};

/// 主题色
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ThemeColor {
    #[default]
    Blue,
    Mint,
    Rose,
    Peach,
    Lavender,
    Slate,
}

/// 字体大小配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontSizeConfig {
    pub display: u32,
    pub options: u32,
    pub input: u32,
}

impl Default for FontSizeConfig {
    fn default() -> Self {
        Self {
            display: 13,
            options: 13,
            input: 13,
        }
    }
}

/// 应用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub theme: Theme,
    #[serde(default)]
    pub theme_color: ThemeColor,
    #[serde(default)]
    pub font_size: FontSizeConfig,
    pub layout: Layout,
    pub display_mode: DisplayMode,
    pub audio_enabled: bool,
    pub audio_file: Option<String>,
    pub window_pinned: bool,
    pub auto_minimize: bool,
    pub splitter_position: f64,
    pub api_keys: ApiKeys,
    #[serde(default)]
    pub api_test_status: ApiTestStatus,
    /// API 提供商优先级顺序（第一个优先级最高）
    #[serde(default)]
    pub provider_order: Vec<String>,
    pub selected_provider: String,
    pub optimize_prompt: String,
    pub enhance_prompt: String,
    /// 自定义选项功能
    #[serde(default)]
    pub custom_options_enabled: bool,
    #[serde(default = "default_custom_options")]
    pub custom_options: Vec<String>,
    /// 文本优化类型配置
    #[serde(default = "default_optimization_types")]
    pub optimization_types: Vec<OptimizationTypeConfig>,
}

/// 默认自定义选项
fn default_custom_options() -> Vec<String> {
    vec![
        "好的，我明白了".to_string(),
        "请继续".to_string(),
        "需要更多信息".to_string(),
        "返回上一步".to_string(),
        "暂停，让我思考一下".to_string(),
    ]
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: Theme::Dark,
            theme_color: ThemeColor::default(),
            font_size: FontSizeConfig::default(),
            layout: Layout::Horizontal,
            display_mode: DisplayMode::Full,
            audio_enabled: true,
            audio_file: None,
            window_pinned: false,
            auto_minimize: false,
            splitter_position: 50.0,
            api_keys: ApiKeys::default(),
            api_test_status: ApiTestStatus::default(),
            provider_order: Vec::new(),
            selected_provider: "openai".to_string(),
            optimize_prompt: String::new(),
            enhance_prompt: String::new(),
            custom_options_enabled: false,
            custom_options: default_custom_options(),
            optimization_types: default_optimization_types(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    Dark,
    Light,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Layout {
    Vertical,
    Horizontal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DisplayMode {
    Simple,
    Full,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApiKeys {
    pub openai: Option<String>,
    pub gemini: Option<String>,
    pub deepseek: Option<String>,
    pub volcengine: Option<String>,
}

/// API 测试状态
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ApiTestStatus {
    #[serde(default)]
    pub openai: bool,
    #[serde(default)]
    pub gemini: bool,
    #[serde(default)]
    pub deepseek: bool,
    #[serde(default)]
    pub volcengine: bool,
}

/// 反馈内容
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FeedbackContent {
    Text { text: String },
    Image { data: String, mime_type: String },
    FileReference { display_name: String, path: String },
}

/// 反馈数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackData {
    pub content: Vec<FeedbackContent>,
}

/// 常用语
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CannedResponse {
    pub id: String,
    pub text: String,
    pub order: i32,
}

/// 处理后的图片
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedImage {
    pub data: String,
    pub mime_type: String,
    pub width: u32,
    pub height: u32,
    pub size: usize,
}

/// 截图区域
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenRegion {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// 文本优化类型配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OptimizationTypeConfig {
    pub id: String,
    pub label: String,
    pub icon: String,
    pub description: String,
    pub prompt: String,
    pub is_system: bool,
    pub enabled: bool,
}

/// 默认提示词类型
fn default_optimization_types() -> Vec<OptimizationTypeConfig> {
    vec![
        // ===== 提示词类 =====
        OptimizationTypeConfig {
            id: "prompt-optimize".to_string(),
            label: "提示词优化".to_string(),
            icon: "i-carbon-edit".to_string(),
            description: "将口语化输入转换为结构化、高质量的提示词".to_string(),
            prompt: "你是一个专业的文本优化助手。请将用户的输入文本改写为结构化、逻辑清晰的指令。只需要输出优化后的文本，不要包含任何技术参数、函数定义或元数据信息。".to_string(),
            is_system: true,
            enabled: true,
        },
        OptimizationTypeConfig {
            id: "prompt-enhance".to_string(),
            label: "提示词增强".to_string(),
            icon: "i-carbon-improve-relevance".to_string(),
            description: "深度增强提示词，扩展和完善内容".to_string(),
            prompt: "你是一个专业的文本增强助手。请对用户的输入文本进行扩展和改写，使其更加完整、详细和专业。

输出要求：
- 保持原意，扩展细节和上下文
- 补充相关的背景信息和使用场景
- 优化表达，使语言更加流畅专业
- 只输出改写结果，不要包含任何技术信息".to_string(),
            is_system: true,
            enabled: true,
        },
        // ===== 代码类 =====
        OptimizationTypeConfig {
            id: "code-review".to_string(),
            label: "代码审查".to_string(),
            icon: "i-carbon-checkmark-outline".to_string(),
            description: "从规范性、安全性、性能等维度审查代码质量".to_string(),
            prompt: r#"你是一位经验丰富的代码审查专家，擅长从规范性、安全性、性能、可维护性、可测试性等多个维度系统性分析代码质量。

技能模块：
- 规范合规：检查是否符合语言标准、团队编码规范
- 安全审查：识别潜在的安全漏洞（如注入、越权、信息泄露等）
- 性能优化：发现低效算法、冗余计算、不必要的资源占用
- 可读性分析：判断代码是否清晰易读，是否存在重复代码或过度复杂逻辑
- 异常与错误处理：检查是否有健壮的错误处理机制

输出格式：
- 使用清晰的 Markdown 格式，分条列出每项问题
- 每个问题包括：问题描述、影响分析、建议改进方案、优先级（高/中/低）
- 在指出问题的同时，也指出代码优点"#.to_string(),
            is_system: true,
            enabled: true,
        },
        OptimizationTypeConfig {
            id: "code-explainer".to_string(),
            label: "代码讲解".to_string(),
            icon: "i-carbon-bot".to_string(),
            description: "逐层深入解释代码的功能、设计意图和运行逻辑".to_string(),
            prompt: r#"你是一位资深代码讲解专家，擅长以清晰、结构化、逐层深入的方式解释任意编程语言的源代码。

你不仅讲"代码做了什么"，还分析"为什么这么写"，并在必要时提供更优雅、安全或可维护的替代方案。

技能模块：
- 语言适应性：支持 Python、Java、Golang、JavaScript、TypeScript、Ruby、C/C++、Rust 等多种语言
- 结构拆解能力：按模块/类/函数/逻辑块逐层讲解
- 运行流程分析：包括输入输出、中间状态、条件判断、循环、递归、异步等控制流讲解
- 设计模式识别：识别如单例、工厂、策略、观察者等设计模式
- 潜在问题提示：指出性能瓶颈、安全漏洞、可读性差等问题

输出风格：
- 使用 Markdown 排版，结构清晰
- 按层级讲解：概览 → 模块结构 → 具体逻辑 → 注意事项
- 使用通俗语言解释专业术语"#.to_string(),
            is_system: true,
            enabled: true,
        },
        OptimizationTypeConfig {
            id: "code-refactor".to_string(),
            label: "代码重构".to_string(),
            icon: "i-carbon-improve-relevance".to_string(),
            description: "优化代码结构，提高可读性、可维护性和扩展性".to_string(),
            prompt: r#"你是一名专业的代码重构专家，专注于优化代码的结构和设计，提高代码的可读性、可维护性和扩展性。你的目标是帮助开发者在不改变代码功能的前提下，消除冗余、简化复杂逻辑、提升性能和代码质量。

技能模块：
- 代码结构优化：重组代码结构，提升模块化和解耦性
- 命名改进：改善变量、函数、类等命名，使其更具描述性
- 重复代码消除：识别并消除代码重复，实现复用
- 简化复杂逻辑：用更简洁、清晰的逻辑代替复杂表达
- 设计模式应用：合理引入设计模式，提升代码灵活性和可扩展性
- 性能优化：在重构过程中发现并改善性能瓶颈

输出要求：
- 说明每次重构改动的目的和预期效果
- 提供前后代码对比示例
- 避免过度重构，保持代码简洁和可理解"#.to_string(),
            is_system: true,
            enabled: true,
        },
        // ===== 专家类 =====
        OptimizationTypeConfig {
            id: "fullstack-expert".to_string(),
            label: "全栈专家".to_string(),
            icon: "i-carbon-code".to_string(),
            description: "提供代码生成、架构设计等全方位技术支持".to_string(),
            prompt: r#"你是一名专业且经验丰富的全栈开发助手，专注于为用户提供代码生成、理解、重构、评审、架构设计等全方位技术支持。熟悉主流语言与框架，精通现代开发流程和架构演进。

主要职责：
- 生成符合最佳实践、易维护的高质量代码
- 清晰解析代码逻辑、性能瓶颈及潜在风险
- 提供代码结构优化、风格统一、模块解耦建议
- 辅助系统架构设计、技术选型与模块划分

技术能力：
- 多语言支持：JavaScript、Python、Go、Rust、Java、TypeScript、C# 等
- 框架理解：React、Vue、Svelte、Spring Boot、Django、Express、FastAPI 等
- 架构设计：微服务、单体、分层架构设计
- CI/CD：GitHub Actions、Jenkins、GitLab CI 配置协助

输出风格：
- 结构化输出（代码块、分点、表格等）
- 复杂问题采用"方案对比 + 推荐理由"形式
- 代码附加必要注释及依赖说明"#.to_string(),
            is_system: true,
            enabled: true,
        },
        OptimizationTypeConfig {
            id: "frontend-expert".to_string(),
            label: "前端专家".to_string(),
            icon: "i-carbon-application-web".to_string(),
            description: "精通现代 Web 技术栈，提供专业的前端解决方案".to_string(),
            prompt: r#"你是一位经验丰富的前端开发专家，精通现代 Web 技术栈，拥有良好的工程实践和审美能力。你可以根据用户需求快速构建响应式、可访问、可维护的用户界面。

技能模块：
- HTML/CSS：语义化标签、Flexbox/Grid、媒体查询、Tailwind CSS
- JavaScript/TypeScript：ES6+、模块化、异步编程、DOM 操作、TS 类型系统
- 现代框架：React、Vue、Svelte、Next.js、Nuxt、组件设计、状态管理
- 构建工具：Vite、Webpack、Babel、ESLint、Prettier
- 性能优化：懒加载、Tree Shaking、代码分割、首屏优化
- 安全/SEO/a11y：防 XSS、ARIA 标签、meta SEO 优化

输出风格：
- 使用 Markdown 输出，排版清晰，配合代码块与要点总结
- 提供可运行的 HTML/CSS/JS 示例或框架代码片段
- 保持模块化结构，避免嵌套混乱与硬编码"#.to_string(),
            is_system: true,
            enabled: true,
        },
        OptimizationTypeConfig {
            id: "backend-expert".to_string(),
            label: "后端专家".to_string(),
            icon: "i-carbon-data-base".to_string(),
            description: "精通分布式系统、微服务和数据库优化".to_string(),
            prompt: r#"你是一位资深后端架构专家，精通分布式系统、微服务、数据库优化和高并发处理。

技能模块：
- 架构设计：微服务、单体、分层架构，技术栈选型
- API 设计：RESTful/GraphQL API 规范，接口文档，版本控制
- 数据库设计：表结构设计，索引优化，分库分表策略
- 中间件：消息队列、缓存、搜索引擎集成
- 性能优化：查询优化、多级缓存、异步处理、连接池配置
- 高可用设计：负载均衡、限流降级、熔断机制、灾备方案
- 安全措施：认证授权（JWT/OAuth）、数据加密、SQL 注入防护

输出风格：
- 结构化输出，方案包含利弊分析
- 提供核心代码示例和配置说明
- 遇到不明确的需求主动询问"#.to_string(),
            is_system: true,
            enabled: true,
        },
        OptimizationTypeConfig {
            id: "mobile-expert".to_string(),
            label: "移动端专家".to_string(),
            icon: "i-carbon-terminal".to_string(),
            description: "专注于 Android、iOS 及跨平台移动应用开发".to_string(),
            prompt: r#"你是一名资深移动应用开发专家，专注于 Android 和 iOS 平台的原生与跨平台开发。

技能模块：
- 原生开发：熟悉 Android（Kotlin/Java）和 iOS（Swift/Objective-C）开发
- 跨平台框架：React Native、Flutter、Xamarin 等技术应用
- 性能优化：提升启动速度、流畅度，降低内存和电池消耗
- UI/UX 设计：符合平台设计规范，提升用户交互体验
- 安全保障：数据加密、权限管理、防篡改及防逆向
- 网络与数据管理：高效数据同步、本地缓存及离线支持
- 测试和发布：单元测试、UI 自动化测试，应用商店发布流程

输出风格：
- 条理清晰，结合具体场景给出示例代码和最佳实践
- 明确区分 Android 与 iOS 细节和平台差异
- 兼顾初学者和高级开发者"#.to_string(),
            is_system: true,
            enabled: true,
        },
        OptimizationTypeConfig {
            id: "architect".to_string(),
            label: "系统架构师".to_string(),
            icon: "i-carbon-chart-network".to_string(),
            description: "提供高质量的系统架构设计、技术选型与性能优化建议".to_string(),
            prompt: r#"你是一位资深软件架构师，专注于提供高质量的系统架构设计、技术选型与性能优化建议。你具备宏观系统设计视角和微观实现能力，能够帮助用户从 0 到 1 设计架构。

技能模块：
- 架构风格：微服务、单体、分层、Serverless、六边形架构、Clean Architecture、CQRS、事件驱动等
- 技术选型：Spring Cloud、K8s、Kafka、Redis、ClickHouse、ElasticSearch 等
- 通信协议：RESTful、gRPC、WebSocket、GraphQL
- 数据库设计：MySQL/PostgreSQL/MongoDB，分库分表，主从与读写分离
- 系统能力构建：限流、熔断、灰度、容灾、观测、扩展性
- 安全与合规性：权限模型、身份认证（OAuth2.0、OIDC）、数据安全设计
- 可观测性：日志链路追踪、分布式监控、故障演练机制

输出格式：
- 使用 Markdown 标题、列表、代码块组织内容
- 提供设计决策理由，明确技术选型的利弊对比
- 如需图示结构，使用 Mermaid 格式生成架构图"#.to_string(),
            is_system: true,
            enabled: true,
        },
        OptimizationTypeConfig {
            id: "tech-doc".to_string(),
            label: "技术文档".to_string(),
            icon: "i-carbon-document".to_string(),
            description: "生成专业、清晰、易于理解的技术文档".to_string(),
            prompt: r#"你是技术文档工程师，擅长编写清晰、专业、易懂的技术文档。

文档类型支持：
- API 文档：接口概述、请求方法、参数说明、响应格式、错误码、调用示例
- 用户手册：快速开始、功能说明、常见问题
- 开发指南：环境搭建、核心概念、最佳实践、代码示例
- 架构设计文档：系统概述、架构图、技术选型、核心流程、数据模型
- 运维手册：部署流程、配置说明、监控告警、故障排查

文档规范：
- 使用 Markdown 格式，结构清晰
- 代码块带语言标识
- 适当使用表格和图表
- 统一术语，版本更新记录

输出专业、规范的技术文档。"#.to_string(),
            is_system: true,
            enabled: true,
        },
    ]
}


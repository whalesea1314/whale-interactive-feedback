// 主题色类型
export type ThemeColor = 'blue' | 'mint' | 'rose' | 'peach' | 'lavender' | 'slate'

// 字体大小配置
export interface FontSizeConfig {
  display: number    // 提示区文字大小
  options: number    // 选项区文字大小
  input: number      // 输入框文字大小
}

// 应用配置类型
export interface AppConfig {
  theme: 'dark' | 'light'
  themeColor: ThemeColor
  fontSize: FontSizeConfig
  layout: 'vertical' | 'horizontal'
  displayMode: 'simple' | 'full'
  audioEnabled: boolean
  audioFile?: string
  windowPinned: boolean
  autoMinimize: boolean
  splitterPosition: number
  apiKeys: Record<string, string>
  apiTestStatus: Record<string, boolean>
  providerOrder: string[]  // API 提供商优先级顺序
  selectedProvider: string
  optimizePrompt: string
  enhancePrompt: string
  // 自定义选项功能
  customOptionsEnabled: boolean
  customOptions: string[]
  // 文本优化类型配置
  optimizationTypes: OptimizationTypeConfig[]
}

// 反馈内容类型
export type FeedbackContentType = 'text' | 'image' | 'file_reference'

export interface TextContent {
  type: 'text'
  text: string
}

export interface ImageContent {
  type: 'image'
  data: string // Base64 encoded
  mimeType: string
}

export interface FileReferenceContent {
  type: 'file_reference'
  display_name: string
  path: string
}

export type FeedbackContent = TextContent | ImageContent | FileReferenceContent

export interface FeedbackData {
  content: FeedbackContent[]
}

// MCP 工具调用类型
export interface McpToolCall {
  name: string
  arguments: {
    message?: string
    full_response?: string
    predefined_options?: string[]
  }
}

// 常用语类型
export interface CannedResponse {
  id: string
  text: string
  order: number
  starred?: boolean
}

// 图片预览类型
export interface ImagePreviewData {
  id: string
  data: string // Base64
  mimeType: string
  width: number
  height: number
  size: number
}

// 文件引用类型
export interface FileReference {
  id: string
  name?: string           // 原始文件名
  displayName: string     // 显示名称（带图标）
  path: string
  size?: number           // 文件大小
  mimeType?: string       // MIME 类型
  isImage: boolean
  isDirectory?: boolean   // 是否为文件夹
}

// 文本优化类型配置
export interface OptimizationTypeConfig {
  id: string                // 唯一标识
  label: string             // 显示名称
  icon: string              // 图标类名
  description: string       // 简短描述
  prompt: string            // 提示词模板
  isSystem: boolean         // 是否系统预设（不可删除）
  enabled: boolean          // 是否启用
}

// 系统预设的优化类型 ID
export type SystemOptimizationType = 'improve' | 'formal' | 'casual' | 'concise' | 'expand'

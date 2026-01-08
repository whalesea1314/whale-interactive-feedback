/**
 * 跨平台支持 composable
 * 
 * 处理平台特定的快捷键和路径
 * Requirements: 15.1, 15.2, 15.3, 15.4
 */

import { computed, ref, onMounted } from 'vue'

// 平台类型
export type Platform = 'macos' | 'windows' | 'linux' | 'unknown'

// 修饰键类型
export type ModifierKey = 'ctrl' | 'cmd' | 'alt' | 'shift' | 'meta'

// 快捷键配置
export interface ShortcutConfig {
  key: string
  modifiers: ModifierKey[]
  description?: string
}

// 平台特定的路径分隔符
export const PATH_SEPARATORS: Record<Platform, string> = {
  macos: '/',
  windows: '\\',
  linux: '/',
  unknown: '/'
}

// 平台特定的主修饰键（用于复制、粘贴等操作）
export const PRIMARY_MODIFIER: Record<Platform, ModifierKey> = {
  macos: 'cmd',
  windows: 'ctrl',
  linux: 'ctrl',
  unknown: 'ctrl'
}

/**
 * 检测当前运行平台
 * @returns 平台类型
 */
export function detectPlatform(): Platform {
  if (typeof navigator === 'undefined') {
    return 'unknown'
  }

  const userAgent = navigator.userAgent.toLowerCase()
  const platform = navigator.platform?.toLowerCase() || ''

  // 检测 macOS
  if (platform.includes('mac') || userAgent.includes('macintosh') || userAgent.includes('mac os')) {
    return 'macos'
  }

  // 检测 Windows
  if (platform.includes('win') || userAgent.includes('windows')) {
    return 'windows'
  }

  // 检测 Linux
  if (platform.includes('linux') || userAgent.includes('linux')) {
    return 'linux'
  }

  return 'unknown'
}

/**
 * 获取平台特定的路径分隔符
 * Requirement 15.4: 使用平台适当的路径分隔符
 * @param platform 平台类型
 * @returns 路径分隔符
 */
export function getPathSeparator(platform: Platform = detectPlatform()): string {
  return PATH_SEPARATORS[platform]
}

/**
 * 规范化路径为当前平台格式
 * Requirement 15.4: 处理平台特定路径
 * @param path 原始路径
 * @param targetPlatform 目标平台（默认为当前平台）
 * @returns 规范化后的路径
 */
export function normalizePath(path: string, targetPlatform: Platform = detectPlatform()): string {
  if (!path) return path

  const separator = getPathSeparator(targetPlatform)
  
  // 将所有路径分隔符统一替换为目标平台的分隔符
  // 先将所有分隔符统一为 /，然后再转换为目标分隔符
  let normalized = path.replace(/[\\/]+/g, '/')
  
  if (separator === '\\') {
    // Windows: 将 / 转换为 \
    normalized = normalized.replace(/\//g, '\\')
  }

  return normalized
}

/**
 * 连接路径片段
 * Requirement 15.4: 使用平台适当的路径分隔符
 * @param parts 路径片段
 * @param platform 目标平台（默认为当前平台）
 * @returns 连接后的路径
 */
export function joinPath(parts: string[], platform: Platform = detectPlatform()): string {
  if (parts.length === 0) return ''
  
  const separator = getPathSeparator(platform)
  
  // 过滤空字符串并连接
  const filteredParts = parts.filter(p => p.length > 0)
  
  // 规范化每个部分，移除首尾的分隔符
  const normalizedParts = filteredParts.map((part, index) => {
    let normalized = part.replace(/[\\/]+/g, separator)
    
    // 保留第一个部分的开头分隔符（如 / 或 C:\）
    if (index > 0) {
      normalized = normalized.replace(new RegExp(`^[${separator.replace('\\', '\\\\')}]+`), '')
    }
    
    // 移除结尾的分隔符
    normalized = normalized.replace(new RegExp(`[${separator.replace('\\', '\\\\')}]+$`), '')
    
    return normalized
  })

  return normalizedParts.join(separator)
}

/**
 * 获取平台特定的主修饰键
 * Requirements: 15.1, 15.2, 15.3
 * @param platform 平台类型
 * @returns 主修饰键
 */
export function getPrimaryModifier(platform: Platform = detectPlatform()): ModifierKey {
  return PRIMARY_MODIFIER[platform]
}

/**
 * 检查键盘事件是否匹配平台特定的粘贴快捷键
 * Requirements: 15.1 (macOS: Cmd+V), 15.2 (Windows: Ctrl+V), 15.3 (Linux: Ctrl+V)
 * @param event 键盘事件
 * @param platform 平台类型（默认为当前平台）
 * @returns 是否匹配粘贴快捷键
 */
export function isPasteShortcut(event: KeyboardEvent, platform: Platform = detectPlatform()): boolean {
  const key = event.key.toLowerCase()
  
  if (key !== 'v') return false

  // macOS 使用 Cmd+V (metaKey)
  // Windows/Linux 使用 Ctrl+V (ctrlKey)
  if (platform === 'macos') {
    return event.metaKey && !event.ctrlKey && !event.altKey
  } else {
    return event.ctrlKey && !event.metaKey && !event.altKey
  }
}

/**
 * 检查键盘事件是否匹配平台特定的复制快捷键
 * @param event 键盘事件
 * @param platform 平台类型（默认为当前平台）
 * @returns 是否匹配复制快捷键
 */
export function isCopyShortcut(event: KeyboardEvent, platform: Platform = detectPlatform()): boolean {
  const key = event.key.toLowerCase()
  
  if (key !== 'c') return false

  if (platform === 'macos') {
    return event.metaKey && !event.ctrlKey && !event.altKey
  } else {
    return event.ctrlKey && !event.metaKey && !event.altKey
  }
}

/**
 * 检查键盘事件是否匹配平台特定的撤销快捷键
 * Requirement 6.5: Ctrl+Z / Cmd+Z 恢复原始文本
 * @param event 键盘事件
 * @param platform 平台类型（默认为当前平台）
 * @returns 是否匹配撤销快捷键
 */
export function isUndoShortcut(event: KeyboardEvent, platform: Platform = detectPlatform()): boolean {
  const key = event.key.toLowerCase()
  
  if (key !== 'z') return false

  if (platform === 'macos') {
    return event.metaKey && !event.ctrlKey && !event.altKey && !event.shiftKey
  } else {
    return event.ctrlKey && !event.metaKey && !event.altKey && !event.shiftKey
  }
}

/**
 * 检查键盘事件是否匹配平台特定的重做快捷键
 * @param event 键盘事件
 * @param platform 平台类型（默认为当前平台）
 * @returns 是否匹配重做快捷键
 */
export function isRedoShortcut(event: KeyboardEvent, platform: Platform = detectPlatform()): boolean {
  const key = event.key.toLowerCase()
  
  if (platform === 'macos') {
    // macOS: Cmd+Shift+Z
    return key === 'z' && event.metaKey && event.shiftKey && !event.ctrlKey && !event.altKey
  } else {
    // Windows/Linux: Ctrl+Y 或 Ctrl+Shift+Z
    if (key === 'y' && event.ctrlKey && !event.metaKey && !event.altKey && !event.shiftKey) {
      return true
    }
    return key === 'z' && event.ctrlKey && event.shiftKey && !event.metaKey && !event.altKey
  }
}

/**
 * 检查键盘事件是否匹配平台特定的全选快捷键
 * @param event 键盘事件
 * @param platform 平台类型（默认为当前平台）
 * @returns 是否匹配全选快捷键
 */
export function isSelectAllShortcut(event: KeyboardEvent, platform: Platform = detectPlatform()): boolean {
  const key = event.key.toLowerCase()
  
  if (key !== 'a') return false

  if (platform === 'macos') {
    return event.metaKey && !event.ctrlKey && !event.altKey
  } else {
    return event.ctrlKey && !event.metaKey && !event.altKey
  }
}

/**
 * 获取快捷键的显示文本
 * @param shortcut 快捷键配置
 * @param platform 平台类型（默认为当前平台）
 * @returns 显示文本（如 "Cmd+V" 或 "Ctrl+V"）
 */
export function formatShortcut(shortcut: ShortcutConfig, platform: Platform = detectPlatform()): string {
  const modifierSymbols: Record<Platform, Record<ModifierKey, string>> = {
    macos: {
      ctrl: '⌃',
      cmd: '⌘',
      alt: '⌥',
      shift: '⇧',
      meta: '⌘'
    },
    windows: {
      ctrl: 'Ctrl',
      cmd: 'Win',
      alt: 'Alt',
      shift: 'Shift',
      meta: 'Win'
    },
    linux: {
      ctrl: 'Ctrl',
      cmd: 'Super',
      alt: 'Alt',
      shift: 'Shift',
      meta: 'Super'
    },
    unknown: {
      ctrl: 'Ctrl',
      cmd: 'Cmd',
      alt: 'Alt',
      shift: 'Shift',
      meta: 'Meta'
    }
  }

  const symbols = modifierSymbols[platform]
  const separator = platform === 'macos' ? '' : '+'
  
  const modifierStr = shortcut.modifiers
    .map(mod => symbols[mod])
    .join(separator)
  
  const keyStr = shortcut.key.length === 1 ? shortcut.key.toUpperCase() : shortcut.key
  
  return modifierStr + (separator || '') + keyStr
}

/**
 * 获取粘贴快捷键的显示文本
 * @param platform 平台类型（默认为当前平台）
 * @returns 显示文本
 */
export function getPasteShortcutText(platform: Platform = detectPlatform()): string {
  const modifier = platform === 'macos' ? 'cmd' : 'ctrl'
  return formatShortcut({ key: 'V', modifiers: [modifier] }, platform)
}

/**
 * 获取撤销快捷键的显示文本
 * @param platform 平台类型（默认为当前平台）
 * @returns 显示文本
 */
export function getUndoShortcutText(platform: Platform = detectPlatform()): string {
  const modifier = platform === 'macos' ? 'cmd' : 'ctrl'
  return formatShortcut({ key: 'Z', modifiers: [modifier] }, platform)
}

/**
 * 跨平台支持 composable
 * 提供响应式的平台信息和快捷键处理
 */
export function usePlatform() {
  const platform = ref<Platform>('unknown')
  const isInitialized = ref(false)

  // 在组件挂载时检测平台
  onMounted(() => {
    platform.value = detectPlatform()
    isInitialized.value = true
  })

  // 计算属性
  const isMacOS = computed(() => platform.value === 'macos')
  const isWindows = computed(() => platform.value === 'windows')
  const isLinux = computed(() => platform.value === 'linux')
  
  const pathSeparator = computed(() => getPathSeparator(platform.value))
  const primaryModifier = computed(() => getPrimaryModifier(platform.value))
  
  const pasteShortcutText = computed(() => getPasteShortcutText(platform.value))
  const undoShortcutText = computed(() => getUndoShortcutText(platform.value))

  return {
    // 状态
    platform,
    isInitialized,
    
    // 平台检测
    isMacOS,
    isWindows,
    isLinux,
    
    // 路径处理
    pathSeparator,
    normalizePath: (path: string) => normalizePath(path, platform.value),
    joinPath: (parts: string[]) => joinPath(parts, platform.value),
    
    // 快捷键
    primaryModifier,
    pasteShortcutText,
    undoShortcutText,
    
    // 快捷键检测方法
    isPasteShortcut: (event: KeyboardEvent) => isPasteShortcut(event, platform.value),
    isCopyShortcut: (event: KeyboardEvent) => isCopyShortcut(event, platform.value),
    isUndoShortcut: (event: KeyboardEvent) => isUndoShortcut(event, platform.value),
    isRedoShortcut: (event: KeyboardEvent) => isRedoShortcut(event, platform.value),
    isSelectAllShortcut: (event: KeyboardEvent) => isSelectAllShortcut(event, platform.value),
    
    // 格式化
    formatShortcut: (shortcut: ShortcutConfig) => formatShortcut(shortcut, platform.value)
  }
}

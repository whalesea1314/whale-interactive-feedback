import { computed, watch, onMounted, getCurrentInstance } from 'vue'
import { useConfigStore } from '@/stores/config'
import type { ThemeColor } from '@/types'

/**
 * 主题色配置 - 淡雅好看的色卡
 */
export const THEME_COLORS: { id: ThemeColor; name: string; color: string }[] = [
  { id: 'blue', name: '天空蓝', color: '#60A5FA' },
  { id: 'mint', name: '薄荷绿', color: '#34D399' },
  { id: 'rose', name: '樱花粉', color: '#F472B6' },
  { id: 'peach', name: '蜜桃橙', color: '#FB923C' },
  { id: 'lavender', name: '薰衣紫', color: '#A78BFA' },
  { id: 'slate', name: '石墨灰', color: '#94A3B8' },
]

/**
 * 主题管理 Composable
 * 
 * Requirements:
 * - 10.2: 主题更改时立即应用新主题颜色
 * - 10.3: 所有对话框和弹窗适应当前主题
 * - 10.4: 应用启动时应用之前保存的主题偏好
 * - 10.5: 如果没有保存主题偏好，默认使用深色主题
 */
export function useTheme() {
  const configStore = useConfigStore()

  const theme = computed(() => configStore.theme)
  const themeColor = computed(() => configStore.themeColor)
  const isDark = computed(() => theme.value === 'dark')
  const isLight = computed(() => theme.value === 'light')

  /**
   * 切换主题
   * Requirement 10.2: 主题更改时立即应用
   */
  function toggleTheme(): void {
    const newTheme = isDark.value ? 'light' : 'dark'
    setTheme(newTheme)
  }

  /**
   * 设置主题
   * Requirement 10.2: 主题更改时立即应用
   */
  function setTheme(newTheme: 'dark' | 'light'): void {
    configStore.setTheme(newTheme)
  }

  /**
   * 设置主题色
   */
  function setThemeColor(newColor: ThemeColor): void {
    configStore.setThemeColor(newColor)
  }

  /**
   * 应用主题到 DOM
   * Requirement 10.2, 10.3: 立即应用主题到所有 UI 组件
   */
  function applyTheme(themeName: 'dark' | 'light', colorName?: ThemeColor): void {
    const root = document.documentElement
    
    // 移除旧主题类
    root.classList.remove('dark', 'light')
    THEME_COLORS.forEach(c => root.classList.remove(`theme-${c.id}`))
    
    // 添加新主题类
    root.classList.add(themeName)
    root.classList.add(`theme-${colorName || themeColor.value || 'blue'}`)
    
    // 同时更新 body 以确保所有组件都能获取主题
    document.body.classList.remove('dark', 'light')
    THEME_COLORS.forEach(c => document.body.classList.remove(`theme-${c.id}`))
    document.body.classList.add(themeName)
    document.body.classList.add(`theme-${colorName || themeColor.value || 'blue'}`)
    
    // 更新 meta theme-color 以适配系统 UI
    const metaThemeColor = document.querySelector('meta[name="theme-color"]')
    if (metaThemeColor) {
      metaThemeColor.setAttribute(
        'content',
        themeName === 'dark' ? '#1a1a2e' : '#ffffff'
      )
    }
  }

  /**
   * 获取当前主题的 CSS 变量值
   * 用于需要在 JS 中访问主题颜色的场景
   */
  function getCssVariable(variableName: string): string {
    return getComputedStyle(document.documentElement)
      .getPropertyValue(variableName)
      .trim()
  }

  /**
   * 获取主题相关的颜色对象
   * 便于在组件中使用
   */
  const themeColors = computed(() => ({
    bgPrimary: getCssVariable('--bg-primary'),
    bgSecondary: getCssVariable('--bg-secondary'),
    textPrimary: getCssVariable('--text-primary'),
    textSecondary: getCssVariable('--text-secondary'),
    accentColor: getCssVariable('--accent-color'),
    borderColor: getCssVariable('--border-color'),
  }))

  // 监听主题变化，立即更新 DOM
  // Requirement 10.2: 主题更改时立即应用
  watch(theme, (newTheme) => {
    applyTheme(newTheme, themeColor.value)
  }, { immediate: true })

  // 监听主题色变化
  watch(themeColor, (newColor) => {
    applyTheme(theme.value, newColor)
  })

  // 组件挂载时确保主题已应用
  // Requirement 10.4: 应用启动时应用之前保存的主题偏好
  // 只在组件上下文中注册 onMounted
  if (getCurrentInstance()) {
    onMounted(() => {
      // 确保主题类已应用
      if (theme.value) {
        applyTheme(theme.value, themeColor.value)
      }
    })
  }

  return {
    // 状态
    theme,
    themeColor,
    isDark,
    isLight,
    themeColors,
    // 方法
    toggleTheme,
    setTheme,
    setThemeColor,
    applyTheme,
    getCssVariable,
  }
}

/**
 * 主题常量
 */
export const THEMES = {
  DARK: 'dark' as const,
  LIGHT: 'light' as const,
}

/**
 * 默认主题
 * Requirement 10.5: 默认使用深色主题
 */
export const DEFAULT_THEME = THEMES.DARK

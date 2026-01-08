import { ref, computed, onMounted, onUnmounted } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { useConfigStore } from '@/stores/config'

/**
 * 窗口控制 Composable
 * 
 * 提供窗口固定（置顶）和自动最小化功能
 * Requirements: 13.1, 13.2, 13.3, 13.4, 13.5
 */
export function useWindowControl() {
  const configStore = useConfigStore()
  
  // 本地状态
  const isAlwaysOnTop = ref(false)
  const isMinimized = ref(false)
  const isInitialized = ref(false)
  
  // 计算属性 - 从配置获取
  const windowPinned = computed(() => configStore.windowPinned)
  const autoMinimize = computed(() => configStore.autoMinimize)
  
  /**
   * 设置窗口置顶状态
   * Requirement 13.1, 13.3: 窗口固定功能
   */
  async function setAlwaysOnTop(value: boolean): Promise<void> {
    try {
      const window = getCurrentWindow()
      await window.setAlwaysOnTop(value)
      isAlwaysOnTop.value = value
      
      // 保存到配置
      configStore.setWindowPinned(value)
    } catch (error) {
      console.error('Failed to set always on top:', error)
    }
  }
  
  /**
   * 切换窗口置顶状态
   */
  async function toggleAlwaysOnTop(): Promise<void> {
    await setAlwaysOnTop(!isAlwaysOnTop.value)
  }
  
  /**
   * 最小化窗口
   * Requirement 13.2, 13.4: 自动最小化功能
   */
  async function minimizeWindow(): Promise<void> {
    try {
      const window = getCurrentWindow()
      await window.minimize()
      isMinimized.value = true
    } catch (error) {
      console.error('Failed to minimize window:', error)
    }
  }
  
  /**
   * 恢复窗口
   */
  async function unminimizeWindow(): Promise<void> {
    try {
      const window = getCurrentWindow()
      await window.unminimize()
      isMinimized.value = false
    } catch (error) {
      console.error('Failed to unminimize window:', error)
    }
  }
  
  /**
   * 设置自动最小化
   * Requirement 13.2: 允许启用提交后自动最小化
   */
  function setAutoMinimize(value: boolean): void {
    configStore.setAutoMinimize(value)
  }
  
  /**
   * 提交后处理
   * Requirement 13.4: 启用自动最小化时，提交后最小化窗口
   */
  async function handleAfterSubmit(): Promise<void> {
    if (autoMinimize.value) {
      await minimizeWindow()
    }
  }
  
  /**
   * 初始化窗口状态
   * Requirement 13.5: 启动时恢复窗口控制设置
   */
  async function initWindowState(): Promise<void> {
    if (isInitialized.value) return
    
    try {
      // 等待配置加载完成
      if (!configStore.isInitialized) {
        await configStore.loadConfig()
      }
      
      // 恢复窗口置顶状态
      if (configStore.windowPinned) {
        await setAlwaysOnTop(true)
      }
      
      // 获取当前窗口状态
      const window = getCurrentWindow()
      isAlwaysOnTop.value = await window.isAlwaysOnTop()
      
      isInitialized.value = true
    } catch (error) {
      console.error('Failed to initialize window state:', error)
    }
  }
  
  /**
   * 同步窗口状态与配置
   */
  async function syncWithConfig(): Promise<void> {
    try {
      const window = getCurrentWindow()
      const currentAlwaysOnTop = await window.isAlwaysOnTop()
      
      // 如果配置与实际状态不一致，以配置为准
      if (currentAlwaysOnTop !== configStore.windowPinned) {
        await window.setAlwaysOnTop(configStore.windowPinned)
        isAlwaysOnTop.value = configStore.windowPinned
      }
    } catch (error) {
      console.error('Failed to sync window state:', error)
    }
  }
  
  return {
    // 状态
    isAlwaysOnTop,
    isMinimized,
    isInitialized,
    
    // 计算属性
    windowPinned,
    autoMinimize,
    
    // 方法
    setAlwaysOnTop,
    toggleAlwaysOnTop,
    minimizeWindow,
    unminimizeWindow,
    setAutoMinimize,
    handleAfterSubmit,
    initWindowState,
    syncWithConfig,
  }
}

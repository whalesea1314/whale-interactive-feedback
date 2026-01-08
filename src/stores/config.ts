import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { AppConfig, ThemeColor, FontSizeConfig, OptimizationTypeConfig } from '@/types'

// 默认配置 - optimization_types 从后端获取
const defaultConfig: AppConfig = {
  theme: 'dark',
  themeColor: 'blue',
  fontSize: {
    display: 13,
    options: 13,
    input: 13,
  },
  layout: 'horizontal',   // 默认水平布局
  displayMode: 'full',    // 默认完整模式
  audioEnabled: true,
  audioFile: undefined,
  windowPinned: false,
  autoMinimize: false,
  splitterPosition: 50,
  apiKeys: {},
  apiTestStatus: {},
  providerOrder: [],
  selectedProvider: 'openai',
  optimizePrompt: '',
  enhancePrompt: '',
  // 自定义选项
  customOptionsEnabled: false,
  customOptions: ['好的，我明白了', '请继续', '需要更多信息', '返回上一步', '暂停，让我思考一下'],
  // 文本优化类型 - 从后端获取，这里设为空数组
  optimizationTypes: [],
}

/**
 * 配置 Store
 * 负责管理应用配置的加载、保存和状态管理
 * Requirements: 14.1, 14.2
 */
export const useConfigStore = defineStore('config', () => {
  // 状态
  const config = ref<AppConfig>({ ...defaultConfig })
  const isLoading = ref(false)
  const error = ref<string | null>(null)
  const isInitialized = ref(false)
  
  // API 密钥相关状态 (Requirements: 7.5, 14.5)
  const configuredProviders = ref<string[]>([])
  const maskedApiKeys = ref<Record<string, string>>({})
  const testedProviders = ref<Record<string, boolean>>({}) // 测试通过的提供商
  const providerOrder = ref<string[]>([]) // 提供商优先级顺序

  // 计算属性
  const theme = computed(() => config.value.theme)
  const themeColor = computed(() => config.value.themeColor)
  const fontSize = computed(() => config.value.fontSize)
  const layout = computed(() => config.value.layout)
  const displayMode = computed(() => config.value.displayMode)
  const audioEnabled = computed(() => config.value.audioEnabled)
  const windowPinned = computed(() => config.value.windowPinned)
  const autoMinimize = computed(() => config.value.autoMinimize)
  const splitterPosition = computed(() => config.value.splitterPosition)
  const selectedProvider = computed(() => config.value.selectedProvider)
  const customOptionsEnabled = computed(() => config.value.customOptionsEnabled)
  const customOptions = computed(() => config.value.customOptions)
  const optimizationTypes = computed(() => config.value.optimizationTypes)
  const enabledOptimizationTypes = computed(() => 
    config.value.optimizationTypes.filter(t => t.enabled)
  )

  /**
   * 加载配置
   * Requirement 14.2: 应用启动时从配置文件加载所有保存的设置
   */
  async function loadConfig(): Promise<void> {
    isLoading.value = true
    error.value = null
    try {
      const loadedConfig = await invoke<AppConfig>('get_config')
      config.value = { ...defaultConfig, ...loadedConfig }
      isInitialized.value = true
      
      // 加载 API 密钥状态 (Requirements: 7.5, 14.5)
      await refreshApiKeyStatus()
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      console.error('Failed to load config:', e)
      // 使用默认配置
      config.value = { ...defaultConfig }
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 保存配置
   * Requirement 14.1: 设置更改时立即持久化到本地配置文件
   */
  async function saveConfig(): Promise<void> {
    isLoading.value = true
    error.value = null
    try {
      await invoke('save_config', { config: config.value })
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      console.error('Failed to save config:', e)
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 设置主题
   */
  function setTheme(newTheme: 'dark' | 'light'): void {
    config.value.theme = newTheme
    saveConfig()
  }

  /**
   * 设置主题色
   */
  function setThemeColor(newColor: ThemeColor): void {
    config.value.themeColor = newColor
    saveConfig()
  }

  /**
   * 设置字体大小
   */
  function setFontSize(area: keyof FontSizeConfig, size: number): void {
    config.value.fontSize[area] = size
    saveConfig()
  }

  /**
   * 设置布局
   */
  function setLayout(newLayout: 'vertical' | 'horizontal'): void {
    config.value.layout = newLayout
    saveConfig()
  }

  /**
   * 设置显示模式
   */
  function setDisplayMode(newMode: 'simple' | 'full'): void {
    config.value.displayMode = newMode
    saveConfig()
  }

  /**
   * 设置分割器位置
   */
  function setSplitterPosition(position: number): void {
    config.value.splitterPosition = position
    saveConfig()
  }

  /**
   * 设置音频启用状态
   */
  function setAudioEnabled(enabled: boolean): void {
    config.value.audioEnabled = enabled
    saveConfig()
  }

  /**
   * 设置音频文件路径
   */
  function setAudioFile(path: string | undefined): void {
    config.value.audioFile = path
    saveConfig()
  }

  /**
   * 设置窗口固定状态
   */
  async function setWindowPinned(pinned: boolean): Promise<void> {
    config.value.windowPinned = pinned
    await saveConfig()
  }

  /**
   * 设置自动最小化
   */
  function setAutoMinimize(minimize: boolean): void {
    config.value.autoMinimize = minimize
    saveConfig()
  }

  // ============================================================================
  // API 密钥管理方法 (Requirements: 7.5, 14.5)
  // 使用系统密钥链安全存储
  // ============================================================================

  /**
   * 刷新 API 密钥状态
   * 获取已配置的提供商列表和掩码密钥
   */
  async function refreshApiKeyStatus(): Promise<void> {
    try {
      // 获取已配置的提供商
      const providers = await invoke<string[]>('get_configured_providers')
      console.log('[ConfigStore] get_configured_providers returned:', providers)
      configuredProviders.value = providers
      
      // 获取优先级顺序
      const order = await invoke<string[]>('get_provider_order')
      // 过滤掉未配置的提供商
      providerOrder.value = order.filter(p => providers.includes(p))
      // 添加新配置但不在顺序中的提供商到末尾
      for (const p of providers) {
        if (!providerOrder.value.includes(p)) {
          providerOrder.value.push(p)
        }
      }
      console.log('[ConfigStore] providerOrder:', providerOrder.value)
      
      // 只获取已配置提供商的掩码密钥和测试状态
      const newMaskedKeys: Record<string, string> = {}
      const newTestedProviders: Record<string, boolean> = {}
      
      for (const provider of providers) {
        // 获取掩码密钥
        const masked = await invoke<string | null>('get_masked_api_key', { provider })
        console.log(`[ConfigStore] get_masked_api_key(${provider}) returned:`, masked)
        if (masked) {
          newMaskedKeys[provider] = masked
        }
        
        // 获取测试状态
        const tested = await invoke<boolean>('get_api_test_status', { provider })
        if (tested) {
          newTestedProviders[provider] = true
        }
      }
      
      // 批量更新
      maskedApiKeys.value = newMaskedKeys
      testedProviders.value = newTestedProviders
      console.log('[ConfigStore] Final state - configuredProviders:', configuredProviders.value, 'maskedApiKeys:', maskedApiKeys.value, 'testedProviders:', testedProviders.value)
    } catch (e) {
      console.error('Failed to refresh API key status:', e)
    }
  }

  /**
   * 保存 API 密钥到系统密钥链
   * Requirement 7.5, 14.5: 使用适当的安全措施存储敏感数据
   */
  async function setApiKey(provider: string, key: string): Promise<void> {
    try {
      console.log(`[ConfigStore] Saving API key for ${provider}...`)
      await invoke('save_api_key', { provider, apiKey: key })
      console.log(`[ConfigStore] API key saved, refreshing status...`)
      await refreshApiKeyStatus()
      console.log(`[ConfigStore] Status refreshed, configuredProviders:`, configuredProviders.value)
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      console.error('Failed to save API key:', e)
      throw e
    }
  }

  /**
   * 获取 API 密钥
   * 从系统密钥链获取
   */
  async function getApiKey(provider: string): Promise<string | null> {
    try {
      return await invoke<string | null>('get_api_key', { provider })
    } catch (e) {
      console.error('Failed to get API key:', e)
      return null
    }
  }

  /**
   * 删除 API 密钥
   */
  async function deleteApiKey(provider: string): Promise<void> {
    try {
      await invoke('delete_api_key', { provider })
      await refreshApiKeyStatus()
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      console.error('Failed to delete API key:', e)
      throw e
    }
  }

  /**
   * 检查提供商是否已配置 API 密钥
   */
  function hasApiKey(provider: string): boolean {
    return configuredProviders.value.includes(provider)
  }

  /**
   * 获取掩码后的 API 密钥（用于 UI 显示）
   */
  function getMaskedApiKey(provider: string): string | undefined {
    return maskedApiKeys.value[provider]
  }

  /**
   * 设置选中的 AI 提供商
   */
  function setSelectedProvider(provider: string): void {
    config.value.selectedProvider = provider
    saveConfig()
  }

  /**
   * 设置优化提示词
   */
  function setOptimizePrompt(prompt: string): void {
    config.value.optimizePrompt = prompt
    saveConfig()
  }

  /**
   * 设置增强提示词
   */
  function setEnhancePrompt(prompt: string): void {
    config.value.enhancePrompt = prompt
    saveConfig()
  }

  /**
   * 设置自定义选项启用状态
   */
  function setCustomOptionsEnabled(enabled: boolean): void {
    config.value.customOptionsEnabled = enabled
    saveConfig()
  }

  /**
   * 设置自定义选项列表
   */
  function setCustomOptions(options: string[]): void {
    config.value.customOptions = options
    saveConfig()
  }

  /**
   * 添加自定义选项
   */
  function addCustomOption(option: string): void {
    if (option.trim() && !config.value.customOptions.includes(option.trim())) {
      config.value.customOptions.push(option.trim())
      saveConfig()
    }
  }

  /**
   * 删除自定义选项
   */
  function removeCustomOption(index: number): void {
    if (index >= 0 && index < config.value.customOptions.length) {
      config.value.customOptions.splice(index, 1)
      saveConfig()
    }
  }

  /**
   * 更新自定义选项
   */
  function updateCustomOption(index: number, value: string): void {
    if (index >= 0 && index < config.value.customOptions.length) {
      config.value.customOptions[index] = value.trim()
      saveConfig()
    }
  }

  // ========== 文本优化类型管理 ==========

  /**
   * 获取优化类型（从后端配置获取）
   */
  function getOptimizationTypes(): OptimizationTypeConfig[] {
    return config.value.optimizationTypes
  }

  /**
   * 添加自定义优化类型
   */
  function addOptimizationType(typeConfig: Omit<OptimizationTypeConfig, 'id' | 'isSystem'>): void {
    const newType: OptimizationTypeConfig = {
      ...typeConfig,
      id: `custom_${Date.now()}_${Math.random().toString(36).slice(2, 11)}`,
      isSystem: false,
    }
    config.value.optimizationTypes.push(newType)
    saveConfig()
  }

  /**
   * 更新优化类型
   */
  function updateOptimizationType(id: string, updates: Partial<OptimizationTypeConfig>): void {
    const index = config.value.optimizationTypes.findIndex(t => t.id === id)
    if (index !== -1) {
      // 系统类型不允许修改 id 和 isSystem
      const { id: _id, isSystem: _isSystem, ...safeUpdates } = updates
      config.value.optimizationTypes[index] = {
        ...config.value.optimizationTypes[index],
        ...safeUpdates,
      }
      saveConfig()
    }
  }

  /**
   * 删除优化类型（仅限自定义类型）
   */
  function removeOptimizationType(id: string): boolean {
    const index = config.value.optimizationTypes.findIndex(t => t.id === id)
    if (index !== -1) {
      const typeConfig = config.value.optimizationTypes[index]
      if (typeConfig.isSystem) {
        console.warn('Cannot remove system optimization type')
        return false
      }
      config.value.optimizationTypes.splice(index, 1)
      saveConfig()
      return true
    }
    return false
  }

  /**
   * 切换优化类型启用状态
   */
  function toggleOptimizationType(id: string): void {
    const index = config.value.optimizationTypes.findIndex(t => t.id === id)
    if (index !== -1) {
      config.value.optimizationTypes[index].enabled = !config.value.optimizationTypes[index].enabled
      saveConfig()
    }
  }

  /**
   * 重置优化类型为系统默认（重新加载后端配置）
   */
  async function resetOptimizationTypes(): Promise<void> {
    try {
      // 从后端重新加载配置获取默认的优化类型
      const loadedConfig = await invoke<AppConfig>('get_config')
      config.value.optimizationTypes = loadedConfig.optimizationTypes || []
      await saveConfig()
    } catch (e) {
      console.error('Failed to reset optimization types:', e)
    }
  }

  /**
   * 获取优化类型的提示词
   */
  function getOptimizationPrompt(id: string, text: string): string {
    const typeConfig = config.value.optimizationTypes.find(t => t.id === id)
    if (!typeConfig) {
      return text
    }
    return typeConfig.prompt.replace('{text}', text)
  }

  /**
   * 重置为默认配置
   */
  async function resetToDefaults(): Promise<void> {
    config.value = { ...defaultConfig }
    await saveConfig()
  }

  /**
   * 设置提供商测试状态（持久化到配置文件）
   */
  async function setProviderTested(provider: string, tested: boolean): Promise<void> {
    try {
      await invoke('set_api_test_status', { provider, tested })
      // 更新本地状态
      testedProviders.value[provider] = tested
      if (!tested) {
        delete testedProviders.value[provider]
      }
    } catch (e) {
      console.error('Failed to set API test status:', e)
    }
  }

  /**
   * 检查提供商是否已测试通过
   */
  function isProviderTested(provider: string): boolean {
    return testedProviders.value[provider] === true
  }

  /**
   * 获取提供商的优先级（1 为最高）
   */
  function getProviderPriority(provider: string): number {
    const index = providerOrder.value.indexOf(provider)
    return index >= 0 ? index + 1 : configuredProviders.value.length
  }

  /**
   * 设置提供商优先级
   * @param provider 提供商 ID
   * @param priority 新优先级（1 为最高）
   */
  async function setProviderPriority(provider: string, priority: number): Promise<void> {
    const newOrder = [...providerOrder.value]
    const currentIndex = newOrder.indexOf(provider)
    
    // 从当前位置移除
    if (currentIndex >= 0) {
      newOrder.splice(currentIndex, 1)
    }
    
    // 插入到新位置
    const targetIndex = Math.max(0, Math.min(priority - 1, newOrder.length))
    newOrder.splice(targetIndex, 0, provider)
    
    // 保存到后端
    try {
      await invoke('set_provider_order', { order: newOrder })
      providerOrder.value = newOrder
      console.log('[ConfigStore] Provider order updated:', newOrder)
    } catch (e) {
      console.error('Failed to set provider order:', e)
      throw e
    }
  }

  /**
   * 批量更新提供商顺序（用于拖拽排序）
   * @param newOrder 新的提供商顺序数组
   */
  async function setProviderOrder(newOrder: string[]): Promise<void> {
    try {
      await invoke('set_provider_order', { order: newOrder })
      // 使用新数组替换以确保响应式更新
      providerOrder.value = [...newOrder]
      console.log('[ConfigStore] Provider order updated:', providerOrder.value)
    } catch (e) {
      console.error('Failed to set provider order:', e)
      throw e
    }
  }

  return {
    // 状态
    config,
    isLoading,
    error,
    isInitialized,
    configuredProviders,
    maskedApiKeys,
    testedProviders,
    providerOrder,
    // 计算属性
    theme,
    themeColor,
    fontSize,
    layout,
    displayMode,
    audioEnabled,
    windowPinned,
    autoMinimize,
    splitterPosition,
    selectedProvider,
    customOptionsEnabled,
    customOptions,
    optimizationTypes,
    enabledOptimizationTypes,
    // 方法
    loadConfig,
    saveConfig,
    setTheme,
    setThemeColor,
    setFontSize,
    setLayout,
    setDisplayMode,
    setSplitterPosition,
    setAudioEnabled,
    setAudioFile,
    setWindowPinned,
    setAutoMinimize,
    // API 密钥管理 (Requirements: 7.5, 14.5)
    refreshApiKeyStatus,
    setApiKey,
    getApiKey,
    deleteApiKey,
    hasApiKey,
    getMaskedApiKey,
    setSelectedProvider,
    setOptimizePrompt,
    setEnhancePrompt,
    setCustomOptionsEnabled,
    setCustomOptions,
    addCustomOption,
    removeCustomOption,
    updateCustomOption,
    // 优化类型管理
    getOptimizationTypes,
    addOptimizationType,
    updateOptimizationType,
    removeOptimizationType,
    toggleOptimizationType,
    resetOptimizationTypes,
    getOptimizationPrompt,
    setProviderTested,
    isProviderTested,
    getProviderPriority,
    setProviderPriority,
    setProviderOrder,
    resetToDefaults,
  }
})

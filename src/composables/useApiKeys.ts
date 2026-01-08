/**
 * API 密钥管理 Composable
 * 
 * 提供安全的 API 密钥存储和管理功能
 * 使用操作系统原生密钥链进行安全存储
 * 
 * Requirements: 7.5, 14.5
 */

import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

// 支持的 AI 提供商
export type ApiProvider = 'openai' | 'gemini' | 'deepseek' | 'volcengine'

// 提供商配置信息
export interface ProviderInfo {
  id: ApiProvider
  name: string
  description: string
  placeholder: string
}

// 所有支持的提供商
export const PROVIDERS: ProviderInfo[] = [
  {
    id: 'openai',
    name: 'OpenAI',
    description: 'GPT-4, GPT-3.5 等模型',
    placeholder: 'sk-...',
  },
  {
    id: 'gemini',
    name: 'Google Gemini',
    description: 'Gemini Pro, Gemini Ultra 等模型',
    placeholder: 'AIza...',
  },
  {
    id: 'deepseek',
    name: 'DeepSeek',
    description: 'DeepSeek Chat, DeepSeek Coder 等模型',
    placeholder: 'sk-...',
  },
  {
    id: 'volcengine',
    name: '火山引擎',
    description: '豆包大模型等',
    placeholder: 'your-api-key',
  },
]

/**
 * API 密钥管理 Composable
 */
export function useApiKeys() {
  // 状态
  const isLoading = ref(false)
  const error = ref<string | null>(null)
  const configuredProviders = ref<string[]>([])
  const maskedKeys = ref<Record<string, string>>({})

  /**
   * 保存 API 密钥
   * 
   * @param provider - 提供商 ID
   * @param apiKey - API 密钥
   */
  async function saveApiKey(provider: ApiProvider, apiKey: string): Promise<void> {
    isLoading.value = true
    error.value = null
    
    try {
      await invoke('save_api_key', { provider, apiKey })
      // 刷新配置状态
      await refreshConfiguredProviders()
      await refreshMaskedKey(provider)
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 获取 API 密钥
   * 
   * @param provider - 提供商 ID
   * @returns API 密钥或 null
   */
  async function getApiKey(provider: ApiProvider): Promise<string | null> {
    isLoading.value = true
    error.value = null
    
    try {
      const key = await invoke<string | null>('get_api_key', { provider })
      return key
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 删除 API 密钥
   * 
   * @param provider - 提供商 ID
   */
  async function deleteApiKey(provider: ApiProvider): Promise<void> {
    isLoading.value = true
    error.value = null
    
    try {
      await invoke('delete_api_key', { provider })
      // 刷新配置状态
      await refreshConfiguredProviders()
      delete maskedKeys.value[provider]
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 检查是否存在 API 密钥
   * 
   * @param provider - 提供商 ID
   * @returns 是否存在
   */
  async function hasApiKey(provider: ApiProvider): Promise<boolean> {
    try {
      return await invoke<boolean>('has_api_key', { provider })
    } catch (e) {
      console.error('Failed to check API key:', e)
      return false
    }
  }

  /**
   * 获取已配置的提供商列表
   */
  async function refreshConfiguredProviders(): Promise<void> {
    try {
      configuredProviders.value = await invoke<string[]>('get_configured_providers')
    } catch (e) {
      console.error('Failed to get configured providers:', e)
      configuredProviders.value = []
    }
  }

  /**
   * 获取掩码后的 API 密钥
   * 
   * @param provider - 提供商 ID
   */
  async function refreshMaskedKey(provider: ApiProvider): Promise<void> {
    try {
      const masked = await invoke<string | null>('get_masked_api_key', { provider })
      if (masked) {
        maskedKeys.value[provider] = masked
      } else {
        delete maskedKeys.value[provider]
      }
    } catch (e) {
      console.error('Failed to get masked API key:', e)
    }
  }

  /**
   * 刷新所有掩码密钥
   */
  async function refreshAllMaskedKeys(): Promise<void> {
    for (const provider of PROVIDERS) {
      await refreshMaskedKey(provider.id)
    }
  }

  /**
   * 初始化 - 加载已配置的提供商和掩码密钥
   */
  async function initialize(): Promise<void> {
    await refreshConfiguredProviders()
    await refreshAllMaskedKeys()
  }

  /**
   * 检查提供商是否已配置
   */
  function isProviderConfigured(provider: ApiProvider): boolean {
    return configuredProviders.value.includes(provider)
  }

  /**
   * 获取提供商的掩码密钥
   */
  function getMaskedKey(provider: ApiProvider): string | undefined {
    return maskedKeys.value[provider]
  }

  /**
   * 测试 API 连接
   * 
   * @param provider - 提供商 ID
   * @returns 测试结果消息
   */
  async function testConnection(provider: ApiProvider): Promise<string> {
    isLoading.value = true
    error.value = null
    
    try {
      const result = await invoke<string>('test_api_connection', { provider })
      return result
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : String(e)
      error.value = errorMsg
      throw new Error(errorMsg)
    } finally {
      isLoading.value = false
    }
  }

  return {
    // 状态
    isLoading,
    error,
    configuredProviders,
    maskedKeys,
    
    // 方法
    saveApiKey,
    getApiKey,
    deleteApiKey,
    hasApiKey,
    refreshConfiguredProviders,
    refreshMaskedKey,
    refreshAllMaskedKeys,
    initialize,
    isProviderConfigured,
    getMaskedKey,
    testConnection,
    
    // 常量
    PROVIDERS,
  }
}

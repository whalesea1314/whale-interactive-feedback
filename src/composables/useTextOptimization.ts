import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

// 支持系统预设和自定义类型
export type OptimizationType = string

export interface TextOptimizationResult {
  original: string
  optimized: string
  type: string
  timestamp: Date
}

export interface OptimizationHistoryItem extends TextOptimizationResult {
  id: string
}

export function useTextOptimization() {
  const isOptimizing = ref(false)
  const lastResult = ref<TextOptimizationResult | null>(null)
  const error = ref<string | null>(null)
  const history = ref<OptimizationHistoryItem[]>([])

  /**
   * 优化文本
   * @param text 要优化的文本
   * @param type 优化类型 ID
   */
  async function optimizeText(text: string, type: string): Promise<string> {
    if (!text.trim()) {
      throw new Error('文本不能为空')
    }

    isOptimizing.value = true
    error.value = null

    try {
      // 传递原始文本和类型 ID，后端负责获取提示词模板并替换 {text}
      const result = await invoke<string>('optimize_text', {
        text,
        optimizationType: type
      })

      const optimizationResult: TextOptimizationResult = {
        original: text,
        optimized: result,
        type,
        timestamp: new Date()
      }

      lastResult.value = optimizationResult

      // 添加到历史记录
      addToHistory(optimizationResult)

      return result
    } catch (e) {
      const errorMessage = e instanceof Error ? e.message : '优化失败，请重试'
      error.value = errorMessage
      throw new Error(errorMessage)
    } finally {
      isOptimizing.value = false
    }
  }

  /**
   * 添加到历史记录
   */
  function addToHistory(result: TextOptimizationResult) {
    const historyItem: OptimizationHistoryItem = {
      ...result,
      id: `${Date.now()}-${Math.random().toString(36).substr(2, 9)}`
    }
    
    history.value.unshift(historyItem)
    
    // 保留最近 10 条记录
    if (history.value.length > 10) {
      history.value = history.value.slice(0, 10)
    }
  }

  /**
   * 清除当前结果
   */
  function clearResult() {
    lastResult.value = null
    error.value = null
  }

  /**
   * 清除历史记录
   */
  function clearHistory() {
    history.value = []
  }

  /**
   * 从历史记录中恢复
   */
  function restoreFromHistory(id: string): TextOptimizationResult | null {
    const item = history.value.find(h => h.id === id)
    if (item) {
      lastResult.value = {
        original: item.original,
        optimized: item.optimized,
        type: item.type,
        timestamp: item.timestamp
      }
      return lastResult.value
    }
    return null
  }

  /**
   * 获取优化类型的显示名称
   * 注意：现在优先从 configStore 获取，这里保留作为兜底
   */
  function getTypeLabel(type: string): string {
    const systemLabels: Record<string, string> = {
      improve: '改进',
      formal: '正式',
      casual: '随意',
      concise: '精简',
      expand: '扩展'
    }
    return systemLabels[type] || type
  }

  return {
    isOptimizing,
    lastResult,
    error,
    history,
    optimizeText,
    clearResult,
    clearHistory,
    restoreFromHistory,
    getTypeLabel
  }
}

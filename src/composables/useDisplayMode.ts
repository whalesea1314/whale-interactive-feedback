import { computed, type Ref, type ComputedRef } from 'vue'
import { useConfigStore } from '@/stores/config'

/**
 * 显示模式内容选择参数
 */
export interface DisplayModeParams {
  message?: string
  fullResponse?: string
}

/**
 * 显示模式 Composable 返回类型
 */
export interface UseDisplayModeReturn {
  /** 当前显示模式 */
  displayMode: ComputedRef<'simple' | 'full'>
  /** 根据显示模式选择的内容 */
  displayContent: ComputedRef<string>
  /** 切换显示模式 */
  toggleDisplayMode: () => void
  /** 设置显示模式 */
  setDisplayMode: (mode: 'simple' | 'full') => void
}

/**
 * 根据显示模式选择要显示的内容
 * 
 * 规则：
 * - 简单模式 (simple): 优先显示 message，如果为空则回退到 fullResponse
 * - 完整模式 (full): 优先显示 fullResponse，如果为空则回退到 message
 * 
 * @param mode - 显示模式
 * @param message - 简洁消息
 * @param fullResponse - 完整响应
 * @returns 选择的内容
 * 
 * Requirements: 11.1, 11.2, 11.3
 */
export function selectDisplayContent(
  mode: 'simple' | 'full',
  message?: string,
  fullResponse?: string
): string {
  const trimmedMessage = message?.trim() || ''
  const trimmedFullResponse = fullResponse?.trim() || ''

  if (mode === 'simple') {
    // Requirement 11.1: 简单模式显示 message 参数内容
    // Requirement 11.3: 主参数为空时自动回退到备选参数
    return trimmedMessage || trimmedFullResponse
  } else {
    // Requirement 11.2: 完整模式显示 full_response 参数内容
    // Requirement 11.3: 主参数为空时自动回退到备选参数
    return trimmedFullResponse || trimmedMessage
  }
}

/**
 * 显示模式 Composable
 * 
 * 管理显示模式的切换和内容选择逻辑
 * 
 * Requirements: 2.1, 11.1, 11.2, 11.3, 11.4, 11.5
 * 
 * @param params - 响应式的显示参数
 * @returns 显示模式相关的状态和方法
 */
export function useDisplayMode(
  params: Ref<DisplayModeParams> | ComputedRef<DisplayModeParams>
): UseDisplayModeReturn {
  const configStore = useConfigStore()

  // 当前显示模式
  const displayMode = computed(() => configStore.displayMode)

  // 根据显示模式选择的内容
  // Requirement 2.1: 根据当前显示模式显示 message 或 full_response 内容
  const displayContent = computed(() => {
    return selectDisplayContent(
      displayMode.value,
      params.value.message,
      params.value.fullResponse
    )
  })

  // 切换显示模式
  // Requirement 11.4: 允许实时切换显示模式
  function toggleDisplayMode(): void {
    const newMode = displayMode.value === 'simple' ? 'full' : 'simple'
    configStore.setDisplayMode(newMode)
  }

  // 设置显示模式
  // Requirement 11.5: 显示模式更改时立即反映新模式
  function setDisplayMode(mode: 'simple' | 'full'): void {
    configStore.setDisplayMode(mode)
  }

  return {
    displayMode,
    displayContent,
    toggleDisplayMode,
    setDisplayMode,
  }
}

import { computed, ref, watch, onMounted, getCurrentInstance } from 'vue'
import { useConfigStore } from '@/stores/config'

/**
 * 布局管理 Composable
 * 
 * Requirements:
 * - 9.1: 允许在垂直布局（上下）和水平布局（左右）之间切换
 * - 9.2: 布局模式更改时立即反映新布局
 * - 9.3: 用户拖拽分割器时调整各区域大小
 * - 9.4: 用户双击分割器时重置为默认比例
 * - 9.5: 应用关闭时保存当前布局状态
 * - 9.6: 应用启动时恢复之前保存的布局状态
 */
export function useLayout() {
  const configStore = useConfigStore()

  // 从 store 获取布局状态
  const layout = computed(() => configStore.layout)
  const isVertical = computed(() => layout.value === 'vertical')
  const isHorizontal = computed(() => layout.value === 'horizontal')
  const splitterPosition = computed(() => configStore.config.splitterPosition)

  // 本地拖拽状态
  const isDragging = ref(false)
  const localSplitterPosition = ref(splitterPosition.value)

  // 监听 store 中的 splitterPosition 变化，同步到本地
  // Requirement 9.6: 应用启动时恢复之前保存的布局状态
  watch(splitterPosition, (newPosition) => {
    if (!isDragging.value) {
      localSplitterPosition.value = newPosition
    }
  }, { immediate: true })

  /**
   * 设置布局模式
   * Requirement 9.1, 9.2: 切换布局并立即反映
   */
  function setLayout(newLayout: 'vertical' | 'horizontal'): void {
    configStore.setLayout(newLayout)
  }

  /**
   * 切换布局模式
   * Requirement 9.1, 9.2: 切换布局并立即反映
   */
  function toggleLayout(): void {
    const newLayout = isVertical.value ? 'horizontal' : 'vertical'
    configStore.setLayout(newLayout)
  }

  /**
   * 开始拖拽
   * Requirement 9.3: 用户拖拽分割器时调整各区域大小
   */
  function startDrag(): void {
    isDragging.value = true
    localSplitterPosition.value = splitterPosition.value
  }

  /**
   * 拖拽中更新位置
   * Requirement 9.3: 用户拖拽分割器时调整各区域大小
   */
  function onDrag(position: number): void {
    if (isDragging.value) {
      // 限制范围 20% - 80%
      localSplitterPosition.value = Math.max(20, Math.min(80, position))
    }
  }

  /**
   * 结束拖拽并持久化
   * Requirement 9.5: 应用关闭时保存当前布局状态
   */
  function endDrag(): void {
    if (isDragging.value) {
      isDragging.value = false
      // 持久化分割器位置
      configStore.setSplitterPosition(localSplitterPosition.value)
    }
  }

  /**
   * 重置分割器到默认位置
   * Requirement 9.4: 用户双击分割器时重置为默认比例
   */
  function resetSplitter(): void {
    localSplitterPosition.value = 50
    configStore.setSplitterPosition(50)
  }

  /**
   * 设置分割器位置
   * Requirement 9.5: 保存布局状态
   */
  function setSplitterPosition(position: number): void {
    const clampedPosition = Math.max(20, Math.min(80, position))
    localSplitterPosition.value = clampedPosition
    configStore.setSplitterPosition(clampedPosition)
  }

  /**
   * 获取当前布局状态（用于持久化）
   */
  function getLayoutState(): LayoutState {
    return {
      layout: layout.value,
      splitterPosition: localSplitterPosition.value,
    }
  }

  /**
   * 恢复布局状态
   * Requirement 9.6: 应用启动时恢复之前保存的布局状态
   */
  function restoreLayoutState(state: LayoutState): void {
    if (state.layout) {
      configStore.setLayout(state.layout)
    }
    if (typeof state.splitterPosition === 'number') {
      const clampedPosition = Math.max(20, Math.min(80, state.splitterPosition))
      localSplitterPosition.value = clampedPosition
      configStore.setSplitterPosition(clampedPosition)
    }
  }

  // 组件挂载时确保布局状态已同步
  if (getCurrentInstance()) {
    onMounted(() => {
      localSplitterPosition.value = splitterPosition.value
    })
  }

  return {
    // 状态
    layout,
    isVertical,
    isHorizontal,
    splitterPosition,
    isDragging,
    localSplitterPosition,
    // 方法
    setLayout,
    toggleLayout,
    startDrag,
    onDrag,
    endDrag,
    resetSplitter,
    setSplitterPosition,
    getLayoutState,
    restoreLayoutState,
  }
}

/**
 * 布局状态类型
 */
export interface LayoutState {
  layout: 'vertical' | 'horizontal'
  splitterPosition: number
}

/**
 * 布局常量
 */
export const LAYOUTS = {
  VERTICAL: 'vertical' as const,
  HORIZONTAL: 'horizontal' as const,
}

/**
 * 默认布局
 */
export const DEFAULT_LAYOUT = LAYOUTS.VERTICAL

/**
 * 默认分割器位置
 */
export const DEFAULT_SPLITTER_POSITION = 50

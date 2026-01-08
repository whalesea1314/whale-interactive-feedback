import { ref, computed, type Ref } from 'vue'

export interface SplitterOptions {
  /** 初始位置（百分比） */
  initialPosition?: number
  /** 最小位置（百分比） */
  minPosition?: number
  /** 最大位置（百分比） */
  maxPosition?: number
  /** 是否垂直方向 */
  isVertical?: Ref<boolean> | boolean
  /** 容器元素引用 */
  containerRef?: Ref<HTMLElement | null>
}

export interface SplitterReturn {
  /** 当前位置（百分比） */
  position: Ref<number>
  /** 是否正在拖拽 */
  isDragging: Ref<boolean>
  /** 鼠标按下事件处理 */
  handleMouseDown: (event: MouseEvent) => void
  /** 重置到初始位置 */
  resetPosition: () => void
  /** 设置位置 */
  setPosition: (pos: number) => void
}

/**
 * 分割线拖拽 composable
 * 处理可拖拽分割线的逻辑
 */
export function useSplitter(options: SplitterOptions = {}): SplitterReturn {
  const {
    initialPosition = 50,
    minPosition = 20,
    maxPosition = 80,
    isVertical = false,
    containerRef
  } = options

  const position = ref(initialPosition)
  const isDragging = ref(false)

  // 获取是否垂直方向的值
  const getIsVertical = (): boolean => {
    if (typeof isVertical === 'boolean') return isVertical
    return isVertical.value
  }

  // 鼠标移动处理
  function handleMouseMove(event: MouseEvent) {
    if (!isDragging.value) return
    
    const container = containerRef?.value
    if (!container) return

    const rect = container.getBoundingClientRect()
    let pos: number

    if (getIsVertical()) {
      pos = ((event.clientY - rect.top) / rect.height) * 100
    } else {
      pos = ((event.clientX - rect.left) / rect.width) * 100
    }

    // 限制范围
    position.value = Math.max(minPosition, Math.min(maxPosition, pos))
  }

  // 鼠标释放处理
  function handleMouseUp() {
    isDragging.value = false
    document.removeEventListener('mousemove', handleMouseMove)
    document.removeEventListener('mouseup', handleMouseUp)
  }

  // 鼠标按下处理
  function handleMouseDown(event: MouseEvent) {
    event.preventDefault()
    isDragging.value = true
    document.addEventListener('mousemove', handleMouseMove)
    document.addEventListener('mouseup', handleMouseUp)
  }

  // 重置位置
  function resetPosition() {
    position.value = initialPosition
  }

  // 设置位置
  function setPosition(pos: number) {
    position.value = Math.max(minPosition, Math.min(maxPosition, pos))
  }

  return {
    position,
    isDragging,
    handleMouseDown,
    resetPosition,
    setPosition
  }
}

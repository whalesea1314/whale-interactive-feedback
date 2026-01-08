<script setup lang="ts">
/**
 * LayoutWrapper 组件
 * 
 * 实现垂直/水平布局切换和可拖拽分割器
 * 
 * Requirements:
 * - 9.1: 允许在垂直布局（上下）和水平布局（左右）之间切换
 * - 9.2: 布局模式更改时立即反映新布局
 * - 9.3: 用户拖拽分割器时调整各区域大小
 * - 9.4: 用户双击分割器时重置为默认比例
 */
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useLayout } from '@/composables/useLayout'

const props = defineProps<{
  minSize?: number // 最小尺寸百分比，默认 20
  maxSize?: number // 最大尺寸百分比，默认 80
}>()

const {
  layout,
  isVertical,
  isDragging,
  localSplitterPosition,
  startDrag,
  onDrag,
  endDrag,
  resetSplitter,
} = useLayout()

const containerRef = ref<HTMLElement | null>(null)

// 计算最小和最大尺寸
const minSize = computed(() => props.minSize ?? 20)
const maxSize = computed(() => props.maxSize ?? 80)

// 计算第一个面板的样式
const firstPanelStyle = computed(() => {
  const size = `${localSplitterPosition.value}%`
  return isVertical.value
    ? { height: size, width: '100%' }
    : { width: size, height: '100%' }
})

// 计算第二个面板的样式
const secondPanelStyle = computed(() => {
  const size = `${100 - localSplitterPosition.value}%`
  return isVertical.value
    ? { height: size, width: '100%' }
    : { width: size, height: '100%' }
})

// 处理鼠标按下事件
function handleMouseDown(event: MouseEvent) {
  event.preventDefault()
  startDrag()
  
  document.addEventListener('mousemove', handleMouseMove)
  document.addEventListener('mouseup', handleMouseUp)
}

// 处理鼠标移动事件
function handleMouseMove(event: MouseEvent) {
  if (!isDragging.value || !containerRef.value) return
  
  const rect = containerRef.value.getBoundingClientRect()
  let position: number
  
  if (isVertical.value) {
    // 垂直布局：计算 Y 轴位置
    position = ((event.clientY - rect.top) / rect.height) * 100
  } else {
    // 水平布局：计算 X 轴位置
    position = ((event.clientX - rect.left) / rect.width) * 100
  }
  
  // 限制在最小和最大范围内
  position = Math.max(minSize.value, Math.min(maxSize.value, position))
  onDrag(position)
}

// 处理鼠标释放事件
function handleMouseUp() {
  endDrag()
  document.removeEventListener('mousemove', handleMouseMove)
  document.removeEventListener('mouseup', handleMouseUp)
}

// 处理双击重置
// Requirement 9.4: 用户双击分割器时重置为默认比例
function handleDoubleClick() {
  resetSplitter()
}

// 清理事件监听器
onUnmounted(() => {
  document.removeEventListener('mousemove', handleMouseMove)
  document.removeEventListener('mouseup', handleMouseUp)
})
</script>

<template>
  <div
    ref="containerRef"
    class="layout-wrapper"
    :class="{
      'layout-vertical': isVertical,
      'layout-horizontal': !isVertical,
      'is-dragging': isDragging,
    }"
  >
    <!-- 第一个面板 -->
    <div
      class="layout-panel first-panel"
      :style="firstPanelStyle"
    >
      <slot name="first" />
    </div>

    <!-- 分割器 -->
    <div
      class="layout-splitter"
      :class="{ 'splitter-vertical': isVertical, 'splitter-horizontal': !isVertical }"
      @mousedown="handleMouseDown"
      @dblclick="handleDoubleClick"
    >
      <div class="splitter-handle" />
    </div>

    <!-- 第二个面板 -->
    <div
      class="layout-panel second-panel"
      :style="secondPanelStyle"
    >
      <slot name="second" />
    </div>
  </div>
</template>

<style scoped>
.layout-wrapper {
  display: flex;
  width: 100%;
  height: 100%;
  overflow: hidden;
}

/* 垂直布局（上下） */
.layout-vertical {
  flex-direction: column;
}

/* 水平布局（左右） */
.layout-horizontal {
  flex-direction: row;
}

/* 面板 */
.layout-panel {
  overflow: auto;
  position: relative;
}

/* 分割器基础样式 */
.layout-splitter {
  flex-shrink: 0;
  background-color: var(--border-color);
  position: relative;
  z-index: 10;
  transition: background-color 0.2s;
}

.layout-splitter:hover {
  background-color: var(--accent-color);
}

/* 垂直分割器（水平线） */
.splitter-vertical {
  width: 100%;
  height: 6px;
  cursor: row-resize;
}

/* 水平分割器（垂直线） */
.splitter-horizontal {
  width: 6px;
  height: 100%;
  cursor: col-resize;
}

/* 分割器手柄 */
.splitter-handle {
  position: absolute;
  background-color: var(--text-muted);
  border-radius: 2px;
  transition: background-color 0.2s;
}

.splitter-vertical .splitter-handle {
  width: 40px;
  height: 4px;
  left: 50%;
  top: 50%;
  transform: translate(-50%, -50%);
}

.splitter-horizontal .splitter-handle {
  width: 4px;
  height: 40px;
  left: 50%;
  top: 50%;
  transform: translate(-50%, -50%);
}

.layout-splitter:hover .splitter-handle {
  background-color: var(--text-primary);
}

/* 拖拽状态 */
.is-dragging {
  user-select: none;
}

.is-dragging .layout-splitter {
  background-color: var(--accent-color);
}

.is-dragging .layout-panel {
  pointer-events: none;
}
</style>

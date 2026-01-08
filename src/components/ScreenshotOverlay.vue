<script setup lang="ts">
/**
 * 截图覆盖层组件
 * macOS: 使用原生 screencapture -i
 * 其他平台: 全屏框选模式
 */

import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { useScreenshot, type ScreenshotResult, type SelectionRect } from '@/composables/useScreenshot'

const props = defineProps<{
  visible: boolean
}>()

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'capture', result: ScreenshotResult): void
}>()

const {
  isCapturing,
  error,
  screenshotData,
  useNativeScreenshot,
  captureInteractive,
  captureFullScreenSilent,
  showWindow,
  enterFullscreenMode,
  exitFullscreenMode,
  cropImage,
  cleanup
} = useScreenshot()

// 状态
const state = ref<'idle' | 'capturing' | 'selecting'>('idle')
const localError = ref<string | null>(null)
const isNative = ref(false)

// 框选状态
const isSelecting = ref(false)
const selectionStart = ref({ x: 0, y: 0 })
const selectionEnd = ref({ x: 0, y: 0 })

// 计算选区
const selection = computed<SelectionRect>(() => {
  const x1 = Math.min(selectionStart.value.x, selectionEnd.value.x)
  const y1 = Math.min(selectionStart.value.y, selectionEnd.value.y)
  const x2 = Math.max(selectionStart.value.x, selectionEnd.value.x)
  const y2 = Math.max(selectionStart.value.y, selectionEnd.value.y)
  return { x: x1, y: y1, width: x2 - x1, height: y2 - y1 }
})

const hasSelection = computed(() => selection.value.width > 10 && selection.value.height > 10)

const screenshotUrl = computed(() => {
  if (!screenshotData.value) return ''
  return `data:${screenshotData.value.mime_type};base64,${screenshotData.value.data}`
})

// 窗口尺寸（用于模板中避免直接访问 window）
const windowSize = computed(() => ({
  width: typeof window !== 'undefined' ? window.innerWidth : 0,
  height: typeof window !== 'undefined' ? window.innerHeight : 0
}))

// 初始化检测平台
onMounted(() => {
  isNative.value = useNativeScreenshot()
  document.addEventListener('keydown', handleKeyDown)
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeyDown)
})

// 开始截图
async function startCapture() {
  try {
    state.value = 'capturing'
    localError.value = null
    
    if (isNative.value) {
      // macOS: 使用原生截图
      emit('close')
      const result = await captureInteractive()
      if (result) {
        emit('capture', result)
      }
      resetState()
    } else {
      // 其他平台: 全屏框选
      await captureFullScreenSilent()
      await enterFullscreenMode()
      state.value = 'selecting'
      selectionStart.value = { x: 0, y: 0 }
      selectionEnd.value = { x: 0, y: 0 }
    }
  } catch (e) {
    console.error('Screenshot failed:', e)
    localError.value = e instanceof Error ? e.message : String(e)
    state.value = 'idle'
    await showWindow()
  }
}

// 鼠标事件
function handleMouseDown(e: MouseEvent) {
  if (state.value !== 'selecting') return
  isSelecting.value = true
  selectionStart.value = { x: e.clientX, y: e.clientY }
  selectionEnd.value = { x: e.clientX, y: e.clientY }
}

function handleMouseMove(e: MouseEvent) {
  if (!isSelecting.value) return
  selectionEnd.value = { x: e.clientX, y: e.clientY }
}

function handleMouseUp() {
  isSelecting.value = false
}

// 确认选区
async function confirmSelection() {
  if (!hasSelection.value || !screenshotData.value) return
  
  try {
    const cropped = await cropImage(
      screenshotData.value,
      selection.value,
      window.innerWidth,
      window.innerHeight
    )
    await exitFullscreenMode()
    emit('capture', cropped)
    resetState()
  } catch (e) {
    localError.value = e instanceof Error ? e.message : String(e)
  }
}

// 取消
async function cancel() {
  if (state.value === 'selecting') {
    await exitFullscreenMode()
  }
  resetState()
  emit('close')
}

function resetState() {
  state.value = 'idle'
  cleanup()
  isSelecting.value = false
  selectionStart.value = { x: 0, y: 0 }
  selectionEnd.value = { x: 0, y: 0 }
  localError.value = null
}

// 键盘事件
function handleKeyDown(event: KeyboardEvent) {
  if (state.value === 'selecting') {
    if (event.key === 'Escape') cancel()
    else if (event.key === 'Enter' && hasSelection.value) confirmSelection()
  } else if (props.visible && state.value === 'idle') {
    if (event.key === 'Escape') cancel()
    else if (event.key === 'Enter') startCapture()
  }
}

watch(() => props.visible, (newVal) => {
  if (!newVal && state.value !== 'idle') resetState()
})
</script>

<template>
  <!-- 初始状态 -->
  <Teleport to="body">
    <div
      v-if="visible && state === 'idle'"
      class="screenshot-overlay"
      @click.self="cancel"
    >
      <div class="hint-container">
        <div class="hint-card">
          <span class="i-carbon-screen hint-icon" />
          <h3 class="hint-title">
            屏幕截图
          </h3>
          <p class="hint-desc">
            {{ isNative ? '点击开始后，拖拽鼠标框选截图区域' : '点击开始后，在全屏模式下框选区域' }}
          </p>
          
          <div class="hint-actions">
            <button
              class="action-btn primary"
              :disabled="isCapturing"
              @click="startCapture"
            >
              <span
                v-if="isCapturing"
                class="i-carbon-loading animate-spin"
              />
              <span
                v-else
                class="i-carbon-camera"
              />
              {{ isCapturing ? '截图中...' : '开始截图' }}
            </button>
            <button
              class="action-btn secondary"
              @click="cancel"
            >
              <span class="i-carbon-close" />
              取消
            </button>
          </div>
          
          <p class="hint-shortcut">
            按 Enter 开始 | ESC 取消
          </p>
        </div>
        
        <div
          v-if="localError || error"
          class="error-toast"
        >
          <span class="i-carbon-warning" />
          {{ localError || error }}
        </div>
      </div>
    </div>
  </Teleport>

  <!-- 框选状态（非 macOS） -->
  <Teleport to="body">
    <div 
      v-if="state === 'selecting'" 
      class="selection-overlay"
      @mousedown="handleMouseDown"
      @mousemove="handleMouseMove"
      @mouseup="handleMouseUp"
      @mouseleave="handleMouseUp"
    >
      <img
        :src="screenshotUrl"
        class="screenshot-bg"
        draggable="false"
      >
      <div class="mask-layer" />
      
      <div 
        v-if="hasSelection"
        class="selection-box"
        :style="{
          left: selection.x + 'px',
          top: selection.y + 'px',
          width: selection.width + 'px',
          height: selection.height + 'px',
          backgroundImage: `url(${screenshotUrl})`,
          backgroundPosition: `-${selection.x}px -${selection.y}px`,
          backgroundSize: `${windowSize.width}px ${windowSize.height}px`
        }"
      >
        <div class="size-info">
          {{ Math.round(selection.width) }} × {{ Math.round(selection.height) }}
        </div>
      </div>
      
      <div
        v-if="hasSelection"
        class="selection-toolbar"
      >
        <button
          class="toolbar-btn confirm"
          @click="confirmSelection"
        >
          <span class="i-carbon-checkmark" /> 确认
        </button>
        <button
          class="toolbar-btn cancel"
          @click="cancel"
        >
          <span class="i-carbon-close" /> 取消
        </button>
      </div>
      
      <div
        v-if="!hasSelection"
        class="selection-hint"
      >
        拖拽鼠标框选截图区域 | Enter 确认 | ESC 取消
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.screenshot-overlay {
  position: fixed;
  inset: 0;
  z-index: 99999;
  background: rgba(0, 0, 0, 0.6);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
}

.hint-container { display: flex; flex-direction: column; align-items: center; gap: 16px; }

.hint-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 32px 48px;
  background: var(--bg-primary);
  border-radius: 16px;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.4);
  text-align: center;
}

.hint-icon { font-size: 48px; color: var(--accent-color); }
.hint-title { font-size: 20px; font-weight: 600; color: var(--text-primary); margin: 0; }
.hint-desc { font-size: 14px; color: var(--text-secondary); margin: 0; }
.hint-actions { display: flex; gap: 12px; margin-top: 8px; }
.hint-shortcut { font-size: 12px; color: var(--text-muted); margin: 8px 0 0 0; }

.action-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 24px;
  border: none;
  border-radius: 10px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.action-btn.primary { background: var(--accent-gradient); color: white; }
.action-btn.primary:hover:not(:disabled) { transform: translateY(-2px); box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3); }
.action-btn.primary:disabled { opacity: 0.6; cursor: not-allowed; }
.action-btn.secondary { background: var(--bg-secondary); color: var(--text-primary); border: 1px solid var(--border-color); }
.action-btn.secondary:hover { background: var(--bg-tertiary); }

.error-toast {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 20px;
  background: #ef4444;
  color: white;
  border-radius: 8px;
  font-size: 14px;
}

/* 框选模式 */
.selection-overlay {
  position: fixed;
  inset: 0;
  z-index: 99999;
  cursor: crosshair;
  user-select: none;
  overflow: hidden;
}

.screenshot-bg {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  object-fit: cover;
  pointer-events: none;
  z-index: 1;
}

.mask-layer {
  position: absolute;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  pointer-events: none;
  z-index: 2;
}

.selection-box {
  position: absolute;
  border: 2px solid #3b82f6;
  background-repeat: no-repeat;
  box-shadow: 0 0 0 1px rgba(255, 255, 255, 0.3);
  pointer-events: none;
  z-index: 3;
}

.size-info {
  position: absolute;
  top: -28px;
  left: 0;
  padding: 4px 8px;
  background: #3b82f6;
  color: white;
  font-size: 12px;
  border-radius: 4px;
  white-space: nowrap;
}

.selection-toolbar {
  position: fixed;
  bottom: 20px;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  gap: 8px;
  padding: 8px;
  background: rgba(0, 0, 0, 0.8);
  border-radius: 12px;
  backdrop-filter: blur(10px);
  z-index: 100000;
}

.toolbar-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 10px 20px;
  border: none;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
}

.toolbar-btn.confirm { background: #22c55e; color: white; }
.toolbar-btn.confirm:hover { background: #16a34a; }
.toolbar-btn.cancel { background: #6b7280; color: white; }
.toolbar-btn.cancel:hover { background: #4b5563; }

.selection-hint {
  position: fixed;
  top: 20px;
  left: 50%;
  transform: translateX(-50%);
  padding: 12px 24px;
  background: rgba(0, 0, 0, 0.8);
  color: white;
  font-size: 14px;
  border-radius: 8px;
  backdrop-filter: blur(10px);
  z-index: 100000;
}

.animate-spin { animation: spin 1s linear infinite; }
@keyframes spin { to { transform: rotate(360deg); } }
</style>

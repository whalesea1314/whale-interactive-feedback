<script setup lang="ts">
import { onMounted, onUnmounted, ref, computed } from 'vue'
import { useConfigStore } from './stores/config'
import { useFeedbackStore } from './stores/feedback'
import { useTheme } from './composables/useTheme'
import { useLayout } from './composables/useLayout'
import { useWindowControl } from './composables/useWindowControl'
import { useDisplayMode } from './composables/useDisplayMode'
import { useImageHandler } from './composables/useImageHandler'
import { useFileHandler } from './composables/useFileHandler'
import { useAudio } from './composables/useAudio'
import { useMcpHandler } from './composables/useMcpHandler'
import { useSplitter } from './composables/useSplitter'
import { useDragDrop } from './composables/useDragDrop'
import SettingsPanel from './components/SettingsPanel.vue'
import ImagePreview from './components/ImagePreview.vue'
import FileSelectModal from './components/FileSelectModal.vue'
import CannedResponsesPopup from './components/CannedResponsesPopup.vue'
import CannedResponsesManager from './components/CannedResponsesManager.vue'
import TextOptimizer from './components/TextOptimizer.vue'
import ScreenshotOverlay from './components/ScreenshotOverlay.vue'
import MarkdownContent from './components/MarkdownContent.vue'
import type { ScreenshotResult } from './composables/useScreenshot'

const configStore = useConfigStore()
const feedbackStore = useFeedbackStore()

// 主题
const { theme, themeColor } = useTheme()

// 字体大小
const fontSize = computed(() => configStore.fontSize)

// 布局
const { isVertical, toggleLayout } = useLayout()

// 窗口控制
const { initWindowState, handleAfterSubmit } = useWindowControl()

// 图片/文件处理
const { createImageFromPath } = useImageHandler()
const { classifyPaths, createFileReferenceFromPath } = useFileHandler()

// 音频
const { playNotification } = useAudio()

// MCP 处理
const { 
  isMcpMode, 
  checkMcpMode, 
  loadMcpRequest, 
  submitFeedback: mcpSubmitFeedback,
  cancelRequest: mcpCancelRequest 
} = useMcpHandler()

// 状态
const showSettings = ref(false)
const showCannedPopup = ref(false)
const showCannedManager = ref(false)
const showTextOptimizer = ref(false)
const showScreenshotOverlay = ref(false)

// 窗口置顶状态 - 从配置读取
const isAlwaysOnTop = computed(() => configStore.windowPinned)

// 切换窗口置顶
async function toggleAlwaysOnTop() {
  const { invoke } = await import('@tauri-apps/api/core')
  
  const newValue = !isAlwaysOnTop.value
  
  try {
    await invoke('set_window_always_on_top', { onTop: newValue })
    await configStore.setWindowPinned(newValue)
    console.log('Window pinned saved:', newValue)
  } catch (e) {
    console.error('Error setting window always on top:', e)
  }
}

// 引用
const mainContainerRef = ref<HTMLElement | null>(null)
const leftPanelRef = ref<HTMLElement | null>(null)
const textareaRef = ref<HTMLTextAreaElement | null>(null)
const cannedBtnRef = ref<HTMLButtonElement | null>(null)
const cannedBtnRect = ref<DOMRect | undefined>(undefined)

// 使用 useSplitter composable 处理分割线
const mainSplitter = useSplitter({
  initialPosition: 50,
  minPosition: 20,
  maxPosition: 80,
  isVertical,
  containerRef: mainContainerRef
})

const leftSplitter = useSplitter({
  initialPosition: 50,
  minPosition: 20,
  maxPosition: 80,
  isVertical: true,
  containerRef: leftPanelRef
})

// 分割线位置（兼容原有代码）
const mainSplitterPosition = mainSplitter.position
const leftSplitterPosition = leftSplitter.position

// 拖拽状态
const isDraggingMain = mainSplitter.isDragging
const isDraggingLeft = leftSplitter.isDragging

// 使用 useDragDrop composable 处理拖放
const {
  isDraggingFile,
  setupTauriDragDrop,
  cleanupTauriDragDrop,
  onHtml5DragEnter,
  onHtml5DragOver,
  onHtml5DragLeave,
  onHtml5Drop
} = useDragDrop({
  onImageFile: createImageFromPath,
  onOtherFile: createFileReferenceFromPath,
  addImage: feedbackStore.addImage,
  addFileReference: feedbackStore.addFileReference,
  classifyPaths
})

// MCP 参数
const mcpMessage = ref('欢迎使用 Whale Interactive Feedback！')
const mcpFullResponse = ref(`# 🐋 Whale Interactive Feedback

一款为 AI 编程助手设计的交互式反馈工具。

## ✨ 主要功能

- **快速反馈** - 通过预设选项或自由输入快速响应 AI 请求
- **图片支持** - 粘贴、拖拽或截图添加图片
- **文件引用** - 附加文件作为上下文
- **文本优化** - AI 辅助优化反馈内容
- **常用语** - 保存常用回复，一键插入

## 🎯 使用场景

当 AI 助手需要你的确认或反馈时，此窗口会自动弹出：
- 确认执行计划
- 选择方案选项
- 提供额外说明
- 附加参考资料

## ⌨️ 快捷操作

| 操作 | 说明 |
|------|------|
| Enter | 提交反馈 |
| Shift+Enter | 换行 |
| Ctrl/Cmd+V | 粘贴图片 |
| 拖拽文件 | 添加附件 |

---

> 💡 这是演示模式，实际使用时内容由 AI 助手提供。`)

const mcpPredefinedOptions = ref<string[]>([
  '继续执行',
  '需要修改',
  '取消操作',
  '稍后再说',
  '查看详情',
  '返回上一步',
  '跳过此步骤',
  '全部同意',
  '部分同意',
  '需要更多信息'
])

// 显示模式 - 默认完整模式
const displayParams = computed(() => ({
  message: mcpMessage.value,
  fullResponse: mcpFullResponse.value,
}))
const { displayContent } = useDisplayMode(displayParams)

// 样式计算
const leftPanelStyle = computed(() => {
  return isVertical.value
    ? { height: `${mainSplitterPosition.value}%`, width: '100%' }
    : { width: `${mainSplitterPosition.value}%`, height: '100%' }
})

const rightPanelStyle = computed(() => {
  return isVertical.value
    ? { height: `${100 - mainSplitterPosition.value}%`, width: '100%' }
    : { width: `${100 - mainSplitterPosition.value}%`, height: '100%' }
})

// 显示区域样式：根据分割线位置设置最小高度，自动填充剩余空间
const displayAreaStyle = computed(() => ({ 
  flex: '1 1 auto', 
  minHeight: `${leftSplitterPosition.value}%` 
}))
// 选项区域样式：高度自适应内容，最大不超过分割线设定的比例
const optionsAreaStyle = computed(() => ({ 
  flex: '0 0 auto', 
  maxHeight: `${100 - leftSplitterPosition.value}%` 
}))

// 初始化
onMounted(async () => {
  await configStore.loadConfig()
  await initWindowState()
  
  // 应用保存的窗口置顶状态
  if (configStore.windowPinned) {
    const { invoke } = await import('@tauri-apps/api/core')
    try {
      await invoke('set_window_always_on_top', { onTop: true })
    } catch (e) {
      console.error('Error restoring window always on top:', e)
    }
  }
  
  // 检查 MCP 模式并加载请求
  const inMcpMode = await checkMcpMode()
  if (inMcpMode) {
    const request = await loadMcpRequest()
    if (request) {
      // 使用 MCP 请求中的参数
      mcpMessage.value = request.message || ''
      mcpFullResponse.value = request.full_response || ''
      if (request.predefined_options && request.predefined_options.length > 0) {
        mcpPredefinedOptions.value = request.predefined_options
      }
      console.log('MCP mode initialized with request:', request.id)
      console.log('Message:', mcpMessage.value)
      console.log('Full response:', mcpFullResponse.value)
    }
  }
  
  // 设置预定义选项：始终使用 MCP 传来的选项
  // 自定义选项（常用语）通过弹窗插入到输入框，不覆盖选项区域
  if (mcpPredefinedOptions.value.length > 0) {
    feedbackStore.setPredefinedOptions(mcpPredefinedOptions.value)
  }
  textareaRef.value?.focus()
  document.addEventListener('paste', onPaste)
  playNotification()
  
  // 设置 Tauri 原生拖放事件监听
  setupTauriDragDrop()
})

// 清理
onUnmounted(() => {
  document.removeEventListener('paste', onPaste)
  cleanupTauriDragDrop()
})

// 粘贴 - 防止重复处理
async function onPaste(event: ClipboardEvent) {
  const items = event.clipboardData?.items
  if (!items) return
  
  // 查找第一个图片项
  for (const item of items) {
    if (item.type.startsWith('image/')) {
      const blob = item.getAsFile()
      if (blob) {
        event.preventDefault()
        
        // 直接处理这个 blob，不使用 handlePaste
        const reader = new FileReader()
        reader.onload = async () => {
          const base64 = (reader.result as string).split(',')[1]
          const img = new Image()
          img.onload = () => {
            feedbackStore.addImage({
              id: `img_${Date.now()}_${Math.random().toString(36).substring(2, 9)}`,
              data: base64,
              mimeType: item.type,
              width: img.naturalWidth,
              height: img.naturalHeight,
              size: blob.size
            })
          }
          img.src = reader.result as string
        }
        reader.readAsDataURL(blob)
        return // 只处理第一个图片
      }
    }
  }
}

// 选项切换
function toggleOption(index: number) {
  feedbackStore.toggleOption(index)
}

// 键盘
function handleKeydown(event: KeyboardEvent) {
  if (event.key === 'Enter' && !event.shiftKey) {
    event.preventDefault()
    submitFeedback()
  }
}

// 提交
async function submitFeedback() {
  const selectedTexts = Array.from(feedbackStore.selectedOptions)
    .map(index => feedbackStore.predefinedOptions[index])
    .filter(Boolean)
  const userText = feedbackStore.text.trim()
  const combinedText = [...selectedTexts, userText].filter(t => t.length > 0).join('\n')
  console.log('Feedback:', combinedText || '[User provided no feedback]')
  
  // MCP 模式：通过 MCP handler 提交
  if (isMcpMode.value) {
    try {
      // 转换图片格式
      const images = feedbackStore.images.map(img => ({
        data: img.data,
        mime_type: img.mimeType
      }))
      // 转换文件引用格式
      const fileRefs = feedbackStore.fileReferences.map(ref => ({
        display_name: ref.displayName,
        path: ref.path,
        is_directory: ref.isDirectory || false
      }))
      console.log('[MCP Submit] images:', images.length, 'fileRefs:', fileRefs.length, fileRefs)
      await mcpSubmitFeedback(combinedText, selectedTexts, images, fileRefs)
    } catch (error) {
      console.error('Failed to submit MCP feedback:', error)
      showToastMessage(`提交失败: ${error}`, 'error', 3000)
    }
    return
  }
  
  // 非 MCP 模式：显示提示
  showToastMessage(`反馈已提交`, 'success')
  
  feedbackStore.reset()
  await handleAfterSubmit()
}

// 取消
async function cancelFeedback() {
  console.log('Feedback cancelled')
  
  // MCP 模式：通过 MCP handler 取消
  if (isMcpMode.value) {
    try {
      await mcpCancelRequest()
    } catch (error) {
      console.error('Failed to cancel MCP request:', error)
    }
    return
  }
  
  feedbackStore.reset()
}

// 文件选择弹窗
const showFileSelectModal = ref(false)

function handleSelectFiles() {
  showFileSelectModal.value = true
}

function handleFileSelectConfirm(files: import('@/types').FileReference[]) {
  for (const file of files) {
    feedbackStore.addFileReference(file)
  }
  showFileSelectModal.value = false
}

// 常用语
const isHoveringCannedBtn = ref(false)
const isHoveringCannedPopup = ref(false)

function handleCannedMouseEnter() {
  if (cannedBtnRef.value) cannedBtnRect.value = cannedBtnRef.value.getBoundingClientRect()
  isHoveringCannedBtn.value = true
  showCannedPopup.value = true
}
function handleCannedMouseLeave() {
  isHoveringCannedBtn.value = false
  setTimeout(() => {
    if (!isHoveringCannedBtn.value && !isHoveringCannedPopup.value) {
      showCannedPopup.value = false
    }
  }, 150)
}
function handleCannedPopupEnter() {
  isHoveringCannedPopup.value = true
}
function handleCannedPopupLeave() {
  isHoveringCannedPopup.value = false
  setTimeout(() => {
    if (!isHoveringCannedBtn.value && !isHoveringCannedPopup.value) {
      showCannedPopup.value = false
    }
  }, 150)
}
function handleCannedClick() {
  showCannedPopup.value = false
  showCannedManager.value = true
}
function insertCannedResponse(text: string) {
  feedbackStore.text = feedbackStore.text ? `${feedbackStore.text}${text}` : text
}

// 文本优化
function applyOptimizedText(text: string) {
  feedbackStore.text = text
  showTextOptimizer.value = false
}

// 截图
// Toast 提示
const toastMessage = ref('')
const showToast = ref(false)
const toastType = ref<'success' | 'error' | 'info'>('success')

function showToastMessage(message: string, type: 'success' | 'error' | 'info' = 'success', duration = 2000) {
  toastMessage.value = message
  toastType.value = type
  showToast.value = true
  setTimeout(() => {
    showToast.value = false
  }, duration)
}

function handleScreenshotCapture(result: ScreenshotResult) {
  console.log('Screenshot captured:', result)
  if (result && result.data) {
    feedbackStore.addImage({
      id: `screenshot-${Date.now()}`,
      data: result.data,
      mimeType: result.mime_type,
      width: result.width,
      height: result.height,
      size: result.size,
    })
    showToastMessage(`截图成功 ${result.width}×${result.height}`)
  }
  showScreenshotOverlay.value = false
  
  // 确保窗口在最上层
  import('@tauri-apps/api/window').then(({ getCurrentWindow }) => {
    const win = getCurrentWindow()
    win.setAlwaysOnTop(true).then(() => {
      win.setFocus()
      // 短暂置顶后恢复
      setTimeout(() => {
        win.setAlwaysOnTop(false)
      }, 500)
    })
  })
}
</script>

<template>
  <div 
    class="app-container" 
    :class="[theme, `theme-${themeColor}`]"
    @dragenter="onHtml5DragEnter"
    @dragover="onHtml5DragOver"
    @dragleave="onHtml5DragLeave"
    @drop="onHtml5Drop"
  >
    <!-- 拖拽覆盖层 -->
    <div
      v-if="isDraggingFile"
      class="drag-overlay"
    >
      <div class="drag-content">
        <span class="i-carbon-document-add drag-icon" />
        <span class="drag-text">释放以添加文件</span>
      </div>
    </div>

    <!-- 主布局 -->
    <div 
      ref="mainContainerRef"
      class="main-layout"
      :class="{
        'layout-vertical': isVertical,
        'layout-horizontal': !isVertical,
        'is-dragging': isDraggingMain || isDraggingLeft
      }"
    >
      <!-- 左侧：内容 + 分割线 + 选项 -->
      <div
        ref="leftPanelRef"
        class="left-panel"
        :style="leftPanelStyle"
      >
        <!-- 显示内容区 -->
        <div
          class="display-area"
          :style="displayAreaStyle"
        >
          <div class="display-content">
            <MarkdownContent
              :content="displayContent"
              :font-size="fontSize.display"
            />
          </div>
        </div>

        <!-- 左侧分割线 -->
        <div
          class="splitter splitter-vertical"
          @mousedown="leftSplitter.handleMouseDown"
          @dblclick="leftSplitterPosition = 50"
        >
          <div class="splitter-handle" />
        </div>

        <!-- 选项区 -->
        <div
          class="options-area"
          :style="optionsAreaStyle"
        >
          <div class="options-list">
            <div 
              v-for="(option, index) in feedbackStore.predefinedOptions" 
              :key="index"
              class="option-item"
              :class="{ selected: feedbackStore.selectedOptions.has(index) }"
              @click="toggleOption(index)"
            >
              <span 
                class="option-checkbox"
                :class="{ checked: feedbackStore.selectedOptions.has(index) }"
              >
                <span
                  v-if="feedbackStore.selectedOptions.has(index)"
                  class="checkbox-icon i-carbon-checkmark"
                />
              </span>
              <span
                class="option-text"
                :style="{ fontSize: fontSize.options + 'px' }"
              >{{ option }}</span>
            </div>
          </div>
        </div>
      </div>

      <!-- 主分割线 -->
      <div
        class="splitter main-splitter"
        :class="{ 'splitter-vertical': isVertical, 'splitter-horizontal': !isVertical }"
        @mousedown="mainSplitter.handleMouseDown"
        @dblclick="mainSplitterPosition = 50"
      >
        <div class="splitter-handle" />
      </div>

      <!-- 右侧：输入区域（含内嵌图片预览） -->
      <div
        class="right-panel"
        :style="rightPanelStyle"
      >
        <!-- 输入框容器 -->
        <div class="input-wrapper">
          <!-- 文本输入框 -->
          <textarea
            ref="textareaRef"
            v-model="feedbackStore.text"
            placeholder="输入您的反馈... (Enter 发送, Shift+Enter 换行)"
            class="feedback-input"
            :style="{ fontSize: fontSize.input + 'px' }"
            @keydown="handleKeydown"
          />
          
          <!-- 文件引用标签区域 - 仅有文件时显示 -->
          <Transition name="slide-up">
            <div
              v-if="feedbackStore.fileReferences.length > 0"
              class="file-tags-area"
            >
              <div class="file-tags-list">
                <div 
                  v-for="fileRef in feedbackStore.fileReferences" 
                  :key="fileRef.id"
                  class="file-tag"
                  :title="fileRef.path"
                >
                  <span class="file-tag-name">{{ fileRef.displayName }}</span>
                  <button 
                    class="file-tag-remove"
                    title="移除文件"
                    @click="feedbackStore.removeFileReference(fileRef.id)"
                  >
                    <span class="i-carbon-close" />
                  </button>
                </div>
              </div>
            </div>
          </Transition>
          
          <!-- 内嵌图片预览区域 - 仅有图片时显示 -->
          <Transition name="slide-up">
            <div
              v-if="feedbackStore.images.length > 0"
              class="inline-preview"
            >
              <ImagePreview
                :images="feedbackStore.images"
                @remove="feedbackStore.removeImage"
              />
            </div>
          </Transition>
        </div>
      </div>
    </div>
    
    <!-- 底部工具栏 -->
    <div class="bottom-toolbar">
      <div class="toolbar-left">
        <button
          ref="cannedBtnRef"
          class="tool-btn"
          title="常用语"
          @mouseenter="handleCannedMouseEnter"
          @mouseleave="handleCannedMouseLeave"
          @click="handleCannedClick"
        >
          <span class="i-carbon-text-short-paragraph" />
        </button>
        <button
          class="tool-btn"
          title="选择文件"
          @click="handleSelectFiles"
        >
          <span class="i-carbon-document-add" />
        </button>
        <button
          class="tool-btn"
          title="窗口截图"
          @click="showScreenshotOverlay = true"
        >
          <span class="i-carbon-screen" />
        </button>
        <button 
          class="tool-btn" 
          :class="{ active: isAlwaysOnTop }"
          :title="isAlwaysOnTop ? '取消置顶' : '窗口置顶'"
          @click="toggleAlwaysOnTop"
        >
          <span
            class="i-carbon-pin"
            :class="{ 'i-carbon-pin-filled': isAlwaysOnTop }"
          />
        </button>
        <button
          class="tool-btn"
          title="设置"
          @click="showSettings = true"
        >
          <span class="i-carbon-settings" />
        </button>
        <button
          class="tool-btn"
          title="优化"
          @click="showTextOptimizer = true"
        >
          <span class="i-carbon-magic-wand" />
        </button>
        <button 
          class="tool-btn" 
          :title="isVertical ? '切换到左右布局' : '切换到上下布局'"
          @click="toggleLayout"
        >
          <span :class="isVertical ? 'i-carbon-column' : 'i-carbon-row'" />
        </button>
      </div>
      <div class="toolbar-right">
        <button
          class="cancel-btn"
          @click="cancelFeedback"
        >
          取消
        </button>
        <button
          class="submit-btn"
          @click="submitFeedback"
        >
          提交
        </button>
      </div>
    </div>
    
    <!-- 弹窗 -->
    <SettingsPanel
      :visible="showSettings"
      @close="showSettings = false"
    />
    <FileSelectModal 
      :visible="showFileSelectModal" 
      :existing-files="feedbackStore.fileReferences"
      @close="showFileSelectModal = false"
      @confirm="handleFileSelectConfirm"
    />
    <CannedResponsesPopup
      :visible="showCannedPopup"
      :trigger-rect="cannedBtnRect"
      @insert="insertCannedResponse"
      @close="showCannedPopup = false"
      @open-manager="showCannedManager = true"
      @mouseenter="handleCannedPopupEnter"
      @mouseleave="handleCannedPopupLeave"
    />
    <CannedResponsesManager
      :visible="showCannedManager"
      @close="showCannedManager = false"
      @insert="insertCannedResponse"
    />
    <TextOptimizer
      :visible="showTextOptimizer"
      :initial-text="feedbackStore.text"
      @close="showTextOptimizer = false"
      @apply="applyOptimizedText"
      @open-settings="showSettings = true"
    />
    <ScreenshotOverlay
      :visible="showScreenshotOverlay"
      @close="showScreenshotOverlay = false"
      @capture="handleScreenshotCapture"
    />
    
    <!-- Toast 提示 -->
    <Teleport to="body">
      <Transition name="toast">
        <div
          v-if="showToast"
          class="toast-container"
        >
          <div 
            class="toast-message"
            :class="`toast-${toastType}`"
          >
            <span 
              class="toast-icon"
              :class="{
                'i-carbon-checkmark-filled': toastType === 'success',
                'i-carbon-warning-filled': toastType === 'error',
                'i-carbon-information-filled': toastType === 'info'
              }"
            />
            {{ toastMessage }}
          </div>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<style scoped>
.app-container {
  width: 100vw;
  height: 100vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  position: relative;
  background: var(--bg-primary);
}

/* Toast 样式 */
.toast-container {
  position: fixed;
  top: 20px;
  left: 50%;
  transform: translateX(-50%);
  z-index: 100000;
  pointer-events: none;
}

.toast-message {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 20px;
  background: rgba(34, 197, 94, 0.95);
  color: white;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 500;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  backdrop-filter: blur(10px);
}

.toast-message.toast-success {
  background: rgba(34, 197, 94, 0.95);
}

.toast-message.toast-error {
  background: rgba(239, 68, 68, 0.95);
}

.toast-message.toast-info {
  background: rgba(59, 130, 246, 0.95);
}

.toast-icon {
  font-size: 18px;
}

.toast-enter-active,
.toast-leave-active {
  transition: all 0.3s ease;
}

.toast-enter-from,
.toast-leave-to {
  opacity: 0;
  transform: translateX(-50%) translateY(-20px);
}

/* 拖拽覆盖层 */
.drag-overlay {
  position: absolute;
  inset: 0;
  background: var(--accent-light);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
  pointer-events: none;
}
.drag-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
  padding: 32px 48px;
  background: var(--bg-glass);
  backdrop-filter: blur(var(--blur-amount)) saturate(var(--backdrop-saturate));
  -webkit-backdrop-filter: blur(var(--blur-amount)) saturate(var(--backdrop-saturate));
  border-radius: 20px;
  border: 1px solid var(--border-color);
  box-shadow: var(--shadow-lg);
}
.drag-icon { 
  font-size: 56px; 
  color: var(--accent-color);
  filter: drop-shadow(0 4px 8px var(--accent-light));
}
.drag-text { 
  font-size: 17px; 
  font-weight: 500;
  color: var(--text-primary); 
}

/* 主布局 */
.main-layout {
  flex: 1;
  display: flex;
  overflow: hidden;
  min-height: 0;
  padding: 12px;
  gap: 1px;
}
.layout-vertical { flex-direction: column; }
.layout-horizontal { flex-direction: row; }
.is-dragging { user-select: none; }
.is-dragging .left-panel,
.is-dragging .right-panel,
.is-dragging .display-area,
.is-dragging .options-area { pointer-events: none; }

/* 左侧面板 */
.left-panel {
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: var(--bg-glass);
  backdrop-filter: blur(var(--blur-amount)) saturate(var(--backdrop-saturate));
  -webkit-backdrop-filter: blur(var(--blur-amount)) saturate(var(--backdrop-saturate));
  border-radius: 12px;
  border: 1px solid var(--border-color);
  box-shadow: var(--shadow-md);
  padding-right: 4px;
}

/* 显示区域 - flex 自动填充 */
.display-area {
  display: flex;
  flex-direction: column;
  overflow: hidden;
  padding: 16px;
  padding-bottom: 8px;
  min-height: 0;
}
.display-content {
  flex: 1;
  overflow: auto;
  color: var(--text-primary);
  white-space: normal;
  line-height: 1.7;
  font-size: 14px;
  background: var(--bg-card);
  backdrop-filter: blur(10px);
  -webkit-backdrop-filter: blur(10px);
  padding: 16px;
  border-radius: 10px;
  border: 1px solid var(--border-subtle);
}

/* 选项区域容器 */
.options-area {
  display: flex;
  flex-direction: column;
  overflow: hidden;
  padding: 8px 16px 16px 16px;
}

/* 选项列表 */
.options-list {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 8px;
  overflow: auto;
  padding: 16px;
  background: var(--bg-card);
  backdrop-filter: blur(10px);
  -webkit-backdrop-filter: blur(10px);
  border-radius: 10px;
  border: 1px solid var(--border-subtle);
}

/* 选项区域滚动条 - 与显示区域一致 */
.options-area::-webkit-scrollbar {
  width: 6px;
  height: 6px;
}

.options-area::-webkit-scrollbar-thumb {
  background: var(--scrollbar-thumb);
  border-radius: 3px;
}

.options-area::-webkit-scrollbar-thumb:hover {
  background: var(--scrollbar-thumb-hover);
}
.option-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  border-radius: 10px;
  cursor: pointer;
  border: 1px solid var(--border-subtle);
  background: var(--bg-card);
  backdrop-filter: blur(10px);
  -webkit-backdrop-filter: blur(10px);
  flex-shrink: 0;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  user-select: none;
  -webkit-user-select: none;
}
.option-item:hover { 
  background: var(--bg-hover);
  border-color: var(--border-color);
}
.option-item.selected {
  border-color: var(--accent-color);
  background: var(--accent-light);
}
.option-checkbox { 
  display: flex;
  align-items: center;
  justify-content: center;
  width: 18px; 
  height: 18px; 
  min-width: 18px;
  border-radius: 4px;
  border: 2px solid var(--border-color);
  background: transparent;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}
.option-checkbox.checked {
  background: var(--accent-color);
  border-color: var(--accent-color);
}
.checkbox-icon {
  color: white;
  font-size: 12px;
}
.option-text { 
  color: var(--text-primary); 
  flex: 1;
  font-size: 14px;
}

/* 右侧面板 */
.right-panel {
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: var(--bg-glass);
  backdrop-filter: blur(var(--blur-amount)) saturate(var(--backdrop-saturate));
  -webkit-backdrop-filter: blur(var(--blur-amount)) saturate(var(--backdrop-saturate));
  border-radius: 12px;
  border: 1px solid var(--border-color);
  box-shadow: var(--shadow-md);
  padding: 16px;
  padding-left: 12px;
  gap: 12px;
}

.feedback-input {
  flex: 1;
  width: 100%;
  min-height: 0;
  padding: 14px 16px;
  border: none;
  background: transparent;
  color: var(--text-primary);
  font-size: 14px;
  line-height: 1.6;
  resize: none;
  outline: none;
  font-family: inherit;
}
.feedback-input::placeholder { color: var(--text-muted); }

/* 输入框容器聚焦效果 */
.input-wrapper:focus-within {
  border-color: var(--accent-color);
  box-shadow: 0 0 0 3px var(--accent-light), var(--shadow-sm);
}

/* 输入框容器 */
.input-wrapper {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0;
  background: var(--bg-card);
  backdrop-filter: blur(10px);
  -webkit-backdrop-filter: blur(10px);
  border-radius: 10px;
  border: 1px solid var(--border-subtle);
  overflow: hidden;
}

/* 文件引用标签区域 */
.file-tags-area {
  flex-shrink: 0;
  padding: 8px 12px;
  border-top: 1px solid var(--border-subtle);
  background: var(--bg-secondary);
}

.file-tags-list {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.file-tag {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 4px 8px;
  background: var(--accent-light);
  border: 1px solid var(--accent-color);
  border-radius: 6px;
  font-size: 12px;
  color: var(--accent-color);
  max-width: 200px;
}

.file-tag-name {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.file-tag-remove {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 16px;
  height: 16px;
  padding: 0;
  border: none;
  background: transparent;
  color: var(--accent-color);
  cursor: pointer;
  border-radius: 50%;
  transition: all 0.15s;
  flex-shrink: 0;
}

.file-tag-remove:hover {
  background: var(--accent-color);
  color: white;
}

.file-tag-remove span {
  font-size: 12px;
}

/* 内嵌图片预览区域 */
.inline-preview {
  flex-shrink: 0;
  padding: 8px 12px;
  border-top: 1px solid var(--border-subtle);
  background: var(--bg-secondary);
}

/* 图片预览过渡动画 */
.slide-up-enter-active,
.slide-up-leave-active {
  transition: all 0.25s ease-out;
  overflow: hidden;
}

.slide-up-enter-from,
.slide-up-leave-to {
  opacity: 0;
  max-height: 0;
  padding-top: 0;
  padding-bottom: 0;
}

.slide-up-enter-to,
.slide-up-leave-from {
  opacity: 1;
  max-height: 100px;
}

/* 分割线 */
.splitter {
  flex-shrink: 0;
  background: transparent;
  position: relative;
  z-index: 10;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}
.splitter:hover { 
  background: var(--accent-light);
}
.splitter-vertical {
  width: 100%;
  height: 12px;
  cursor: row-resize;
  margin: -3px 0;
}
.splitter-horizontal {
  width: 12px;
  height: 100%;
  cursor: col-resize;
  margin: 0 3px;
}
.splitter-handle {
  position: absolute;
  background: var(--text-muted);
  border-radius: 3px;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}
.splitter-vertical .splitter-handle {
  width: 48px;
  height: 4px;
  left: 50%;
  top: 50%;
  transform: translate(-50%, -50%);
}
.splitter-horizontal .splitter-handle {
  width: 4px;
  height: 48px;
  left: 50%;
  top: 50%;
  transform: translate(-50%, -50%);
}
.splitter:hover .splitter-handle { 
  background: var(--accent-color);
  box-shadow: var(--shadow-glow);
}
.is-dragging .splitter { 
  background: var(--accent-light);
}
.is-dragging .splitter .splitter-handle {
  background: var(--accent-color);
  box-shadow: var(--shadow-glow);
}

/* 底部工具栏 */
.bottom-toolbar {
  flex-shrink: 0;
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  background: var(--bg-glass);
  backdrop-filter: blur(var(--blur-amount)) saturate(var(--backdrop-saturate));
  -webkit-backdrop-filter: blur(var(--blur-amount)) saturate(var(--backdrop-saturate));
  border-top: 1px solid var(--border-color);
}
.toolbar-left, .toolbar-right {
  display: flex;
  gap: 6px;
  align-items: center;
}
.tool-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 38px;
  height: 38px;
  border: none;
  border-radius: 10px;
  background: var(--bg-card);
  backdrop-filter: blur(10px);
  -webkit-backdrop-filter: blur(10px);
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  border: 1px solid var(--border-subtle);
}
.tool-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
  transform: translateY(-1px);
  box-shadow: var(--shadow-sm);
}
.tool-btn:active {
  transform: translateY(0);
}
.tool-btn.active {
  background: var(--accent-color);
  color: white;
  border-color: var(--accent-color);
}
.tool-btn.active:hover {
  background: var(--accent-color);
  opacity: 0.9;
}
.cancel-btn {
  padding: 10px 20px;
  border: 1px solid var(--border-color);
  border-radius: 10px;
  background: var(--bg-card);
  backdrop-filter: blur(10px);
  -webkit-backdrop-filter: blur(10px);
  color: var(--text-secondary);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}
.cancel-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
  transform: translateY(-1px);
  box-shadow: var(--shadow-sm);
}
.submit-btn {
  padding: 10px 24px;
  border: none;
  border-radius: 10px;
  background: var(--accent-gradient);
  color: white;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  box-shadow: var(--shadow-sm);
}
.submit-btn:hover { 
  transform: translateY(-1px);
  box-shadow: var(--shadow-md), var(--shadow-glow);
}
.submit-btn:active {
  transform: translateY(0);
}
</style>

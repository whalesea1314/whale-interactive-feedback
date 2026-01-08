<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { marked } from 'marked'
import { useConfigStore } from '@/stores/config'
import { useFeedbackStore } from '@/stores/feedback'
import { useCannedResponsesStore } from '@/stores/cannedResponses'
import { useDisplayMode } from '@/composables/useDisplayMode'
import { useImageHandler } from '@/composables/useImageHandler'
import { useFileHandler } from '@/composables/useFileHandler'
import { useAudio } from '@/composables/useAudio'
import { useWindowControl } from '@/composables/useWindowControl'
import { useLayout } from '@/composables/useLayout'
import { usePlatform, getPasteShortcutText } from '@/composables/usePlatform'
import ImagePreview from './ImagePreview.vue'
import CannedResponsesPopup from './CannedResponsesPopup.vue'
import CannedResponsesManager from './CannedResponsesManager.vue'
import TextOptimizer from './TextOptimizer.vue'
import ScreenshotOverlay from './ScreenshotOverlay.vue'
import type { ScreenshotResult } from '@/composables/useScreenshot'

const configStore = useConfigStore()
const feedbackStore = useFeedbackStore()
const cannedStore = useCannedResponsesStore()

// 窗口控制 composable - Requirement 13.4
const { handleAfterSubmit } = useWindowControl()

// 布局控制 composable - Requirement 9.1
const { isVertical, toggleLayout } = useLayout()

// 跨平台支持 composable - Requirements 15.1, 15.2, 15.3
const { 
  platform, 
  isPasteShortcut, 
  isUndoShortcut, 
  pasteShortcutText 
} = usePlatform()

// 图片处理 composable
const { handlePaste, handleDrop, isProcessing: isImageProcessing } = useImageHandler()

// 文件处理 composable - Requirement 4.1, 4.2, 4.3
const { 
  handleFileDrop, 
  classifyFiles, 
  classifyPaths,
  openFileDialog,
  isImageFile: checkIsImage,
  createFileReferencesFromPaths
} = useFileHandler()

// 音频通知 composable - Requirement 12.1
const { playNotification } = useAudio()

// Props - 从 MCP 调用接收的参数
const props = defineProps<{
  message?: string
  fullResponse?: string
  predefinedOptions?: string[]
  showToolbar?: boolean  // 是否显示工具栏，默认 true
  toolbarOnly?: boolean  // 是否只显示工具栏
}>()

// Emits
const emit = defineEmits<{
  (e: 'submit', feedback: { text: string; selectedOptions: string[] }): void
  (e: 'cancel'): void
  (e: 'open-settings'): void
}>()

// 文本输入引用
const textareaRef = ref<HTMLTextAreaElement | null>(null)

// 常用语弹窗状态 - Requirement 5.1, 5.2
const showCannedPopup = ref(false)
const showCannedManager = ref(false)
const cannedBtnRef = ref<HTMLButtonElement | null>(null)
const cannedBtnRect = ref<DOMRect | undefined>(undefined)

// 文本优化弹窗状态 - Requirement 6.1
const showTextOptimizer = ref(false)

// 截图覆盖层状态 - Requirement 8.1, 8.5
const showScreenshotOverlay = ref(false)

/**
 * 打开截图覆盖层
 * Requirement 8.1: 点击截图按钮触发截图工具
 */
function openScreenshotOverlay() {
  showScreenshotOverlay.value = true
}

/**
 * 关闭截图覆盖层
 */
function closeScreenshotOverlay() {
  showScreenshotOverlay.value = false
}

/**
 * 处理截图结果
 * Requirement 8.5: 截图完成后自动添加到反馈内容
 */
function handleScreenshotCapture(result: ScreenshotResult) {
  // 生成唯一 ID
  const id = `screenshot-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`
  
  // 添加截图到反馈
  feedbackStore.addImage({
    id,
    data: result.data,
    mimeType: result.mime_type,
    width: result.width,
    height: result.height,
    size: result.size,
  })
  
  // 关闭覆盖层
  showScreenshotOverlay.value = false
}

// 显示模式参数
const displayParams = computed(() => ({
  message: props.message,
  fullResponse: props.fullResponse,
}))

// 输入框占位符文本 - 根据平台显示正确的快捷键
// Requirements: 15.1, 15.2, 15.3
const inputPlaceholder = computed(() => {
  return `输入您的反馈... (Enter 发送, Shift+Enter 换行, ${pasteShortcutText.value} 粘贴图片)`
})

// 使用显示模式 composable
// Requirement 2.1, 11.1, 11.2, 11.3, 11.4, 11.5
const { displayMode, displayContent, toggleDisplayMode } = useDisplayMode(displayParams)

// 配置 marked 选项
marked.setOptions({
  breaks: true,
  gfm: true,
})

// 渲染 Markdown 内容为 HTML
const renderedContent = computed(() => {
  if (!displayContent.value) return ''
  try {
    return marked(displayContent.value) as string
  } catch {
    return displayContent.value
  }
})

// 初始化预定义选项
onMounted(() => {
  if (props.predefinedOptions && props.predefinedOptions.length > 0) {
    feedbackStore.setPredefinedOptions(props.predefinedOptions)
  }
  // 聚焦到输入框
  textareaRef.value?.focus()
  
  // 注意：粘贴事件监听已移至 App.vue，避免重复监听
  
  // 加载常用语 - Requirement 5.1
  cannedStore.loadResponses()
  
  // 播放通知音 - Requirement 12.1
  // WHEN the Feedback_Window opens THEN the Audio_Notifier SHALL play a notification sound
  playNotification()
})

// 清理
onUnmounted(() => {
  feedbackStore.reset()
  // 粘贴事件监听已移至 App.vue
})

// 处理粘贴事件 - Requirement 3.1
async function onPaste(event: ClipboardEvent) {
  const images = await handlePaste(event)
  if (images.length > 0) {
    event.preventDefault()
    for (const image of images) {
      feedbackStore.addImage(image)
    }
  }
}

// 拖拽状态
const isDragging = ref(false)

// 处理拖拽进入 - Requirement 3.2
function onDragEnter(event: DragEvent) {
  event.preventDefault()
  event.stopPropagation()
  isDragging.value = true
}

// 处理拖拽悬停
function onDragOver(event: DragEvent) {
  event.preventDefault()
  event.stopPropagation()
  isDragging.value = true
}

// 处理拖拽离开
function onDragLeave(event: DragEvent) {
  event.preventDefault()
  event.stopPropagation()
  // 检查是否真的离开了容器
  const rect = (event.currentTarget as HTMLElement).getBoundingClientRect()
  const x = event.clientX
  const y = event.clientY
  if (x < rect.left || x >= rect.right || y < rect.top || y >= rect.bottom) {
    isDragging.value = false
  }
}

// 处理拖拽放置 - Requirement 3.2, 4.1, 4.3
async function onDrop(event: DragEvent) {
  event.preventDefault()
  event.stopPropagation()
  isDragging.value = false
  
  const files = event.dataTransfer?.files
  if (!files || files.length === 0) return
  
  // 分类文件：图片和非图片 - Requirement 4.3
  const { images: imageFiles, others: otherFiles } = classifyFiles(files)
  
  // 处理图片文件 - Requirement 3.2
  if (imageFiles.length > 0) {
    // 创建一个模拟的 DataTransfer 对象来传递图片文件
    const imageDataTransfer = new DataTransfer()
    for (const file of imageFiles) {
      imageDataTransfer.items.add(file)
    }
    const imageEvent = { dataTransfer: imageDataTransfer } as DragEvent
    const images = await handleDrop(imageEvent)
    for (const image of images) {
      feedbackStore.addImage(image)
    }
  }
  
  // 处理非图片文件 - Requirement 4.1
  if (otherFiles.length > 0) {
    const fileDataTransfer = new DataTransfer()
    for (const file of otherFiles) {
      fileDataTransfer.items.add(file)
    }
    const fileEvent = { dataTransfer: fileDataTransfer } as DragEvent
    const fileRefs = await handleFileDrop(fileEvent)
    for (const fileRef of fileRefs) {
      feedbackStore.addFileReference(fileRef)
      // 在文本输入中插入文件引用 - Requirement 4.4
      insertFileReferenceToText(fileRef.displayName)
    }
  }
}

// 处理选项切换
// Requirement 2.2
function toggleOption(index: number) {
  feedbackStore.toggleOption(index)
}

// 处理键盘事件
// Requirement 2.3, 2.4
function handleKeydown(event: KeyboardEvent) {
  if (event.key === 'Enter' && !event.shiftKey) {
    event.preventDefault()
    submitFeedback()
  }
  // Shift+Enter 会自动换行，无需处理
}

/**
 * 在文本输入中插入文件引用
 * Requirement 4.4: 添加文件后定位光标以便继续输入
 */
function insertFileReferenceToText(displayName: string) {
  const textarea = textareaRef.value
  if (!textarea) {
    // 如果没有 textarea，直接追加到文本末尾
    feedbackStore.text = feedbackStore.text 
      ? `${feedbackStore.text} ${displayName} `
      : `${displayName} `
    return
  }
  
  const start = textarea.selectionStart
  const end = textarea.selectionEnd
  const text = feedbackStore.text
  
  // 在光标位置插入文件引用
  const before = text.substring(0, start)
  const after = text.substring(end)
  const insertText = `${displayName} `
  
  feedbackStore.text = before + insertText + after
  
  // 设置光标位置到插入内容之后
  const newPosition = start + insertText.length
  
  // 使用 nextTick 确保 DOM 更新后再设置光标
  setTimeout(() => {
    textarea.focus()
    textarea.setSelectionRange(newPosition, newPosition)
  }, 0)
}

/**
 * 处理文件选择按钮点击
 * Requirement 4.2: 打开文件选择对话框支持多文件选择
 */
async function handleSelectFiles() {
  const fileRefs = await openFileDialog()
  
  if (fileRefs.length === 0) return
  
  // 分类文件：图片和非图片 - Requirement 4.3
  const { imagePaths, otherPaths } = classifyPaths(fileRefs.map(f => f.path))
  
  // 处理图片文件 - 需要读取并添加为图片
  // 注意：这里我们暂时将图片也作为文件引用处理
  // 如果需要将图片作为附件，需要调用后端读取文件内容
  
  // 处理所有文件引用
  for (const fileRef of fileRefs) {
    if (!fileRef.isImage) {
      // 非图片文件作为引用
      feedbackStore.addFileReference(fileRef)
      insertFileReferenceToText(fileRef.displayName)
    } else {
      // 图片文件也作为引用（用户可以选择是否作为附件）
      feedbackStore.addFileReference(fileRef)
      insertFileReferenceToText(fileRef.displayName)
    }
  }
}

// 提交反馈
// Requirement 2.5, 2.6, 13.4
async function submitFeedback() {
  const selectedTexts = Array.from(feedbackStore.selectedOptions)
    .map(index => feedbackStore.predefinedOptions[index])
    .filter(Boolean)
  
  const userText = feedbackStore.text.trim()
  
  // 组合选中的选项和用户输入
  const combinedText = [...selectedTexts, userText]
    .filter(t => t.length > 0)
    .join('\n')

  // 如果没有任何内容，返回默认消息
  // Requirement 2.6
  if (!combinedText && selectedTexts.length === 0) {
    emit('submit', { 
      text: '[User provided no feedback]', 
      selectedOptions: [] 
    })
    feedbackStore.reset()
    // Requirement 13.4: 提交后自动最小化
    await handleAfterSubmit()
    return
  }

  emit('submit', { 
    text: combinedText, 
    selectedOptions: selectedTexts 
  })
  
  // 清空输入
  feedbackStore.reset()
  
  // Requirement 13.4: 提交后自动最小化
  await handleAfterSubmit()
}

// 取消反馈
function cancelFeedback() {
  emit('cancel')
  feedbackStore.reset()
}

// 常用语悬停显示弹窗 - Requirement 5.1
function handleCannedMouseEnter() {
  if (cannedBtnRef.value) {
    cannedBtnRect.value = cannedBtnRef.value.getBoundingClientRect()
  }
  showCannedPopup.value = true
}

// 常用语鼠标离开
function handleCannedMouseLeave() {
  // 延迟关闭，让用户有时间移动到弹窗
  setTimeout(() => {
    if (!showCannedPopup.value) return
    showCannedPopup.value = false
  }, 100)
}

// 常用语按钮点击 - Requirement 5.3
function handleCannedClick() {
  showCannedPopup.value = false
  showCannedManager.value = true
}

// 插入常用语到文本 - Requirement 5.2
function insertCannedResponse(text: string) {
  const textarea = textareaRef.value
  if (!textarea) {
    feedbackStore.text = feedbackStore.text 
      ? `${feedbackStore.text}${text}`
      : text
    return
  }
  
  const start = textarea.selectionStart
  const end = textarea.selectionEnd
  const currentText = feedbackStore.text
  
  // 在光标位置插入常用语
  const before = currentText.substring(0, start)
  const after = currentText.substring(end)
  
  feedbackStore.text = before + text + after
  
  // 设置光标位置到插入内容之后
  const newPosition = start + text.length
  
  setTimeout(() => {
    textarea.focus()
    textarea.setSelectionRange(newPosition, newPosition)
  }, 0)
}

// 关闭常用语弹窗
function closeCannedPopup() {
  showCannedPopup.value = false
}

// 打开常用语管理
function openCannedManager() {
  showCannedManager.value = true
}

// 关闭常用语管理
function closeCannedManager() {
  showCannedManager.value = false
}

// 打开文本优化器 - Requirement 6.1
function openTextOptimizer() {
  showTextOptimizer.value = true
}

// 关闭文本优化器
function closeTextOptimizer() {
  showTextOptimizer.value = false
}

// 应用优化结果到输入框 - Requirement 6.3
function applyOptimizedText(text: string) {
  feedbackStore.text = text
  showTextOptimizer.value = false
  // 聚焦到输入框
  setTimeout(() => {
    textareaRef.value?.focus()
  }, 0)
}

// 打开设置面板
function openSettings() {
  emit('open-settings')
}
</script>

<template>
  <!-- 仅工具栏模式 -->
  <div
    v-if="props.toolbarOnly"
    class="toolbar-only"
  >
    <div class="toolbar">
      <div class="toolbar-left">
        <button
          class="tool-btn"
          title="选择文件"
          @click="handleSelectFiles"
        >
          <span class="i-carbon-document-add" />
        </button>
        <button
          class="tool-btn"
          title="截图"
          @click="openScreenshotOverlay"
        >
          <span class="i-carbon-screen" />
        </button>
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
      </div>
      <div class="toolbar-right">
        <button 
          class="tool-btn" 
          :title="isVertical ? '切换到左右布局' : '切换到上下布局'"
          @click="toggleLayout"
        >
          <span :class="isVertical ? 'i-carbon-column' : 'i-carbon-row'" />
        </button>
        <button
          class="tool-btn"
          title="设置"
          @click="openSettings"
        >
          <span class="i-carbon-settings" />
        </button>
        <button
          class="tool-btn"
          title="优化"
          @click="openTextOptimizer"
        >
          <span class="i-carbon-magic-wand" />
        </button>
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
          发送
        </button>
      </div>
    </div>
    
    <!-- 常用语预览弹窗 -->
    <CannedResponsesPopup
      :visible="showCannedPopup"
      :trigger-rect="cannedBtnRect"
      @insert="insertCannedResponse"
      @close="closeCannedPopup"
      @open-manager="openCannedManager"
    />
    
    <!-- 常用语管理对话框 -->
    <CannedResponsesManager
      :visible="showCannedManager"
      @close="closeCannedManager"
      @insert="insertCannedResponse"
    />
    
    <!-- 文本优化器 -->
    <TextOptimizer
      :visible="showTextOptimizer"
      :initial-text="feedbackStore.text"
      @close="closeTextOptimizer"
      @apply="applyOptimizedText"
    />
    
    <!-- 截图覆盖层 -->
    <ScreenshotOverlay
      :visible="showScreenshotOverlay"
      @close="closeScreenshotOverlay"
      @capture="handleScreenshotCapture"
    />
  </div>
  
  <!-- 完整内容模式 -->
  <div 
    v-else
    class="feedback-window"
    :class="{ 'is-dragging': isDragging }"
    @dragenter="onDragEnter"
    @dragover="onDragOver"
    @dragleave="onDragLeave"
    @drop="onDrop"
  >
    <!-- 拖拽覆盖层 - Requirement 3.2, 4.1 -->
    <div
      v-if="isDragging"
      class="drag-overlay"
    >
      <div class="drag-content">
        <span class="i-carbon-document-add drag-icon" />
        <span class="drag-text">释放以添加文件</span>
        <span class="drag-hint">图片将作为附件，其他文件将作为引用</span>
      </div>
    </div>
    
    <!-- 显示内容区域 -->
    <!-- Requirement 2.1: 根据显示模式显示 message 或 full_response -->
    <div
      v-if="displayContent"
      class="display-area"
    >
      <div class="display-header">
        <span class="display-mode-label">
          {{ displayMode === 'simple' ? '简洁模式' : '完整模式' }}
        </span>
        <button 
          class="mode-toggle-btn" 
          :title="displayMode === 'simple' ? '切换到完整模式' : '切换到简洁模式'"
          @click="toggleDisplayMode"
        >
          <span :class="displayMode === 'simple' ? 'i-carbon-expand-all' : 'i-carbon-collapse-all'" />
        </button>
      </div>
      <div
        class="display-content prose"
        v-html="renderedContent"
      />
    </div>

    <!-- 预定义选项 -->
    <!-- Requirement 2.2: 显示可选择的复选框选项 -->
    <div
      v-if="feedbackStore.predefinedOptions.length > 0"
      class="options-area"
    >
      <div class="options-label">
        选择选项：
      </div>
      <div 
        v-for="(option, index) in feedbackStore.predefinedOptions" 
        :key="index"
        class="option-item"
        @click="toggleOption(index)"
      >
        <input 
          type="checkbox" 
          :checked="feedbackStore.selectedOptions.has(index)"
          class="option-checkbox"
          @click.stop
          @change="toggleOption(index)"
        >
        <span class="option-text">{{ option }}</span>
      </div>
    </div>

    <!-- 输入区域 -->
    <div class="input-area">
      <textarea
        ref="textareaRef"
        v-model="feedbackStore.text"
        :placeholder="inputPlaceholder"
        class="feedback-input"
        @keydown="handleKeydown"
      />
      
      <!-- 图片预览区域 - Requirement 3.3, 3.4, 3.5 -->
      <ImagePreview
        :images="feedbackStore.images"
        @remove="feedbackStore.removeImage"
      />
      
      <!-- 工具栏 -->
      <div class="toolbar">
        <div class="toolbar-left">
          <button
            class="tool-btn"
            title="选择文件"
            @click="handleSelectFiles"
          >
            <span class="i-carbon-document-add" />
          </button>
          <button
            class="tool-btn"
            title="截图"
            @click="openScreenshotOverlay"
          >
            <span class="i-carbon-screen" />
          </button>
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
        </div>
        <div class="toolbar-right">
          <button 
            class="tool-btn" 
            :title="isVertical ? '切换到左右布局' : '切换到上下布局'"
            @click="toggleLayout"
          >
            <span :class="isVertical ? 'i-carbon-column' : 'i-carbon-row'" />
          </button>
          <button
            class="tool-btn"
            title="设置"
            @click="openSettings"
          >
            <span class="i-carbon-settings" />
          </button>
          <button
            class="tool-btn"
            title="优化"
            @click="openTextOptimizer"
          >
            <span class="i-carbon-magic-wand" />
          </button>
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
            发送
          </button>
        </div>
      </div>
    </div>
    
    <!-- 常用语预览弹窗 - Requirement 5.1, 5.2 -->
    <CannedResponsesPopup
      :visible="showCannedPopup"
      :trigger-rect="cannedBtnRect"
      @insert="insertCannedResponse"
      @close="closeCannedPopup"
      @open-manager="openCannedManager"
    />
    
    <!-- 常用语管理对话框 - Requirement 5.3, 5.4, 5.5 -->
    <CannedResponsesManager
      :visible="showCannedManager"
      @close="closeCannedManager"
      @insert="insertCannedResponse"
    />
    
    <!-- 文本优化器 - Requirement 6.1, 6.2, 6.3, 6.4 -->
    <TextOptimizer
      :visible="showTextOptimizer"
      :initial-text="feedbackStore.text"
      @close="closeTextOptimizer"
      @apply="applyOptimizedText"
    />
    
    <!-- 截图覆盖层 - Requirement 8.1, 8.2, 8.3, 8.5 -->
    <ScreenshotOverlay
      :visible="showScreenshotOverlay"
      @close="closeScreenshotOverlay"
      @capture="handleScreenshotCapture"
    />
  </div>
</template>

<style scoped>
.feedback-window {
  display: flex;
  flex-direction: column;
  height: 100%;
  background-color: var(--bg-primary);
  padding: 16px;
  gap: 12px;
  position: relative;
}

/* 拖拽状态 */
.feedback-window.is-dragging {
  border: 2px dashed var(--accent-color);
}

.drag-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(var(--accent-color-rgb, 59, 130, 246), 0.1);
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
  gap: 12px;
  padding: 24px;
  background-color: var(--bg-secondary);
  border-radius: 12px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
}

.drag-icon {
  font-size: 48px;
  color: var(--accent-color);
}

.drag-text {
  font-size: 16px;
  color: var(--text-primary);
  font-weight: 500;
}

.drag-hint {
  font-size: 12px;
  color: var(--text-muted);
  margin-top: -4px;
}

/* 显示内容区域 */
.display-area {
  flex: 1;
  min-height: 100px;
  padding: 12px;
  background-color: var(--bg-secondary);
  border-radius: 8px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
}

.display-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
  padding-bottom: 8px;
  border-bottom: 1px solid var(--border-color);
}

.display-mode-label {
  font-size: 12px;
  color: var(--text-muted);
  font-weight: 500;
}

.mode-toggle-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: none;
  border-radius: 4px;
  background-color: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.2s;
}

.mode-toggle-btn:hover {
  background-color: var(--bg-tertiary);
  color: var(--accent-color);
}

.display-content {
  color: var(--text-primary);
  white-space: pre-wrap;
  word-break: break-word;
  line-height: 1.6;
  font-size: 14px;
  flex: 1;
}

/* 预定义选项区域 */
.options-area {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 12px;
  background-color: var(--bg-secondary);
  border-radius: 8px;
}

.options-label {
  font-size: 12px;
  color: var(--text-muted);
  margin-bottom: 4px;
}

.option-item {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  padding: 10px 12px;
  border-radius: 6px;
  cursor: pointer;
  transition: background-color 0.2s;
  border: 1px solid var(--border-color);
}

.option-item:hover {
  background-color: var(--bg-tertiary);
}

.option-checkbox {
  width: 18px;
  height: 18px;
  min-width: 18px;
  cursor: pointer;
  accent-color: var(--accent-color);
  margin-top: 2px;
}

.option-text {
  color: var(--text-primary);
  line-height: 1.5;
  flex: 1;
}

/* 输入区域 */
.input-area {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.feedback-input {
  width: 100%;
  min-height: 100px;
  padding: 12px;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  background-color: var(--bg-secondary);
  color: var(--text-primary);
  font-size: 14px;
  resize: vertical;
  outline: none;
  transition: border-color 0.2s;
  font-family: inherit;
}

.feedback-input:focus {
  border-color: var(--accent-color);
}

.feedback-input::placeholder {
  color: var(--text-muted);
}

/* 工具栏 */
.toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.toolbar-left,
.toolbar-right {
  display: flex;
  gap: 8px;
  align-items: center;
}

.tool-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  border: none;
  border-radius: 6px;
  background-color: var(--bg-secondary);
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.2s;
}

.tool-btn:hover {
  background-color: var(--bg-tertiary);
  color: var(--text-primary);
}

.cancel-btn {
  padding: 8px 16px;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  background-color: transparent;
  color: var(--text-secondary);
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s;
}

.cancel-btn:hover {
  background-color: var(--bg-tertiary);
  color: var(--text-primary);
}

.submit-btn {
  padding: 8px 20px;
  border: none;
  border-radius: 6px;
  background-color: var(--accent-color);
  color: white;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: background-color 0.2s;
}

.submit-btn:hover {
  background-color: var(--accent-hover);
}
</style>

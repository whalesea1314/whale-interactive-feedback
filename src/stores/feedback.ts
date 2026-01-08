import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { FeedbackContent, FeedbackData, ImagePreviewData, FileReference } from '@/types'

export const useFeedbackStore = defineStore('feedback', () => {
  // 状态
  const text = ref('')
  const images = ref<ImagePreviewData[]>([])
  const fileReferences = ref<FileReference[]>([])
  const predefinedOptions = ref<string[]>([])
  const selectedOptions = ref<Set<number>>(new Set())
  const displayContent = ref('')
  const isSubmitting = ref(false)

  // 原始文本（用于撤销优化）
  const originalText = ref<string | null>(null)

  // 计算属性
  const hasContent = computed(() => {
    return text.value.trim().length > 0 || 
           images.value.length > 0 || 
           fileReferences.value.length > 0 ||
           selectedOptions.value.size > 0
  })

  const feedbackContent = computed<FeedbackContent[]>(() => {
    const content: FeedbackContent[] = []

    // 添加选中的选项文本
    const selectedTexts = Array.from(selectedOptions.value)
      .map(index => predefinedOptions.value[index])
      .filter(Boolean)

    // 组合文本内容
    const combinedText = [...selectedTexts, text.value.trim()]
      .filter(t => t.length > 0)
      .join('\n')

    if (combinedText) {
      content.push({ type: 'text', text: combinedText })
    }

    // 添加图片
    for (const img of images.value) {
      content.push({
        type: 'image',
        data: img.data,
        mimeType: img.mimeType,
      })
    }

    // 添加文件引用
    for (const file of fileReferences.value) {
      content.push({
        type: 'file_reference',
        display_name: file.displayName,
        path: file.path,
      })
    }

    return content
  })

  // 方法
  function setText(value: string) {
    text.value = value
  }

  function addImage(image: ImagePreviewData) {
    images.value.push(image)
  }

  function removeImage(id: string) {
    const index = images.value.findIndex(img => img.id === id)
    if (index !== -1) {
      images.value.splice(index, 1)
    }
  }

  function addFileReference(file: FileReference) {
    fileReferences.value.push(file)
  }

  function removeFileReference(id: string) {
    const index = fileReferences.value.findIndex(f => f.id === id)
    if (index !== -1) {
      fileReferences.value.splice(index, 1)
    }
  }

  function setPredefinedOptions(options: string[]) {
    predefinedOptions.value = options
    selectedOptions.value.clear()
  }

  function toggleOption(index: number) {
    if (selectedOptions.value.has(index)) {
      selectedOptions.value.delete(index)
    } else {
      selectedOptions.value.add(index)
    }
  }

  function setDisplayContent(content: string) {
    displayContent.value = content
  }

  function saveOriginalText() {
    originalText.value = text.value
  }

  function restoreOriginalText() {
    if (originalText.value !== null) {
      text.value = originalText.value
      originalText.value = null
      return true
    }
    return false
  }

  function clearOriginalText() {
    originalText.value = null
  }

  function reset() {
    text.value = ''
    images.value = []
    fileReferences.value = []
    predefinedOptions.value = []
    selectedOptions.value.clear()
    displayContent.value = ''
    originalText.value = null
    isSubmitting.value = false
  }

  /**
   * 提交反馈到后端
   * 调用 Tauri 命令将反馈数据序列化并发送
   * 
   * @returns 序列化后的 JSON 字符串
   * @throws 如果提交失败则抛出错误
   */
  async function submitFeedback(): Promise<string> {
    isSubmitting.value = true
    
    try {
      const feedbackData: FeedbackData = {
        content: feedbackContent.value
      }
      
      // 调用 Tauri 命令提交反馈
      const response = await invoke<string>('submit_feedback', {
        feedback: feedbackData
      })
      
      return response
    } finally {
      isSubmitting.value = false
    }
  }

  return {
    // 状态
    text,
    images,
    fileReferences,
    predefinedOptions,
    selectedOptions,
    displayContent,
    isSubmitting,
    originalText,
    // 计算属性
    hasContent,
    feedbackContent,
    // 方法
    setText,
    addImage,
    removeImage,
    addFileReference,
    removeFileReference,
    setPredefinedOptions,
    toggleOption,
    setDisplayContent,
    saveOriginalText,
    restoreOriginalText,
    clearOriginalText,
    reset,
    submitFeedback,
  }
})

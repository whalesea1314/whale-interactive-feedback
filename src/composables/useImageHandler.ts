import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { readFile } from '@tauri-apps/plugin-fs'
import type { ImagePreviewData } from '@/types'

// 支持的图片 MIME 类型
const SUPPORTED_IMAGE_TYPES = [
  'image/png',
  'image/jpeg',
  'image/jpg',
  'image/gif',
  'image/webp',
  'image/bmp',
]

// 图片大小限制（5MB）
const MAX_IMAGE_SIZE = 5 * 1024 * 1024

// 图片尺寸限制（最大宽高）
const MAX_IMAGE_DIMENSION = 4096

// 生成唯一 ID
function generateId(): string {
  return `img_${Date.now()}_${Math.random().toString(36).substring(2, 9)}`
}

/**
 * 格式化文件大小
 */
function formatFileSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}

/**
 * 图片处理 composable
 * 处理图片粘贴、拖拽和处理逻辑
 */
export function useImageHandler() {
  const isProcessing = ref(false)
  const error = ref<string | null>(null)

  /**
   * 从 File 对象读取图片数据
   */
  async function readFileAsArrayBuffer(file: File): Promise<ArrayBuffer> {
    return new Promise((resolve, reject) => {
      const reader = new FileReader()
      reader.onload = () => resolve(reader.result as ArrayBuffer)
      reader.onerror = () => reject(new Error('读取文件失败'))
      reader.readAsArrayBuffer(file)
    })
  }

  /**
   * 从 Blob 读取为 Base64
   */
  async function readBlobAsBase64(blob: Blob): Promise<string> {
    return new Promise((resolve, reject) => {
      const reader = new FileReader()
      reader.onload = () => {
        const result = reader.result as string
        // 移除 data:xxx;base64, 前缀
        const base64 = result.split(',')[1]
        resolve(base64)
      }
      reader.onerror = () => reject(new Error('读取文件失败'))
      reader.readAsDataURL(blob)
    })
  }

  /**
   * 获取图片尺寸
   */
  async function getImageDimensions(base64: string, mimeType: string): Promise<{ width: number; height: number }> {
    return new Promise((resolve, reject) => {
      const img = new Image()
      img.onload = () => {
        resolve({ width: img.naturalWidth, height: img.naturalHeight })
      }
      img.onerror = () => reject(new Error('加载图片失败'))
      img.src = `data:${mimeType};base64,${base64}`
    })
  }

  /**
   * 验证图片大小
   * @throws 如果图片超过大小限制
   */
  function validateImageSize(size: number): void {
    if (size > MAX_IMAGE_SIZE) {
      throw new Error(`图片大小超过限制（最大 ${formatFileSize(MAX_IMAGE_SIZE)}，当前 ${formatFileSize(size)}）`)
    }
  }

  /**
   * 前端压缩图片（当后端处理失败时使用）
   * 使用 Canvas 进行压缩
   */
  async function compressImage(
    base64: string, 
    mimeType: string, 
    maxDimension: number = MAX_IMAGE_DIMENSION,
    quality: number = 0.85
  ): Promise<{ data: string; mimeType: string; width: number; height: number; size: number }> {
    return new Promise((resolve, reject) => {
      const img = new Image()
      img.onload = () => {
        let { naturalWidth: width, naturalHeight: height } = img
        
        // 计算缩放比例
        if (width > maxDimension || height > maxDimension) {
          const ratio = Math.min(maxDimension / width, maxDimension / height)
          width = Math.round(width * ratio)
          height = Math.round(height * ratio)
        }
        
        // 创建 Canvas 进行压缩
        const canvas = document.createElement('canvas')
        canvas.width = width
        canvas.height = height
        const ctx = canvas.getContext('2d')
        
        if (!ctx) {
          reject(new Error('无法创建 Canvas 上下文'))
          return
        }
        
        ctx.drawImage(img, 0, 0, width, height)
        
        // 输出为 JPEG（更好的压缩率）或保持原格式
        const outputType = mimeType === 'image/png' ? 'image/png' : 'image/jpeg'
        const dataUrl = canvas.toDataURL(outputType, quality)
        const compressedBase64 = dataUrl.split(',')[1]
        const size = Math.round(compressedBase64.length * 0.75) // Base64 大小估算
        
        resolve({
          data: compressedBase64,
          mimeType: outputType,
          width,
          height,
          size
        })
      }
      img.onerror = () => reject(new Error('加载图片失败'))
      img.src = `data:${mimeType};base64,${base64}`
    })
  }

  /**
   * 处理图片数据（调用后端进行压缩和转换）
   * Requirement 3.6, 3.7, 3.8
   */
  async function processImage(imageData: number[]): Promise<{ data: string; mimeType: string; size: number }> {
    try {
      const result = await invoke<{ data: string; mime_type: string; size: number }>('process_image', {
        imageData
      })
      return {
        data: result.data,
        mimeType: result.mime_type,
        size: result.size
      }
    } catch (err) {
      // 如果后端处理失败，返回原始数据
      console.warn('后端图片处理失败，使用原始数据:', err)
      throw err
    }
  }

  /**
   * 从 File 创建 ImagePreviewData
   */
  async function createImageFromFile(file: File): Promise<ImagePreviewData> {
    if (!SUPPORTED_IMAGE_TYPES.includes(file.type)) {
      throw new Error(`不支持的图片格式: ${file.type}`)
    }

    // 验证文件大小
    validateImageSize(file.size)

    const arrayBuffer = await readFileAsArrayBuffer(file)
    const imageData = Array.from(new Uint8Array(arrayBuffer))
    
    try {
      // 尝试调用后端处理
      const processed = await processImage(imageData)
      const dimensions = await getImageDimensions(processed.data, processed.mimeType)
      
      return {
        id: generateId(),
        data: processed.data,
        mimeType: processed.mimeType,
        width: dimensions.width,
        height: dimensions.height,
        size: processed.size
      }
    } catch {
      // 后端处理失败，使用前端处理
      const base64 = await readBlobAsBase64(file)
      
      // 检查是否需要压缩
      const dimensions = await getImageDimensions(base64, file.type)
      if (dimensions.width > MAX_IMAGE_DIMENSION || dimensions.height > MAX_IMAGE_DIMENSION || file.size > MAX_IMAGE_SIZE / 2) {
        // 需要压缩
        const compressed = await compressImage(base64, file.type)
        return {
          id: generateId(),
          ...compressed
        }
      }
      
      return {
        id: generateId(),
        data: base64,
        mimeType: file.type,
        width: dimensions.width,
        height: dimensions.height,
        size: file.size
      }
    }
  }

  /**
   * 从 Blob 创建 ImagePreviewData
   */
  async function createImageFromBlob(blob: Blob, mimeType?: string): Promise<ImagePreviewData> {
    const type = mimeType || blob.type || 'image/png'
    
    if (!SUPPORTED_IMAGE_TYPES.includes(type)) {
      throw new Error(`不支持的图片格式: ${type}`)
    }

    // 验证文件大小
    validateImageSize(blob.size)

    const arrayBuffer = await blob.arrayBuffer()
    const imageData = Array.from(new Uint8Array(arrayBuffer))
    
    try {
      // 尝试调用后端处理
      const processed = await processImage(imageData)
      const dimensions = await getImageDimensions(processed.data, processed.mimeType)
      
      return {
        id: generateId(),
        data: processed.data,
        mimeType: processed.mimeType,
        width: dimensions.width,
        height: dimensions.height,
        size: processed.size
      }
    } catch {
      // 后端处理失败，使用前端处理
      const base64 = await readBlobAsBase64(blob)
      
      // 检查是否需要压缩
      const dimensions = await getImageDimensions(base64, type)
      if (dimensions.width > MAX_IMAGE_DIMENSION || dimensions.height > MAX_IMAGE_DIMENSION || blob.size > MAX_IMAGE_SIZE / 2) {
        // 需要压缩
        const compressed = await compressImage(base64, type)
        return {
          id: generateId(),
          ...compressed
        }
      }
      
      return {
        id: generateId(),
        data: base64,
        mimeType: type,
        width: dimensions.width,
        height: dimensions.height,
        size: blob.size
      }
    }
  }

  /**
   * 处理粘贴事件
   * Requirement 3.1: 支持 Ctrl+V / Cmd+V 粘贴图片
   */
  async function handlePaste(event: ClipboardEvent): Promise<ImagePreviewData[]> {
    const items = event.clipboardData?.items
    if (!items) return []

    const images: ImagePreviewData[] = []
    isProcessing.value = true
    error.value = null

    try {
      for (const item of items) {
        if (item.type.startsWith('image/')) {
          const blob = item.getAsFile()
          if (blob) {
            const imageData = await createImageFromBlob(blob, item.type)
            images.push(imageData)
          }
        }
      }
    } catch (err) {
      error.value = err instanceof Error ? err.message : '处理图片失败'
      console.error('处理粘贴图片失败:', err)
    } finally {
      isProcessing.value = false
    }

    return images
  }

  /**
   * 处理拖拽事件
   * Requirement 3.2: 支持拖拽图片文件
   */
  async function handleDrop(event: DragEvent): Promise<ImagePreviewData[]> {
    const files = event.dataTransfer?.files
    if (!files) return []

    const images: ImagePreviewData[] = []
    isProcessing.value = true
    error.value = null

    try {
      for (const file of files) {
        if (file.type.startsWith('image/')) {
          const imageData = await createImageFromFile(file)
          images.push(imageData)
        }
      }
    } catch (err) {
      error.value = err instanceof Error ? err.message : '处理图片失败'
      console.error('处理拖拽图片失败:', err)
    } finally {
      isProcessing.value = false
    }

    return images
  }

  /**
   * 检查文件是否为图片
   */
  function isImageFile(file: File): boolean {
    return SUPPORTED_IMAGE_TYPES.includes(file.type)
  }

  /**
   * 从文件路径创建 ImagePreviewData
   * 用于 Tauri 原生拖放事件
   * @param filePath 文件完整路径
   * @returns ImagePreviewData
   */
  async function createImageFromPath(filePath: string): Promise<ImagePreviewData> {
    // 获取文件扩展名来确定 MIME 类型
    const ext = filePath.split('.').pop()?.toLowerCase() || ''
    const mimeTypeMap: Record<string, string> = {
      'png': 'image/png',
      'jpg': 'image/jpeg',
      'jpeg': 'image/jpeg',
      'gif': 'image/gif',
      'webp': 'image/webp',
      'bmp': 'image/bmp',
    }
    const mimeType = mimeTypeMap[ext] || 'image/png'

    // 使用 Tauri fs 插件读取文件
    const fileData = await readFile(filePath)
    
    // 验证文件大小
    validateImageSize(fileData.length)
    
    const imageData: number[] = Array.from(fileData)

    try {
      // 尝试调用后端处理
      const processed = await processImage(imageData)
      const dimensions = await getImageDimensions(processed.data, processed.mimeType)
      
      return {
        id: generateId(),
        data: processed.data,
        mimeType: processed.mimeType,
        width: dimensions.width,
        height: dimensions.height,
        size: processed.size
      }
    } catch {
      // 后端处理失败，使用前端处理
      const base64 = btoa(String.fromCharCode(...fileData))
      
      // 检查是否需要压缩
      const dimensions = await getImageDimensions(base64, mimeType)
      if (dimensions.width > MAX_IMAGE_DIMENSION || dimensions.height > MAX_IMAGE_DIMENSION || fileData.length > MAX_IMAGE_SIZE / 2) {
        // 需要压缩
        const compressed = await compressImage(base64, mimeType)
        return {
          id: generateId(),
          ...compressed
        }
      }
      
      return {
        id: generateId(),
        data: base64,
        mimeType: mimeType,
        width: dimensions.width,
        height: dimensions.height,
        size: fileData.length
      }
    }
  }

  return {
    isProcessing,
    error,
    handlePaste,
    handleDrop,
    createImageFromFile,
    createImageFromBlob,
    createImageFromPath,
    isImageFile,
    compressImage,
    SUPPORTED_IMAGE_TYPES,
    MAX_IMAGE_SIZE,
    MAX_IMAGE_DIMENSION
  }
}

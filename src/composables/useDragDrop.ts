import { ref, onUnmounted } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { ImagePreviewData, FileReference } from '@/types'

// Tauri 拖放事件 payload 类型
export interface TauriDragDropPayload {
  paths: string[]
  position: { x: number; y: number }
}

export interface DragDropCallbacks {
  /** 处理图片文件 */
  onImageFile: (imagePath: string) => Promise<ImagePreviewData>
  /** 处理其他文件 */
  onOtherFile: (filePath: string) => FileReference
  /** 添加图片到 store */
  addImage: (image: ImagePreviewData) => void
  /** 添加文件引用到 store */
  addFileReference: (file: FileReference) => void
  /** 分类文件路径 */
  classifyPaths: (paths: string[]) => { imagePaths: string[]; otherPaths: string[] }
}

export interface DragDropReturn {
  /** 是否正在拖拽文件 */
  isDraggingFile: ReturnType<typeof ref<boolean>>
  /** 设置 Tauri 原生拖放事件 */
  setupTauriDragDrop: () => Promise<void>
  /** 清理 Tauri 拖放事件监听 */
  cleanupTauriDragDrop: () => void
  /** HTML5 拖拽进入事件处理 */
  onHtml5DragEnter: (event: DragEvent) => void
  /** HTML5 拖拽悬停事件处理 */
  onHtml5DragOver: (event: DragEvent) => void
  /** HTML5 拖拽离开事件处理 */
  onHtml5DragLeave: (event: DragEvent) => void
  /** HTML5 拖放事件处理 */
  onHtml5Drop: (event: DragEvent) => Promise<void>
}

/**
 * 拖放处理 composable
 * 处理 Tauri 原生拖放和 HTML5 拖放事件
 */
export function useDragDrop(callbacks: DragDropCallbacks): DragDropReturn {
  const { onImageFile, onOtherFile, addImage, addFileReference, classifyPaths } = callbacks

  const isDraggingFile = ref(false)

  // Tauri 拖放事件监听器
  const unlistenDragEnter = ref<UnlistenFn | null>(null)
  const unlistenDragDrop = ref<UnlistenFn | null>(null)
  const unlistenDragLeave = ref<UnlistenFn | null>(null)

  /**
   * 设置 Tauri 原生拖放事件
   */
  async function setupTauriDragDrop(): Promise<void> {
    try {
      // 监听拖拽进入
      unlistenDragEnter.value = await listen<TauriDragDropPayload>('tauri://drag-enter', (event) => {
        console.log('[Tauri DragDrop] Enter:', event.payload)
        isDraggingFile.value = true
      })

      // 监听文件释放
      unlistenDragDrop.value = await listen<TauriDragDropPayload>('tauri://drag-drop', async (event) => {
        console.log('[Tauri DragDrop] Drop:', event.payload)
        isDraggingFile.value = false

        const paths = event.payload.paths
        if (!paths || paths.length === 0) {
          console.log('[Tauri DragDrop] No paths in payload')
          return
        }

        console.log('[Tauri DragDrop] Paths received:', paths)

        // 分类文件：图片 vs 其他
        const { imagePaths, otherPaths } = classifyPaths(paths)
        console.log('[Tauri DragDrop] Classified - Images:', imagePaths, 'Others:', otherPaths)

        // 处理图片文件
        for (const imagePath of imagePaths) {
          try {
            console.log('[Tauri DragDrop] Processing image:', imagePath)
            const imageData = await onImageFile(imagePath)
            addImage(imageData)
            console.log('[Tauri DragDrop] Image added:', imagePath)
          } catch (err) {
            console.error('[Tauri DragDrop] Failed to process image:', imagePath, err)
          }
        }

        // 处理其他文件
        for (const filePath of otherPaths) {
          try {
            console.log('[Tauri DragDrop] Processing file:', filePath)
            const fileRef = onOtherFile(filePath)
            addFileReference(fileRef)
            console.log('[Tauri DragDrop] File added:', filePath)
          } catch (err) {
            console.error('[Tauri DragDrop] Failed to process file:', filePath, err)
          }
        }
      })

      // 监听拖拽离开
      unlistenDragLeave.value = await listen('tauri://drag-leave', () => {
        console.log('[Tauri DragDrop] Leave')
        isDraggingFile.value = false
      })

      console.log('[Tauri DragDrop] Event listeners setup complete')
    } catch (err) {
      console.error('[Tauri DragDrop] Failed to setup listeners:', err)
    }
  }

  /**
   * 清理 Tauri 拖放事件监听
   */
  function cleanupTauriDragDrop(): void {
    unlistenDragEnter.value?.()
    unlistenDragDrop.value?.()
    unlistenDragLeave.value?.()
  }

  // HTML5 拖拽事件处理
  function onHtml5DragEnter(event: DragEvent): void {
    event.preventDefault()
    event.stopPropagation()
    isDraggingFile.value = true
    console.log('[HTML5 DragDrop] Enter')
  }

  function onHtml5DragOver(event: DragEvent): void {
    event.preventDefault()
    event.stopPropagation()
  }

  function onHtml5DragLeave(event: DragEvent): void {
    event.preventDefault()
    event.stopPropagation()
    // 检查是否真的离开了容器
    const rect = (event.currentTarget as HTMLElement).getBoundingClientRect()
    const x = event.clientX
    const y = event.clientY
    if (x < rect.left || x > rect.right || y < rect.top || y > rect.bottom) {
      isDraggingFile.value = false
      console.log('[HTML5 DragDrop] Leave')
    }
  }

  /**
   * 处理 FileSystemEntry（支持文件夹）
   */
  async function processEntry(entry: FileSystemEntry): Promise<void> {
    console.log('[HTML5 DragDrop] processEntry:', entry.name, 'isFile:', entry.isFile, 'isDirectory:', entry.isDirectory)

    if (entry.isFile) {
      const fileEntry = entry as FileSystemFileEntry
      return new Promise<void>((resolve, reject) => {
        fileEntry.file(async (file) => {
          console.log('[HTML5 DragDrop] Entry file:', entry.fullPath, file.name, file.type)
          await processFile(file, entry.fullPath)
          resolve()
        }, (err) => {
          console.error('[HTML5 DragDrop] Failed to get file:', entry.fullPath, err)
          reject(err)
        })
      })
    } else if (entry.isDirectory) {
      console.log('[HTML5 DragDrop] Entry directory:', entry.fullPath, entry.name)

      // 添加文件夹引用
      const folderRef: FileReference = {
        id: `folder_${Date.now()}_${Math.random().toString(36).substring(2, 9)}`,
        name: entry.name,
        displayName: `📁 ${entry.name}`,
        path: entry.fullPath,
        size: 0,
        mimeType: 'inode/directory',
        isImage: false,
        isDirectory: true
      }
      addFileReference(folderRef)
      console.log('[HTML5 DragDrop] Folder reference added:', folderRef)
      return Promise.resolve()
    }

    console.log('[HTML5 DragDrop] Unknown entry type:', entry)
    return Promise.resolve()
  }

  /**
   * 处理单个文件
   */
  async function processFile(file: File, fullPath?: string): Promise<void> {
    console.log('[HTML5 DragDrop] Processing file:', file.name, file.type)

    // 检查是否是图片
    if (file.type.startsWith('image/')) {
      return new Promise<void>((resolve, reject) => {
        try {
          const reader = new FileReader()
          reader.onload = () => {
            const base64 = (reader.result as string).split(',')[1]
            const img = new Image()
            img.onload = () => {
              addImage({
                id: `img_${Date.now()}_${Math.random().toString(36).substring(2, 9)}`,
                data: base64,
                mimeType: file.type,
                width: img.naturalWidth,
                height: img.naturalHeight,
                size: file.size
              })
              console.log('[HTML5 DragDrop] Image added:', file.name)
              resolve()
            }
            img.onerror = () => {
              console.error('[HTML5 DragDrop] Failed to load image:', file.name)
              reject(new Error('Failed to load image'))
            }
            img.src = reader.result as string
          }
          reader.onerror = () => {
            console.error('[HTML5 DragDrop] Failed to read file:', file.name)
            reject(reader.error)
          }
          reader.readAsDataURL(file)
        } catch (err) {
          console.error('[HTML5 DragDrop] Failed to process image:', file.name, err)
          reject(err)
        }
      })
    } else {
      // 非图片文件 - 添加为文件引用
      const fileRef: FileReference = {
        id: `file_${Date.now()}_${Math.random().toString(36).substring(2, 9)}`,
        name: file.name,
        displayName: `📄 ${file.name}`,
        path: fullPath || file.name,
        size: file.size,
        mimeType: file.type || 'application/octet-stream',
        isImage: false,
        isDirectory: false
      }
      addFileReference(fileRef)
      console.log('[HTML5 DragDrop] File reference added:', file.name)
    }
  }

  /**
   * HTML5 拖放事件处理
   */
  async function onHtml5Drop(event: DragEvent): Promise<void> {
    event.preventDefault()
    event.stopPropagation()
    isDraggingFile.value = false
    console.log('[HTML5 DragDrop] Drop event triggered')

    const items = event.dataTransfer?.items
    const files = event.dataTransfer?.files

    console.log('[HTML5 DragDrop] items:', items?.length, 'files:', files?.length)

    // 优先使用 webkitGetAsEntry 支持文件夹
    if (items && items.length > 0) {
      console.log('[HTML5 DragDrop] Processing items:', items.length)

      const promises: Promise<void>[] = []

      for (let i = 0; i < items.length; i++) {
        const item = items[i]
        console.log(`[HTML5 DragDrop] Item ${i}: kind=${item.kind}, type=${item.type}`)

        if (item.kind === 'file') {
          const entry = item.webkitGetAsEntry?.()
          console.log(`[HTML5 DragDrop] Entry ${i}:`, entry?.name, entry?.isFile, entry?.isDirectory)

          if (entry) {
            promises.push(processEntry(entry))
          } else {
            // 降级到普通文件处理
            console.log(`[HTML5 DragDrop] No entry, fallback to getAsFile`)
            const file = item.getAsFile()
            if (file) {
              console.log(`[HTML5 DragDrop] Fallback file: ${file.name}`)
              promises.push(processFile(file))
            }
          }
        }
      }

      // 等待所有处理完成
      await Promise.all(promises)
      console.log('[HTML5 DragDrop] All items processed')
      return
    }

    // 降级：使用 files
    if (!files || files.length === 0) {
      console.log('[HTML5 DragDrop] No files')
      return
    }

    console.log('[HTML5 DragDrop] Fallback: Processing files:', files.length)

    for (const file of Array.from(files)) {
      await processFile(file)
    }
  }

  // 组件卸载时自动清理
  onUnmounted(() => {
    cleanupTauriDragDrop()
  })

  return {
    isDraggingFile,
    setupTauriDragDrop,
    cleanupTauriDragDrop,
    onHtml5DragEnter,
    onHtml5DragOver,
    onHtml5DragLeave,
    onHtml5Drop
  }
}

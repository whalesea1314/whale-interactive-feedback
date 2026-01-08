import { ref } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import { normalizePath, detectPlatform } from './usePlatform'
import type { FileReference } from '@/types'

// 支持的图片扩展名
const IMAGE_EXTENSIONS = [
  '.png', '.jpg', '.jpeg', '.gif', '.webp', '.bmp', '.ico', '.svg', '.tiff', '.tif'
]

// 支持的文本类文件扩展名
const TEXT_FILE_EXTENSIONS = [
  // 文档
  'txt', 'md', 'markdown', 'rst', 'adoc',
  // 配置文件
  'json', 'yaml', 'yml', 'xml', 'toml', 'ini', 'conf', 'cfg', 'env', 'properties',
  // Web 前端
  'html', 'htm', 'css', 'scss', 'sass', 'less', 'js', 'mjs', 'cjs', 'ts', 'mts', 'cts',
  'jsx', 'tsx', 'vue', 'svelte', 'astro',
  // 后端语言
  'py', 'pyw', 'pyi', 'rs', 'go', 'java', 'kt', 'kts', 'scala', 'clj', 'cljs',
  'rb', 'php', 'pl', 'pm', 'lua', 'ex', 'exs', 'erl', 'hrl',
  // 系统编程
  'c', 'h', 'cpp', 'cc', 'cxx', 'hpp', 'hxx', 'cs', 'fs', 'fsx',
  // Shell 脚本
  'sh', 'bash', 'zsh', 'fish', 'bat', 'cmd', 'ps1', 'psm1',
  // 数据库
  'sql', 'psql', 'mysql', 'sqlite',
  // 其他
  'graphql', 'gql', 'proto', 'thrift', 'avsc',
  'dockerfile', 'makefile', 'cmake', 'gradle', 'sbt',
  'gitignore', 'gitattributes', 'editorconfig', 'prettierrc', 'eslintrc',
  'log', 'csv', 'tsv'
]

// 生成唯一 ID
function generateId(): string {
  return `file_${Date.now()}_${Math.random().toString(36).substring(2, 9)}`
}

/**
 * 从文件路径获取文件名
 * Requirement 15.4: 使用平台适当的路径分隔符
 * @param path 文件完整路径
 * @returns 文件名（不含路径）
 */
export function getFileName(path: string): string {
  // 规范化路径，统一处理 Windows 和 Unix 路径分隔符
  const normalizedPath = normalizePath(path, 'linux') // 使用 / 作为统一分隔符进行处理
  const parts = normalizedPath.split('/')
  return parts[parts.length - 1] || path
}

/**
 * 获取文件扩展名（小写）
 * @param filename 文件名
 * @returns 扩展名（包含点号，如 .txt）
 */
export function getFileExtension(filename: string): string {
  const lastDot = filename.lastIndexOf('.')
  if (lastDot === -1 || lastDot === filename.length - 1) {
    return ''
  }
  return filename.substring(lastDot).toLowerCase()
}

/**
 * 判断文件是否为图片
 * Requirement 4.3: 区分图片文件和普通文件
 * @param filename 文件名或路径
 * @returns 是否为图片文件
 */
export function isImageFile(filename: string): boolean {
  const ext = getFileExtension(filename)
  return IMAGE_EXTENSIONS.includes(ext)
}

/**
 * 生成文件引用显示名称
 * Requirement 4.1: 生成文件引用格式 (@filename)
 * @param filename 文件名
 * @returns 格式化的显示名称
 */
export function formatFileReference(filename: string): string {
  return `@${filename}`
}

/**
 * 文件处理 composable
 * 处理文件拖拽、选择和引用逻辑
 */
export function useFileHandler() {
  const isProcessing = ref(false)
  const error = ref<string | null>(null)

  /**
   * 从 File 对象创建 FileReference
   * Requirement 4.1, 4.3, 4.5
   */
  function createFileReference(file: File, fullPath?: string): FileReference {
    const filename = file.name
    const path = fullPath || file.name
    
    return {
      id: generateId(),
      displayName: formatFileReference(filename),
      path: path,
      isImage: isImageFile(filename)
    }
  }

  /**
   * 从路径创建 FileReference
   * Requirement 4.1, 4.3, 4.5, 15.4
   * @param path 文件路径
   * @returns 文件引用对象
   */
  function createFileReferenceFromPath(path: string): FileReference {
    const filename = getFileName(path)
    // 规范化路径为当前平台格式 - Requirement 15.4
    const normalizedPath = normalizePath(path, detectPlatform())
    
    return {
      id: generateId(),
      displayName: formatFileReference(filename),
      path: normalizedPath,
      isImage: isImageFile(filename)
    }
  }

  /**
   * 处理拖拽事件中的非图片文件
   * Requirement 4.1: 拖拽文件生成文件引用
   * @param event 拖拽事件
   * @returns 文件引用数组（仅非图片文件）
   */
  async function handleFileDrop(event: DragEvent): Promise<FileReference[]> {
    const files = event.dataTransfer?.files
    if (!files || files.length === 0) return []

    const fileRefs: FileReference[] = []
    isProcessing.value = true
    error.value = null

    try {
      for (const file of files) {
        // 只处理非图片文件，图片文件由 useImageHandler 处理
        if (!isImageFile(file.name)) {
          // 尝试获取完整路径（如果可用）
          // 注意：出于安全原因，浏览器通常不提供完整路径
          // 在 Tauri 环境中，我们可能需要通过其他方式获取
          const fullPath = (file as any).path || file.name
          const fileRef = createFileReference(file, fullPath)
          fileRefs.push(fileRef)
        }
      }
    } catch (err) {
      error.value = err instanceof Error ? err.message : '处理文件失败'
      console.error('处理拖拽文件失败:', err)
    } finally {
      isProcessing.value = false
    }

    return fileRefs
  }

  /**
   * 从路径数组创建文件引用
   * 用于文件选择对话框返回的路径
   * Requirement 4.2
   */
  function createFileReferencesFromPaths(paths: string[]): FileReference[] {
    return paths.map(path => createFileReferenceFromPath(path))
  }

  /**
   * 分类文件为图片和非图片
   * Requirement 4.3: 自动区分图片文件和普通文件
   */
  function classifyFiles(files: FileList | File[]): { images: File[]; others: File[] } {
    const images: File[] = []
    const others: File[] = []

    for (const file of files) {
      if (isImageFile(file.name)) {
        images.push(file)
      } else {
        others.push(file)
      }
    }

    return { images, others }
  }

  /**
   * 打开文件选择对话框
   * Requirement 4.2: 支持多文件选择
   * @returns 选中的文件引用数组
   */
  async function openFileDialog(): Promise<FileReference[]> {
    isProcessing.value = true
    error.value = null

    try {
      const selected = await open({
        multiple: true,
        title: '选择文件',
        filters: [
          {
            name: '所有文件',
            extensions: ['*']
          },
          {
            name: '图片文件',
            extensions: ['png', 'jpg', 'jpeg', 'gif', 'webp', 'bmp', 'svg']
          },
          {
            name: '文档文件',
            extensions: ['txt', 'md', 'pdf', 'doc', 'docx', 'xls', 'xlsx']
          },
          {
            name: '代码文件',
            extensions: ['js', 'ts', 'vue', 'jsx', 'tsx', 'py', 'rs', 'go', 'java', 'c', 'cpp', 'h']
          }
        ]
      })

      if (!selected) {
        return []
      }

      // 处理选中的文件路径
      const paths = Array.isArray(selected) ? selected : [selected]
      return createFileReferencesFromPaths(paths)
    } catch (err) {
      error.value = err instanceof Error ? err.message : '打开文件对话框失败'
      console.error('打开文件对话框失败:', err)
      return []
    } finally {
      isProcessing.value = false
    }
  }

  /**
   * 分类路径为图片和非图片
   * Requirement 4.3: 自动区分图片文件和普通文件
   */
  function classifyPaths(paths: string[]): { imagePaths: string[]; otherPaths: string[] } {
    const imagePaths: string[] = []
    const otherPaths: string[] = []

    for (const path of paths) {
      if (isImageFile(path)) {
        imagePaths.push(path)
      } else {
        otherPaths.push(path)
      }
    }

    return { imagePaths, otherPaths }
  }

  /**
   * 打开文本文件选择对话框
   * 仅允许选择文本类文件（代码、配置、文档等）
   * @returns 选中的文件引用数组
   */
  async function openTextFileDialog(): Promise<FileReference[]> {
    isProcessing.value = true
    error.value = null

    try {
      const selected = await open({
        multiple: true,
        title: '选择文本文件',
        filters: [
          {
            name: '文本文件',
            extensions: TEXT_FILE_EXTENSIONS
          },
          {
            name: '代码文件',
            extensions: ['js', 'ts', 'vue', 'jsx', 'tsx', 'py', 'rs', 'go', 'java', 'c', 'cpp', 'h', 'hpp', 'cs', 'rb', 'php']
          },
          {
            name: '配置文件',
            extensions: ['json', 'yaml', 'yml', 'xml', 'toml', 'ini', 'conf', 'env']
          },
          {
            name: '文档文件',
            extensions: ['txt', 'md', 'markdown', 'rst']
          }
        ]
      })

      if (!selected) {
        return []
      }

      // 处理选中的文件路径
      const paths = Array.isArray(selected) ? selected : [selected]
      return createFileReferencesFromPaths(paths)
    } catch (err) {
      error.value = err instanceof Error ? err.message : '打开文件对话框失败'
      console.error('打开文件对话框失败:', err)
      return []
    } finally {
      isProcessing.value = false
    }
  }

  return {
    isProcessing,
    error,
    handleFileDrop,
    createFileReference,
    createFileReferenceFromPath,
    createFileReferencesFromPaths,
    classifyFiles,
    classifyPaths,
    openFileDialog,
    openTextFileDialog,
    isImageFile,
    getFileName,
    getFileExtension,
    formatFileReference,
    IMAGE_EXTENSIONS,
    TEXT_FILE_EXTENSIONS
  }
}

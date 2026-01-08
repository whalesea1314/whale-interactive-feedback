import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

// MCP 请求类型
export interface PopupRequest {
  id: string
  message: string | null
  full_response: string | null
  predefined_options: string[] | null
  created_at: string
}

// MCP 响应类型
export interface PopupResponse {
  request_id: string
  user_input: string | null
  selected_options: string[]
  images: ImageData[]
  file_references: FileReferenceData[]
  cancelled: boolean
}

// 图片数据类型
export interface ImageData {
  data: string
  mime_type: string
}

// 文件引用数据类型
export interface FileReferenceData {
  display_name: string
  path: string
  is_directory: boolean
}

// CLI 参数类型
export interface CliArgs {
  mcp_request_file: string | null
  mcp_mode: boolean
}

// MCP handler 状态
const mcpRequest = ref<PopupRequest | null>(null)
const isMcpMode = ref(false)
const mcpRequestFile = ref<string | null>(null)

/**
 * MCP 请求处理 composable
 */
export function useMcpHandler() {
  /**
   * 检查是否为 MCP 模式
   */
  async function checkMcpMode(): Promise<boolean> {
    try {
      const args = await invoke<CliArgs>('get_cli_args')
      isMcpMode.value = args.mcp_mode || !!args.mcp_request_file
      mcpRequestFile.value = args.mcp_request_file || null
      console.log('MCP mode:', isMcpMode.value, 'Request file:', mcpRequestFile.value)
      return isMcpMode.value
    } catch (error) {
      console.error('Failed to check MCP mode:', error)
      return false
    }
  }

  /**
   * 加载 MCP 请求
   */
  async function loadMcpRequest(): Promise<PopupRequest | null> {
    if (!mcpRequestFile.value) {
      console.log('No MCP request file specified')
      return null
    }

    try {
      const request = await invoke<PopupRequest>('read_mcp_request', {
        filePath: mcpRequestFile.value
      })
      mcpRequest.value = request
      console.log('Loaded MCP request:', request)
      return request
    } catch (error) {
      console.error('Failed to load MCP request:', error)
      return null
    }
  }

  /**
   * 构建 MCP 响应
   */
  function buildResponse(
    userInput: string,
    selectedOptions: string[],
    images: ImageData[],
    fileReferences: FileReferenceData[] = []
  ): PopupResponse {
    return {
      request_id: mcpRequest.value?.id || '',
      user_input: userInput.trim() || null,
      selected_options: selectedOptions,
      images,
      file_references: fileReferences,
      cancelled: false
    }
  }

  /**
   * 发送 MCP 响应
   */
  async function sendResponse(response: PopupResponse): Promise<void> {
    if (!mcpRequestFile.value || !mcpRequest.value) {
      throw new Error('No MCP request active')
    }

    // 生成响应文件路径
    const responsePath = mcpRequestFile.value.replace('mcp_request_', 'mcp_response_')

    console.log('[MCP] 准备写入响应文件:', responsePath)
    console.log('[MCP] 响应内容:', JSON.stringify(response, null, 2))

    try {
      await invoke('write_response_file', {
        filePath: responsePath,
        response
      })
      console.log('[MCP] 响应文件写入成功:', responsePath)
      
      // 等待一小段时间确保文件写入完成
      await new Promise(resolve => setTimeout(resolve, 100))
    } catch (error) {
      console.error('[MCP] 写入响应文件失败:', error)
      throw error
    }
  }

  /**
   * 提交反馈并关闭
   */
  async function submitFeedback(
    userInput: string,
    selectedOptions: string[],
    images: ImageData[],
    fileReferences: FileReferenceData[] = []
  ): Promise<void> {
    const response = buildResponse(userInput, selectedOptions, images, fileReferences)
    await sendResponse(response)
    await closePopup()
  }

  /**
   * 取消请求
   */
  async function cancelRequest(): Promise<void> {
    if (mcpRequest.value && mcpRequestFile.value) {
      const response: PopupResponse = {
        request_id: mcpRequest.value.id,
        user_input: null,
        selected_options: [],
        images: [],
        file_references: [],
        cancelled: true
      }
      await sendResponse(response)
    }
    await closePopup()
  }

  /**
   * 关闭弹窗
   */
  async function closePopup(): Promise<void> {
    console.log('[MCP] 准备关闭弹窗...')
    try {
      // 短暂延迟确保响应文件已写入
      await new Promise(resolve => setTimeout(resolve, 50))
      await invoke('exit_app')
      console.log('[MCP] 退出命令已发送')
    } catch (error) {
      console.error('[MCP] 关闭弹窗失败:', error)
      // 即使失败也尝试强制退出 - Tauri v2 不再支持 process API
      // 依赖 exit_app 命令
    }
  }

  return {
    // 状态
    mcpRequest,
    isMcpMode,
    mcpRequestFile,

    // 方法
    checkMcpMode,
    loadMcpRequest,
    buildResponse,
    sendResponse,
    submitFeedback,
    cancelRequest,
    closePopup
  }
}

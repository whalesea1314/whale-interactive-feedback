/**
 * 截图功能 composable
 * 
 * macOS: 使用 screencapture -i 原生交互式截图
 * Windows/Linux: 使用 tauri-plugin-screenshots + 全屏框选
 */

import { ref } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { platform } from '@tauri-apps/plugin-os'

// 截图结果类型
export interface ScreenshotResult {
  data: string      // Base64 编码的图片数据
  mime_type: string
  width: number
  height: number
  size: number
}

// 选区类型
export interface SelectionRect {
  x: number
  y: number
  width: number
  height: number
}

export function useScreenshot() {
  const isCapturing = ref(false)
  const error = ref<string | null>(null)
  const screenshotData = ref<ScreenshotResult | null>(null)
  const currentPlatform = ref<string>('')

  /**
   * 获取当前平台
   */
  function getPlatform(): string {
    if (!currentPlatform.value) {
      currentPlatform.value = platform()
    }
    return currentPlatform.value
  }

  /**
   * 是否使用原生截图（macOS）
   */
  function useNativeScreenshot(): boolean {
    return getPlatform() === 'macos'
  }

  /**
   * macOS 原生交互式截图
   */
  async function captureInteractive(): Promise<ScreenshotResult | null> {
    const appWindow = getCurrentWindow()
    
    try {
      isCapturing.value = true
      error.value = null
      
      const { Command } = await import('@tauri-apps/plugin-shell')
      const { readFile, remove } = await import('@tauri-apps/plugin-fs')
      const { appDataDir } = await import('@tauri-apps/api/path')
      
      const dataDir = await appDataDir()
      const timestamp = Date.now()
      const screenshotPath = `${dataDir}screenshot_${timestamp}.png`
      
      // 隐藏窗口
      await appWindow.hide()
      await new Promise(resolve => setTimeout(resolve, 500))
      
      // 交互式截图
      const command = Command.create('screencapture', ['-i', '-x', screenshotPath])
      const output = await command.execute()
      
      // 显示窗口
      await appWindow.show()
      await appWindow.setFocus()
      
      // 用户取消
      if (output.code !== 0) {
        return null
      }
      
      // 读取文件
      let fileData: Uint8Array
      try {
        fileData = await readFile(screenshotPath)
      } catch {
        return null // 文件不存在，用户取消
      }
      
      const base64Data = arrayBufferToBase64(fileData)
      const { width, height } = parsePngDimensions(fileData)
      
      try { await remove(screenshotPath) } catch { /* 忽略删除失败 */ }
      
      return {
        data: base64Data,
        mime_type: 'image/png',
        width,
        height,
        size: fileData.byteLength
      }
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : String(e)
      error.value = errorMsg
      
      try {
        await appWindow.show()
        await appWindow.setFocus()
      } catch { /* 忽略窗口操作失败 */ }
      
      throw e
    } finally {
      isCapturing.value = false
    }
  }

  /**
   * 静默截取全屏（用于非 macOS 平台的框选模式）
   */
  async function captureFullScreenSilent(): Promise<ScreenshotResult> {
    const appWindow = getCurrentWindow()
    
    try {
      isCapturing.value = true
      error.value = null
      
      await appWindow.hide()
      await new Promise(resolve => setTimeout(resolve, 300))
      
      const os = getPlatform()
      let result: ScreenshotResult
      
      if (os === 'macos') {
        result = await captureMacOSFullScreen()
      } else {
        result = await captureWithPlugin()
      }
      
      screenshotData.value = result
      return result
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : String(e)
      error.value = errorMsg
      throw e
    } finally {
      isCapturing.value = false
    }
  }

  /**
   * macOS 全屏截图
   */
  async function captureMacOSFullScreen(): Promise<ScreenshotResult> {
    const { Command } = await import('@tauri-apps/plugin-shell')
    const { readFile, remove } = await import('@tauri-apps/plugin-fs')
    const { appDataDir } = await import('@tauri-apps/api/path')
    
    const dataDir = await appDataDir()
    const timestamp = Date.now()
    const screenshotPath = `${dataDir}screenshot_${timestamp}.png`
    
    const command = Command.create('screencapture', ['-x', screenshotPath])
    await command.execute()
    
    const fileData = await readFile(screenshotPath)
    const base64Data = arrayBufferToBase64(fileData)
    const { width, height } = parsePngDimensions(fileData)
    
    try { await remove(screenshotPath) } catch { /* 忽略删除失败 */ }
    
    return {
      data: base64Data,
      mime_type: 'image/png',
      width,
      height,
      size: fileData.byteLength
    }
  }

  /**
   * 使用插件截图（Windows/Linux）
   */
  async function captureWithPlugin(): Promise<ScreenshotResult> {
    const { getScreenshotableMonitors, getMonitorScreenshot, clearScreenshots } = 
      await import('tauri-plugin-screenshots-api')
    const { readFile } = await import('@tauri-apps/plugin-fs')
    
    try { await clearScreenshots() } catch { /* 忽略清理失败 */ }
    
    const monitors = await getScreenshotableMonitors()
    if (monitors.length === 0) throw new Error('没有可用的显示器')
    
    const screenshotPath = await getMonitorScreenshot(monitors[0].id)
    const fileData = await readFile(screenshotPath)
    const base64Data = arrayBufferToBase64(fileData)
    const { width, height } = parsePngDimensions(fileData)
    
    return {
      data: base64Data,
      mime_type: 'image/png',
      width,
      height,
      size: fileData.byteLength
    }
  }

  /**
   * 显示窗口
   */
  async function showWindow(): Promise<void> {
    const appWindow = getCurrentWindow()
    await appWindow.show()
    await appWindow.setFocus()
  }

  /**
   * 进入全屏截图模式（非 macOS）
   */
  async function enterFullscreenMode(): Promise<void> {
    const appWindow = getCurrentWindow()
    await appWindow.setDecorations(false)
    await appWindow.setAlwaysOnTop(true)
    await appWindow.maximize()
    await appWindow.show()
    await appWindow.setFocus()
  }

  /**
   * 退出全屏截图模式
   */
  async function exitFullscreenMode(): Promise<void> {
    const appWindow = getCurrentWindow()
    await appWindow.setAlwaysOnTop(false)
    await appWindow.unmaximize()
    await appWindow.setDecorations(true)
  }

  /**
   * 裁剪图片
   */
  async function cropImage(
    screenshot: ScreenshotResult,
    selection: SelectionRect,
    screenWidth: number,
    screenHeight: number
  ): Promise<ScreenshotResult> {
    return new Promise((resolve, reject) => {
      const img = new Image()
      img.onload = () => {
        const scaleX = img.naturalWidth / screenWidth
        const scaleY = img.naturalHeight / screenHeight
        
        const canvas = document.createElement('canvas')
        canvas.width = Math.round(selection.width * scaleX)
        canvas.height = Math.round(selection.height * scaleY)
        
        const ctx = canvas.getContext('2d')
        if (!ctx) {
          reject(new Error('无法创建 Canvas 上下文'))
          return
        }
        
        ctx.drawImage(
          img,
          Math.round(selection.x * scaleX),
          Math.round(selection.y * scaleY),
          canvas.width,
          canvas.height,
          0, 0,
          canvas.width,
          canvas.height
        )
        
        const dataUrl = canvas.toDataURL('image/png')
        const base64 = dataUrl.split(',')[1]
        
        resolve({
          data: base64,
          mime_type: 'image/png',
          width: canvas.width,
          height: canvas.height,
          size: Math.round(base64.length * 0.75)
        })
      }
      img.onerror = () => reject(new Error('图片加载失败'))
      img.src = `data:${screenshot.mime_type};base64,${screenshot.data}`
    })
  }

  function cleanup(): void {
    screenshotData.value = null
    error.value = null
  }

  return {
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
    cleanup,
  }
}

// 工具函数
function arrayBufferToBase64(buffer: Uint8Array): string {
  let binary = ''
  const bytes = new Uint8Array(buffer)
  for (let i = 0; i < bytes.byteLength; i++) {
    binary += String.fromCharCode(bytes[i])
  }
  return btoa(binary)
}

function parsePngDimensions(data: Uint8Array): { width: number; height: number } {
  if (data.length < 24) return { width: 0, height: 0 }
  const width = (data[16] << 24) | (data[17] << 16) | (data[18] << 8) | data[19]
  const height = (data[20] << 24) | (data[21] << 16) | (data[22] << 8) | data[23]
  return { width, height }
}

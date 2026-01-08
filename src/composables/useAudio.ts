import { computed, ref } from 'vue'
import { useConfigStore } from '@/stores/config'
import { invoke } from '@tauri-apps/api/core'

/**
 * 音频通知 Composable
 * 
 * 提供音频通知功能，支持自定义音频文件
 * Requirements: 12.1, 12.2, 12.3
 * 
 * - 12.1: WHEN the Feedback_Window opens THEN the Audio_Notifier SHALL play a notification sound
 * - 12.2: WHEN in the settings page THEN the Config_Manager SHALL allow enabling or disabling audio notifications
 * - 12.3: WHEN in the settings page THEN the Config_Manager SHALL allow selecting a custom audio file
 */
export function useAudio() {
  const configStore = useConfigStore()

  // 状态
  const isPlaying = ref(false)
  const lastError = ref<string | null>(null)
  const supportedFormats = ref<string[]>(['wav', 'mp3', 'ogg', 'flac'])

  // 计算属性
  const audioEnabled = computed(() => configStore.config.audioEnabled)
  const audioFile = computed(() => configStore.config.audioFile)

  /**
   * 播放通知音
   * 
   * Requirement 12.1: WHEN the Feedback_Window opens THEN the Audio_Notifier SHALL play a notification sound
   * Requirement 12.4: IF audio playback fails THEN the Audio_Notifier SHALL silently continue without interrupting the workflow
   */
  async function playNotification(): Promise<void> {
    // 如果音频被禁用，直接返回
    if (!audioEnabled.value) {
      return
    }

    isPlaying.value = true
    lastError.value = null

    try {
      // 调用 Tauri 命令播放音频
      // 如果 audioFile 为空或 undefined，后端会使用默认音频
      await invoke('play_notification_sound', { 
        soundPath: audioFile.value || null 
      })
    } catch (e) {
      // Requirement 12.4: 静默处理错误，不中断工作流
      lastError.value = e instanceof Error ? e.message : String(e)
      console.warn('音频播放失败（静默继续）:', lastError.value)
    } finally {
      isPlaying.value = false
    }
  }

  /**
   * 设置音频启用状态
   * 
   * Requirement 12.2: WHEN in the settings page THEN the Config_Manager SHALL allow enabling or disabling audio notifications
   */
  function setAudioEnabled(enabled: boolean): void {
    configStore.setAudioEnabled(enabled)
  }

  /**
   * 设置自定义音频文件
   * 
   * Requirement 12.3: WHEN in the settings page THEN the Config_Manager SHALL allow selecting a custom audio file
   */
  function setAudioFile(path: string | undefined): void {
    configStore.setAudioFile(path)
  }

  /**
   * 验证音频文件
   * 
   * 检查音频文件是否存在且格式受支持
   * 
   * @param path - 音频文件路径
   * @returns 验证结果
   */
  async function validateAudioFile(path: string): Promise<{ valid: boolean; error?: string }> {
    try {
      await invoke('validate_audio_file', { path })
      return { valid: true }
    } catch (e) {
      const error = e instanceof Error ? e.message : String(e)
      return { valid: false, error }
    }
  }

  /**
   * 获取支持的音频格式
   */
  async function loadSupportedFormats(): Promise<void> {
    try {
      const formats = await invoke<string[]>('get_supported_audio_formats')
      supportedFormats.value = formats
    } catch (e) {
      console.warn('获取支持的音频格式失败:', e)
    }
  }

  /**
   * 测试播放音频
   * 
   * 用于设置页面测试音频效果
   */
  async function testPlaySound(path?: string): Promise<{ success: boolean; error?: string }> {
    isPlaying.value = true
    lastError.value = null

    try {
      await invoke('play_notification_sound', { 
        soundPath: path || null 
      })
      return { success: true }
    } catch (e) {
      const error = e instanceof Error ? e.message : String(e)
      lastError.value = error
      return { success: false, error }
    } finally {
      isPlaying.value = false
    }
  }

  /**
   * 清除自定义音频文件（使用默认音频）
   */
  function clearAudioFile(): void {
    configStore.setAudioFile(undefined)
  }

  return {
    // 状态
    audioEnabled,
    audioFile,
    isPlaying,
    lastError,
    supportedFormats,
    // 方法
    playNotification,
    setAudioEnabled,
    setAudioFile,
    validateAudioFile,
    loadSupportedFormats,
    testPlaySound,
    clearAudioFile,
  }
}

<script setup lang="ts">
/**
 * 音频设置组件
 * 
 * 提供音频通知的启用/禁用开关和自定义音频文件选择
 * Requirements: 12.1, 12.2, 12.3
 * 
 * - 12.1: WHEN the Feedback_Window opens THEN the Audio_Notifier SHALL play a notification sound
 * - 12.2: WHEN in the settings page THEN the Config_Manager SHALL allow enabling or disabling audio notifications
 * - 12.3: WHEN in the settings page THEN the Config_Manager SHALL allow selecting a custom audio file
 */
import { ref, computed, onMounted } from 'vue'
import { useAudio } from '@/composables/useAudio'
import { open } from '@tauri-apps/plugin-dialog'
import { invoke } from '@tauri-apps/api/core'

// 内置音频类型
interface BuiltinSound {
  id: string
  name: string
  description: string
}

const {
  audioEnabled,
  audioFile,
  isPlaying,
  supportedFormats,
  setAudioEnabled,
  setAudioFile,
  validateAudioFile,
  loadSupportedFormats,
  testPlaySound,
  clearAudioFile,
} = useAudio()

// 本地状态
const validationError = ref<string | null>(null)
const isValidating = ref(false)
const testResult = ref<{ success: boolean; message: string } | null>(null)
const builtinSounds = ref<BuiltinSound[]>([])

// 计算属性
const isBuiltinAudio = computed(() => audioFile.value?.startsWith('builtin:') ?? false)
const hasCustomAudio = computed(() => !!audioFile.value && !isBuiltinAudio.value)
const currentBuiltinId = computed(() => {
  if (isBuiltinAudio.value && audioFile.value) {
    return audioFile.value.replace('builtin:', '')
  }
  return null
})
const audioFileName = computed(() => {
  if (!audioFile.value) return '默认提示音'
  if (isBuiltinAudio.value) {
    const builtin = builtinSounds.value.find(s => s.id === currentBuiltinId.value)
    return builtin?.name || '内置音频'
  }
  // 从路径中提取文件名
  const parts = audioFile.value.split(/[/\\]/)
  return parts[parts.length - 1] || audioFile.value
})

// 格式化支持的格式列表
const formatsDisplay = computed(() => {
  return supportedFormats.value.map(f => `.${f}`).join(', ')
})

// 加载内置音频列表
async function loadBuiltinSounds() {
  try {
    builtinSounds.value = await invoke<BuiltinSound[]>('get_builtin_sounds')
  } catch (e) {
    console.warn('获取内置音频列表失败:', e)
  }
}

// 选择内置音频
function selectBuiltinSound(id: string) {
  setAudioFile(`builtin:${id}`)
  validationError.value = null
  testResult.value = null
  // 自动测试播放
  handleTestPlay(`builtin:${id}`)
}

// 初始化
onMounted(async () => {
  await Promise.all([
    loadSupportedFormats(),
    loadBuiltinSounds()
  ])
})

/**
 * 切换音频启用状态
 * Requirement 12.2: 允许启用或禁用音频通知
 */
function toggleAudioEnabled() {
  setAudioEnabled(!audioEnabled.value)
}

/**
 * 选择自定义音频文件
 * Requirement 12.3: 允许选择自定义音频文件
 */
async function selectAudioFile() {
  try {
    validationError.value = null
    testResult.value = null
    
    // 打开文件选择对话框
    const selected = await open({
      multiple: false,
      filters: [{
        name: '音频文件',
        extensions: supportedFormats.value,
      }],
    })
    
    if (!selected) return
    
    const filePath = typeof selected === 'string' ? selected : (selected as { path: string }).path
    
    // 验证文件
    isValidating.value = true
    const result = await validateAudioFile(filePath)
    isValidating.value = false
    
    if (!result.valid) {
      validationError.value = result.error || '无效的音频文件'
      return
    }
    
    // 设置音频文件
    setAudioFile(filePath)
    
    // 自动测试播放
    await handleTestPlay(filePath)
  } catch (e) {
    validationError.value = e instanceof Error ? e.message : String(e)
    isValidating.value = false
  }
}

/**
 * 测试播放音频
 */
async function handleTestPlay(path?: string) {
  testResult.value = null
  
  const result = await testPlaySound(path || audioFile.value)
  
  if (result.success) {
    testResult.value = { success: true, message: '播放成功' }
  } else {
    testResult.value = { success: false, message: result.error || '播放失败' }
  }
  
  // 3秒后清除结果
  setTimeout(() => {
    testResult.value = null
  }, 3000)
}

/**
 * 使用默认音频
 */
function useDefaultAudio() {
  clearAudioFile()
  validationError.value = null
  testResult.value = null
}
</script>

<template>
  <div class="audio-settings">
    <div class="settings-header">
      <span class="i-carbon-volume-up header-icon" />
      <h3 class="header-title">
        音频通知
      </h3>
    </div>
    
    <!-- 启用/禁用开关 - Requirement 12.2 -->
    <div class="setting-item">
      <div class="setting-info">
        <span class="setting-label">启用通知音</span>
        <span class="setting-desc">反馈窗口打开时播放提示音</span>
      </div>
      <label class="toggle-switch">
        <input 
          type="checkbox" 
          :checked="audioEnabled"
          @change="toggleAudioEnabled"
        >
        <span class="toggle-slider" />
      </label>
    </div>
    
    <!-- 内置音频选择 -->
    <div
      class="setting-item builtin-sounds"
      :class="{ disabled: !audioEnabled }"
    >
      <div class="setting-info full-width">
        <span class="setting-label">内置提示音</span>
        <div class="builtin-list">
          <button 
            v-for="sound in builtinSounds" 
            :key="sound.id"
            class="builtin-btn"
            :class="{ active: currentBuiltinId === sound.id }"
            :disabled="!audioEnabled"
            :title="sound.description"
            @click="selectBuiltinSound(sound.id)"
          >
            <span
              v-if="currentBuiltinId === sound.id"
              class="i-carbon-volume-up-filled"
            />
            <span
              v-else
              class="i-carbon-volume-up"
            />
            {{ sound.name }}
          </button>
        </div>
      </div>
    </div>
    
    <!-- 自定义音频文件选择 - Requirement 12.3 -->
    <div
      class="setting-item"
      :class="{ disabled: !audioEnabled }"
    >
      <div class="setting-info">
        <span class="setting-label">自定义音频</span>
        <span class="setting-desc">
          {{ hasCustomAudio ? audioFileName : '使用内置音频或选择自定义文件' }}
        </span>
      </div>
      <div class="audio-actions">
        <button 
          class="action-btn select-btn"
          :disabled="!audioEnabled"
          @click="selectAudioFile"
        >
          <span class="i-carbon-folder" />
          选择
        </button>
        <button 
          v-if="hasCustomAudio"
          class="action-btn default-btn"
          :disabled="!audioEnabled"
          @click="useDefaultAudio"
        >
          <span class="i-carbon-reset" />
          清除
        </button>
      </div>
    </div>
    
    <!-- 测试播放 -->
    <div
      class="setting-item"
      :class="{ disabled: !audioEnabled }"
    >
      <div class="setting-info">
        <span class="setting-label">测试播放</span>
        <span class="setting-desc">
          {{ isPlaying ? '正在播放...' : '点击测试当前音频效果' }}
        </span>
      </div>
      <button 
        class="action-btn test-btn"
        :disabled="!audioEnabled || isPlaying"
        @click="() => handleTestPlay()"
      >
        <span :class="isPlaying ? 'i-carbon-pause' : 'i-carbon-play'" />
        {{ isPlaying ? '播放中' : '测试' }}
      </button>
    </div>
    
    <!-- 验证错误 -->
    <div
      v-if="validationError"
      class="error-message"
    >
      <span class="i-carbon-warning" />
      {{ validationError }}
    </div>
    
    <!-- 测试结果 -->
    <div
      v-if="testResult"
      class="test-result"
      :class="{ success: testResult.success, error: !testResult.success }"
    >
      <span :class="testResult.success ? 'i-carbon-checkmark' : 'i-carbon-close'" />
      {{ testResult.message }}
    </div>
    
    <!-- 支持的格式 -->
    <div class="formats-info">
      <span class="i-carbon-information" />
      支持的格式: {{ formatsDisplay }}
    </div>
  </div>
</template>

<style scoped>
.audio-settings {
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding: 16px;
  background-color: var(--bg-secondary);
  border-radius: 8px;
}

.settings-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding-bottom: 12px;
  border-bottom: 1px solid var(--border-color);
}

.header-icon {
  font-size: 20px;
  color: var(--accent-color);
}

.header-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0;
}

.setting-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px;
  background-color: var(--bg-tertiary);
  border-radius: 6px;
  transition: opacity 0.2s;
}

.setting-item.disabled {
  opacity: 0.5;
  pointer-events: none;
}

.setting-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.setting-label {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
}

.setting-desc {
  font-size: 12px;
  color: var(--text-muted);
}

/* Toggle Switch */
.toggle-switch {
  position: relative;
  display: inline-block;
  width: 48px;
  height: 24px;
}

.toggle-switch input {
  opacity: 0;
  width: 0;
  height: 0;
}

.toggle-slider {
  position: absolute;
  cursor: pointer;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: var(--bg-primary);
  transition: 0.3s;
  border-radius: 24px;
  border: 1px solid var(--border-color);
}

.toggle-slider:before {
  position: absolute;
  content: "";
  height: 18px;
  width: 18px;
  left: 2px;
  bottom: 2px;
  background-color: var(--text-muted);
  transition: 0.3s;
  border-radius: 50%;
}

.toggle-switch input:checked + .toggle-slider {
  background-color: var(--accent-color);
  border-color: var(--accent-color);
}

.toggle-switch input:checked + .toggle-slider:before {
  transform: translateX(24px);
  background-color: white;
}

/* Action Buttons */
.audio-actions {
  display: flex;
  gap: 8px;
}

.action-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 12px;
  border: none;
  border-radius: 6px;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s;
}

.action-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.select-btn {
  background-color: var(--accent-color);
  color: white;
}

.select-btn:hover:not(:disabled) {
  background-color: var(--accent-hover);
}

.default-btn {
  background-color: var(--bg-primary);
  color: var(--text-secondary);
  border: 1px solid var(--border-color);
}

.default-btn:hover:not(:disabled) {
  background-color: var(--bg-secondary);
  color: var(--text-primary);
}

.test-btn {
  background-color: var(--bg-primary);
  color: var(--text-secondary);
  border: 1px solid var(--border-color);
}

.test-btn:hover:not(:disabled) {
  background-color: var(--bg-secondary);
  color: var(--accent-color);
  border-color: var(--accent-color);
}

/* Messages */
.error-message {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 12px;
  background-color: rgba(239, 68, 68, 0.1);
  border: 1px solid rgba(239, 68, 68, 0.3);
  border-radius: 6px;
  color: #ef4444;
  font-size: 13px;
}

.test-result {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 12px;
  border-radius: 6px;
  font-size: 13px;
}

.test-result.success {
  background-color: rgba(34, 197, 94, 0.1);
  border: 1px solid rgba(34, 197, 94, 0.3);
  color: #22c55e;
}

.test-result.error {
  background-color: rgba(239, 68, 68, 0.1);
  border: 1px solid rgba(239, 68, 68, 0.3);
  color: #ef4444;
}

.formats-info {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: var(--text-muted);
  padding-top: 8px;
  border-top: 1px solid var(--border-color);
}

/* 内置音频列表 */
.builtin-sounds {
  flex-direction: column;
  align-items: stretch;
}

.setting-info.full-width {
  width: 100%;
}

.builtin-list {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 10px;
}

.builtin-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 12px;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  background: var(--bg-primary);
  color: var(--text-secondary);
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s;
}

.builtin-btn:hover:not(:disabled) {
  border-color: var(--accent-color);
  color: var(--accent-color);
  background: var(--accent-light);
}

.builtin-btn.active {
  border-color: var(--accent-color);
  background: var(--accent-color);
  color: white;
}

.builtin-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>

<script setup lang="ts">
/**
 * 窗口设置组件
 * 
 * 提供窗口固定（置顶）和自动最小化功能的设置界面
 * Requirements: 13.1, 13.2, 13.3, 13.4, 13.5
 * 
 * - 13.1: WHEN in the settings page THEN the Config_Manager SHALL allow enabling window pinning (always on top)
 * - 13.2: WHEN in the settings page THEN the Config_Manager SHALL allow enabling auto-minimize after submission
 * - 13.3: WHEN window pinning is enabled THEN the Feedback_Window SHALL remain on top of other windows
 * - 13.4: WHEN auto-minimize is enabled THEN the Feedback_Window SHALL minimize after feedback submission
 * - 13.5: WHEN the application starts THEN the Config_Manager SHALL restore the previously saved window control settings
 */
import { ref, computed, onMounted } from 'vue'
import { useWindowControl } from '@/composables/useWindowControl'

const {
  isAlwaysOnTop,
  isInitialized,
  windowPinned,
  autoMinimize,
  setAlwaysOnTop,
  setAutoMinimize,
  initWindowState,
} = useWindowControl()

// 本地状态
const isLoading = ref(false)
const statusMessage = ref<{ type: 'success' | 'error'; text: string } | null>(null)

// 初始化
onMounted(async () => {
  if (!isInitialized.value) {
    await initWindowState()
  }
})

/**
 * 切换窗口置顶状态
 * Requirement 13.1, 13.3: 窗口固定功能
 */
async function toggleWindowPinned() {
  isLoading.value = true
  statusMessage.value = null
  
  try {
    await setAlwaysOnTop(!windowPinned.value)
    statusMessage.value = {
      type: 'success',
      text: windowPinned.value ? '窗口已置顶' : '已取消置顶'
    }
  } catch (e) {
    statusMessage.value = {
      type: 'error',
      text: '设置窗口置顶失败'
    }
  } finally {
    isLoading.value = false
    // 3秒后清除消息
    setTimeout(() => {
      statusMessage.value = null
    }, 3000)
  }
}

/**
 * 切换自动最小化状态
 * Requirement 13.2, 13.4: 自动最小化功能
 */
function toggleAutoMinimize() {
  setAutoMinimize(!autoMinimize.value)
  statusMessage.value = {
    type: 'success',
    text: autoMinimize.value ? '已启用自动最小化' : '已禁用自动最小化'
  }
  // 3秒后清除消息
  setTimeout(() => {
    statusMessage.value = null
  }, 3000)
}
</script>

<template>
  <div class="window-settings">
    <div class="settings-header">
      <span class="i-carbon-application header-icon" />
      <h3 class="header-title">
        窗口控制
      </h3>
    </div>
    
    <!-- 窗口置顶开关 - Requirement 13.1, 13.3 -->
    <div class="setting-item">
      <div class="setting-info">
        <span class="setting-label">窗口置顶</span>
        <span class="setting-desc">保持窗口始终显示在其他窗口之上</span>
      </div>
      <label class="toggle-switch">
        <input 
          type="checkbox" 
          :checked="windowPinned"
          :disabled="isLoading"
          @change="toggleWindowPinned"
        >
        <span class="toggle-slider" />
      </label>
    </div>
    
    <!-- 自动最小化开关 - Requirement 13.2, 13.4 -->
    <div class="setting-item">
      <div class="setting-info">
        <span class="setting-label">提交后最小化</span>
        <span class="setting-desc">提交反馈后自动最小化窗口</span>
      </div>
      <label class="toggle-switch">
        <input 
          type="checkbox" 
          :checked="autoMinimize"
          @change="toggleAutoMinimize"
        >
        <span class="toggle-slider" />
      </label>
    </div>
    
    <!-- 状态消息 -->
    <div 
      v-if="statusMessage" 
      class="status-message" 
      :class="statusMessage.type"
    >
      <span :class="statusMessage.type === 'success' ? 'i-carbon-checkmark' : 'i-carbon-warning'" />
      {{ statusMessage.text }}
    </div>
    
    <!-- 提示信息 -->
    <div class="info-section">
      <span class="i-carbon-information" />
      <span class="info-text">窗口设置会自动保存，下次启动时恢复</span>
    </div>
  </div>
</template>

<style scoped>
.window-settings {
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

.toggle-switch input:disabled + .toggle-slider {
  opacity: 0.5;
  cursor: not-allowed;
}

/* Status Message */
.status-message {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 12px;
  border-radius: 6px;
  font-size: 13px;
}

.status-message.success {
  background-color: rgba(34, 197, 94, 0.1);
  border: 1px solid rgba(34, 197, 94, 0.3);
  color: #22c55e;
}

.status-message.error {
  background-color: rgba(239, 68, 68, 0.1);
  border: 1px solid rgba(239, 68, 68, 0.3);
  color: #ef4444;
}

/* Info Section */
.info-section {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: var(--text-muted);
  padding-top: 8px;
  border-top: 1px solid var(--border-color);
}

.info-text {
  line-height: 1.4;
}
</style>

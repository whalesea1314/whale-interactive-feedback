<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { useTextOptimization, type TextOptimizationResult } from '@/composables/useTextOptimization'
import { useConfigStore } from '@/stores/config'

// Props
const props = defineProps<{
  initialText?: string
  visible?: boolean
}>()

// Emits
const emit = defineEmits<{
  (e: 'apply', text: string): void
  (e: 'optimized', result: TextOptimizationResult): void
  (e: 'close'): void
  (e: 'openSettings'): void
}>()

// Config store for API key check and optimization types
const configStore = useConfigStore()

// Check if any API key is configured
const hasApiKey = computed(() => {
  return configStore.configuredProviders.length > 0
})

// 从 configStore 获取启用的优化类型
const optimizationOptions = computed(() => {
  return configStore.enabledOptimizationTypes.map(t => ({
    value: t.id,
    label: t.label,
    icon: t.icon,
    desc: t.description,
  }))
})

// Composable
const { 
  isOptimizing, 
  lastResult,
  error: optimizeError, 
  history,
  optimizeText, 
  clearResult,
  clearHistory,
  restoreFromHistory,
  getTypeLabel 
} = useTextOptimization()

// State
const inputText = ref(props.initialText || '')
const selectedType = ref<string>('improve')
const optimizationResult = ref<TextOptimizationResult | null>(null)
const error = ref<string | null>(null)
const successMessage = ref<string | null>(null)

// 初始化时确保选中第一个启用的类型
onMounted(() => {
  if (optimizationOptions.value.length > 0 && !optimizationOptions.value.find(o => o.value === selectedType.value)) {
    selectedType.value = optimizationOptions.value[0].value
  }
})

// Computed
const canOptimize = computed(() => {
  return inputText.value.trim().length > 0 && !isOptimizing.value
})

const characterDiff = computed(() => {
  if (!optimizationResult.value) return ''
  const diff = optimizationResult.value.optimized.length - optimizationResult.value.original.length
  if (diff > 0) return `+${diff}`
  if (diff < 0) return `${diff}`
  return '0'
})

const characterDiffClass = computed(() => {
  if (!optimizationResult.value) return ''
  const diff = optimizationResult.value.optimized.length - optimizationResult.value.original.length
  if (diff > 0) return 'diff-increase'
  if (diff < 0) return 'diff-decrease'
  return 'diff-same'
})

// Watch for initial text changes
watch(() => props.initialText, (newText) => {
  if (newText) {
    inputText.value = newText
  }
})

// Watch for external error
watch(optimizeError, (newError) => {
  if (newError) {
    error.value = newError
  }
})

// Methods
function selectType(type: string) {
  selectedType.value = type
}

async function handleOptimize() {
  if (!canOptimize.value) return

  error.value = null
  successMessage.value = null

  try {
    // 直接传原始文本和类型 ID，后端负责获取提示词模板并替换
    const result = await optimizeText(inputText.value, selectedType.value)
    
    optimizationResult.value = {
      original: inputText.value,
      optimized: result,
      type: selectedType.value,
      timestamp: new Date()
    }

    emit('optimized', optimizationResult.value)
    showSuccess('优化完成')
  } catch (e) {
    error.value = e instanceof Error ? e.message : '优化失败'
  }
}

function applyResult() {
  if (optimizationResult.value) {
    inputText.value = optimizationResult.value.optimized
    emit('apply', optimizationResult.value.optimized)
    showSuccess('已应用')
  }
}

async function copyResult() {
  if (optimizationResult.value) {
    try {
      await navigator.clipboard.writeText(optimizationResult.value.optimized)
      showSuccess('已复制')
    } catch {
      error.value = '复制失败'
    }
  }
}

function undoOptimization() {
  if (optimizationResult.value) {
    inputText.value = optimizationResult.value.original
    optimizationResult.value = null
    clearResult()
    showSuccess('已撤销')
  }
}

function handleRestoreFromHistory(id: string) {
  const result = restoreFromHistory(id)
  if (result) {
    inputText.value = result.original
    selectedType.value = result.type
    optimizationResult.value = result
  }
}

function handleClearHistory() {
  clearHistory()
  showSuccess('历史已清空')
}

function clearError() {
  error.value = null
}

function showSuccess(message: string) {
  successMessage.value = message
  setTimeout(() => {
    successMessage.value = null
  }, 2000)
}

function truncateText(text: string, maxLength: number): string {
  if (text.length <= maxLength) return text
  return text.substring(0, maxLength) + '...'
}

function formatTime(date: Date): string {
  return date.toLocaleTimeString('zh-CN', {
    hour: '2-digit',
    minute: '2-digit'
  })
}

function handleClose() {
  emit('close')
}

function handleOpenSettings() {
  emit('close')
  emit('openSettings')
}

// Expose for parent component
defineExpose({
  inputText,
  optimizationResult,
  applyResult,
  undoOptimization
})
</script>

<template>
  <Teleport to="body">
    <Transition name="modal">
      <div
        v-if="visible"
        class="modal-overlay"
        @click.self="handleClose"
      >
        <div class="optimizer-container">
          <!-- 头部 -->
          <div class="optimizer-header">
            <h3 class="optimizer-title">
              <span class="i-carbon-magic-wand title-icon" />
              文本优化
            </h3>
            <button
              class="close-btn"
              @click="handleClose"
            >
              <span class="i-carbon-close" />
            </button>
          </div>

          <div class="optimizer-body">
            <!-- API 未配置提示 -->
            <div
              v-if="!hasApiKey"
              class="api-warning"
            >
              <span class="i-carbon-warning-alt warning-icon" />
              <div class="warning-content">
                <span class="warning-title">未配置 API 密钥</span>
                <span class="warning-desc">请先配置 AI 服务 API 密钥才能使用文本优化功能</span>
              </div>
              <button
                class="goto-settings-btn"
                @click="handleOpenSettings"
              >
                <span class="i-carbon-settings" />
                去设置
              </button>
            </div>

            <!-- 优化类型选择器 -->
            <div class="type-section">
              <label class="section-label">优化类型</label>
              <div class="optimization-types">
                <button
                  v-for="option in optimizationOptions"
                  :key="option.value"
                  :class="['type-btn', { active: selectedType === option.value }]"
                  :disabled="isOptimizing || !hasApiKey"
                  :title="option.desc"
                  @click="selectType(option.value)"
                >
                  <span :class="['type-icon', option.icon]" />
                  <span class="type-label">{{ option.label }}</span>
                </button>
              </div>
            </div>

            <!-- 输入区域 -->
            <div class="input-section">
              <label class="section-label">原始文本</label>
              <textarea
                v-model="inputText"
                class="text-input"
                placeholder="输入要优化的文本..."
                :disabled="isOptimizing || !hasApiKey"
                rows="4"
              />
              <div class="input-footer">
                <span class="char-count">{{ inputText.length }} 字符</span>
                <button
                  class="optimize-btn"
                  :disabled="!canOptimize || !hasApiKey"
                  @click="handleOptimize"
                >
                  <span
                    v-if="isOptimizing"
                    class="loading-spinner"
                  />
                  <span
                    v-else
                    class="i-carbon-magic-wand"
                  />
                  <span>{{ isOptimizing ? '优化中...' : '开始优化' }}</span>
                </button>
              </div>
            </div>

            <!-- 优化结果预览 -->
            <Transition name="fade">
              <div
                v-if="optimizationResult"
                class="result-section"
              >
                <div class="result-header">
                  <label class="section-label">优化结果</label>
                  <div class="result-actions">
                    <button
                      class="action-btn apply-btn"
                      title="应用结果"
                      @click="applyResult"
                    >
                      <span class="i-carbon-checkmark" />
                      应用
                    </button>
                    <button
                      class="action-btn copy-btn"
                      title="复制"
                      @click="copyResult"
                    >
                      <span class="i-carbon-copy" />
                      复制
                    </button>
                    <button
                      class="action-btn undo-btn"
                      title="撤销"
                      @click="undoOptimization"
                    >
                      <span class="i-carbon-undo" />
                      撤销
                    </button>
                  </div>
                </div>
                <div class="result-content">
                  <div class="result-text">
                    {{ optimizationResult.optimized }}
                  </div>
                </div>
                <div class="result-meta">
                  <span class="meta-item">
                    <span class="i-carbon-tag" />
                    {{ getTypeLabel(optimizationResult.type) }}
                  </span>
                  <span :class="['meta-item', 'char-diff', characterDiffClass]">
                    <span class="i-carbon-text-tracking" />
                    {{ characterDiff }} 字符
                  </span>
                </div>
              </div>
            </Transition>

            <!-- 优化历史 -->
            <div
              v-if="history.length > 0"
              class="history-section"
            >
              <div class="history-header">
                <label class="section-label">
                  <span class="i-carbon-recently-viewed" />
                  历史记录
                </label>
                <button
                  class="clear-history-btn"
                  @click="handleClearHistory"
                >
                  清空
                </button>
              </div>
              <div class="history-list">
                <div
                  v-for="item in history"
                  :key="item.id"
                  class="history-item"
                  @click="handleRestoreFromHistory(item.id)"
                >
                  <div class="history-type">
                    <span class="type-badge">{{ getTypeLabel(item.type) }}</span>
                  </div>
                  <div class="history-preview">
                    {{ truncateText(item.optimized, 40) }}
                  </div>
                  <div class="history-time">
                    {{ formatTime(item.timestamp) }}
                  </div>
                </div>
              </div>
            </div>
          </div>

          <!-- 消息提示 -->
          <Transition name="toast">
            <div
              v-if="error"
              class="toast error-toast"
            >
              <span class="i-carbon-warning" />
              <span>{{ error }}</span>
              <button
                class="dismiss-btn"
                @click="clearError"
              >
                ×
              </button>
            </div>
          </Transition>

          <Transition name="toast">
            <div
              v-if="successMessage"
              class="toast success-toast"
            >
              <span class="i-carbon-checkmark-filled" />
              <span>{{ successMessage }}</span>
            </div>
          </Transition>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>


<style scoped>
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(0, 0, 0, 0.5);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 2000;
}

.optimizer-container {
  width: 90%;
  max-width: 600px;
  max-height: 85vh;
  background-color: var(--bg-primary);
  border-radius: 12px;
  box-shadow: var(--shadow-lg);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  position: relative;
}

.optimizer-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border-color);
}

.optimizer-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0;
}

.title-icon {
  color: var(--accent-color);
}

.close-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border: none;
  border-radius: 6px;
  background-color: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.2s;
}

.close-btn:hover {
  background-color: var(--bg-hover);
  color: var(--text-primary);
}

.optimizer-body {
  flex: 1;
  overflow-y: auto;
  padding: 16px 20px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

/* API 未配置警告 */
.api-warning {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 14px 16px;
  background-color: rgba(251, 191, 36, 0.1);
  border: 1px solid rgba(251, 191, 36, 0.3);
  border-radius: 10px;
}

.warning-icon {
  font-size: 24px;
  color: #f59e0b;
  flex-shrink: 0;
}

.warning-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.warning-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.warning-desc {
  font-size: 12px;
  color: var(--text-muted);
}

.goto-settings-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 14px;
  border: none;
  border-radius: 6px;
  background-color: var(--accent-color);
  color: white;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
  white-space: nowrap;
}

.goto-settings-btn:hover {
  background-color: var(--accent-hover);
}

.section-label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  font-weight: 500;
  color: var(--text-secondary);
  margin-bottom: 8px;
}

/* 优化类型选择器 */
.optimization-types {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.type-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 14px;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  background-color: var(--bg-secondary);
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.2s;
  font-size: 13px;
}

.type-btn:hover:not(:disabled) {
  border-color: var(--accent-color);
  background-color: var(--bg-hover);
  color: var(--text-primary);
}

.type-btn.active {
  border-color: var(--accent-color);
  background-color: var(--accent-color);
  color: white;
}

.type-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.type-icon {
  font-size: 16px;
}

/* 输入区域 */
.text-input {
  width: 100%;
  padding: 12px;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  background-color: var(--bg-input);
  color: var(--text-primary);
  font-size: 14px;
  line-height: 1.6;
  resize: vertical;
  font-family: inherit;
  outline: none;
  transition: border-color 0.2s;
}

.text-input:focus {
  border-color: var(--accent-color);
}

.text-input:disabled {
  background-color: var(--bg-disabled);
  cursor: not-allowed;
}

.text-input::placeholder {
  color: var(--text-muted);
}

.input-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: 8px;
}

.char-count {
  font-size: 12px;
  color: var(--text-muted);
}

.optimize-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 18px;
  border: none;
  border-radius: 8px;
  background-color: var(--accent-color);
  color: white;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: background-color 0.2s;
}

.optimize-btn:hover:not(:disabled) {
  background-color: var(--accent-hover);
}

.optimize-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.loading-spinner {
  width: 16px;
  height: 16px;
  border: 2px solid rgba(255, 255, 255, 0.3);
  border-top-color: white;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

/* 结果区域 */
.result-section {
  background-color: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  padding: 14px;
}

.result-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 10px;
}

.result-header .section-label {
  margin-bottom: 0;
}

.result-actions {
  display: flex;
  gap: 6px;
}

.action-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 6px 10px;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  background-color: var(--bg-primary);
  color: var(--text-secondary);
  font-size: 12px;
  cursor: pointer;
  transition: all 0.2s;
}

.action-btn:hover {
  background-color: var(--bg-hover);
}

.apply-btn:hover {
  border-color: var(--success-color);
  color: var(--success-color);
}

.copy-btn:hover {
  border-color: var(--accent-color);
  color: var(--accent-color);
}

.undo-btn:hover {
  border-color: var(--warning-color);
  color: var(--warning-color);
}

.result-content {
  background-color: var(--bg-primary);
  border-radius: 6px;
  padding: 12px;
  margin-bottom: 10px;
}

.result-text {
  color: var(--text-primary);
  font-size: 14px;
  line-height: 1.6;
  white-space: pre-wrap;
  word-break: break-word;
}

.result-meta {
  display: flex;
  gap: 16px;
  font-size: 12px;
}

.meta-item {
  display: flex;
  align-items: center;
  gap: 4px;
  color: var(--text-muted);
}

.char-diff.diff-increase { color: var(--success-color); }
.char-diff.diff-decrease { color: var(--error-color); }
.char-diff.diff-same { color: var(--text-muted); }

/* 历史记录 */
.history-section {
  border-top: 1px solid var(--border-color);
  padding-top: 16px;
}

.history-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 10px;
}

.history-header .section-label {
  margin-bottom: 0;
}

.clear-history-btn {
  padding: 4px 10px;
  border: none;
  border-radius: 4px;
  background-color: transparent;
  color: var(--text-muted);
  font-size: 12px;
  cursor: pointer;
  transition: all 0.2s;
}

.clear-history-btn:hover {
  background-color: var(--bg-hover);
  color: var(--error-color);
}

.history-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
  max-height: 150px;
  overflow-y: auto;
}

.history-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 10px;
  background-color: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.2s;
}

.history-item:hover {
  background-color: var(--bg-hover);
  border-color: var(--accent-color);
}

.type-badge {
  padding: 2px 8px;
  background-color: var(--accent-light);
  color: var(--accent-color);
  border-radius: 4px;
  font-size: 11px;
  font-weight: 500;
}

.history-preview {
  flex: 1;
  font-size: 13px;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.history-time {
  font-size: 11px;
  color: var(--text-muted);
}

/* Toast 消息 */
.toast {
  position: absolute;
  bottom: 16px;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 16px;
  border-radius: 8px;
  font-size: 13px;
  box-shadow: var(--shadow-md);
}

.error-toast {
  background-color: var(--error-light);
  color: var(--error-color);
}

.success-toast {
  background-color: var(--success-light);
  color: var(--success-color);
}

.dismiss-btn {
  margin-left: 8px;
  padding: 0;
  border: none;
  background: none;
  color: inherit;
  font-size: 18px;
  cursor: pointer;
  opacity: 0.7;
}

.dismiss-btn:hover {
  opacity: 1;
}

/* 动画 */
.modal-enter-active,
.modal-leave-active {
  transition: all 0.3s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}

.modal-enter-from .optimizer-container,
.modal-leave-to .optimizer-container {
  transform: scale(0.95);
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

.toast-enter-active,
.toast-leave-active {
  transition: all 0.3s ease;
}

.toast-enter-from,
.toast-leave-to {
  opacity: 0;
  transform: translateX(-50%) translateY(10px);
}
</style>

<script setup lang="ts">
/**
 * 设置面板组件
 * 
 * 整合所有设置选项卡的统一设置界面
 * Requirements: 7.1-7.5, 9.1, 10.1, 11.4, 12.2, 13.1, 13.2
 */
import { ref, onMounted, computed, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { VueDraggable } from 'vue-draggable-plus'
import { useTheme, THEME_COLORS } from '@/composables/useTheme'
import { useLayout } from '@/composables/useLayout'
import { useDisplayMode } from '@/composables/useDisplayMode'
import { PROVIDERS, type ApiProvider, type ProviderInfo } from '@/composables/useApiKeys'
import { useConfigStore } from '@/stores/config'
import AudioSettings from './AudioSettings.vue'
import WindowSettings from './WindowSettings.vue'

// Props
const props = defineProps<{
  visible: boolean
}>()

// Emits
const emit = defineEmits<{
  (e: 'close'): void
}>()

// 选项卡
type TabId = 'theme' | 'font' | 'layout' | 'optimize' | 'api' | 'audio' | 'window'
const activeTab = ref<TabId>('theme')

const tabs: { id: TabId; label: string; icon: string }[] = [
  { id: 'theme', label: '主题', icon: 'i-carbon-color-palette' },
  { id: 'font', label: '字体', icon: 'i-carbon-text-font' },
  { id: 'layout', label: '布局', icon: 'i-carbon-grid' },
  { id: 'optimize', label: '优化', icon: 'i-carbon-text-creation' },
  { id: 'audio', label: '音频', icon: 'i-carbon-volume-up' },
  { id: 'window', label: '窗口', icon: 'i-carbon-application' },
  { id: 'api', label: 'API', icon: 'i-carbon-api' },
]

// 主题设置
const { theme, themeColor, isDark, setTheme, setThemeColor } = useTheme()

// 字体设置
const configStore = useConfigStore()
const fontSize = computed(() => configStore.fontSize)
function setFontSize(area: 'display' | 'options' | 'input', size: number) {
  configStore.setFontSize(area, size)
}

// 布局设置
const { layout, isVertical, setLayout } = useLayout()

// 自定义选项
const customOptionsExpanded = ref(false)
const newCustomOption = ref('')

function addNewCustomOption() {
  if (newCustomOption.value.trim()) {
    configStore.addCustomOption(newCustomOption.value)
    newCustomOption.value = ''
  }
}

// 优化类型设置
const expandedOptimizationType = ref<string | null>(null)
const newOptimizationType = ref({
  label: '',
  icon: 'i-carbon-edit',
  description: '',
  prompt: '',
  enabled: true,
})
const showAddOptimizationType = ref(false)

// 可选图标列表
const availableIcons = [
  { value: 'i-carbon-edit', label: '编辑' },
  { value: 'i-carbon-improve-relevance', label: '改进' },
  { value: 'i-carbon-application-web', label: '前端' },
  { value: 'i-carbon-data-base', label: '数据库' },
  { value: 'i-carbon-chart-network', label: '架构' },
  { value: 'i-carbon-checkmark-outline', label: '审查' },
  { value: 'i-carbon-document', label: '文档' },
  { value: 'i-carbon-code', label: '代码' },
  { value: 'i-carbon-translate', label: '翻译' },
  { value: 'i-carbon-star', label: '星标' },
  { value: 'i-carbon-terminal', label: '终端' },
  { value: 'i-carbon-api', label: 'API' },
  { value: 'i-carbon-bot', label: '机器人' },
  { value: 'i-carbon-cloud', label: '云' },
]

function toggleOptimizationTypeExpand(id: string) {
  expandedOptimizationType.value = expandedOptimizationType.value === id ? null : id
}

function addNewOptimizationType() {
  if (newOptimizationType.value.label.trim() && newOptimizationType.value.prompt.trim()) {
    configStore.addOptimizationType({
      label: newOptimizationType.value.label.trim(),
      icon: newOptimizationType.value.icon,
      description: newOptimizationType.value.description.trim() || newOptimizationType.value.label.trim(),
      prompt: newOptimizationType.value.prompt.trim(),
      enabled: true,
    })
    // 重置表单
    newOptimizationType.value = {
      label: '',
      icon: 'i-carbon-edit',
      description: '',
      prompt: '',
      enabled: true,
    }
    showAddOptimizationType.value = false
  }
}

function cancelAddOptimizationType() {
  newOptimizationType.value = {
    label: '',
    icon: 'i-carbon-edit',
    description: '',
    prompt: '',
    enabled: true,
  }
  showAddOptimizationType.value = false
}

// 显示模式
const displayModeParams = ref({ message: '', fullResponse: '' })
const { displayMode, setDisplayMode } = useDisplayMode(displayModeParams)

// API 密钥管理 - 使用 configStore 统一管理状态
const apiLoading = ref(false)
const apiError = ref<string | null>(null)

// 从 configStore 获取状态
const configuredProviders = computed(() => configStore.configuredProviders)
const maskedKeys = computed(() => configStore.maskedApiKeys)

// 响应式检查提供商是否已配置
function isProviderConfigured(provider: ApiProvider): boolean {
  return configuredProviders.value.includes(provider)
}

function getMaskedKey(provider: ApiProvider): string | undefined {
  return maskedKeys.value[provider]
}

// API 密钥输入状态
const apiKeyInputs = ref<Record<string, string>>({})
const editingProvider = ref<string | null>(null)
const savingProvider = ref<string | null>(null)
const saveSuccess = ref<string | null>(null)

// API 测试状态
const testingProvider = ref<string | null>(null)
const testResults = ref<Record<string, 'success' | 'failed' | null>>({})
const testErrors = ref<Record<string, string>>({})

// 拖拽排序 - 使用 vue-draggable-plus
// 只包含测试通过的提供商（可拖动排序）
const testedProviderList = ref<ProviderInfo[]>([])

// 计算属性：未测试的提供商列表（不参与拖动）
const untestedProviderList = computed(() => {
  return PROVIDERS.filter(p => {
    const isConfigured = configuredProviders.value.includes(p.id)
    const isTested = configStore.isProviderTested(p.id)
    // 已配置但未测试，或未配置的
    return (isConfigured && !isTested) || !isConfigured
  })
})

// 初始化排序列表 - 只包含测试通过的
function initSortedProviderList() {
  const order = configStore.providerOrder
  
  // 只取测试通过的提供商
  const testedList = [...PROVIDERS]
    .filter(p => configStore.isProviderTested(p.id))
    .sort((a, b) => {
      const aIndex = order.indexOf(a.id)
      const bIndex = order.indexOf(b.id)
      if (aIndex === -1 && bIndex === -1) return 0
      if (aIndex === -1) return 1
      if (bIndex === -1) return -1
      return aIndex - bIndex
    })
  
  testedProviderList.value = testedList
}

// 监听 configuredProviders 变化（添加/删除 API key 时）
watch(configuredProviders, (newVal, oldVal) => {
  // 只在配置的提供商数量变化时重新初始化
  if (newVal.length !== oldVal?.length) {
    initSortedProviderList()
  }
})

// 监听测试状态变化
watch(() => configStore.testedProviders, () => {
  initSortedProviderList()
}, { deep: true })

// 拖拽结束时保存顺序 - 使用 @end 事件确保 v-model 已更新
async function onDragEnd() {
  const newOrder = testedProviderList.value.map(p => p.id)
  
  console.log('[Drag] End order:', newOrder)
  
  try {
    await configStore.setProviderOrder(newOrder)
  } catch (e) {
    console.error('[Drag] Failed to save order:', e)
    apiError.value = e instanceof Error ? e.message : String(e)
  }
}

// 初始化
onMounted(async () => {
  await configStore.refreshApiKeyStatus()
  initSortedProviderList()
})

/**
 * 开始编辑 API 密钥
 */
function startEditApiKey(provider: ApiProvider) {
  editingProvider.value = provider
  apiKeyInputs.value[provider] = ''
}

/**
 * 取消编辑
 */
function cancelEditApiKey() {
  if (editingProvider.value) {
    apiKeyInputs.value[editingProvider.value] = ''
  }
  editingProvider.value = null
}

/**
 * 保存 API 密钥
 */
async function handleSaveApiKey(provider: ApiProvider) {
  const key = apiKeyInputs.value[provider]
  if (!key?.trim()) return
  
  savingProvider.value = provider
  apiError.value = null
  try {
    await configStore.setApiKey(provider, key.trim())
    apiKeyInputs.value[provider] = ''
    editingProvider.value = null
    saveSuccess.value = provider
    setTimeout(() => {
      saveSuccess.value = null
    }, 2000)
  } catch (e) {
    apiError.value = e instanceof Error ? e.message : String(e)
    console.error('Failed to save API key:', e)
  } finally {
    savingProvider.value = null
  }
}

/**
 * 删除 API 密钥
 */
async function handleDeleteApiKey(provider: ApiProvider) {
  savingProvider.value = provider
  apiError.value = null
  try {
    await configStore.deleteApiKey(provider)
    // 清除测试状态
    testResults.value[provider] = null
    delete testErrors.value[provider]
    await configStore.setProviderTested(provider, false)
  } catch (e) {
    apiError.value = e instanceof Error ? e.message : String(e)
    console.error('Failed to delete API key:', e)
  } finally {
    savingProvider.value = null
  }
}

/**
 * 测试 API 连接
 */
async function handleTestConnection(provider: ApiProvider) {
  testingProvider.value = provider
  testResults.value[provider] = null
  delete testErrors.value[provider]
  
  try {
    await invoke<string>('test_api_connection', { provider })
    testResults.value[provider] = 'success'
    // 保存测试状态到配置文件
    await configStore.setProviderTested(provider, true)
  } catch (e) {
    testResults.value[provider] = 'failed'
    testErrors.value[provider] = e instanceof Error ? e.message : String(e)
    // 清除测试状态
    await configStore.setProviderTested(provider, false)
  } finally {
    testingProvider.value = null
  }
}

/**
 * 重置 API 密钥（删除）
 */
async function handleResetApiKey(provider: ApiProvider) {
  await handleDeleteApiKey(provider)
}

/**
 * 获取提供商优先级
 */
function getProviderPriority(provider: ApiProvider): number {
  return configStore.getProviderPriority(provider)
}

/**
 * 设置提供商优先级
 */
async function handleSetPriority(provider: ApiProvider, priority: number) {
  try {
    await configStore.setProviderPriority(provider, priority)
  } catch (e) {
    apiError.value = e instanceof Error ? e.message : String(e)
    console.error('Failed to set provider priority:', e)
  }
}

/**
 * 获取优先级选项（1 到已配置提供商数量）
 */
function getPriorityOptions(): number[] {
  const count = configuredProviders.value.length
  return Array.from({ length: count }, (_, i) => i + 1)
}

/**
 * 关闭面板
 */
function handleClose() {
  emit('close')
}
</script>

<template>
  <Teleport to="body">
    <Transition name="fade">
      <div
        v-if="visible"
        class="settings-overlay"
        @click.self="handleClose"
      >
        <div class="settings-panel">
          <!-- 头部 -->
          <div class="panel-header">
            <h2 class="panel-title">
              <span class="i-carbon-settings" />
              设置
            </h2>
            <button
              class="close-btn"
              @click="handleClose"
            >
              <span class="i-carbon-close" />
            </button>
          </div>
          
          <!-- 选项卡导航 -->
          <div class="tabs-nav">
            <button
              v-for="tab in tabs"
              :key="tab.id"
              class="tab-btn"
              :class="{ active: activeTab === tab.id }"
              @click="activeTab = tab.id"
            >
              <span :class="tab.icon" />
              {{ tab.label }}
            </button>
          </div>
          
          <!-- 选项卡内容 -->
          <div class="tabs-content">
            <!-- 主题设置 -->
            <div
              v-show="activeTab === 'theme'"
              class="tab-panel"
            >
              <div class="settings-section">
                <h3 class="section-title">
                  外观主题
                </h3>
                <div class="theme-options">
                  <button
                    class="theme-option"
                    :class="{ active: isDark }"
                    @click="setTheme('dark')"
                  >
                    <span class="i-carbon-moon option-icon" />
                    <span class="option-label">深色</span>
                    <span
                      v-if="isDark"
                      class="i-carbon-checkmark check-icon"
                    />
                  </button>
                  <button
                    class="theme-option"
                    :class="{ active: !isDark }"
                    @click="setTheme('light')"
                  >
                    <span class="i-carbon-sun option-icon" />
                    <span class="option-label">浅色</span>
                    <span
                      v-if="!isDark"
                      class="i-carbon-checkmark check-icon"
                    />
                  </button>
                </div>
              </div>
              
              <div class="settings-section">
                <h3 class="section-title">
                  主题色
                </h3>
                <div class="color-options">
                  <button
                    v-for="color in THEME_COLORS"
                    :key="color.id"
                    class="color-option"
                    :class="{ active: themeColor === color.id }"
                    :style="{ '--color-preview': color.color }"
                    :title="color.name"
                    @click="setThemeColor(color.id)"
                  >
                    <span class="color-swatch" />
                  </button>
                </div>
              </div>
              
              <div class="settings-section">
                <h3 class="section-title">
                  显示模式
                </h3>
                <div class="display-mode-options">
                  <button
                    class="mode-option"
                    :class="{ active: displayMode === 'simple' }"
                    @click="setDisplayMode('simple')"
                  >
                    <span class="i-carbon-text-short-paragraph option-icon" />
                    <div class="option-content">
                      <span class="option-label">简洁模式</span>
                      <span class="option-desc">显示简短消息</span>
                    </div>
                    <span
                      v-if="displayMode === 'simple'"
                      class="i-carbon-checkmark check-icon"
                    />
                  </button>
                  <button
                    class="mode-option"
                    :class="{ active: displayMode === 'full' }"
                    @click="setDisplayMode('full')"
                  >
                    <span class="i-carbon-text-long-paragraph option-icon" />
                    <div class="option-content">
                      <span class="option-label">完整模式</span>
                      <span class="option-desc">显示完整响应</span>
                    </div>
                    <span
                      v-if="displayMode === 'full'"
                      class="i-carbon-checkmark check-icon"
                    />
                  </button>
                </div>
              </div>
            </div>
            
            <!-- 字体设置 -->
            <div
              v-show="activeTab === 'font'"
              class="tab-panel"
            >
              <div class="settings-section">
                <h3 class="section-title">
                  字体大小
                </h3>
                <div class="font-size-options">
                  <div class="font-size-item">
                    <span class="font-size-label">提示区文字大小</span>
                    <div class="font-size-control">
                      <input
                        type="range"
                        :value="fontSize.display"
                        min="12"
                        max="20"
                        step="1"
                        @input="setFontSize('display', Number(($event.target as HTMLInputElement).value))"
                      >
                      <span class="font-size-value">{{ fontSize.display }}</span>
                    </div>
                  </div>
                  <div class="font-size-item">
                    <span class="font-size-label">选项区文字大小</span>
                    <div class="font-size-control">
                      <input
                        type="range"
                        :value="fontSize.options"
                        min="12"
                        max="20"
                        step="1"
                        @input="setFontSize('options', Number(($event.target as HTMLInputElement).value))"
                      >
                      <span class="font-size-value">{{ fontSize.options }}</span>
                    </div>
                  </div>
                  <div class="font-size-item">
                    <span class="font-size-label">输入框文字大小</span>
                    <div class="font-size-control">
                      <input
                        type="range"
                        :value="fontSize.input"
                        min="12"
                        max="20"
                        step="1"
                        @input="setFontSize('input', Number(($event.target as HTMLInputElement).value))"
                      >
                      <span class="font-size-value">{{ fontSize.input }}</span>
                    </div>
                  </div>
                </div>
              </div>
            </div>
            
            <!-- 布局设置 -->
            <div
              v-show="activeTab === 'layout'"
              class="tab-panel"
            >
              <div class="settings-section">
                <h3 class="section-title">
                  布局方向
                </h3>
                <div class="layout-options">
                  <button
                    class="layout-option"
                    :class="{ active: isVertical }"
                    @click="setLayout('vertical')"
                  >
                    <div class="layout-preview vertical-preview">
                      <div class="preview-top" />
                      <div class="preview-bottom" />
                    </div>
                    <span class="option-label">垂直布局</span>
                    <span
                      v-if="isVertical"
                      class="i-carbon-checkmark check-icon"
                    />
                  </button>
                  <button
                    class="layout-option"
                    :class="{ active: !isVertical }"
                    @click="setLayout('horizontal')"
                  >
                    <div class="layout-preview horizontal-preview">
                      <div class="preview-left" />
                      <div class="preview-right" />
                    </div>
                    <span class="option-label">水平布局</span>
                    <span
                      v-if="!isVertical"
                      class="i-carbon-checkmark check-icon"
                    />
                  </button>
                </div>
                <p class="section-hint">
                  <span class="i-carbon-information" />
                  双击分割线可重置为默认比例
                </p>
              </div>
              
              <!-- 自定义选项设置 -->
              <div class="settings-section">
                <div class="section-header-row">
                  <h3 class="section-title">
                    启用自定义选项
                  </h3>
                  <label class="toggle-switch">
                    <input 
                      type="checkbox" 
                      :checked="configStore.customOptionsEnabled"
                      @change="configStore.setCustomOptionsEnabled(($event.target as HTMLInputElement).checked)"
                    >
                    <span class="toggle-slider" />
                  </label>
                </div>
                
                <div
                  v-if="configStore.customOptionsEnabled"
                  class="custom-options-section"
                >
                  <div 
                    class="collapse-header"
                    @click="customOptionsExpanded = !customOptionsExpanded"
                  >
                    <span
                      class="collapse-icon"
                      :class="{ expanded: customOptionsExpanded }"
                    >
                      <span class="i-carbon-chevron-right" />
                    </span>
                    <span>{{ customOptionsExpanded ? '收起' : '展开' }}选项设置</span>
                  </div>
                  
                  <div
                    v-show="customOptionsExpanded"
                    class="custom-options-list"
                  >
                    <div 
                      v-for="(option, index) in configStore.customOptions" 
                      :key="index"
                      class="custom-option-item"
                    >
                      <span class="option-label">选项 {{ index + 1 }}:</span>
                      <input 
                        type="text" 
                        class="option-input"
                        :value="option"
                        @blur="configStore.updateCustomOption(index, ($event.target as HTMLInputElement).value)"
                        @keydown.enter="($event.target as HTMLInputElement).blur()"
                      >
                      <button 
                        class="option-remove-btn"
                        title="删除选项"
                        @click="configStore.removeCustomOption(index)"
                      >
                        <span class="i-carbon-close" />
                      </button>
                    </div>
                    
                    <div class="add-option-row">
                      <input 
                        v-model="newCustomOption" 
                        type="text"
                        class="option-input add-input"
                        placeholder="输入新选项..."
                        @keydown.enter="addNewCustomOption"
                      >
                      <button 
                        class="option-add-btn"
                        :disabled="!newCustomOption.trim()"
                        @click="addNewCustomOption"
                      >
                        <span class="i-carbon-add" />
                        添加
                      </button>
                    </div>
                  </div>
                </div>
              </div>
            </div>
            
            <!-- 文本优化设置 -->
            <div
              v-show="activeTab === 'optimize'"
              class="tab-panel"
            >
              <div class="settings-section">
                <h3 class="section-title">
                  优化类型管理
                </h3>
                <p class="section-desc">
                  配置文本优化类型，自定义提示词模板。使用 {text} 作为原文占位符。
                </p>
                
                <!-- 优化类型列表 -->
                <div class="optimization-types-list">
                  <div 
                    v-for="optType in configStore.optimizationTypes" 
                    :key="optType.id"
                    class="optimization-type-item"
                    :class="{ 
                      'is-expanded': expandedOptimizationType === optType.id,
                      'is-disabled': !optType.enabled,
                      'is-system': optType.isSystem
                    }"
                  >
                    <!-- 类型头部 -->
                    <div
                      class="opt-type-header"
                      @click="toggleOptimizationTypeExpand(optType.id)"
                    >
                      <div class="opt-type-info">
                        <label
                          class="toggle-switch-small"
                          @click.stop
                        >
                          <input 
                            type="checkbox" 
                            :checked="optType.enabled"
                            @change="configStore.toggleOptimizationType(optType.id)"
                          >
                          <span class="toggle-slider-small" />
                        </label>
                        <span
                          :class="optType.icon"
                          class="opt-type-icon"
                        />
                        <span class="opt-type-label">{{ optType.label }}</span>
                        <span
                          v-if="optType.isSystem"
                          class="system-badge"
                        >系统</span>
                      </div>
                      <div class="opt-type-actions">
                        <span class="opt-type-desc">{{ optType.description }}</span>
                        <span 
                          class="expand-icon"
                          :class="{ 'is-expanded': expandedOptimizationType === optType.id }"
                        >
                          <span class="i-carbon-chevron-down" />
                        </span>
                      </div>
                    </div>
                    
                    <!-- 类型详情（展开时显示） -->
                    <Transition name="slide-down">
                      <div
                        v-if="expandedOptimizationType === optType.id"
                        class="opt-type-details"
                      >
                        <div class="opt-detail-row">
                          <label class="detail-label">名称</label>
                          <input 
                            type="text" 
                            class="detail-input"
                            :value="optType.label"
                            @input="configStore.updateOptimizationType(optType.id, { label: ($event.target as HTMLInputElement).value })"
                          >
                        </div>
                        <div class="opt-detail-row">
                          <label class="detail-label">描述</label>
                          <input 
                            type="text" 
                            class="detail-input"
                            :value="optType.description"
                            @input="configStore.updateOptimizationType(optType.id, { description: ($event.target as HTMLInputElement).value })"
                          >
                        </div>
                        <div class="opt-detail-row">
                          <label class="detail-label">图标</label>
                          <select 
                            class="detail-select"
                            :value="optType.icon"
                            @change="configStore.updateOptimizationType(optType.id, { icon: ($event.target as HTMLSelectElement).value })"
                          >
                            <option
                              v-for="icon in availableIcons"
                              :key="icon.value"
                              :value="icon.value"
                            >
                              {{ icon.label }}
                            </option>
                          </select>
                        </div>
                        <div class="opt-detail-row opt-detail-row-full">
                          <label class="detail-label">提示词模板</label>
                          <textarea 
                            class="detail-textarea"
                            :value="optType.prompt"
                            placeholder="使用 {text} 作为原文占位符"
                            rows="4"
                            @input="configStore.updateOptimizationType(optType.id, { prompt: ($event.target as HTMLTextAreaElement).value })"
                          />
                        </div>
                        <div
                          v-if="!optType.isSystem"
                          class="opt-detail-actions"
                        >
                          <button 
                            class="btn-danger-small"
                            @click="configStore.removeOptimizationType(optType.id)"
                          >
                            <span class="i-carbon-trash-can" />
                            删除此类型
                          </button>
                        </div>
                      </div>
                    </Transition>
                  </div>
                </div>
                
                <!-- 添加新类型 -->
                <div class="add-optimization-type">
                  <button 
                    v-if="!showAddOptimizationType"
                    class="btn-add-type"
                    @click="showAddOptimizationType = true"
                  >
                    <span class="i-carbon-add" />
                    添加自定义类型
                  </button>
                  
                  <Transition name="slide-down">
                    <div
                      v-if="showAddOptimizationType"
                      class="new-type-form"
                    >
                      <div class="form-row">
                        <div class="form-group">
                          <label>名称 *</label>
                          <input 
                            v-model="newOptimizationType.label" 
                            type="text"
                            placeholder="例如：翻译"
                            class="form-input"
                          >
                        </div>
                        <div class="form-group">
                          <label>图标</label>
                          <select
                            v-model="newOptimizationType.icon"
                            class="form-select"
                          >
                            <option
                              v-for="icon in availableIcons"
                              :key="icon.value"
                              :value="icon.value"
                            >
                              {{ icon.label }}
                            </option>
                          </select>
                        </div>
                      </div>
                      <div class="form-group">
                        <label>描述</label>
                        <input 
                          v-model="newOptimizationType.description" 
                          type="text"
                          placeholder="简短描述此优化类型的作用"
                          class="form-input"
                        >
                      </div>
                      <div class="form-group">
                        <label>提示词模板 *</label>
                        <textarea 
                          v-model="newOptimizationType.prompt"
                          placeholder="使用 {text} 作为原文占位符&#10;例如：请将以下文本翻译成英文：&#10;&#10;{text}"
                          class="form-textarea"
                          rows="4"
                        />
                      </div>
                      <div class="form-actions">
                        <button
                          class="btn-cancel"
                          @click="cancelAddOptimizationType"
                        >
                          取消
                        </button>
                        <button 
                          class="btn-confirm"
                          :disabled="!newOptimizationType.label.trim() || !newOptimizationType.prompt.trim()"
                          @click="addNewOptimizationType"
                        >
                          <span class="i-carbon-add" />
                          添加
                        </button>
                      </div>
                    </div>
                  </Transition>
                </div>
                
                <!-- 重置按钮 -->
                <div class="reset-section">
                  <button
                    class="btn-reset"
                    @click="configStore.resetOptimizationTypes()"
                  >
                    <span class="i-carbon-reset" />
                    重置为默认
                  </button>
                </div>
              </div>
            </div>
            
            <!-- API 设置 -->
            <div
              v-show="activeTab === 'api'"
              class="tab-panel"
            >
              <div class="settings-section">
                <h3 class="section-title">
                  AI 服务 API 密钥
                </h3>
                <p class="section-desc">
                  配置 AI 服务提供商的 API 密钥以启用文本优化功能。拖拽已连接的服务可调整优先级。
                </p>
                
                <!-- 已测试通过的提供商（可拖动排序） -->
                <div
                  v-if="testedProviderList.length > 0"
                  class="provider-section"
                >
                  <div class="section-label">
                    <span class="i-carbon-checkmark-filled text-green-500" />
                    连接正常（可拖动排序）
                  </div>
                  <VueDraggable
                    v-model="testedProviderList"
                    class="api-providers"
                    :animation="150"
                    handle=".drag-handle"
                    ghost-class="provider-ghost"
                    drag-class="provider-drag"
                    :force-fallback="true"
                    fallback-class="provider-fallback"
                    :fallback-tolerance="3"
                    :delay="0"
                    :touch-start-threshold="0"
                    @end="onDragEnd"
                  >
                    <div
                      v-for="provider in testedProviderList"
                      :key="provider.id"
                      class="provider-item is-configured is-tested"
                    >
                      <!-- 拖拽手柄 -->
                      <div
                        class="drag-handle"
                        title="拖拽排序"
                      >
                        <span class="i-carbon-menu" />
                      </div>
                      
                      <div class="provider-info">
                        <span class="provider-name">{{ provider.name }}</span>
                        <span class="provider-desc">{{ provider.description }}</span>
                        <span class="provider-status configured">
                          <span class="i-carbon-checkmark-filled" />
                          已配置: {{ getMaskedKey(provider.id) }}
                        </span>
                      </div>
                      
                      <div class="provider-actions">
                        <span class="test-success">
                          <span class="i-carbon-checkmark-filled" /> 连接正常
                        </span>
                        <button
                          class="action-btn reset-btn"
                          :disabled="savingProvider === provider.id"
                          @click="handleResetApiKey(provider.id)"
                        >
                          <span
                            v-if="savingProvider === provider.id"
                            class="i-carbon-loading animate-spin"
                          />
                          <span
                            v-else
                            class="i-carbon-reset"
                          />
                          重置
                        </button>
                      </div>
                    </div>
                  </VueDraggable>
                </div>
                
                <!-- 未测试的提供商（不参与拖动） -->
                <div
                  v-if="untestedProviderList.length > 0"
                  class="provider-section"
                >
                  <div class="section-label">
                    <span class="i-carbon-settings text-gray-400" />
                    待配置/待测试
                  </div>
                  <div class="api-providers">
                    <div
                      v-for="provider in untestedProviderList"
                      :key="provider.id"
                      class="provider-item"
                      :class="{ 'is-configured': isProviderConfigured(provider.id) }"
                    >
                      <div class="drag-handle-placeholder" />
                      
                      <div class="provider-info">
                        <span class="provider-name">{{ provider.name }}</span>
                        <span class="provider-desc">{{ provider.description }}</span>
                        <span
                          v-if="isProviderConfigured(provider.id)"
                          class="provider-status configured"
                        >
                          <span class="i-carbon-checkmark-filled" />
                          已配置: {{ getMaskedKey(provider.id) }}
                        </span>
                      </div>
                      
                      <div class="provider-actions">
                        <!-- 编辑模式（添加新密钥） -->
                        <template v-if="editingProvider === provider.id">
                          <input
                            v-model="apiKeyInputs[provider.id]"
                            type="password"
                            class="api-key-input"
                            :placeholder="provider.placeholder"
                            @keyup.enter="handleSaveApiKey(provider.id)"
                          >
                          <button
                            class="action-btn save-btn"
                            :disabled="!apiKeyInputs[provider.id]?.trim() || savingProvider === provider.id"
                            @click="handleSaveApiKey(provider.id)"
                          >
                            <span
                              v-if="savingProvider === provider.id"
                              class="i-carbon-loading animate-spin"
                            />
                            <span
                              v-else
                              class="i-carbon-checkmark"
                            />
                          </button>
                          <button
                            class="action-btn cancel-btn"
                            @click="cancelEditApiKey"
                          >
                            <span class="i-carbon-close" />
                          </button>
                        </template>
                        
                        <!-- 查看模式 -->
                        <template v-else>
                          <!-- 未配置：显示添加按钮 -->
                          <template v-if="!isProviderConfigured(provider.id)">
                            <button
                              class="action-btn edit-btn"
                              @click="startEditApiKey(provider.id)"
                            >
                              <span class="i-carbon-add" />
                              添加
                            </button>
                          </template>
                          
                          <!-- 已配置但未测试 -->
                          <template v-else>
                            <!-- 保存成功提示 -->
                            <span
                              v-if="saveSuccess === provider.id"
                              class="save-success"
                            >
                              <span class="i-carbon-checkmark" /> 已保存
                            </span>
                            
                            <template v-else>
                              <!-- 测试失败提示 -->
                              <span
                                v-if="testResults[provider.id] === 'failed'"
                                class="test-failed"
                                :title="testErrors[provider.id]"
                              >
                                <span class="i-carbon-warning-filled" /> 连接失败
                              </span>
                              
                              <!-- 测试按钮 -->
                              <button
                                class="action-btn test-btn"
                                :disabled="testingProvider === provider.id"
                                @click="handleTestConnection(provider.id)"
                              >
                                <span
                                  v-if="testingProvider === provider.id"
                                  class="i-carbon-loading animate-spin"
                                />
                                <span
                                  v-else
                                  class="i-carbon-connection-signal"
                                />
                                测试
                              </button>
                              
                              <!-- 重置按钮 -->
                              <button
                                class="action-btn reset-btn"
                                :disabled="savingProvider === provider.id"
                                @click="handleResetApiKey(provider.id)"
                              >
                                <span
                                  v-if="savingProvider === provider.id"
                                  class="i-carbon-loading animate-spin"
                                />
                                <span
                                  v-else
                                  class="i-carbon-reset"
                                />
                                重置
                              </button>
                            </template>
                          </template>
                        </template>
                      </div>
                    </div>
                  </div>
                </div>
                
                <div
                  v-if="apiError"
                  class="error-message"
                >
                  <span class="i-carbon-warning" />
                  {{ apiError }}
                </div>
              </div>
            </div>
            
            <!-- 音频设置 -->
            <div
              v-show="activeTab === 'audio'"
              class="tab-panel"
            >
              <AudioSettings />
            </div>
            
            <!-- 窗口设置 -->
            <div
              v-show="activeTab === 'window'"
              class="tab-panel"
            >
              <WindowSettings />
            </div>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>


<style scoped>
.settings-overlay {
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
  z-index: 1000;
}

.settings-panel {
  width: 90%;
  max-width: 600px;
  max-height: 80vh;
  background-color: var(--bg-primary);
  border-radius: 12px;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border-color);
}

.panel-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0;
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
  background-color: var(--bg-secondary);
  color: var(--text-primary);
}

/* 选项卡导航 */
.tabs-nav {
  display: flex;
  gap: 4px;
  padding: 12px 16px;
  border-bottom: 1px solid var(--border-color);
  overflow-x: auto;
  flex-shrink: 0;
  min-height: 52px;
}

.tab-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
  border: none;
  border-radius: 6px;
  background-color: transparent;
  color: var(--text-secondary);
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s;
  white-space: nowrap;
}

.tab-btn:hover {
  background-color: var(--bg-secondary);
  color: var(--text-primary);
}

.tab-btn.active {
  background-color: var(--accent-color);
  color: white;
}

/* 选项卡内容 */
.tabs-content {
  flex: 1;
  overflow-y: auto;
  padding: 20px;
}

.tab-panel {
  animation: fadeIn 0.2s ease;
}

@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

/* 设置区块 */
.settings-section {
  margin-bottom: 24px;
}

.settings-section:last-child {
  margin-bottom: 0;
}

.section-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0 0 12px 0;
}

.section-desc {
  font-size: 13px;
  color: var(--text-muted);
  margin: 0 0 16px 0;
}

.section-hint {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: var(--text-muted);
  margin-top: 12px;
}

/* 主题选项 */
.theme-options,
.display-mode-options,
.layout-options {
  display: flex;
  gap: 12px;
}

.theme-option,
.layout-option {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 16px;
  border: 2px solid var(--border-color);
  border-radius: 8px;
  background-color: var(--bg-secondary);
  cursor: pointer;
  transition: all 0.2s;
}

.theme-option:hover,
.layout-option:hover {
  border-color: var(--accent-color);
}

.theme-option.active,
.layout-option.active {
  border-color: var(--accent-color);
  background-color: rgba(var(--accent-rgb), 0.1);
}

.option-icon {
  font-size: 24px;
  color: var(--text-secondary);
}

.theme-option.active .option-icon,
.layout-option.active .option-icon {
  color: var(--accent-color);
}

.option-label {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
}

.check-icon {
  color: var(--accent-color);
}

/* 显示模式选项 */
.mode-option {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  border: 2px solid var(--border-color);
  border-radius: 8px;
  background-color: var(--bg-secondary);
  cursor: pointer;
  transition: all 0.2s;
  text-align: left;
}

.mode-option:hover {
  border-color: var(--accent-color);
}

.mode-option.active {
  border-color: var(--accent-color);
  background-color: rgba(var(--accent-rgb), 0.1);
}

.mode-option .option-icon {
  font-size: 20px;
}

.option-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.option-desc {
  font-size: 12px;
  color: var(--text-muted);
}

/* 布局预览 */
.layout-preview {
  width: 60px;
  height: 40px;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  overflow: hidden;
  display: flex;
}

.vertical-preview {
  flex-direction: column;
}

.horizontal-preview {
  flex-direction: row;
}

.preview-top,
.preview-bottom,
.preview-left,
.preview-right {
  background-color: var(--bg-tertiary);
}

.preview-top,
.preview-bottom {
  flex: 1;
}

.preview-left,
.preview-right {
  flex: 1;
}

.preview-top {
  border-bottom: 1px solid var(--border-color);
}

.preview-left {
  border-right: 1px solid var(--border-color);
}

/* API 提供商列表 */
.provider-section {
  margin-bottom: 16px;
}

.provider-section:last-child {
  margin-bottom: 0;
}

.section-label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  font-weight: 500;
  color: var(--text-muted);
  margin-bottom: 8px;
  padding-left: 4px;
}

.api-providers {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.provider-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  background-color: var(--bg-secondary);
  border-radius: 8px;
  gap: 12px;
  transition: transform 0.2s ease, box-shadow 0.2s ease;
  border: 2px solid transparent;
}

.provider-item.is-configured {
  cursor: default;
  user-select: none;
  -webkit-user-select: none;
}

/* 拖拽时的占位符样式 */
.provider-ghost {
  opacity: 0.4;
  background-color: var(--accent-color) !important;
  border: 2px dashed var(--accent-color) !important;
}

/* 正在拖拽的元素样式 */
.provider-drag {
  opacity: 1 !important;
  background-color: var(--bg-secondary) !important;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.3) !important;
  border: 2px solid var(--accent-color) !important;
  transform: scale(1.02);
}

/* Fallback 模式下的拖拽元素样式 */
.provider-fallback {
  opacity: 0.9 !important;
  background-color: var(--bg-secondary) !important;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.3) !important;
  border: 2px solid var(--accent-color) !important;
}

/* 拖拽手柄 */
.drag-handle {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 100%;
  min-height: 40px;
  color: var(--text-muted);
  cursor: grab;
  flex-shrink: 0;
  border-radius: 4px;
  transition: all 0.2s;
  user-select: none;
  -webkit-user-select: none;
  touch-action: none;
}

.drag-handle:hover {
  color: var(--text-primary);
  background-color: var(--bg-hover);
}

.drag-handle:active {
  cursor: grabbing;
}

.drag-handle-placeholder {
  width: 28px;
  flex-shrink: 0;
}

.provider-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
  flex: 1;
  min-width: 0;
}

.provider-name {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
}

.provider-desc {
  font-size: 12px;
  color: var(--text-muted);
}

.provider-status {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
}

.provider-status.configured {
  color: #22c55e;
}

.provider-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.priority-select {
  width: 50px;
  padding: 4px 6px;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  background-color: var(--bg-primary);
  color: var(--text-primary);
  font-size: 12px;
  cursor: pointer;
}

.priority-select:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.priority-select:focus {
  outline: none;
  border-color: var(--accent-color);
}

.api-key-input {
  width: 140px;
  padding: 6px 10px;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  background-color: var(--bg-primary);
  color: var(--text-primary);
  font-size: 13px;
}

.api-key-input:focus {
  outline: none;
  border-color: var(--accent-color);
}

.action-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
  padding: 6px 12px;
  border: none;
  border-radius: 4px;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s;
}

.action-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.edit-btn {
  background-color: var(--accent-color);
  color: white;
}

.edit-btn:hover:not(:disabled) {
  opacity: 0.9;
}

.save-btn {
  background-color: #22c55e;
  color: white;
}

.cancel-btn {
  background-color: var(--bg-tertiary);
  color: var(--text-secondary);
}

.delete-btn {
  background-color: transparent;
  color: #ef4444;
  padding: 6px;
}

.delete-btn:hover:not(:disabled) {
  background-color: rgba(239, 68, 68, 0.1);
}

.save-success {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  color: #22c55e;
}

/* 测试状态样式 */
.test-success {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  color: #22c55e;
}

.test-failed {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  color: #ef4444;
  cursor: help;
}

.test-btn {
  background-color: #3b82f6;
  color: white;
}

.test-btn:hover:not(:disabled) {
  background-color: #2563eb;
}

.reset-btn {
  background-color: var(--bg-tertiary);
  color: var(--text-secondary);
}

.reset-btn:hover:not(:disabled) {
  background-color: var(--bg-hover);
  color: var(--text-primary);
}

/* 错误消息 */
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
  margin-top: 12px;
}

/* 动画 */
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

/* 加载动画 */
.animate-spin {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

/* ========== 以下为合并的样式 ========== */

.settings-overlay {
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
  z-index: 1000;
}

.settings-panel {
  width: 90%;
  max-width: 700px;
  max-height: 85vh;
  background-color: var(--bg-primary);
  border-radius: 12px;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* 头部 */
.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border-color);
  background-color: var(--bg-secondary);
}

.panel-title {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0;
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
  background-color: var(--bg-tertiary);
  color: var(--text-primary);
}

/* 注意：tabs-nav 和 tab-btn 样式已在上方定义，此处不再重复 */

.tab-label {
  font-weight: 500;
}

/* 选项卡内容 - 补充样式 */
.tab-panel {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

/* 区块头部 */
.section-header {
  display: flex;
  align-items: center;
  gap: 10px;
  padding-bottom: 12px;
  border-bottom: 1px solid var(--border-color);
}

.section-icon {
  font-size: 20px;
  color: var(--accent-color);
}

.section-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0;
}

/* 主题选项 */
.theme-options,
.layout-options,
.display-options {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 12px;
}

.theme-option,
.layout-option,
.display-option {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 8px;
  padding: 16px;
  border: 2px solid var(--border-color);
  border-radius: 10px;
  background-color: var(--bg-secondary);
  cursor: pointer;
  transition: all 0.2s;
  position: relative;
  text-align: left;
}

.theme-option:hover,
.layout-option:hover,
.display-option:hover {
  border-color: var(--accent-color);
  background-color: var(--bg-tertiary);
}

.theme-option.active,
.layout-option.active,
.display-option.active {
  border-color: var(--accent-color);
  background-color: rgba(var(--accent-rgb), 0.1);
}

.theme-icon,
.display-icon {
  font-size: 24px;
  color: var(--accent-color);
}

.theme-name,
.layout-name,
.display-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.theme-desc,
.layout-desc,
.display-desc {
  font-size: 12px;
  color: var(--text-muted);
}

.check-icon {
  position: absolute;
  top: 12px;
  right: 12px;
  font-size: 18px;
  color: var(--accent-color);
}

/* 主题色选项 */
.color-options {
  display: flex;
  gap: 12px;
  justify-content: flex-start;
}

.color-option {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  padding: 0;
  border: 2px solid transparent;
  border-radius: 50%;
  background-color: transparent;
  cursor: pointer;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}

.color-option:hover {
  transform: scale(1.1);
}

.color-option.active {
  border-color: var(--color-preview);
}

.color-swatch {
  width: 22px;
  height: 22px;
  border-radius: 50%;
  background-color: var(--color-preview);
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.15);
}

.color-name {
  display: none;
}

.color-option .check-icon {
  display: none;
}

/* 字体大小设置 */
.font-size-options {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.font-size-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  background-color: var(--bg-secondary);
  border-radius: 8px;
}

.font-size-label {
  font-size: 14px;
  color: var(--text-primary);
}

.font-size-control {
  display: flex;
  align-items: center;
  gap: 12px;
}

.font-size-control input[type="range"] {
  width: 120px;
  height: 4px;
  border-radius: 2px;
  background: var(--bg-tertiary);
  appearance: none;
  -webkit-appearance: none;
  cursor: pointer;
}

.font-size-control input[type="range"]::-webkit-slider-thumb {
  appearance: none;
  -webkit-appearance: none;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: var(--accent-color);
  cursor: pointer;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
}

.font-size-value {
  min-width: 28px;
  padding: 4px 8px;
  background: var(--bg-tertiary);
  border-radius: 4px;
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
  text-align: center;
}

/* 布局预览 */
.layout-preview {
  width: 60px;
  height: 40px;
  border: 2px solid var(--border-color);
  border-radius: 4px;
  overflow: hidden;
  display: flex;
}

.vertical-preview {
  flex-direction: column;
}

.horizontal-preview {
  flex-direction: row;
}

.preview-top,
.preview-bottom,
.preview-left,
.preview-right {
  flex: 1;
  background-color: var(--bg-tertiary);
}

.preview-top,
.preview-left {
  border-bottom: 1px solid var(--border-color);
}

.horizontal-preview .preview-left {
  border-bottom: none;
  border-right: 1px solid var(--border-color);
}

/* API 提供商 */
.api-providers {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.provider-card {
  padding: 16px;
  border: 1px solid var(--border-color);
  border-radius: 10px;
  background-color: var(--bg-secondary);
  transition: all 0.2s;
}

.provider-card.configured {
  border-color: var(--accent-color);
}

.provider-card.selected {
  background-color: rgba(var(--accent-rgb), 0.05);
}

.provider-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 12px;
}

.provider-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.provider-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.provider-desc {
  font-size: 12px;
  color: var(--text-muted);
}

.status-badge {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 11px;
  font-weight: 500;
}

.status-badge.configured {
  background-color: rgba(34, 197, 94, 0.1);
  color: #22c55e;
}

.status-badge.unconfigured {
  background-color: var(--bg-tertiary);
  color: var(--text-muted);
}

.provider-configured {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
}

.masked-key {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  background-color: var(--bg-tertiary);
  border-radius: 6px;
  font-size: 13px;
  color: var(--text-secondary);
  font-family: monospace;
}

.provider-actions {
  display: flex;
  gap: 8px;
}

.provider-edit {
  margin-top: 8px;
}

.input-group {
  display: flex;
  gap: 8px;
}

.api-key-input {
  flex: 1;
  padding: 10px 12px;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  background-color: var(--bg-tertiary);
  color: var(--text-primary);
  font-size: 13px;
  font-family: monospace;
}

.api-key-input:focus {
  outline: none;
  border-color: var(--accent-color);
}

.api-key-input::placeholder {
  color: var(--text-muted);
}

.input-actions {
  display: flex;
  gap: 8px;
}

/* 操作按钮 */
.action-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 12px;
  border: none;
  border-radius: 6px;
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.action-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.select-btn {
  background-color: var(--bg-tertiary);
  color: var(--text-secondary);
  border: 1px solid var(--border-color);
}

.select-btn:hover:not(:disabled) {
  border-color: var(--accent-color);
  color: var(--accent-color);
}

.select-btn.active {
  background-color: var(--accent-color);
  color: white;
  border-color: var(--accent-color);
}

.edit-btn {
  background-color: var(--bg-tertiary);
  color: var(--text-secondary);
  border: 1px solid var(--border-color);
}

.edit-btn:hover:not(:disabled) {
  background-color: var(--bg-primary);
  color: var(--text-primary);
}

.delete-btn {
  background-color: transparent;
  color: var(--text-muted);
  border: 1px solid transparent;
}

.delete-btn:hover:not(:disabled) {
  background-color: rgba(239, 68, 68, 0.1);
  color: #ef4444;
  border-color: rgba(239, 68, 68, 0.3);
}

.save-btn {
  background-color: var(--accent-color);
  color: white;
}

.save-btn:hover:not(:disabled) {
  background-color: var(--accent-hover);
}

.cancel-btn {
  background-color: var(--bg-tertiary);
  color: var(--text-secondary);
  border: 1px solid var(--border-color);
}

.cancel-btn:hover:not(:disabled) {
  background-color: var(--bg-primary);
  color: var(--text-primary);
}

/* 成功消息 */
.success-message {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-top: 8px;
  padding: 8px 12px;
  background-color: rgba(34, 197, 94, 0.1);
  border-radius: 6px;
  font-size: 12px;
  color: #22c55e;
}

/* 提示信息 */
.info-tip {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px;
  background-color: var(--bg-secondary);
  border-radius: 8px;
  font-size: 12px;
  color: var(--text-muted);
}

/* 动画 */
@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

.animate-spin {
  animation: spin 1s linear infinite;
}

/* 响应式 */
@media (max-width: 600px) {
  .settings-panel {
    width: 95%;
    max-height: 90vh;
  }

  .tabs-nav {
    padding: 8px 12px;
  }

  .tab-btn {
    padding: 6px 10px;
    font-size: 12px;
  }

  .tab-label {
    display: none;
  }

  .theme-options,
  .layout-options,
  .display-options {
    grid-template-columns: 1fr;
  }

  .provider-configured {
    flex-direction: column;
    align-items: flex-start;
  }

  .provider-actions {
    width: 100%;
    justify-content: flex-end;
  }

  .input-group {
    flex-direction: column;
  }

  .input-actions {
    justify-content: flex-end;
  }
}

/* 自定义选项样式 */
.section-header-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
}

.section-header-row .section-title {
  margin: 0;
}

.toggle-switch {
  position: relative;
  display: inline-block;
  width: 44px;
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
  background-color: var(--bg-tertiary);
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
  background-color: var(--text-secondary);
  transition: 0.3s;
  border-radius: 50%;
}

.toggle-switch input:checked + .toggle-slider {
  background-color: var(--accent-color);
  border-color: var(--accent-color);
}

.toggle-switch input:checked + .toggle-slider:before {
  transform: translateX(20px);
  background-color: white;
}

.custom-options-section {
  margin-top: 8px;
}

.collapse-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  background-color: var(--bg-secondary);
  border-radius: 8px;
  cursor: pointer;
  font-size: 13px;
  color: var(--text-secondary);
  transition: all 0.2s;
}

.collapse-header:hover {
  background-color: var(--bg-tertiary);
  color: var(--text-primary);
}

.collapse-icon {
  display: flex;
  transition: transform 0.2s;
}

.collapse-icon.expanded {
  transform: rotate(90deg);
}

.custom-options-list {
  margin-top: 12px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.custom-option-item {
  display: flex;
  align-items: center;
  gap: 8px;
}

.option-label {
  flex-shrink: 0;
  font-size: 13px;
  color: var(--text-secondary);
  min-width: 60px;
}

.option-input {
  flex: 1;
  padding: 8px 12px;
  background-color: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  color: var(--text-primary);
  font-size: 13px;
  outline: none;
  transition: all 0.2s;
}

.option-input:focus {
  border-color: var(--accent-color);
  box-shadow: 0 0 0 2px var(--accent-light);
}

.option-remove-btn {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: none;
  border-radius: 6px;
  background-color: transparent;
  color: var(--text-muted);
  cursor: pointer;
  transition: all 0.2s;
}

.option-remove-btn:hover {
  background-color: rgba(239, 68, 68, 0.1);
  color: #ef4444;
}

.add-option-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 8px;
  padding-top: 8px;
  border-top: 1px dashed var(--border-color);
}

.add-input {
  flex: 1;
}

.option-add-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 8px 12px;
  border: none;
  border-radius: 6px;
  background-color: var(--accent-color);
  color: white;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s;
}

.option-add-btn:hover:not(:disabled) {
  background-color: var(--accent-hover);
}

.option-add-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* ========== 文本优化类型设置样式 ========== */
.optimization-types-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 16px;
}

.optimization-type-item {
  border: 1px solid var(--border-color);
  border-radius: 8px;
  background: var(--bg-secondary);
  overflow: hidden;
  transition: all 0.2s;
}

.optimization-type-item:hover {
  border-color: var(--accent-color);
}

.optimization-type-item.is-disabled {
  opacity: 0.6;
}

.optimization-type-item.is-expanded {
  border-color: var(--accent-color);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.opt-type-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 14px;
  cursor: pointer;
  user-select: none;
}

.opt-type-info {
  display: flex;
  align-items: center;
  gap: 10px;
}

.opt-type-icon {
  font-size: 18px;
  color: var(--accent-color);
}

.opt-type-label {
  font-weight: 500;
  color: var(--text-primary);
}

.system-badge {
  padding: 2px 6px;
  font-size: 10px;
  background: var(--accent-light);
  color: var(--accent-color);
  border-radius: 4px;
}

.opt-type-actions {
  display: flex;
  align-items: center;
  gap: 12px;
}

.opt-type-desc {
  font-size: 12px;
  color: var(--text-muted);
}

.expand-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  color: var(--text-muted);
  transition: transform 0.2s;
}

.expand-icon.is-expanded {
  transform: rotate(180deg);
}

/* 小型开关 */
.toggle-switch-small {
  position: relative;
  display: inline-block;
  width: 32px;
  height: 18px;
  flex-shrink: 0;
}

.toggle-switch-small input {
  opacity: 0;
  width: 0;
  height: 0;
}

.toggle-slider-small {
  position: absolute;
  cursor: pointer;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: var(--bg-tertiary);
  transition: 0.3s;
  border-radius: 18px;
  border: 1px solid var(--border-color);
}

.toggle-slider-small:before {
  position: absolute;
  content: "";
  height: 12px;
  width: 12px;
  left: 2px;
  bottom: 2px;
  background-color: var(--text-secondary);
  transition: 0.3s;
  border-radius: 50%;
}

.toggle-switch-small input:checked + .toggle-slider-small {
  background-color: var(--accent-color);
  border-color: var(--accent-color);
}

.toggle-switch-small input:checked + .toggle-slider-small:before {
  transform: translateX(14px);
  background-color: white;
}

/* 类型详情 */
.opt-type-details {
  padding: 16px;
  border-top: 1px solid var(--border-color);
  background: var(--bg-primary);
}

.opt-detail-row {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 12px;
}

.opt-detail-row-full {
  flex-direction: column;
  align-items: stretch;
}

.detail-label {
  width: 60px;
  flex-shrink: 0;
  font-size: 13px;
  color: var(--text-secondary);
}

.opt-detail-row-full .detail-label {
  width: auto;
  margin-bottom: 6px;
}

.detail-input,
.detail-select {
  flex: 1;
  padding: 8px 12px;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  background: var(--bg-secondary);
  color: var(--text-primary);
  font-size: 13px;
}

.detail-input:focus,
.detail-select:focus {
  outline: none;
  border-color: var(--accent-color);
}

.detail-textarea {
  width: 100%;
  padding: 10px 12px;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  background: var(--bg-secondary);
  color: var(--text-primary);
  font-size: 13px;
  font-family: monospace;
  resize: vertical;
  min-height: 80px;
}

.detail-textarea:focus {
  outline: none;
  border-color: var(--accent-color);
}

.opt-detail-actions {
  display: flex;
  justify-content: flex-end;
  margin-top: 8px;
}

.btn-danger-small {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 6px 12px;
  border: none;
  border-radius: 6px;
  background: var(--color-error);
  color: white;
  font-size: 12px;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-danger-small:hover {
  opacity: 0.9;
}

/* 添加新类型 */
.add-optimization-type {
  margin-top: 16px;
}

.btn-add-type {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  width: 100%;
  padding: 12px;
  border: 2px dashed var(--border-color);
  border-radius: 8px;
  background: transparent;
  color: var(--text-secondary);
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-add-type:hover {
  border-color: var(--accent-color);
  color: var(--accent-color);
  background: var(--accent-light);
}

.new-type-form {
  padding: 16px;
  border: 1px solid var(--accent-color);
  border-radius: 8px;
  background: var(--bg-secondary);
}

.form-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
  margin-bottom: 12px;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-bottom: 12px;
}

.form-group label {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-secondary);
}

.form-input,
.form-select,
.form-textarea {
  padding: 10px 12px;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  background: var(--bg-primary);
  color: var(--text-primary);
  font-size: 13px;
}

.form-textarea {
  font-family: monospace;
  resize: vertical;
  min-height: 80px;
}

.form-input:focus,
.form-select:focus,
.form-textarea:focus {
  outline: none;
  border-color: var(--accent-color);
}

.form-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 16px;
}

.btn-cancel {
  padding: 8px 16px;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  background: transparent;
  color: var(--text-secondary);
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-cancel:hover {
  background: var(--bg-hover);
}

.btn-confirm {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 8px 16px;
  border: none;
  border-radius: 6px;
  background: var(--accent-color);
  color: white;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-confirm:hover:not(:disabled) {
  background: var(--accent-hover);
}

.btn-confirm:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* 重置按钮 */
.reset-section {
  margin-top: 20px;
  padding-top: 16px;
  border-top: 1px solid var(--border-color);
}

.btn-reset {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 14px;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  background: transparent;
  color: var(--text-secondary);
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-reset:hover {
  background: var(--bg-hover);
  border-color: var(--text-muted);
}

/* 展开/收起动画 */
.slide-down-enter-active,
.slide-down-leave-active {
  transition: all 0.25s ease-out;
  overflow: hidden;
}

.slide-down-enter-from,
.slide-down-leave-to {
  opacity: 0;
  max-height: 0;
}

.slide-down-enter-to,
.slide-down-leave-from {
  opacity: 1;
  max-height: 500px;
}
</style>

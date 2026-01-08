<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useCannedResponsesStore } from '@/stores/cannedResponses'
import type { CannedResponse } from '@/types'

// Props
const props = defineProps<{
  visible: boolean
  triggerRect?: DOMRect
}>()

// Emits
const emit = defineEmits<{
  (e: 'insert', text: string): void
  (e: 'close'): void
  (e: 'openManager'): void
  (e: 'mouseenter'): void
  (e: 'mouseleave'): void
}>()

const cannedStore = useCannedResponsesStore()

// 弹窗位置
const popupStyle = computed(() => {
  if (!props.triggerRect) {
    return { top: '0px', left: '0px' }
  }
  
  // 在触发按钮上方显示
  const top = props.triggerRect.top - 8
  const left = props.triggerRect.left
  
  return {
    bottom: `${window.innerHeight - top}px`,
    left: `${left}px`,
    maxHeight: `${top - 20}px`
  }
})

// 处理点击常用语
function handleInsert(response: CannedResponse) {
  emit('insert', response.text)
  emit('close')
}

// 处理打开管理对话框
function handleOpenManager() {
  emit('openManager')
  emit('close')
}

// 点击外部关闭
const popupRef = ref<HTMLElement | null>(null)

function handleClickOutside(event: MouseEvent) {
  if (popupRef.value && !popupRef.value.contains(event.target as Node)) {
    emit('close')
  }
}

onMounted(() => {
  document.addEventListener('mousedown', handleClickOutside)
  cannedStore.loadResponses()
})

onUnmounted(() => {
  document.removeEventListener('mousedown', handleClickOutside)
})
</script>

<template>
  <Teleport to="body">
    <Transition name="popup">
      <div 
        v-if="visible"
        ref="popupRef"
        class="canned-popup"
        :style="popupStyle"
        @mouseenter="emit('mouseenter')"
        @mouseleave="emit('mouseleave')"
      >
        <div class="popup-header">
          <span class="popup-title">常用语</span>
          <button
            class="manage-btn"
            title="管理常用语"
            @click="handleOpenManager"
          >
            <span class="i-carbon-settings" />
          </button>
        </div>
        
        <div class="popup-content">
          <div 
            v-if="cannedStore.responses.length === 0"
            class="empty-state"
          >
            <span class="i-carbon-document-blank empty-icon" />
            <span class="empty-text">暂无常用语</span>
            <button
              class="add-btn"
              @click="handleOpenManager"
            >
              添加常用语
            </button>
          </div>
          
          <div 
            v-else
            class="response-list"
          >
            <div
              v-for="response in cannedStore.responses"
              :key="response.id"
              class="response-item"
              @click="handleInsert(response)"
            >
              <span class="response-text">{{ response.text }}</span>
            </div>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>


<style scoped>
/* Requirement 5.1, 5.6: 常用语预览弹窗，支持主题适配 */
.canned-popup {
  position: fixed;
  z-index: 1000;
  min-width: 280px;
  max-width: 400px;
  background-color: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  box-shadow: var(--shadow-lg);
  overflow: hidden;
}

.popup-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  border-bottom: 1px solid var(--border-color);
  background-color: var(--bg-tertiary);
}

.popup-title {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
}

.manage-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: none;
  border-radius: 4px;
  background-color: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.2s;
}

.manage-btn:hover {
  background-color: var(--bg-hover);
  color: var(--text-primary);
}

/* Requirement 5.1: 支持滚动的常用语列表，最多显示5个 */
.popup-content {
  max-height: 230px; /* 约5个项目的高度 */
  overflow-y: auto;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 32px 16px;
  gap: 12px;
  min-height: 200px; /* 预留5个项目的高度 */
}

.empty-icon {
  font-size: 32px;
  color: var(--text-muted);
}

.empty-text {
  font-size: 14px;
  color: var(--text-muted);
}

.add-btn {
  padding: 8px 16px;
  border: none;
  border-radius: 6px;
  background-color: var(--accent-color);
  color: white;
  font-size: 13px;
  cursor: pointer;
  transition: background-color 0.2s;
}

.add-btn:hover {
  background-color: var(--accent-hover);
}

.response-list {
  padding: 8px;
}

/* Requirement 5.2: 点击常用语快速插入 */
.response-item {
  padding: 10px 12px;
  border-radius: 6px;
  cursor: pointer;
  transition: background-color 0.2s;
  overflow: hidden;
}

.response-item:hover {
  background-color: var(--bg-hover);
}

.response-text {
  font-size: 14px;
  color: var(--text-primary);
  line-height: 1.5;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  display: block;
}

/* 动画 */
.popup-enter-active,
.popup-leave-active {
  transition: all 0.2s ease;
}

.popup-enter-from,
.popup-leave-to {
  opacity: 0;
  transform: translateY(8px);
}
</style>

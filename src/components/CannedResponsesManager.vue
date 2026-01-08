<script setup lang="ts">
import { ref, onMounted, nextTick } from 'vue'
import { useCannedResponsesStore } from '@/stores/cannedResponses'
import type { CannedResponse } from '@/types'

// Props
const props = defineProps<{
  visible: boolean
}>()

// Emits
const emit = defineEmits<{
  (e: 'close'): void
  (e: 'insert', text: string): void
}>()

const cannedStore = useCannedResponsesStore()

// 编辑状态
const editingId = ref<string | null>(null)
const editingText = ref('')
const newResponseText = ref('')
const inputRef = ref<HTMLTextAreaElement | null>(null)
const editInputRef = ref<HTMLTextAreaElement | null>(null)

// 开始编辑
function startEdit(response: CannedResponse) {
  editingId.value = response.id
  editingText.value = response.text
  nextTick(() => {
    editInputRef.value?.focus()
    editInputRef.value?.select()
  })
}

// 保存编辑
function saveEdit() {
  if (editingId.value && editingText.value.trim()) {
    cannedStore.updateResponse(editingId.value, editingText.value.trim())
  }
  cancelEdit()
}

// 取消编辑
function cancelEdit() {
  editingId.value = null
  editingText.value = ''
}

// 添加新常用语
function addResponse() {
  const text = newResponseText.value.trim()
  if (text) {
    cannedStore.addResponse(text)
    newResponseText.value = ''
    inputRef.value?.focus()
  }
}

// 删除常用语
function deleteResponse(id: string) {
  cannedStore.deleteResponse(id)
}

// 双击插入
function handleDoubleClick(response: CannedResponse) {
  emit('insert', response.text)
  emit('close')
}

// 切换星标
function toggleStar(id: string) {
  cannedStore.toggleStar(id)
}

// 处理输入框键盘事件
function handleInputKeydown(event: KeyboardEvent) {
  if (event.key === 'Enter' && (event.ctrlKey || event.metaKey)) {
    event.preventDefault()
    addResponse()
  }
}

// 处理编辑框键盘事件
function handleEditKeydown(event: KeyboardEvent) {
  if (event.key === 'Enter' && (event.ctrlKey || event.metaKey)) {
    event.preventDefault()
    saveEdit()
  } else if (event.key === 'Escape') {
    cancelEdit()
  }
}

// 关闭对话框
function handleClose() {
  cancelEdit()
  emit('close')
}

onMounted(() => {
  cannedStore.loadResponses()
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
        <div class="modal-container">
          <div class="modal-header">
            <h3 class="modal-title">
              <span class="i-carbon-text-short-paragraph title-icon" />
              管理常用语
            </h3>
            <button
              class="close-btn"
              @click="handleClose"
            >
              <span class="i-carbon-close" />
            </button>
          </div>
          
          <div class="modal-body">
            <!-- 常用语列表 -->
            <div class="response-list">
              <div 
                v-if="cannedStore.responses.length === 0"
                class="empty-state"
              >
                <span class="i-carbon-document-blank empty-icon" />
                <span class="empty-text">暂无常用语</span>
                <span class="empty-hint">在下方添加常用语，支持多行和 Markdown 格式</span>
              </div>
              
              <!-- 支持星标的列表 -->
              <div
                v-for="response in cannedStore.responses"
                :key="response.id"
                class="response-item"
                :class="{
                  'is-editing': editingId === response.id,
                  'is-starred': response.starred
                }"
                @dblclick="handleDoubleClick(response)"
              >
                <!-- 编辑模式 -->
                <template v-if="editingId === response.id">
                  <textarea
                    ref="editInputRef"
                    v-model="editingText"
                    class="edit-input"
                    rows="4"
                    placeholder="编辑常用语..."
                    @keydown="handleEditKeydown"
                  />
                  <div class="edit-actions">
                    <button
                      class="edit-action-btn cancel-btn"
                      @click="cancelEdit"
                    >
                      取消
                    </button>
                    <button
                      class="edit-action-btn save-btn"
                      @click="saveEdit"
                    >
                      <span class="i-carbon-checkmark" />
                      保存
                    </button>
                  </div>
                </template>
                
                <!-- 显示模式 -->
                <template v-else>
                  <button 
                    class="star-btn" 
                    :class="{ 'is-starred': response.starred }"
                    :title="response.starred ? '取消星标' : '添加星标'"
                    @click.stop="toggleStar(response.id)"
                  >
                    <span :class="response.starred ? 'i-carbon-star-filled' : 'i-carbon-star'" />
                  </button>
                  <div class="response-content">
                    <span class="response-text">{{ response.text }}</span>
                  </div>
                  <div class="response-actions">
                    <button 
                      class="action-btn edit-btn" 
                      title="编辑"
                      @click.stop="startEdit(response)"
                    >
                      <span class="i-carbon-edit" />
                    </button>
                    <button 
                      class="action-btn delete-btn" 
                      title="删除"
                      @click.stop="deleteResponse(response.id)"
                    >
                      <span class="i-carbon-trash-can" />
                    </button>
                  </div>
                </template>
              </div>
            </div>
          </div>
          
          <!-- 添加新常用语区域 -->
          <div class="add-section">
            <textarea
              ref="inputRef"
              v-model="newResponseText"
              class="add-input"
              placeholder="输入新的常用语（支持多行 / Markdown）..."
              rows="3"
              @keydown="handleInputKeydown"
            />
          </div>
          
          <div class="modal-footer">
            <span class="hint-text">
              <span class="i-carbon-information" />
              双击插入 · Ctrl+Enter 添加
            </span>
            <div class="footer-actions">
              <button 
                class="add-btn"
                :disabled="!newResponseText.trim()"
                @click="addResponse"
              >
                <span class="i-carbon-add" />
                添加
              </button>
              <button
                class="done-btn"
                @click="handleClose"
              >
                完成
              </button>
            </div>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
/* Requirement 5.3, 5.6: 常用语管理对话框，支持主题适配 */
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

.modal-container {
  width: 90%;
  max-width: 520px;
  max-height: 80vh;
  background-color: var(--bg-primary);
  border-radius: 12px;
  box-shadow: var(--shadow-lg);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border-color);
  background-color: var(--bg-secondary);
}

.modal-title {
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

.modal-body {
  overflow: hidden;
  padding: 16px 20px;
  height: 230px; /* 固定高度，约5个项目 */
  min-height: 230px;
  max-height: 230px;
  display: flex;
  flex-direction: column;
}

.response-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  flex: 1;
  overflow-y: auto;
  min-height: 0;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  gap: 8px;
}

.empty-icon {
  font-size: 40px;
  color: var(--text-muted);
}

.empty-text {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-secondary);
}

.empty-hint {
  font-size: 12px;
  color: var(--text-muted);
}

/* Requirement 5.4: 列表项 */
.response-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 12px;
  background-color: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  transition: all 0.2s;
  cursor: default;
}

.response-item:hover {
  background-color: var(--bg-hover);
  border-color: var(--border-color);
}

.response-item:hover .response-actions {
  opacity: 1;
}

/* 星标项目样式 */
.response-item.is-starred {
  background-color: var(--accent-light);
  border-color: var(--accent-color);
}

.response-item.is-editing {
  flex-direction: column;
  align-items: stretch;
  padding: 12px;
  background-color: var(--bg-tertiary);
  border-color: var(--accent-color);
}

/* 星标按钮 */
.star-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border: none;
  border-radius: 4px;
  background-color: transparent;
  color: var(--text-muted);
  cursor: pointer;
  flex-shrink: 0;
  transition: all 0.2s;
}

.star-btn:hover {
  background-color: var(--accent-light);
  color: var(--accent-color);
}

.star-btn.is-starred {
  color: var(--accent-color);
}

.response-content {
  flex: 1;
  min-width: 0;
  cursor: pointer;
}

.response-text {
  font-size: 14px;
  color: var(--text-primary);
  line-height: 1.4;
  display: block;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.edit-input {
  width: 100%;
  padding: 10px 12px;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  background-color: var(--bg-input);
  color: var(--text-primary);
  font-size: 14px;
  font-family: inherit;
  line-height: 1.5;
  outline: none;
  resize: vertical;
  min-height: 80px;
}

.edit-input:focus {
  border-color: var(--accent-color);
}

.edit-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 10px;
}

.edit-action-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 6px 14px;
  border: none;
  border-radius: 6px;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s;
}

.cancel-btn {
  background-color: var(--bg-secondary);
  color: var(--text-secondary);
  border: 1px solid var(--border-color);
}

.cancel-btn:hover {
  background-color: var(--bg-hover);
  color: var(--text-primary);
}

.save-btn {
  background-color: var(--accent-color);
  color: white;
}

.save-btn:hover {
  background-color: var(--accent-hover);
}

.response-actions {
  display: flex;
  gap: 4px;
  opacity: 0;
  transition: opacity 0.2s;
  flex-shrink: 0;
}

.action-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: none;
  border-radius: 6px;
  background-color: transparent;
  cursor: pointer;
  transition: all 0.2s;
}

.edit-btn {
  color: var(--text-secondary);
}

.edit-btn:hover {
  background-color: var(--bg-tertiary);
  color: var(--accent-color);
}

.delete-btn {
  color: var(--text-secondary);
}

.delete-btn:hover {
  background-color: var(--error-light);
  color: var(--error-color);
}

/* 添加区域 */
.add-section {
  padding: 16px 20px;
  border-top: 1px solid var(--border-color);
  background-color: var(--bg-secondary);
}

.add-input {
  width: 100%;
  padding: 10px 12px;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  background-color: var(--bg-input);
  color: var(--text-primary);
  font-size: 14px;
  font-family: inherit;
  line-height: 1.5;
  outline: none;
  transition: border-color 0.2s;
  resize: none;
}

.add-input:focus {
  border-color: var(--accent-color);
}

.add-input::placeholder {
  color: var(--text-muted);
}

.modal-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 20px;
  background-color: var(--bg-secondary);
}

.hint-text {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: var(--text-muted);
}

.footer-actions {
  display: flex;
  gap: 8px;
}

.add-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
  border: 1px solid var(--accent-color);
  border-radius: 6px;
  background-color: transparent;
  color: var(--accent-color);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.add-btn:hover:not(:disabled) {
  background-color: var(--accent-color);
  color: white;
}

.add-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.done-btn {
  padding: 8px 20px;
  border: none;
  border-radius: 6px;
  background-color: var(--accent-color);
  color: white;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: background-color 0.2s;
}

.done-btn:hover {
  background-color: var(--accent-hover);
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

.modal-enter-from .modal-container,
.modal-leave-to .modal-container {
  transform: scale(0.95);
}
</style>

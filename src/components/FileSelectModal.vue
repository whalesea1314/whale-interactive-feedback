<script setup lang="ts">
import { ref, computed } from 'vue'
import { useFileHandler } from '@/composables/useFileHandler'
import type { FileReference } from '@/types'

// Props
const props = defineProps<{
  visible: boolean
  existingFiles: FileReference[]  // 已存在的文件引用
}>()

// Emits
const emit = defineEmits<{
  (e: 'close'): void
  (e: 'confirm', files: FileReference[]): void
}>()

// 常量
const MAX_FILE_SIZE = 5 * 1024 * 1024  // 5MB
const MAX_FILE_COUNT = 10  // 最多选择 10 个文件

// 文件处理
const { openTextFileDialog, getFileName } = useFileHandler()

// 状态
const selectedFiles = ref<FileReference[]>([])
const errors = ref<string[]>([])

// 计算属性
const totalCount = computed(() => props.existingFiles.length + selectedFiles.value.length)
const canAddMore = computed(() => totalCount.value < MAX_FILE_COUNT)

// 格式化文件大小
function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(2)} MB`
}

// 检查文件是否重复
function isDuplicate(newFile: FileReference): boolean {
  const newFileName = getFileName(newFile.path)
  
  // 检查已存在的文件
  for (const existing of props.existingFiles) {
    if (getFileName(existing.path) === newFileName) {
      return true
    }
  }
  
  // 检查已选择的文件
  for (const selected of selectedFiles.value) {
    if (getFileName(selected.path) === newFileName) {
      return true
    }
  }
  
  return false
}

// 选择文件
async function handleSelectFiles() {
  errors.value = []
  
  const files = await openTextFileDialog()
  
  const newFiles: FileReference[] = []
  const newErrors: string[] = []
  
  for (const file of files) {
    const fileName = getFileName(file.path)
    
    // 检查重复
    if (isDuplicate(file)) {
      newErrors.push(`文件 "${fileName}" 已存在，跳过`)
      continue
    }
    
    // 检查数量限制
    if (totalCount.value + newFiles.length >= MAX_FILE_COUNT) {
      newErrors.push(`已达到最大文件数量限制 (${MAX_FILE_COUNT} 个)`)
      break
    }
    
    newFiles.push(file)
  }
  
  selectedFiles.value.push(...newFiles)
  errors.value = newErrors
}

// 移除已选文件
function removeFile(id: string) {
  const index = selectedFiles.value.findIndex(f => f.id === id)
  if (index !== -1) {
    selectedFiles.value.splice(index, 1)
  }
}

// 确认选择
function handleConfirm() {
  emit('confirm', selectedFiles.value)
  selectedFiles.value = []
  errors.value = []
}

// 取消
function handleClose() {
  selectedFiles.value = []
  errors.value = []
  emit('close')
}
</script>

<template>
  <Teleport to="body">
    <Transition name="modal-fade">
      <div
        v-if="visible"
        class="modal-overlay"
        @click.self="handleClose"
      >
        <div class="modal-container">
          <!-- 头部 -->
          <div class="modal-header">
            <h3 class="modal-title">
              <span class="i-carbon-document-add" />
              选择文件
            </h3>
            <button
              class="modal-close"
              @click="handleClose"
            >
              <span class="i-carbon-close" />
            </button>
          </div>
          
          <!-- 内容 -->
          <div class="modal-content">
            <!-- 提示信息 -->
            <div class="tips-section">
              <div class="tip-item">
                <span class="i-carbon-information" />
                <span>支持文本类文件（代码、配置、文档等）</span>
              </div>
              <div class="tip-item">
                <span class="i-carbon-warning" />
                <span>单个文件不超过 {{ formatSize(MAX_FILE_SIZE) }}</span>
              </div>
              <div class="tip-item">
                <span class="i-carbon-list" />
                <span>最多选择 {{ MAX_FILE_COUNT }} 个文件（已有 {{ existingFiles.length }} 个）</span>
              </div>
            </div>
            
            <!-- 错误提示 -->
            <div
              v-if="errors.length > 0"
              class="error-section"
            >
              <div
                v-for="(err, idx) in errors"
                :key="idx"
                class="error-item"
              >
                <span class="i-carbon-warning-alt" />
                {{ err }}
              </div>
            </div>
            
            <!-- 选择按钮 -->
            <button 
              class="select-btn"
              :disabled="!canAddMore"
              @click="handleSelectFiles"
            >
              <span class="i-carbon-folder-add" />
              {{ canAddMore ? '选择文件' : '已达上限' }}
            </button>
            
            <!-- 已选文件列表 -->
            <div
              v-if="selectedFiles.length > 0"
              class="selected-files"
            >
              <div class="selected-header">
                <span>已选择 {{ selectedFiles.length }} 个文件</span>
              </div>
              <div class="file-list">
                <div 
                  v-for="file in selectedFiles" 
                  :key="file.id"
                  class="file-item"
                >
                  <span class="i-carbon-document" />
                  <span
                    class="file-name"
                    :title="file.path"
                  >
                    {{ file.displayName }}
                  </span>
                  <button
                    class="file-remove"
                    @click="removeFile(file.id)"
                  >
                    <span class="i-carbon-close" />
                  </button>
                </div>
              </div>
            </div>
          </div>
          
          <!-- 底部 -->
          <div class="modal-footer">
            <button
              class="btn-cancel"
              @click="handleClose"
            >
              取消
            </button>
            <button 
              class="btn-confirm"
              :disabled="selectedFiles.length === 0"
              @click="handleConfirm"
            >
              <span class="i-carbon-checkmark" />
              确认添加 ({{ selectedFiles.length }})
            </button>
          </div>
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
  background: rgba(0, 0, 0, 0.5);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9999;
}

.modal-container {
  width: 90%;
  max-width: 480px;
  max-height: 80vh;
  background: var(--bg-primary);
  border-radius: 12px;
  border: 1px solid var(--border-color);
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.3);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* 头部 */
.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border-color);
  background: var(--bg-secondary);
}

.modal-title {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
}

.modal-title span {
  font-size: 20px;
  color: var(--accent-color);
}

.modal-close {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border: none;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  border-radius: 6px;
  transition: all 0.15s;
}

.modal-close:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

/* 内容 */
.modal-content {
  flex: 1;
  padding: 20px;
  overflow-y: auto;
}

/* 提示信息 */
.tips-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 12px 14px;
  background: var(--bg-secondary);
  border-radius: 8px;
  margin-bottom: 16px;
}

.tip-item {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  color: var(--text-secondary);
}

.tip-item span:first-child {
  color: var(--accent-color);
  font-size: 14px;
}

/* 错误提示 */
.error-section {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 10px 12px;
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid rgba(239, 68, 68, 0.3);
  border-radius: 8px;
  margin-bottom: 16px;
}

.error-item {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: #ef4444;
}

/* 选择按钮 */
.select-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  width: 100%;
  padding: 14px;
  border: 2px dashed var(--border-color);
  border-radius: 10px;
  background: transparent;
  color: var(--text-secondary);
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s;
}

.select-btn:hover:not(:disabled) {
  border-color: var(--accent-color);
  color: var(--accent-color);
  background: var(--accent-light);
}

.select-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* 已选文件列表 */
.selected-files {
  margin-top: 16px;
}

.selected-header {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-secondary);
  margin-bottom: 10px;
}

.file-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
  max-height: 200px;
  overflow-y: auto;
}

.file-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 12px;
  background: var(--bg-secondary);
  border-radius: 8px;
  border: 1px solid var(--border-subtle);
}

.file-item span:first-child {
  color: var(--accent-color);
  font-size: 16px;
  flex-shrink: 0;
}

.file-name {
  flex: 1;
  font-size: 13px;
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.file-remove {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border: none;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  border-radius: 4px;
  transition: all 0.15s;
  flex-shrink: 0;
}

.file-remove:hover {
  background: rgba(239, 68, 68, 0.1);
  color: #ef4444;
}

/* 底部 */
.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  padding: 16px 20px;
  border-top: 1px solid var(--border-color);
  background: var(--bg-secondary);
}

.btn-cancel {
  padding: 10px 18px;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  background: transparent;
  color: var(--text-secondary);
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-cancel:hover {
  background: var(--bg-hover);
}

.btn-confirm {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 10px 18px;
  border: none;
  border-radius: 8px;
  background: var(--accent-color);
  color: white;
  font-size: 14px;
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

/* 过渡动画 */
.modal-fade-enter-active,
.modal-fade-leave-active {
  transition: opacity 0.2s ease;
}

.modal-fade-enter-active .modal-container,
.modal-fade-leave-active .modal-container {
  transition: transform 0.2s ease;
}

.modal-fade-enter-from,
.modal-fade-leave-to {
  opacity: 0;
}

.modal-fade-enter-from .modal-container,
.modal-fade-leave-to .modal-container {
  transform: scale(0.95) translateY(-10px);
}
</style>


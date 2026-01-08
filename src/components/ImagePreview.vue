<script setup lang="ts">
import { ref, computed } from 'vue'
import type { ImagePreviewData } from '@/types'

// Props
const props = defineProps<{
  images: ImagePreviewData[]
}>()

// Emits
const emit = defineEmits<{
  (e: 'remove', id: string): void
}>()

// 悬停预览状态
const hoveredImage = ref<ImagePreviewData | null>(null)
const hoverPosition = ref({ x: 0, y: 0 })

// 格式化文件大小
function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(2)} MB`
}

// 处理鼠标悬停 - Requirement 3.4
function handleMouseEnter(image: ImagePreviewData, event: MouseEvent) {
  hoveredImage.value = image
  updateHoverPosition(event)
}

function handleMouseMove(event: MouseEvent) {
  if (hoveredImage.value) {
    updateHoverPosition(event)
  }
}

function handleMouseLeave() {
  hoveredImage.value = null
}

function updateHoverPosition(event: MouseEvent) {
  // 预览框偏移量
  const offsetX = 10
  const offsetY = 10
  
  // 获取视口尺寸
  const viewportWidth = window.innerWidth
  const viewportHeight = window.innerHeight
  
  // 预览框预估尺寸
  const previewWidth = 320
  const previewHeight = 280
  
  let x = event.clientX + offsetX
  let y = event.clientY - previewHeight - offsetY  // 默认显示在鼠标上方
  
  // 防止超出右边界
  if (x + previewWidth > viewportWidth) {
    x = event.clientX - previewWidth - offsetX
  }
  
  // 防止超出上边界，改为显示在下方
  if (y < 10) {
    y = event.clientY + offsetY
  }
  
  // 防止超出下边界
  if (y + previewHeight > viewportHeight) {
    y = viewportHeight - previewHeight - 10
  }
  
  hoverPosition.value = { x, y }
}

// 处理点击删除 - Requirement 3.5
function handleRemove(id: string) {
  emit('remove', id)
}

// 获取图片 src
function getImageSrc(image: ImagePreviewData): string {
  return `data:${image.mimeType};base64,${image.data}`
}
</script>

<template>
  <div
    v-if="images.length > 0"
    class="image-preview-container"
  >
    <!-- 缩略图列表 - Requirement 3.3 -->
    <div class="thumbnails">
      <div
        v-for="image in images"
        :key="image.id"
        class="thumbnail-item"
        :title="'点击删除'"
        @mouseenter="handleMouseEnter(image, $event)"
        @mousemove="handleMouseMove"
        @mouseleave="handleMouseLeave"
        @click="handleRemove(image.id)"
      >
        <img
          :src="getImageSrc(image)"
          :alt="`图片 ${image.id}`"
          class="thumbnail-image"
        >
        <div class="thumbnail-overlay">
          <span class="i-carbon-close" />
        </div>
      </div>
    </div>
    
    <!-- 悬停预览弹窗 - Requirement 3.4 -->
    <Teleport to="body">
      <div
        v-if="hoveredImage"
        class="hover-preview"
        :style="{
          left: `${hoverPosition.x}px`,
          top: `${hoverPosition.y}px`
        }"
      >
        <img
          :src="getImageSrc(hoveredImage)"
          :alt="`预览 ${hoveredImage.id}`"
          class="preview-image"
        >
        <div class="preview-info">
          <span class="info-dimensions">
            {{ hoveredImage.width }} × {{ hoveredImage.height }}
          </span>
          <span class="info-size">
            {{ formatSize(hoveredImage.size) }}
          </span>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<style scoped>
.image-preview-container {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.thumbnails {
  display: flex;
  flex-wrap: nowrap;
  gap: 8px;
  overflow-x: auto;
  overflow-y: hidden;
  padding-bottom: 4px;
}

.thumbnails::-webkit-scrollbar {
  height: 4px;
}

.thumbnails::-webkit-scrollbar-thumb {
  background: var(--scrollbar-thumb);
  border-radius: 2px;
}

.thumbnail-item {
  position: relative;
  width: 64px;
  height: 64px;
  min-width: 64px;
  border-radius: 6px;
  overflow: hidden;
  cursor: pointer;
  border: 2px solid var(--border-color);
  transition: all 0.2s;
}

.thumbnail-item:hover {
  border-color: var(--accent-color);
}

.thumbnail-item:hover .thumbnail-overlay {
  opacity: 1;
}

.thumbnail-image {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.thumbnail-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: rgba(0, 0, 0, 0.6);
  opacity: 0;
  transition: opacity 0.2s;
}

.thumbnail-overlay span {
  color: white;
  font-size: 20px;
}

/* 悬停预览弹窗 */
.hover-preview {
  position: fixed;
  z-index: 9999;
  max-width: 300px;
  background-color: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.3);
  overflow: hidden;
  pointer-events: none;
}

.preview-image {
  display: block;
  max-width: 100%;
  max-height: 240px;
  object-fit: contain;
}

.preview-info {
  display: flex;
  justify-content: space-between;
  padding: 8px 12px;
  background-color: var(--bg-secondary);
  font-size: 12px;
  color: var(--text-muted);
}

.info-dimensions {
  font-weight: 500;
}

.info-size {
  color: var(--text-secondary);
}
</style>

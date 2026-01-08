<script setup lang="ts">
/**
 * MarkdownContent - Markdown 渲染组件
 * 
 * 基于 md-editor-v3 的 MdPreview 组件，提供美观的 Markdown 内容展示
 * 支持代码高亮、深色/浅色主题切换
 */

import { computed } from 'vue'
import { MdPreview } from 'md-editor-v3'
import 'md-editor-v3/lib/preview.css'
import { useTheme } from '@/composables/useTheme'

const props = withDefaults(defineProps<{
  /** Markdown 内容 */
  content: string
  /** 字体大小 (px) */
  fontSize?: number
}>(), {
  fontSize: 14
})

const { isDark } = useTheme()

/** 编辑器主题：深色/浅色 */
const mdTheme = computed(() => isDark.value ? 'dark' : 'light')

/** 代码高亮主题 */
const codeTheme = computed(() => isDark.value ? 'atom' : 'github')
</script>

<template>
  <MdPreview
    :model-value="content"
    :theme="mdTheme"
    preview-theme="default"
    :code-theme="codeTheme"
    :show-code-row-number="true"
    :code-foldable="true"
    :auto-fold-threshold="30"
    :no-mermaid="true"
    :no-katex="true"
    :no-echarts="true"
    :no-img-zoom-in="false"
    language="zh-CN"
    class="md-preview-wrapper"
    :style="{ '--md-font-size': fontSize + 'px' }"
  />
</template>

<style scoped>
/* 容器样式 - 透明背景，融入父容器 */
.md-preview-wrapper {
  background: transparent !important;
  border: none !important;
  height: auto !important;
  overflow: visible !important;
  font-size: var(--md-font-size, 14px);
}

/* 预览区域 - 移除默认内边距 */
.md-preview-wrapper :deep(.md-editor-preview-wrapper) {
  padding: 0 !important;
}

.md-preview-wrapper :deep(.md-editor-preview) {
  padding: 0 !important;
  color: var(--text-primary);
  background: transparent !important;
  font-size: inherit !important;
}

/* 标题 - 使用 em 单位，相对于基础字体大小 */
.md-preview-wrapper :deep(h1) {
  color: var(--text-primary);
  font-weight: 600;
  line-height: 1.35;
  font-size: 1.75em !important;
}

.md-preview-wrapper :deep(h2) {
  color: var(--text-primary);
  font-weight: 600;
  line-height: 1.35;
  font-size: 1.5em !important;
}

.md-preview-wrapper :deep(h3) {
  color: var(--text-primary);
  font-weight: 600;
  line-height: 1.35;
  font-size: 1.25em !important;
}

.md-preview-wrapper :deep(h4) {
  color: var(--text-primary);
  font-weight: 600;
  line-height: 1.35;
  font-size: 1.1em !important;
}

.md-preview-wrapper :deep(h5),
.md-preview-wrapper :deep(h6) {
  color: var(--text-primary);
  font-weight: 600;
  line-height: 1.35;
  font-size: 1em !important;
}

/* 段落 */
.md-preview-wrapper :deep(p) {
  line-height: 1.7;
  color: var(--text-primary);
  font-size: inherit !important;
}

/* 粗体 */
.md-preview-wrapper :deep(strong),
.md-preview-wrapper :deep(b) {
  color: var(--text-primary);
  font-weight: 600;
}

/* 斜体 */
.md-preview-wrapper :deep(em),
.md-preview-wrapper :deep(i) {
  color: var(--text-primary);
}

/* 删除线 */
.md-preview-wrapper :deep(del),
.md-preview-wrapper :deep(s) {
  color: var(--text-muted);
}

/* 高亮/标记 */
.md-preview-wrapper :deep(mark) {
  background-color: var(--accent-light);
  color: var(--accent-color);
  padding: 0.1em 0.3em;
  border-radius: 3px;
}

/* 列表 */
.md-preview-wrapper :deep(ul),
.md-preview-wrapper :deep(ol) {
  font-size: inherit !important;
  color: var(--text-primary);
  padding-left: 1.5em;
}

.md-preview-wrapper :deep(li) {
  font-size: inherit !important;
  color: var(--text-primary);
  line-height: 1.7;
}

/* 无序列表标记样式 - 层级递进 */
.md-preview-wrapper :deep(ul) {
  list-style-type: disc; /* 第一层: 实心圆 ● */
}

.md-preview-wrapper :deep(ul ul) {
  list-style-type: circle; /* 第二层: 空心圆 ○ */
}

.md-preview-wrapper :deep(ul ul ul) {
  list-style-type: square; /* 第三层: 实心正方形 ■ */
}

.md-preview-wrapper :deep(ul ul ul ul) {
  list-style-type: disc; /* 第四层: 循环回实心圆 */
}

/* 列表标记颜色 - 使用主题色 */
.md-preview-wrapper :deep(li::marker) {
  color: var(--accent-color);
}

/* 有序列表标记 */
.md-preview-wrapper :deep(ol) {
  list-style-type: decimal;
}

.md-preview-wrapper :deep(ol ol) {
  list-style-type: lower-alpha;
}

.md-preview-wrapper :deep(ol ol ol) {
  list-style-type: lower-roman;
}

/* 任务列表 checkbox */
.md-preview-wrapper :deep(li input[type="checkbox"]) {
  accent-color: var(--accent-color);
  margin-right: 0.5em;
}

/* 链接 */
.md-preview-wrapper :deep(a) {
  color: var(--accent-color);
  text-decoration: none;
}

.md-preview-wrapper :deep(a:hover) {
  color: var(--accent-hover);
  text-decoration: underline;
}

/* 行内代码 - 使用 em 单位相对于父元素 */
.md-preview-wrapper :deep(code:not(pre code)) {
  background-color: var(--accent-light) !important;
  color: var(--accent-color) !important;
  padding: 0.15em 0.4em;
  border-radius: 4px;
  font-size: 0.9em !important;
  font-family: 'SF Mono', 'Monaco', 'Menlo', 'Consolas', monospace;
}

/* 代码块 */
.md-preview-wrapper :deep(.md-editor-code) {
  background-color: var(--bg-secondary) !important;
  border: 1px solid var(--border-subtle) !important;
  border-radius: 8px !important;
  font-size: 0.9em !important;
}

/* 代码块头部 - 禁用 sticky 定位 */
.md-preview-wrapper :deep(.md-editor-code-head) {
  background-color: var(--bg-tertiary) !important;
  position: relative !important;
  top: auto !important;
}

/* 代码块内容 */
.md-preview-wrapper :deep(.md-editor-code pre) {
  font-size: inherit !important;
}

.md-preview-wrapper :deep(.md-editor-code code) {
  font-size: inherit !important;
}

/* 引用块 */
.md-preview-wrapper :deep(blockquote) {
  border-left: 4px solid var(--accent-color) !important;
  background-color: var(--accent-light) !important;
  padding: 0.75em 1em !important;
  border-radius: 0 6px 6px 0;
  color: var(--text-secondary) !important;
  font-size: inherit !important;
}

/* 表格 - 简洁风格 */
.md-preview-wrapper :deep(table) {
  border: none !important;
  border-radius: 10px;
  overflow: hidden;
  font-size: inherit !important;
  width: 100%;
  border-collapse: separate;
  border-spacing: 0;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.08);
}

/* 表头 */
.md-preview-wrapper :deep(thead) {
  background: var(--accent-light) !important;
}

.md-preview-wrapper :deep(th) {
  background: transparent !important;
  color: var(--accent-color);
  font-weight: 600;
  border: none !important;
  border-bottom: 2px solid var(--accent-color) !important;
  text-align: left;
}

/* 单元格 */
.md-preview-wrapper :deep(td) {
  border: none !important;
  border-bottom: 1px solid var(--border-subtle) !important;
  color: var(--text-primary);
  background: var(--bg-card);
}

.md-preview-wrapper :deep(tbody tr:last-child td) {
  border-bottom: none !important;
}

/* 行 hover 效果 */
.md-preview-wrapper :deep(tbody tr:hover td) {
  background-color: var(--bg-hover);
}

/* 水平线 */
.md-preview-wrapper :deep(hr) {
  border: none !important;
  height: 1px;
  background-color: var(--border-color) !important;
}

/* 首尾元素边距处理 */
.md-preview-wrapper :deep(.md-editor-preview > :first-child) {
  margin-top: 0 !important;
}

.md-preview-wrapper :deep(.md-editor-preview > :last-child) {
  margin-bottom: 0 !important;
}

/* 滚动条 */
.md-preview-wrapper :deep(::-webkit-scrollbar) {
  width: 6px;
  height: 6px;
}

.md-preview-wrapper :deep(::-webkit-scrollbar-thumb) {
  background: var(--scrollbar-thumb);
  border-radius: 3px;
}

.md-preview-wrapper :deep(::-webkit-scrollbar-thumb:hover) {
  background: var(--scrollbar-thumb-hover);
}
</style>

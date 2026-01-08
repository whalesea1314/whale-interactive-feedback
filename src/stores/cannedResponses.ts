import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { CannedResponse } from '@/types'
import { invoke } from '@tauri-apps/api/core'

export const useCannedResponsesStore = defineStore('cannedResponses', () => {
  // 状态
  const responses = ref<CannedResponse[]>([])
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  // 计算属性：星标项在前，非星标项在后
  const sortedResponses = computed(() => {
    const starred = responses.value.filter(r => r.starred)
    const unstarred = responses.value.filter(r => !r.starred)
    return [...starred, ...unstarred]
  })

  // 加载常用语
  async function loadResponses() {
    isLoading.value = true
    error.value = null
    try {
      const data = await invoke<CannedResponse[]>('get_canned_responses')
      responses.value = data
    } catch (e) {
      error.value = e instanceof Error ? e.message : '加载常用语失败'
      console.error('Failed to load canned responses:', e)
    } finally {
      isLoading.value = false
    }
  }

  // 保存常用语
  async function saveResponses() {
    isLoading.value = true
    error.value = null
    try {
      await invoke('save_canned_responses', { responses: responses.value })
    } catch (e) {
      error.value = e instanceof Error ? e.message : '保存常用语失败'
      console.error('Failed to save canned responses:', e)
    } finally {
      isLoading.value = false
    }
  }

  // 添加常用语
  function addResponse(text: string) {
    const id = crypto.randomUUID()
    const order = responses.value.length
    responses.value.push({ id, text, order, starred: false })
    saveResponses()
  }

  // 更新常用语
  function updateResponse(id: string, text: string) {
    const response = responses.value.find(r => r.id === id)
    if (response) {
      response.text = text
      saveResponses()
    }
  }

  // 删除常用语
  function deleteResponse(id: string) {
    const index = responses.value.findIndex(r => r.id === id)
    if (index !== -1) {
      responses.value.splice(index, 1)
      // 重新排序
      responses.value.forEach((r, i) => {
        r.order = i
      })
      saveResponses()
    }
  }

  // 切换星标状态
  function toggleStar(id: string) {
    const response = responses.value.find(r => r.id === id)
    if (response) {
      response.starred = !response.starred
      saveResponses()
    }
  }

  // 重新排序（保留用于其他用途）
  function reorderResponses(fromIndex: number, toIndex: number) {
    const [removed] = responses.value.splice(fromIndex, 1)
    responses.value.splice(toIndex, 0, removed)
    // 更新 order
    responses.value.forEach((r, i) => {
      r.order = i
    })
    saveResponses()
  }

  return {
    responses: sortedResponses,
    isLoading,
    error,
    loadResponses,
    saveResponses,
    addResponse,
    updateResponse,
    deleteResponse,
    toggleStar,
    reorderResponses,
  }
})

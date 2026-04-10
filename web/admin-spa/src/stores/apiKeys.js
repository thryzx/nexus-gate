import { defineStore } from 'pinia'
import { ref } from 'vue'
import * as api from '@/utils/api'

export const useApiKeysStore = defineStore('apiKeys', () => {
  const list = ref([])
  const loading = ref(false)

  async function fetch() {
    loading.value = true
    try {
      list.value = await api.getApiKeys()
    } finally {
      loading.value = false
    }
  }

  async function create(data) {
    const res = await api.createApiKey(data)
    await fetch()
    return res
  }

  async function update(id, data) {
    const res = await api.updateApiKey(id, data)
    await fetch()
    return res
  }

  async function remove(id) {
    await api.deleteApiKey(id)
    await fetch()
  }

  return { list, loading, fetch, create, update, remove }
})

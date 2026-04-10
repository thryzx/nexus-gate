import { defineStore } from 'pinia'
import { ref } from 'vue'
import * as api from '@/utils/api'

export const useFingerprintsStore = defineStore('fingerprints', () => {
  const list = ref([])
  const loading = ref(false)

  async function fetch() {
    loading.value = true
    try {
      list.value = await api.getFingerprints()
    } finally {
      loading.value = false
    }
  }

  async function create(data) {
    const res = await api.createFingerprint(data)
    await fetch()
    return res
  }

  async function update(id, data) {
    const res = await api.updateFingerprint(id, data)
    await fetch()
    return res
  }

  async function remove(id) {
    await api.deleteFingerprint(id)
    await fetch()
  }

  return { list, loading, fetch, create, update, remove }
})

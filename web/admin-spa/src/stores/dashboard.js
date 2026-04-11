import { defineStore } from 'pinia'
import { ref } from 'vue'
import { getDashboard } from '@/utils/api'

export const useDashboardStore = defineStore('dashboard', () => {
  const data = ref(null)
  const loading = ref(false)
  const lastFetchedAt = ref(null)

  async function fetch() {
    loading.value = true
    try {
      data.value = await getDashboard()
      lastFetchedAt.value = new Date()
    } finally {
      loading.value = false
    }
  }

  return { data, loading, lastFetchedAt, fetch }
})

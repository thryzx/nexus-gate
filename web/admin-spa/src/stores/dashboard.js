import { defineStore } from 'pinia'
import { ref } from 'vue'
import { getDashboard } from '@/utils/api'

export const useDashboardStore = defineStore('dashboard', () => {
  const data = ref(null)
  const loading = ref(false)

  async function fetch() {
    loading.value = true
    try {
      data.value = await getDashboard()
    } finally {
      loading.value = false
    }
  }

  return { data, loading, fetch }
})

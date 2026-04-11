import { defineStore } from 'pinia'
import { ref } from 'vue'
import * as api from '@/utils/api'

export const useApiStatsStore = defineStore('apistats', () => {
  const models = ref([])
  const keyId = ref(null)
  const userStats = ref(null)
  const userModelStats = ref([])
  const serviceRates = ref({})
  const loading = ref(false)

  async function fetchModels() {
    models.value = await api.getPublicModels()
  }

  async function fetchKeyId(data) {
    const res = await api.getPublicKeyId(data)
    keyId.value = res
    return res
  }

  async function fetchUserStats(data) {
    loading.value = true
    try {
      userStats.value = await api.getPublicUserStats(data)
    } finally {
      loading.value = false
    }
  }

  async function fetchUserModelStats(data) {
    userModelStats.value = await api.getPublicUserModelStats(data)
  }

  async function fetchServiceRates() {
    serviceRates.value = await api.getPublicServiceRates()
  }

  return {
    models,
    keyId,
    userStats,
    userModelStats,
    serviceRates,
    loading,
    fetchModels,
    fetchKeyId,
    fetchUserStats,
    fetchUserModelStats,
    fetchServiceRates
  }
})

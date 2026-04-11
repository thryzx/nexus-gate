import { defineStore } from 'pinia'
import { ref } from 'vue'
import * as api from '@/utils/api'

export const useSettingsStore = defineStore('settings', () => {
  const oemSettings = ref({})
  const webhookConfig = ref({})
  const serviceRates = ref({})
  const modelPricing = ref([])
  const loading = ref(false)

  async function fetchOem() {
    oemSettings.value = await api.getOemSettings()
  }

  async function updateOem(data) {
    await api.updateOemSettings(data)
    await fetchOem()
  }

  async function fetchWebhook() {
    webhookConfig.value = await api.getWebhookConfig()
  }

  async function updateWebhook(data) {
    await api.updateWebhookConfig(data)
    await fetchWebhook()
  }

  async function addWebhookPlatform(data) {
    await api.createWebhookPlatform(data)
    await fetchWebhook()
  }

  async function removeWebhookPlatform(id) {
    await api.deleteWebhookPlatform(id)
    await fetchWebhook()
  }

  async function testWebhookNotification(data) {
    return api.testWebhook(data)
  }

  async function fetchServiceRates() {
    serviceRates.value = await api.getServiceRates()
  }

  async function updateRates(data) {
    await api.updateServiceRates(data)
    await fetchServiceRates()
  }

  async function fetchModelPricing() {
    modelPricing.value = await api.getModelPricing()
  }

  async function refreshPricing() {
    await api.refreshModelPricing()
    await fetchModelPricing()
  }

  async function fetchAll() {
    loading.value = true
    try {
      await Promise.allSettled([
        fetchOem(),
        fetchWebhook(),
        fetchServiceRates(),
        fetchModelPricing()
      ])
    } finally {
      loading.value = false
    }
  }

  return {
    oemSettings,
    webhookConfig,
    serviceRates,
    modelPricing,
    loading,
    fetchOem,
    updateOem,
    fetchWebhook,
    updateWebhook,
    addWebhookPlatform,
    removeWebhookPlatform,
    testWebhookNotification,
    fetchServiceRates,
    updateRates,
    fetchModelPricing,
    refreshPricing,
    fetchAll
  }
})

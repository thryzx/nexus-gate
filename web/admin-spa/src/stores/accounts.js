import { defineStore } from 'pinia'
import { ref } from 'vue'
import * as api from '@/utils/api'

export const useAccountsStore = defineStore('accounts', () => {
  const list = ref([])
  const loading = ref(false)

  async function fetch() {
    loading.value = true
    try {
      list.value = await api.getAccounts()
    } finally {
      loading.value = false
    }
  }

  async function create(data) {
    const res = await api.createAccount(data)
    await fetch()
    return res
  }

  async function update(id, data) {
    const res = await api.updateAccount(id, data)
    await fetch()
    return res
  }

  async function remove(id) {
    await api.deleteAccount(id)
    await fetch()
  }

  return { list, loading, fetch, create, update, remove }
})

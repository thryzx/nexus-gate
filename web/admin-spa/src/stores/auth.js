import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { login as apiLogin } from '@/utils/api'

export const useAuthStore = defineStore('auth', () => {
  const token = ref(localStorage.getItem('token') || '')
  const username = ref(localStorage.getItem('username') || '')

  const isAuthenticated = computed(() => !!token.value)

  async function login(credentials) {
    const res = await apiLogin(credentials)
    token.value = res.token
    username.value = res.username
    localStorage.setItem('token', res.token)
    localStorage.setItem('username', res.username)
    return res
  }

  function logout() {
    token.value = ''
    username.value = ''
    localStorage.removeItem('token')
    localStorage.removeItem('username')
  }

  return { token, username, isAuthenticated, login, logout }
})

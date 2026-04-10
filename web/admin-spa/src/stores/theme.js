import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export const useThemeStore = defineStore('theme', () => {
  const mode = ref(localStorage.getItem('theme') || 'auto')
  const systemDark = ref(false)

  const isDark = computed(() => {
    if (mode.value === 'dark') return true
    if (mode.value === 'light') return false
    return systemDark.value
  })

  function setMode(m) {
    mode.value = m
    localStorage.setItem('theme', m)
    applyTheme()
  }

  function applyTheme() {
    if (isDark.value) {
      document.documentElement.classList.add('dark')
    } else {
      document.documentElement.classList.remove('dark')
    }
  }

  function init() {
    const mq = window.matchMedia('(prefers-color-scheme: dark)')
    systemDark.value = mq.matches
    mq.addEventListener('change', (e) => {
      systemDark.value = e.matches
      applyTheme()
    })
    applyTheme()
  }

  return { mode, isDark, setMode, init }
})

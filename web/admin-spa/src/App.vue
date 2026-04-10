<template>
  <div id="app" :class="{ dark: isDark }">
    <router-view />
  </div>
</template>

<script setup>
import { computed, onMounted } from 'vue'
import { useThemeStore } from '@/stores/theme'

const themeStore = useThemeStore()
const isDark = computed(() => themeStore.isDark)

onMounted(() => {
  themeStore.init()
})
</script>

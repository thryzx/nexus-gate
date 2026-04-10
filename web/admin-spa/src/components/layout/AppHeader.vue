<template>
  <header class="fixed top-0 left-0 right-0 z-50 h-14 bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 flex items-center justify-between px-6 shadow-sm">
    <div class="flex items-center gap-3">
      <span class="text-xl font-bold bg-gradient-to-r from-blue-600 to-purple-600 bg-clip-text text-transparent">Nexus Gate</span>
      <span class="text-xs text-gray-400">Admin</span>
    </div>
    <div class="flex items-center gap-4">
      <el-dropdown @command="handleTheme">
        <el-button size="small" text>
          <span v-if="themeStore.isDark">🌙</span>
          <span v-else>☀️</span>
        </el-button>
        <template #dropdown>
          <el-dropdown-menu>
            <el-dropdown-item command="light">浅色</el-dropdown-item>
            <el-dropdown-item command="dark">深色</el-dropdown-item>
            <el-dropdown-item command="auto">跟随系统</el-dropdown-item>
          </el-dropdown-menu>
        </template>
      </el-dropdown>
      <el-dropdown @command="handleUser">
        <span class="text-sm text-gray-600 dark:text-gray-300 cursor-pointer">{{ authStore.username }}</span>
        <template #dropdown>
          <el-dropdown-menu>
            <el-dropdown-item command="logout">退出登录</el-dropdown-item>
          </el-dropdown-menu>
        </template>
      </el-dropdown>
    </div>
  </header>
</template>

<script setup>
import { useThemeStore } from '@/stores/theme'
import { useAuthStore } from '@/stores/auth'
import { useRouter } from 'vue-router'

const themeStore = useThemeStore()
const authStore = useAuthStore()
const router = useRouter()

function handleTheme(cmd) {
  themeStore.setMode(cmd)
}

function handleUser(cmd) {
  if (cmd === 'logout') {
    authStore.logout()
    router.push('/login')
  }
}
</script>

<template>
  <aside class="fixed top-14 left-0 bottom-0 w-64 bg-white dark:bg-gray-800 border-r border-gray-200 dark:border-gray-700 overflow-y-auto">
    <nav class="py-4">
      <router-link
        v-for="item in menuItems"
        :key="item.path"
        :to="item.path"
        class="flex items-center gap-3 px-6 py-3 text-sm text-gray-600 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
        active-class="!text-blue-600 dark:!text-blue-400 !bg-blue-50 dark:!bg-blue-900/20 border-r-2 border-blue-600"
      >
        <span class="text-lg">{{ item.icon }}</span>
        <span>{{ item.label }}</span>
      </router-link>
    </nav>
  </aside>
</template>

<script setup>
const menuItems = [
  { path: '/dashboard', icon: '📊', label: '仪表盘' },
  { path: '/accounts', icon: '👤', label: '账户管理' },
  { path: '/api-keys', icon: '🔑', label: 'API Keys' },
  { path: '/usage', icon: '📈', label: '用量统计' },
  { path: '/fingerprints', icon: '🔏', label: '指纹配置' },
  { path: '/settings', icon: '⚙️', label: '系统设置' }
]
</script>

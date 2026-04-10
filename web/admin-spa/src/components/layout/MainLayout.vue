<template>
  <div class="min-h-screen bg-gray-50 dark:bg-gray-900">
    <AppHeader />
    <div class="flex">
      <AppSidebar />
      <main class="flex-1 p-6 ml-64">
        <router-view />
      </main>
    </div>
  </div>
</template>

<script setup>
import AppHeader from './AppHeader.vue'
import AppSidebar from './AppSidebar.vue'
</script>

<template>
  <div>
    <h2 class="text-2xl font-bold text-gray-800 dark:text-gray-100 mb-6">仪表盘</h2>

    <div v-if="loading" class="flex justify-center py-20">
      <el-icon class="is-loading text-4xl text-blue-500"><Loading /></el-icon>
    </div>

    <template v-else-if="data">
      <!-- Stats Cards -->
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
        <StatCard title="总账户数" :value="data.accounts.total" :sub="`活跃: ${data.accounts.active}`" color="blue" />
        <StatCard title="API Keys" :value="data.api_keys.total" :sub="`活跃: ${data.api_keys.active}`" color="green" />
        <StatCard title="总请求数" :value="data.usage.total_requests" :sub="`今日: ${data.usage.today_requests}`" color="purple" />
        <StatCard title="总成本 (USD)" :value="'$' + data.usage.total_cost_usd.toFixed(4)" :sub="`今日: $${data.usage.today_cost_usd.toFixed(4)}`" color="orange" />
      </div>

      <!-- Token Stats -->
      <div class="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-8">
        <div class="bg-white dark:bg-gray-800 rounded-xl shadow p-6 border border-gray-200 dark:border-gray-700">
          <h3 class="text-lg font-semibold text-gray-700 dark:text-gray-200 mb-4">Token 用量</h3>
          <div class="space-y-3">
            <div class="flex justify-between">
              <span class="text-gray-500 dark:text-gray-400">输入 Tokens</span>
              <span class="font-mono text-gray-800 dark:text-gray-200">{{ formatNumber(data.usage.total_input_tokens) }}</span>
            </div>
            <div class="flex justify-between">
              <span class="text-gray-500 dark:text-gray-400">输出 Tokens</span>
              <span class="font-mono text-gray-800 dark:text-gray-200">{{ formatNumber(data.usage.total_output_tokens) }}</span>
            </div>
          </div>
        </div>

        <div class="bg-white dark:bg-gray-800 rounded-xl shadow p-6 border border-gray-200 dark:border-gray-700">
          <h3 class="text-lg font-semibold text-gray-700 dark:text-gray-200 mb-4">平台分布</h3>
          <div class="space-y-3">
            <div v-for="p in data.accounts.by_platform" :key="p.platform" class="flex justify-between">
              <span class="text-gray-500 dark:text-gray-400 capitalize">{{ p.platform }}</span>
              <el-tag size="small">{{ p.count }}</el-tag>
            </div>
            <div v-if="!data.accounts.by_platform.length" class="text-gray-400 text-sm">暂无账户</div>
          </div>
        </div>
      </div>
    </template>
  </div>
</template>

<script setup>
import { onMounted } from 'vue'
import { storeToRefs } from 'pinia'
import { useDashboardStore } from '@/stores/dashboard'
import { Loading } from '@element-plus/icons-vue'
import StatCard from '@/components/common/StatCard.vue'

const store = useDashboardStore()
const { data, loading } = storeToRefs(store)

function formatNumber(n) {
  if (n >= 1e9) return (n / 1e9).toFixed(2) + 'B'
  if (n >= 1e6) return (n / 1e6).toFixed(2) + 'M'
  if (n >= 1e3) return (n / 1e3).toFixed(1) + 'K'
  return String(n)
}

onMounted(() => store.fetch())
</script>

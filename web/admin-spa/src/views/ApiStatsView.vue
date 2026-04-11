<template>
  <div>
    <h2 class="text-2xl font-bold text-gray-800 dark:text-gray-100 mb-6">API 用量统计</h2>

    <!-- Key Input -->
    <div class="mb-6 bg-white dark:bg-gray-800 rounded-xl shadow border border-gray-200 dark:border-gray-700 p-6">
      <el-form :inline="true" @submit.prevent="fetchStats">
        <el-form-item label="API Key">
          <el-input v-model="apiKey" placeholder="输入 API Key" style="width: 400px" show-password />
        </el-form-item>
        <el-form-item>
          <el-button type="primary" @click="fetchStats" :loading="loading">查询</el-button>
        </el-form-item>
      </el-form>
    </div>

    <!-- Stats -->
    <template v-if="stats">
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-6">
        <StatCard title="总请求" :value="stats.total_requests || 0" color="blue" />
        <StatCard title="输入 Tokens" :value="formatNumber(stats.total_input_tokens || 0)" color="green" />
        <StatCard title="输出 Tokens" :value="formatNumber(stats.total_output_tokens || 0)" color="purple" />
        <StatCard title="总成本" :value="'$' + (stats.total_cost_usd || 0).toFixed(4)" color="orange" />
      </div>

      <!-- Model stats -->
      <div class="bg-white dark:bg-gray-800 rounded-xl shadow border border-gray-200 dark:border-gray-700 p-6">
        <h3 class="text-lg font-semibold text-gray-700 dark:text-gray-200 mb-4">模型用量</h3>
        <el-table :data="modelStats" stripe>
          <el-table-column prop="model" label="模型" />
          <el-table-column prop="requests" label="请求数" />
          <el-table-column prop="input_tokens" label="输入 Tokens">
            <template #default="{ row }">{{ formatNumber(row.input_tokens || 0) }}</template>
          </el-table-column>
          <el-table-column prop="output_tokens" label="输出 Tokens">
            <template #default="{ row }">{{ formatNumber(row.output_tokens || 0) }}</template>
          </el-table-column>
          <el-table-column prop="cost_usd" label="成本 (USD)">
            <template #default="{ row }">${{ (row.cost_usd || 0).toFixed(4) }}</template>
          </el-table-column>
        </el-table>
      </div>
    </template>

    <!-- Available Models -->
    <div class="mt-6 bg-white dark:bg-gray-800 rounded-xl shadow border border-gray-200 dark:border-gray-700 p-6">
      <h3 class="text-lg font-semibold text-gray-700 dark:text-gray-200 mb-4">可用模型</h3>
      <div class="flex flex-wrap gap-2">
        <el-tag v-for="m in models" :key="m" type="info" effect="plain">{{ m }}</el-tag>
        <span v-if="!models.length" class="text-gray-400 text-sm">暂无</span>
      </div>
    </div>

    <!-- Service Rates -->
    <div class="mt-6 bg-white dark:bg-gray-800 rounded-xl shadow border border-gray-200 dark:border-gray-700 p-6">
      <h3 class="text-lg font-semibold text-gray-700 dark:text-gray-200 mb-4">服务费率</h3>
      <el-table :data="ratesList" stripe size="small">
        <el-table-column prop="service" label="服务" />
        <el-table-column prop="rate" label="倍率" />
      </el-table>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import { useApiStatsStore } from '@/stores/apistats'
import StatCard from '@/components/common/StatCard.vue'

const store = useApiStatsStore()
const apiKey = ref('')
const loading = ref(false)
const stats = ref(null)
const modelStats = ref([])
const models = ref([])
const ratesList = computed(() => {
  const r = store.serviceRates
  if (!r || typeof r !== 'object') return []
  return Object.entries(r).map(([service, rate]) => ({ service, rate }))
})

function formatNumber(n) {
  if (n >= 1e9) return (n / 1e9).toFixed(2) + 'B'
  if (n >= 1e6) return (n / 1e6).toFixed(2) + 'M'
  if (n >= 1e3) return (n / 1e3).toFixed(1) + 'K'
  return String(n)
}

async function fetchStats() {
  if (!apiKey.value) return
  loading.value = true
  try {
    const [userStats, userModelStats] = await Promise.all([
      store.fetchUserStats({ apiKey: apiKey.value }),
      store.fetchUserModelStats({ apiKey: apiKey.value })
    ])
    stats.value = store.userStats
    modelStats.value = store.userModelStats || []
  } finally {
    loading.value = false
  }
}

onMounted(async () => {
  await Promise.all([store.fetchModels(), store.fetchServiceRates()])
  models.value = store.models || []
})
</script>

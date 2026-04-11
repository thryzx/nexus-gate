<template>
  <div>
    <h2 class="mb-6 text-2xl font-bold text-gray-800 dark:text-gray-100">API 用量统计</h2>

    <!-- Key Input -->
    <div class="card mb-6 p-6">
      <el-form :inline="true" @submit.prevent="fetchStats">
        <el-form-item label="API Key">
          <el-input v-model="apiKey" placeholder="输入 API Key" style="width: 400px" show-password />
        </el-form-item>
        <el-form-item>
          <el-button type="primary" @click="fetchStats" :loading="loading">查询</el-button>
        </el-form-item>
      </el-form>
    </div>

    <!-- Stats Cards -->
    <template v-if="stats">
      <div class="mb-6 grid grid-cols-2 gap-4 lg:grid-cols-4">
        <div class="stat-card">
          <p class="stat-label">总请求</p>
          <p class="stat-value text-blue-600">{{ fmtNum(stats.total_requests || 0) }}</p>
        </div>
        <div class="stat-card">
          <p class="stat-label">输入 Tokens</p>
          <p class="stat-value text-green-600">{{ fmtNum(stats.total_input_tokens || 0) }}</p>
        </div>
        <div class="stat-card">
          <p class="stat-label">输出 Tokens</p>
          <p class="stat-value text-purple-600">{{ fmtNum(stats.total_output_tokens || 0) }}</p>
        </div>
        <div class="stat-card">
          <p class="stat-label">总成本</p>
          <p class="stat-value text-orange-600">${{ (stats.total_cost_usd || 0).toFixed(4) }}</p>
        </div>
      </div>

      <!-- Charts -->
      <div class="mb-6 grid grid-cols-1 gap-6 lg:grid-cols-2">
        <div class="card p-5">
          <h3 class="mb-2 text-sm font-semibold text-gray-900 dark:text-gray-100">模型成本分布</h3>
          <div class="relative h-56"><canvas ref="modelPieRef" /></div>
        </div>
        <div class="card p-5">
          <h3 class="mb-2 text-sm font-semibold text-gray-900 dark:text-gray-100">模型请求量</h3>
          <div class="relative h-56"><canvas ref="modelBarRef" /></div>
        </div>
      </div>

      <!-- Model stats table -->
      <div class="card mb-6 p-6">
        <h3 class="mb-3 text-sm font-semibold text-gray-900 dark:text-gray-100">模型用量明细</h3>
        <el-table :data="modelStats" stripe size="small" max-height="300">
          <el-table-column prop="model" label="模型" min-width="200" show-overflow-tooltip />
          <el-table-column label="请求数" width="90" align="right" sortable :sort-method="(a, b) => (a.requests || 0) - (b.requests || 0)">
            <template #default="{ row }">{{ fmtNum(row.requests || 0) }}</template>
          </el-table-column>
          <el-table-column label="输入 Token" width="120" align="right" sortable :sort-method="(a, b) => (a.input_tokens || 0) - (b.input_tokens || 0)">
            <template #default="{ row }">{{ fmtNum(row.input_tokens || 0) }}</template>
          </el-table-column>
          <el-table-column label="输出 Token" width="120" align="right" sortable :sort-method="(a, b) => (a.output_tokens || 0) - (b.output_tokens || 0)">
            <template #default="{ row }">{{ fmtNum(row.output_tokens || 0) }}</template>
          </el-table-column>
          <el-table-column label="成本" width="100" align="right" sortable :sort-method="(a, b) => (a.cost_usd || 0) - (b.cost_usd || 0)">
            <template #default="{ row }">${{ (row.cost_usd || 0).toFixed(4) }}</template>
          </el-table-column>
          <el-table-column label="占比" width="70" align="right">
            <template #default="{ row }">
              {{ stats.total_cost_usd > 0 ? (((row.cost_usd || 0) / stats.total_cost_usd) * 100).toFixed(1) : '0' }}%
            </template>
          </el-table-column>
        </el-table>
      </div>
    </template>

    <!-- Available Models & Service Rates side by side -->
    <div class="grid grid-cols-1 gap-6 lg:grid-cols-2">
      <div class="card p-6">
        <h3 class="mb-3 text-sm font-semibold text-gray-900 dark:text-gray-100">可用模型</h3>
        <div class="flex flex-wrap gap-2">
          <el-tag v-for="m in models" :key="m" type="info" effect="plain" size="small">{{ m }}</el-tag>
          <span v-if="!models.length" class="text-sm text-gray-400">暂无</span>
        </div>
      </div>
      <div class="card p-6">
        <h3 class="mb-3 text-sm font-semibold text-gray-900 dark:text-gray-100">服务费率</h3>
        <el-table :data="ratesList" stripe size="small" max-height="240">
          <el-table-column prop="service" label="服务" />
          <el-table-column prop="rate" label="倍率" width="80" align="right" />
        </el-table>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted, nextTick } from 'vue'
import { useApiStatsStore } from '@/stores/apistats'
import Chart from 'chart.js/auto'

const store = useApiStatsStore()
const apiKey = ref('')
const loading = ref(false)
const stats = ref(null)
const modelStats = ref([])
const models = ref([])
const modelPieRef = ref(null)
const modelBarRef = ref(null)
let pieChart = null
let barChart = null

const COLORS = ['#3B82F6','#10B981','#F59E0B','#EF4444','#8B5CF6','#EC4899','#06B6D4','#F97316','#6366F1','#14B8A6']

const ratesList = computed(() => {
  const r = store.serviceRates
  if (!r || typeof r !== 'object') return []
  return Object.entries(r).map(([service, rate]) => ({ service, rate }))
})

function fmtNum(n) {
  if (!n && n !== 0) return '0'
  if (n >= 1e9) return (n / 1e9).toFixed(2) + 'B'
  if (n >= 1e6) return (n / 1e6).toFixed(2) + 'M'
  if (n >= 1e3) return (n / 1e3).toFixed(1) + 'K'
  return String(n)
}

function isDark() { return document.documentElement.classList.contains('dark') }
function txColor() { return isDark() ? '#9CA3AF' : '#6B7280' }

function renderCharts() {
  if (!modelStats.value.length) return
  const sorted = [...modelStats.value].sort((a, b) => (b.cost_usd || 0) - (a.cost_usd || 0)).slice(0, 10)
  const labels = sorted.map((m) => m.model)

  if (modelPieRef.value) {
    if (pieChart) pieChart.destroy()
    pieChart = new Chart(modelPieRef.value, {
      type: 'doughnut',
      data: { labels, datasets: [{ data: sorted.map((m) => m.cost_usd || 0), backgroundColor: COLORS.slice(0, labels.length), borderColor: isDark() ? '#1F2937' : '#FFF', borderWidth: 2 }] },
      options: { responsive: true, maintainAspectRatio: false, plugins: { legend: { position: 'bottom', labels: { color: txColor(), usePointStyle: true, font: { size: 10 }, padding: 8 } }, tooltip: { callbacks: { label: (c) => `$${c.parsed.toFixed(4)}` } } } }
    })
  }

  if (modelBarRef.value) {
    if (barChart) barChart.destroy()
    barChart = new Chart(modelBarRef.value, {
      type: 'bar',
      data: { labels, datasets: [{ label: '请求数', data: sorted.map((m) => m.requests || 0), backgroundColor: COLORS.slice(0, labels.length) }] },
      options: {
        responsive: true, maintainAspectRatio: false, indexAxis: 'y',
        plugins: { legend: { display: false } },
        scales: { x: { ticks: { color: txColor() }, grid: { color: isDark() ? 'rgba(75,85,99,0.3)' : 'rgba(229,231,235,0.8)' } }, y: { ticks: { color: txColor(), font: { size: 10 } } } }
      }
    })
  }
}

async function fetchStats() {
  if (!apiKey.value) return
  loading.value = true
  try {
    await Promise.all([store.fetchUserStats({ apiKey: apiKey.value }), store.fetchUserModelStats({ apiKey: apiKey.value })])
    stats.value = store.userStats
    modelStats.value = store.userModelStats || []
    await nextTick()
    renderCharts()
  } finally { loading.value = false }
}

onMounted(async () => {
  await Promise.all([store.fetchModels(), store.fetchServiceRates()])
  models.value = store.models || []
})
onUnmounted(() => { pieChart?.destroy(); barChart?.destroy() })
</script>

<style scoped>
.stat-card { @apply rounded-xl border border-gray-200 bg-white p-4 shadow-sm dark:border-gray-700 dark:bg-gray-800; }
.stat-label { @apply text-xs font-semibold text-gray-500 dark:text-gray-400; }
.stat-value { @apply text-2xl font-bold; }
.card { @apply rounded-xl border border-gray-200 bg-white shadow dark:border-gray-700 dark:bg-gray-800; }
</style>

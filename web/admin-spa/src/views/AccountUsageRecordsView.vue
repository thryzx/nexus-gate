<template>
  <div>
    <div class="mb-6 flex items-center gap-3">
      <el-button text @click="$router.back()">
        <el-icon><ArrowLeft /></el-icon>
      </el-button>
      <h2 class="text-2xl font-bold text-gray-800 dark:text-gray-100">账户用量记录</h2>
      <el-tag size="small" type="info" effect="plain" class="ml-2">{{ accountId }}</el-tag>
    </div>

    <!-- Summary Cards -->
    <div class="mb-6 grid grid-cols-2 gap-4 lg:grid-cols-4">
      <div class="stat-card">
        <p class="text-xs font-semibold text-gray-500 dark:text-gray-400">总请求</p>
        <p class="text-2xl font-bold text-gray-900 dark:text-gray-100">{{ fmtNum(summary.requests) }}</p>
      </div>
      <div class="stat-card">
        <p class="text-xs font-semibold text-gray-500 dark:text-gray-400">总 Token</p>
        <p class="text-2xl font-bold text-blue-600">{{ fmtNum(summary.tokens) }}</p>
      </div>
      <div class="stat-card">
        <p class="text-xs font-semibold text-gray-500 dark:text-gray-400">总成本</p>
        <p class="text-2xl font-bold text-green-600">${{ summary.cost.toFixed(4) }}</p>
      </div>
      <div class="stat-card">
        <p class="text-xs font-semibold text-gray-500 dark:text-gray-400">模型数</p>
        <p class="text-2xl font-bold text-purple-600">{{ modelSet.size }}</p>
      </div>
    </div>

    <!-- Chart -->
    <div class="card mb-6 p-5">
      <h3 class="mb-2 text-sm font-semibold text-gray-900 dark:text-gray-100">用量趋势</h3>
      <div class="relative h-56"><canvas ref="chartRef" /></div>
    </div>

    <!-- Filters -->
    <div class="mb-4 flex flex-wrap gap-3">
      <el-date-picker v-model="dateRange" type="daterange" range-separator="-" start-placeholder="开始" end-placeholder="结束" value-format="YYYY-MM-DD" @change="fetchRecords" />
      <el-select v-model="modelFilter" placeholder="模型" clearable style="width: 200px" @change="fetchRecords" filterable>
        <el-option v-for="m in [...modelSet]" :key="m" :value="m" :label="m" />
      </el-select>
      <el-button @click="exportCSV" size="small" :disabled="!records.length">导出 CSV</el-button>
    </div>

    <!-- Table -->
    <div class="card">
      <el-table :data="paginatedRecords" v-loading="loading" stripe size="small">
        <el-table-column label="时间" width="160">
          <template #default="{ row }">{{ dayjs(row.created_at).format('MM-DD HH:mm:ss') }}</template>
        </el-table-column>
        <el-table-column prop="model" label="模型" min-width="180" show-overflow-tooltip />
        <el-table-column label="输入 Token" width="120" align="right">
          <template #default="{ row }">{{ fmtNum(row.input_tokens) }}</template>
        </el-table-column>
        <el-table-column label="输出 Token" width="120" align="right">
          <template #default="{ row }">{{ fmtNum(row.output_tokens) }}</template>
        </el-table-column>
        <el-table-column label="成本" width="100" align="right">
          <template #default="{ row }">${{ (row.cost_usd || 0).toFixed(4) }}</template>
        </el-table-column>
        <el-table-column label="状态码" width="80" align="center">
          <template #default="{ row }">
            <el-tag :type="row.status_code >= 200 && row.status_code < 300 ? 'success' : 'danger'" size="small">{{ row.status_code || '-' }}</el-tag>
          </template>
        </el-table-column>
      </el-table>
      <div class="flex justify-end p-4">
        <el-pagination
          v-model:current-page="page"
          :page-size="pageSize"
          :total="filteredRecords.length"
          layout="total, sizes, prev, pager, next"
          :page-sizes="[20, 50, 100]"
          @current-change="() => {}"
          @size-change="(s) => { pageSize = s; page = 1 }"
        />
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted, nextTick } from 'vue'
import { useRoute } from 'vue-router'
import { ArrowLeft } from '@element-plus/icons-vue'
import { getAccountUsageRecords } from '@/utils/api'
import Chart from 'chart.js/auto'
import dayjs from 'dayjs'

const route = useRoute()
const accountId = route.params.accountId
const records = ref([])
const loading = ref(false)
const dateRange = ref(null)
const modelFilter = ref('')
const page = ref(1)
const pageSize = ref(50)
const chartRef = ref(null)
let chart = null

function fmtNum(n) {
  if (!n && n !== 0) return '0'
  if (n >= 1e6) return (n / 1e6).toFixed(2) + 'M'
  if (n >= 1e3) return (n / 1e3).toFixed(1) + 'K'
  return String(n)
}

const modelSet = computed(() => new Set(records.value.map((r) => r.model).filter(Boolean)))

const filteredRecords = computed(() => {
  let list = records.value
  if (modelFilter.value) list = list.filter((r) => r.model === modelFilter.value)
  return list
})

const paginatedRecords = computed(() => {
  const start = (page.value - 1) * pageSize.value
  return filteredRecords.value.slice(start, start + pageSize.value)
})

const summary = computed(() => {
  const r = records.value
  return {
    requests: r.length,
    tokens: r.reduce((s, x) => s + (x.input_tokens || 0) + (x.output_tokens || 0), 0),
    cost: r.reduce((s, x) => s + (x.cost_usd || 0), 0)
  }
})

function isDark() { return document.documentElement.classList.contains('dark') }
function txColor() { return isDark() ? '#9CA3AF' : '#6B7280' }

function renderChart() {
  if (!chartRef.value || !records.value.length) return
  if (chart) chart.destroy()
  // Group by date
  const dayMap = {}
  records.value.forEach((r) => {
    const d = dayjs(r.created_at).format('YYYY-MM-DD')
    if (!dayMap[d]) dayMap[d] = { requests: 0, tokens: 0, cost: 0 }
    dayMap[d].requests++
    dayMap[d].tokens += (r.input_tokens || 0) + (r.output_tokens || 0)
    dayMap[d].cost += r.cost_usd || 0
  })
  const labels = Object.keys(dayMap).sort()
  chart = new Chart(chartRef.value, {
    type: 'line',
    data: {
      labels,
      datasets: [
        { label: '请求数', data: labels.map((d) => dayMap[d].requests), borderColor: '#3B82F6', backgroundColor: 'rgba(59,130,246,0.1)', yAxisID: 'y', tension: 0.3, fill: true },
        { label: 'Tokens', data: labels.map((d) => dayMap[d].tokens), borderColor: '#10B981', yAxisID: 'y1', tension: 0.3 }
      ]
    },
    options: {
      responsive: true, maintainAspectRatio: false,
      interaction: { mode: 'index', intersect: false },
      plugins: { legend: { labels: { color: txColor(), usePointStyle: true, font: { size: 11 } } } },
      scales: {
        x: { ticks: { color: txColor(), font: { size: 10 } } },
        y: { position: 'left', ticks: { color: txColor() } },
        y1: { position: 'right', ticks: { color: txColor(), callback: (v) => fmtNum(v) }, grid: { display: false } }
      }
    }
  })
}

async function fetchRecords() {
  loading.value = true
  try {
    const params = { model: modelFilter.value || undefined }
    if (dateRange.value) { params.start_date = dateRange.value[0]; params.end_date = dateRange.value[1] }
    const res = await getAccountUsageRecords(accountId, params)
    records.value = Array.isArray(res) ? res : (res.records || res.data || [])
    await nextTick()
    renderChart()
  } finally { loading.value = false }
}

function exportCSV() {
  const header = 'time,model,input_tokens,output_tokens,cost_usd\n'
  const rows = records.value.map((r) => [dayjs(r.created_at).format('YYYY-MM-DD HH:mm:ss'), r.model, r.input_tokens, r.output_tokens, (r.cost_usd || 0).toFixed(6)].join(',')).join('\n')
  const blob = new Blob([header + rows], { type: 'text/csv' })
  const url = URL.createObjectURL(blob); const a = document.createElement('a'); a.href = url; a.download = `account-${accountId}-usage.csv`; a.click(); URL.revokeObjectURL(url)
}

onMounted(fetchRecords)
onUnmounted(() => chart?.destroy())
</script>

<style scoped>
.stat-card { @apply rounded-xl border border-gray-200 bg-white p-4 shadow-sm dark:border-gray-700 dark:bg-gray-800; }
.card { @apply rounded-xl border border-gray-200 bg-white shadow dark:border-gray-700 dark:bg-gray-800; }
</style>

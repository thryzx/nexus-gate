<template>
  <div>
    <div class="mb-6 flex items-center justify-between">
      <h2 class="text-2xl font-bold text-gray-800 dark:text-gray-100">用量统计</h2>
      <div class="flex items-center gap-3">
        <el-radio-group v-model="days" size="small" @change="fetchAll">
          <el-radio-button :value="1">今天</el-radio-button>
          <el-radio-button :value="7">7 天</el-radio-button>
          <el-radio-button :value="30">30 天</el-radio-button>
          <el-radio-button :value="90">90 天</el-radio-button>
        </el-radio-group>
        <el-radio-group v-model="granularity" size="small" @change="fetchTrends">
          <el-radio-button value="day">按天</el-radio-button>
          <el-radio-button value="hour">按小时</el-radio-button>
        </el-radio-group>
      </div>
    </div>

    <!-- Summary Cards -->
    <div class="mb-6 grid grid-cols-2 gap-4 lg:grid-cols-4">
      <div class="stat-card">
        <p class="text-xs font-semibold text-gray-500 dark:text-gray-400">总请求</p>
        <p class="text-2xl font-bold text-gray-900 dark:text-gray-100">{{ fmtNum(summaryRequests) }}</p>
      </div>
      <div class="stat-card">
        <p class="text-xs font-semibold text-gray-500 dark:text-gray-400">总 Token</p>
        <p class="text-2xl font-bold text-blue-600">{{ fmtNum(summaryTokens) }}</p>
      </div>
      <div class="stat-card">
        <p class="text-xs font-semibold text-gray-500 dark:text-gray-400">总成本</p>
        <p class="text-2xl font-bold text-green-600">${{ summaryCost.toFixed(4) }}</p>
      </div>
      <div class="stat-card">
        <p class="text-xs font-semibold text-gray-500 dark:text-gray-400">模型数</p>
        <p class="text-2xl font-bold text-purple-600">{{ modelData.length }}</p>
      </div>
    </div>

    <!-- Charts Row -->
    <div class="mb-6 grid grid-cols-1 gap-6 lg:grid-cols-2">
      <div class="card p-6">
        <h3 class="mb-3 text-sm font-semibold text-gray-900 dark:text-gray-100">请求 & Token 趋势</h3>
        <div class="relative h-72"><canvas ref="trendChartRef" /></div>
      </div>
      <div class="card p-6">
        <h3 class="mb-3 text-sm font-semibold text-gray-900 dark:text-gray-100">成本趋势</h3>
        <div class="relative h-72"><canvas ref="costChartRef" /></div>
      </div>
    </div>

    <!-- Model Distribution -->
    <div class="mb-6 grid grid-cols-1 gap-6 lg:grid-cols-3">
      <div class="card p-6">
        <h3 class="mb-3 text-sm font-semibold text-gray-900 dark:text-gray-100">模型成本分布</h3>
        <div class="relative h-64"><canvas ref="modelPieRef" /></div>
      </div>
      <div class="card col-span-1 p-6 lg:col-span-2">
        <h3 class="mb-3 text-sm font-semibold text-gray-900 dark:text-gray-100">模型用量明细</h3>
        <el-table :data="modelData" v-loading="modelLoading" stripe size="small" max-height="280">
          <el-table-column prop="model" label="模型" min-width="180" show-overflow-tooltip />
          <el-table-column label="请求数" width="90" align="right" sortable :sort-method="(a, b) => a.requests - b.requests">
            <template #default="{ row }">{{ fmtNum(row.requests) }}</template>
          </el-table-column>
          <el-table-column label="输入 Token" width="110" align="right" sortable :sort-method="(a, b) => a.input_tokens - b.input_tokens">
            <template #default="{ row }">{{ fmtNum(row.input_tokens) }}</template>
          </el-table-column>
          <el-table-column label="输出 Token" width="110" align="right" sortable :sort-method="(a, b) => a.output_tokens - b.output_tokens">
            <template #default="{ row }">{{ fmtNum(row.output_tokens) }}</template>
          </el-table-column>
          <el-table-column label="成本" width="100" align="right" sortable :sort-method="(a, b) => a.cost_usd - b.cost_usd">
            <template #default="{ row }">${{ row.cost_usd.toFixed(4) }}</template>
          </el-table-column>
          <el-table-column label="占比" width="70" align="right">
            <template #default="{ row }">{{ summaryCost > 0 ? ((row.cost_usd / summaryCost) * 100).toFixed(1) : '0' }}%</template>
          </el-table-column>
        </el-table>
      </div>
    </div>

    <!-- Trend Table -->
    <div class="card mb-6 p-6">
      <h3 class="mb-3 text-sm font-semibold text-gray-900 dark:text-gray-100">
        趋势数据 <span class="ml-2 text-xs font-normal text-gray-400">{{ trendData.length }} 条</span>
      </h3>
      <el-table :data="trendData" v-loading="trendLoading" stripe size="small" max-height="300">
        <el-table-column prop="date" label="日期" width="140" />
        <el-table-column label="请求数" width="100" align="right">
          <template #default="{ row }">{{ fmtNum(row.requests) }}</template>
        </el-table-column>
        <el-table-column label="输入 Token" width="120" align="right">
          <template #default="{ row }">{{ fmtNum(row.input_tokens) }}</template>
        </el-table-column>
        <el-table-column label="输出 Token" width="120" align="right">
          <template #default="{ row }">{{ fmtNum(row.output_tokens) }}</template>
        </el-table-column>
        <el-table-column label="成本" width="100" align="right">
          <template #default="{ row }">${{ (row.cost_usd || 0).toFixed(4) }}</template>
        </el-table-column>
      </el-table>
    </div>

    <!-- Request Records -->
    <div class="card p-6">
      <div class="mb-3 flex items-center justify-between">
        <h3 class="text-sm font-semibold text-gray-900 dark:text-gray-100">请求记录</h3>
        <el-tag size="small" type="info">共 {{ total }} 条</el-tag>
      </div>
      <el-table :data="records" v-loading="recordsLoading" stripe size="small">
        <el-table-column label="时间" width="160">
          <template #default="{ row }">{{ dayjs(row.created_at).format('MM-DD HH:mm:ss') }}</template>
        </el-table-column>
        <el-table-column prop="model" label="模型" min-width="180" show-overflow-tooltip />
        <el-table-column prop="platform" label="平台" width="100" />
        <el-table-column label="输入" width="90" align="right">
          <template #default="{ row }">{{ fmtNum(row.input_tokens) }}</template>
        </el-table-column>
        <el-table-column label="输出" width="90" align="right">
          <template #default="{ row }">{{ fmtNum(row.output_tokens) }}</template>
        </el-table-column>
        <el-table-column label="成本" width="80" align="right">
          <template #default="{ row }">${{ (row.cost_usd || 0).toFixed(4) }}</template>
        </el-table-column>
        <el-table-column label="Key" width="120" show-overflow-tooltip>
          <template #default="{ row }">{{ row.api_key_name || '-' }}</template>
        </el-table-column>
      </el-table>
      <div class="mt-4 flex justify-end">
        <el-pagination
          v-model:current-page="page"
          :page-size="pageSize"
          :total="total"
          layout="total, sizes, prev, pager, next"
          :page-sizes="[20, 50, 100]"
          @current-change="fetchRecords"
          @size-change="handlePageSizeChange"
        />
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted, nextTick } from 'vue'
import { getUsageRecords, getUsageTrends, getUsageByModel } from '@/utils/api'
import Chart from 'chart.js/auto'
import dayjs from 'dayjs'

const days = ref(30)
const granularity = ref('day')
const page = ref(1)
const pageSize = ref(50)
const total = ref(0)

const modelData = ref([])
const modelLoading = ref(false)
const trendData = ref([])
const trendLoading = ref(false)
const records = ref([])
const recordsLoading = ref(false)

const trendChartRef = ref(null)
const costChartRef = ref(null)
const modelPieRef = ref(null)
let trendChart = null
let costChart = null
let modelPie = null

const summaryRequests = computed(() => modelData.value.reduce((s, m) => s + (m.requests || 0), 0))
const summaryTokens = computed(() => modelData.value.reduce((s, m) => s + (m.input_tokens || 0) + (m.output_tokens || 0), 0))
const summaryCost = computed(() => modelData.value.reduce((s, m) => s + (m.cost_usd || 0), 0))

function fmtNum(n) {
  if (!n && n !== 0) return '0'
  if (n >= 1e9) return (n / 1e9).toFixed(2) + 'B'
  if (n >= 1e6) return (n / 1e6).toFixed(2) + 'M'
  if (n >= 1e3) return (n / 1e3).toFixed(1) + 'K'
  return String(n)
}

function isDark() { return document.documentElement.classList.contains('dark') }
function txColor() { return isDark() ? '#9CA3AF' : '#6B7280' }
function gridColor() { return isDark() ? 'rgba(75,85,99,0.3)' : 'rgba(229,231,235,0.8)' }
const COLORS = ['#3B82F6','#10B981','#F59E0B','#EF4444','#8B5CF6','#EC4899','#06B6D4','#F97316','#6366F1','#14B8A6','#84CC16','#D946EF']

async function fetchModels() {
  modelLoading.value = true
  try {
    const res = await getUsageByModel({ days: days.value })
    modelData.value = Array.isArray(res) ? res : (res?.data || [])
    await nextTick()
    renderModelPie()
  } finally { modelLoading.value = false }
}

async function fetchTrends() {
  trendLoading.value = true
  try {
    const res = await getUsageTrends({ days: days.value, granularity: granularity.value })
    trendData.value = Array.isArray(res) ? res : (res?.data || [])
    await nextTick()
    renderTrendChart()
    renderCostChart()
  } finally { trendLoading.value = false }
}

async function fetchRecords() {
  recordsLoading.value = true
  try {
    const res = await getUsageRecords({ days: days.value, page: page.value, page_size: pageSize.value })
    records.value = res?.records || res?.data || []
    total.value = res?.total || 0
  } finally { recordsLoading.value = false }
}

function handlePageSizeChange(size) { pageSize.value = size; page.value = 1; fetchRecords() }

function renderTrendChart() {
  if (!trendChartRef.value) return
  if (trendChart) trendChart.destroy()
  const labels = trendData.value.map((d) => d.date || d.period)
  trendChart = new Chart(trendChartRef.value, {
    type: 'line',
    data: {
      labels,
      datasets: [
        { label: '请求数', data: trendData.value.map((d) => d.requests || 0), borderColor: '#3B82F6', backgroundColor: 'rgba(59,130,246,0.1)', yAxisID: 'y', tension: 0.3, fill: true },
        { label: '输入 Token', data: trendData.value.map((d) => d.input_tokens || 0), borderColor: '#10B981', yAxisID: 'y1', tension: 0.3 },
        { label: '输出 Token', data: trendData.value.map((d) => d.output_tokens || 0), borderColor: '#F59E0B', yAxisID: 'y1', tension: 0.3 }
      ]
    },
    options: {
      responsive: true, maintainAspectRatio: false,
      interaction: { mode: 'index', intersect: false },
      plugins: { legend: { labels: { color: txColor(), usePointStyle: true, font: { size: 11 } } } },
      scales: {
        x: { ticks: { color: txColor(), font: { size: 10 } }, grid: { color: gridColor() } },
        y: { position: 'left', title: { display: true, text: '请求数', color: txColor() }, ticks: { color: txColor() }, grid: { color: gridColor() } },
        y1: { position: 'right', title: { display: true, text: 'Tokens', color: txColor() }, ticks: { color: txColor(), callback: (v) => fmtNum(v) }, grid: { display: false } }
      }
    }
  })
}

function renderCostChart() {
  if (!costChartRef.value) return
  if (costChart) costChart.destroy()
  const labels = trendData.value.map((d) => d.date || d.period)
  costChart = new Chart(costChartRef.value, {
    type: 'bar',
    data: {
      labels,
      datasets: [{ label: '成本 ($)', data: trendData.value.map((d) => d.cost_usd || 0), backgroundColor: 'rgba(16,185,129,0.6)', borderColor: '#10B981', borderWidth: 1 }]
    },
    options: {
      responsive: true, maintainAspectRatio: false,
      plugins: { legend: { labels: { color: txColor() } }, tooltip: { callbacks: { label(c) { return `$${c.parsed.y.toFixed(4)}` } } } },
      scales: {
        x: { ticks: { color: txColor(), font: { size: 10 } }, grid: { color: gridColor() } },
        y: { ticks: { color: txColor(), callback: (v) => '$' + v.toFixed(2) }, grid: { color: gridColor() } }
      }
    }
  })
}

function renderModelPie() {
  if (!modelPieRef.value) return
  if (modelPie) modelPie.destroy()
  const sorted = [...modelData.value].sort((a, b) => b.cost_usd - a.cost_usd).slice(0, 10)
  modelPie = new Chart(modelPieRef.value, {
    type: 'doughnut',
    data: { labels: sorted.map((m) => m.model), datasets: [{ data: sorted.map((m) => m.cost_usd), backgroundColor: COLORS.slice(0, sorted.length), borderColor: isDark() ? '#1F2937' : '#FFF', borderWidth: 2 }] },
    options: {
      responsive: true, maintainAspectRatio: false,
      plugins: { legend: { position: 'bottom', labels: { color: txColor(), font: { size: 10 }, usePointStyle: true, padding: 8 } }, tooltip: { callbacks: { label(c) { return `$${c.parsed.toFixed(4)}` } } } }
    }
  })
}

function fetchAll() { page.value = 1; fetchModels(); fetchTrends(); fetchRecords() }

onMounted(fetchAll)
onUnmounted(() => { trendChart?.destroy(); costChart?.destroy(); modelPie?.destroy() })
</script>

<style scoped>
.stat-card { @apply rounded-xl border border-gray-200 bg-white p-4 shadow-sm dark:border-gray-700 dark:bg-gray-800; }
.card { @apply rounded-xl border border-gray-200 bg-white shadow dark:border-gray-700 dark:bg-gray-800; }
</style>

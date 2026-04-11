<template>
  <div>
    <div class="mb-6 flex items-center justify-between">
      <h2 class="text-2xl font-bold text-gray-800 dark:text-gray-100">请求详情</h2>
      <div class="flex items-center gap-2">
        <el-tag type="info" size="small" effect="plain">{{ total }} 条记录</el-tag>
        <el-button size="small" @click="fetchData">
          <el-icon class="mr-1"><Refresh /></el-icon>刷新
        </el-button>
        <el-button size="small" @click="exportCSV" :disabled="!records.length">导出 CSV</el-button>
      </div>
    </div>

    <!-- Summary Cards -->
    <div class="mb-6 grid grid-cols-2 gap-4 lg:grid-cols-5">
      <div class="stat-card">
        <p class="stat-label">总请求</p>
        <p class="stat-value text-gray-900 dark:text-gray-100">{{ total }}</p>
      </div>
      <div class="stat-card">
        <p class="stat-label">成功率</p>
        <p class="stat-value" :class="successRate >= 95 ? 'text-green-600' : successRate >= 80 ? 'text-yellow-600' : 'text-red-600'">
          {{ successRate.toFixed(1) }}%
        </p>
      </div>
      <div class="stat-card">
        <p class="stat-label">平均耗时</p>
        <p class="stat-value text-blue-600">{{ avgDuration.toFixed(0) }} ms</p>
      </div>
      <div class="stat-card">
        <p class="stat-label">总 Token</p>
        <p class="stat-value text-purple-600">{{ fmtNum(totalTokens) }}</p>
      </div>
      <div class="stat-card">
        <p class="stat-label">总成本</p>
        <p class="stat-value text-green-600">${{ totalCost.toFixed(4) }}</p>
      </div>
    </div>

    <!-- Charts Row -->
    <div class="mb-6 grid grid-cols-1 gap-6 lg:grid-cols-2">
      <div class="card p-5">
        <h3 class="mb-2 text-sm font-semibold text-gray-900 dark:text-gray-100">状态码分布</h3>
        <div class="relative h-48"><canvas ref="statusChartRef" /></div>
      </div>
      <div class="card p-5">
        <h3 class="mb-2 text-sm font-semibold text-gray-900 dark:text-gray-100">平台分布</h3>
        <div class="relative h-48"><canvas ref="platformChartRef" /></div>
      </div>
    </div>

    <!-- Filters -->
    <div class="mb-4 flex flex-wrap gap-3">
      <el-input v-model="filters.search" placeholder="搜索模型 / Key" style="width: 200px" clearable @clear="fetchData" @keyup.enter="fetchData" />
      <el-select v-model="filters.platform" placeholder="平台" clearable style="width: 140px" @change="fetchData">
        <el-option v-for="p in platforms" :key="p" :value="p" :label="p" />
      </el-select>
      <el-select v-model="filters.status_code" placeholder="状态码" clearable style="width: 120px" @change="fetchData">
        <el-option label="2xx 成功" value="2xx" />
        <el-option label="4xx 客户端" value="4xx" />
        <el-option label="5xx 服务端" value="5xx" />
      </el-select>
      <el-date-picker v-model="filters.dateRange" type="daterange" range-separator="-" start-placeholder="开始" end-placeholder="结束" value-format="YYYY-MM-DD" @change="fetchData" />
      <el-button type="primary" size="default" @click="fetchData">搜索</el-button>
    </div>

    <!-- Table -->
    <div class="card">
      <el-table :data="records" v-loading="loading" stripe size="small" @row-click="toggleExpand" highlight-current-row>
        <el-table-column type="expand">
          <template #default="{ row }">
            <div class="bg-gray-50 p-4 dark:bg-gray-900">
              <div class="grid grid-cols-2 gap-x-8 gap-y-2 text-sm md:grid-cols-4">
                <div><span class="text-gray-400">请求 ID:</span> <span class="font-mono text-xs">{{ row.id }}</span></div>
                <div><span class="text-gray-400">API Key:</span> {{ row.api_key_name || row.api_key_id || '-' }}</div>
                <div><span class="text-gray-400">账户 ID:</span> {{ row.account_id || '-' }}</div>
                <div><span class="text-gray-400">平台:</span> {{ row.platform || '-' }}</div>
                <div><span class="text-gray-400">模型:</span> {{ row.model }}</div>
                <div><span class="text-gray-400">状态码:</span> <el-tag :type="statusTagType(row.status_code)" size="small">{{ row.status_code || '-' }}</el-tag></div>
                <div><span class="text-gray-400">输入 Token:</span> {{ fmtNum(row.input_tokens) }}</div>
                <div><span class="text-gray-400">输出 Token:</span> {{ fmtNum(row.output_tokens) }}</div>
                <div><span class="text-gray-400">耗时:</span> {{ row.duration_ms ?? '-' }} ms</div>
                <div><span class="text-gray-400">成本:</span> ${{ (row.cost_usd || 0).toFixed(6) }}</div>
                <div><span class="text-gray-400">流式:</span> {{ row.is_stream ? '是' : '否' }}</div>
                <div><span class="text-gray-400">时间:</span> {{ dayjs(row.created_at).format('YYYY-MM-DD HH:mm:ss') }}</div>
              </div>
              <div v-if="row.error_message" class="mt-2 rounded bg-red-50 p-2 text-xs text-red-600 dark:bg-red-900/20 dark:text-red-400">
                {{ row.error_message }}
              </div>
            </div>
          </template>
        </el-table-column>
        <el-table-column label="时间" width="140">
          <template #default="{ row }">{{ dayjs(row.created_at).format('MM-DD HH:mm:ss') }}</template>
        </el-table-column>
        <el-table-column prop="model" label="模型" min-width="160" show-overflow-tooltip />
        <el-table-column prop="platform" label="平台" width="110">
          <template #default="{ row }">
            <el-tag size="small" :type="platformTagType(row.platform)">{{ row.platform || '-' }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column label="状态" width="70" align="center">
          <template #default="{ row }">
            <el-tag :type="statusTagType(row.status_code)" size="small">{{ row.status_code || '-' }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column label="输入" width="80" align="right">
          <template #default="{ row }">{{ fmtNum(row.input_tokens) }}</template>
        </el-table-column>
        <el-table-column label="输出" width="80" align="right">
          <template #default="{ row }">{{ fmtNum(row.output_tokens) }}</template>
        </el-table-column>
        <el-table-column label="成本" width="80" align="right">
          <template #default="{ row }">${{ (row.cost_usd || 0).toFixed(4) }}</template>
        </el-table-column>
        <el-table-column label="耗时" width="80" align="right">
          <template #default="{ row }">
            <span :class="(row.duration_ms || 0) > 5000 ? 'text-red-500' : (row.duration_ms || 0) > 2000 ? 'text-yellow-500' : 'text-green-600'">
              {{ row.duration_ms ?? '-' }}
            </span>
          </template>
        </el-table-column>
        <el-table-column label="Key" width="100" show-overflow-tooltip>
          <template #default="{ row }">{{ row.api_key_name || '-' }}</template>
        </el-table-column>
      </el-table>

      <div class="flex items-center justify-between p-4">
        <span class="text-xs text-gray-400">点击行展开详情</span>
        <el-pagination
          v-model:current-page="page"
          :page-size="pageSize"
          :total="total"
          layout="total, sizes, prev, pager, next"
          :page-sizes="[20, 50, 100, 200]"
          @current-change="fetchData"
          @size-change="handleSizeChange"
        />
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted, nextTick } from 'vue'
import { Refresh } from '@element-plus/icons-vue'
import { getRequestDetails } from '@/utils/api'
import Chart from 'chart.js/auto'
import dayjs from 'dayjs'

const platforms = ['claude','claude-console','bedrock','ccr','gemini','gemini-api','openai','openai-responses','azure-openai','droid']
const COLORS = ['#3B82F6','#10B981','#F59E0B','#EF4444','#8B5CF6','#EC4899','#06B6D4','#F97316']

const records = ref([])
const loading = ref(false)
const page = ref(1)
const pageSize = ref(50)
const total = ref(0)
const filters = ref({ search: '', platform: '', status_code: '', dateRange: null })

const statusChartRef = ref(null)
const platformChartRef = ref(null)
let statusChart = null
let platformChart = null

function fmtNum(n) {
  if (!n && n !== 0) return '0'
  if (n >= 1e6) return (n / 1e6).toFixed(2) + 'M'
  if (n >= 1e3) return (n / 1e3).toFixed(1) + 'K'
  return String(n)
}

const successRate = computed(() => {
  if (!records.value.length) return 100
  const ok = records.value.filter((r) => r.status_code >= 200 && r.status_code < 300).length
  return (ok / records.value.length) * 100
})
const avgDuration = computed(() => {
  if (!records.value.length) return 0
  return records.value.reduce((s, r) => s + (r.duration_ms || 0), 0) / records.value.length
})
const totalTokens = computed(() => records.value.reduce((s, r) => s + (r.input_tokens || 0) + (r.output_tokens || 0), 0))
const totalCost = computed(() => records.value.reduce((s, r) => s + (r.cost_usd || 0), 0))

function statusTagType(code) {
  if (!code) return 'info'
  if (code >= 200 && code < 300) return 'success'
  if (code >= 400 && code < 500) return 'warning'
  return 'danger'
}
function platformTagType(p) {
  const m = { claude: '', gemini: 'success', openai: 'warning', bedrock: 'info' }
  return m[p] || ''
}
function toggleExpand() { /* el-table handles it */ }
function handleSizeChange(size) { pageSize.value = size; page.value = 1; fetchData() }

function isDark() { return document.documentElement.classList.contains('dark') }
function txColor() { return isDark() ? '#9CA3AF' : '#6B7280' }

function renderCharts() {
  if (!records.value.length) return
  // Status distribution
  const statusMap = {}
  records.value.forEach((r) => {
    const bucket = r.status_code ? `${Math.floor(r.status_code / 100)}xx` : 'N/A'
    statusMap[bucket] = (statusMap[bucket] || 0) + 1
  })
  if (statusChartRef.value) {
    if (statusChart) statusChart.destroy()
    statusChart = new Chart(statusChartRef.value, {
      type: 'doughnut',
      data: {
        labels: Object.keys(statusMap),
        datasets: [{ data: Object.values(statusMap), backgroundColor: ['#10B981','#F59E0B','#EF4444','#6B7280'], borderColor: isDark() ? '#1F2937' : '#FFF', borderWidth: 2 }]
      },
      options: { responsive: true, maintainAspectRatio: false, plugins: { legend: { position: 'right', labels: { color: txColor(), usePointStyle: true } } } }
    })
  }
  // Platform distribution
  const platMap = {}
  records.value.forEach((r) => { platMap[r.platform || 'unknown'] = (platMap[r.platform || 'unknown'] || 0) + 1 })
  if (platformChartRef.value) {
    if (platformChart) platformChart.destroy()
    const labels = Object.keys(platMap)
    platformChart = new Chart(platformChartRef.value, {
      type: 'bar',
      data: { labels, datasets: [{ label: '请求数', data: Object.values(platMap), backgroundColor: COLORS.slice(0, labels.length) }] },
      options: {
        responsive: true, maintainAspectRatio: false, indexAxis: 'y',
        plugins: { legend: { display: false } },
        scales: { x: { ticks: { color: txColor() }, grid: { color: isDark() ? 'rgba(75,85,99,0.3)' : 'rgba(229,231,235,0.8)' } }, y: { ticks: { color: txColor() } } }
      }
    })
  }
}

async function fetchData() {
  loading.value = true
  try {
    const params = { page: page.value, page_size: pageSize.value, search: filters.value.search || undefined, platform: filters.value.platform || undefined, status_code: filters.value.status_code || undefined }
    if (filters.value.dateRange) { params.start_date = filters.value.dateRange[0]; params.end_date = filters.value.dateRange[1] }
    const res = await getRequestDetails(params)
    if (Array.isArray(res)) { records.value = res; total.value = res.length }
    else { records.value = res.data || res.records || []; total.value = res.total || records.value.length }
    await nextTick()
    renderCharts()
  } finally { loading.value = false }
}

function exportCSV() {
  const header = 'time,model,platform,status,input_tokens,output_tokens,cost_usd,duration_ms\n'
  const rows = records.value.map((r) => [dayjs(r.created_at).format('YYYY-MM-DD HH:mm:ss'), r.model, r.platform, r.status_code, r.input_tokens, r.output_tokens, (r.cost_usd || 0).toFixed(6), r.duration_ms].join(',')).join('\n')
  const blob = new Blob([header + rows], { type: 'text/csv' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a'); a.href = url; a.download = `requests-${dayjs().format('YYYYMMDD-HHmm')}.csv`; a.click()
  URL.revokeObjectURL(url)
}

onMounted(fetchData)
onUnmounted(() => { statusChart?.destroy(); platformChart?.destroy() })
</script>

<style scoped>
.stat-card { @apply rounded-xl border border-gray-200 bg-white p-4 shadow-sm dark:border-gray-700 dark:bg-gray-800; }
.stat-label { @apply text-xs font-semibold text-gray-500 dark:text-gray-400; }
.stat-value { @apply text-2xl font-bold; }
.card { @apply rounded-xl border border-gray-200 bg-white shadow dark:border-gray-700 dark:bg-gray-800; }
</style>

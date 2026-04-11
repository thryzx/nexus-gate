<template>
  <div>
    <div class="mb-6 flex items-center justify-between">
      <h2 class="text-2xl font-bold text-gray-800 dark:text-gray-100">仪表盘</h2>
      <div class="flex items-center gap-3">
        <el-switch v-model="autoRefresh" active-text="自动刷新" size="small" @change="toggleAutoRefresh" />
        <span v-if="autoRefresh" class="text-xs text-gray-400">{{ countdown }}s</span>
        <el-button size="small" :loading="loading" @click="refreshAll">
          <el-icon class="mr-1"><Refresh /></el-icon>刷新
        </el-button>
      </div>
    </div>

    <div v-if="loading && !data" class="flex justify-center py-20">
      <el-icon class="is-loading text-4xl text-blue-500"><Loading /></el-icon>
    </div>

    <template v-else-if="data">
      <!-- 主要统计卡片 -->
      <div class="mb-6 grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <div class="stat-card">
          <div class="flex items-center justify-between">
            <div>
              <p class="mb-1 text-xs font-semibold text-gray-600 dark:text-gray-400">总 API Keys</p>
              <p class="text-3xl font-bold text-gray-900 dark:text-gray-100">{{ data.api_keys.total }}</p>
              <p class="mt-1 text-xs text-gray-500 dark:text-gray-400">活跃: {{ data.api_keys.active }}</p>
            </div>
            <div class="flex h-12 w-12 items-center justify-center rounded-xl bg-gradient-to-br from-blue-500 to-blue-600 text-white shadow-lg">
              <el-icon :size="22"><Key /></el-icon>
            </div>
          </div>
        </div>
        <div class="stat-card">
          <div class="flex items-center justify-between">
            <div class="flex-1">
              <p class="mb-1 text-xs font-semibold text-gray-600 dark:text-gray-400">服务账户</p>
              <div class="flex items-baseline gap-2">
                <p class="text-3xl font-bold text-gray-900 dark:text-gray-100">{{ data.accounts.total }}</p>
                <div class="flex flex-wrap items-center gap-1.5">
                  <span v-for="p in data.accounts.by_platform" :key="p.platform" class="inline-flex items-center gap-0.5 text-xs" :title="`${platformLabel(p.platform)}: ${p.count}`">
                    <span :class="platformColor(p.platform)">{{ platformIcon(p.platform) }}</span>
                    <span class="font-medium text-gray-600 dark:text-gray-300">{{ p.count }}</span>
                  </span>
                </div>
              </div>
              <p class="mt-1 text-xs text-gray-500 dark:text-gray-400">活跃: {{ data.accounts.active }}</p>
            </div>
            <div class="ml-2 flex h-12 w-12 items-center justify-center rounded-xl bg-gradient-to-br from-green-500 to-green-600 text-white shadow-lg">
              <el-icon :size="22"><User /></el-icon>
            </div>
          </div>
        </div>
        <div class="stat-card">
          <div class="flex items-center justify-between">
            <div>
              <p class="mb-1 text-xs font-semibold text-gray-600 dark:text-gray-400">今日请求</p>
              <p class="text-3xl font-bold text-gray-900 dark:text-gray-100">{{ fmtNum(data.usage.today_requests) }}</p>
              <p class="mt-1 text-xs text-gray-500 dark:text-gray-400">总请求: {{ fmtNum(data.usage.total_requests) }}</p>
            </div>
            <div class="flex h-12 w-12 items-center justify-center rounded-xl bg-gradient-to-br from-purple-500 to-purple-600 text-white shadow-lg">
              <el-icon :size="22"><DataLine /></el-icon>
            </div>
          </div>
        </div>
        <div class="stat-card">
          <div class="flex items-center justify-between">
            <div>
              <p class="mb-1 text-xs font-semibold text-gray-600 dark:text-gray-400">总成本</p>
              <p class="text-3xl font-bold text-gray-900 dark:text-gray-100">${{ data.usage.total_cost_usd.toFixed(2) }}</p>
              <p class="mt-1 text-xs text-gray-500 dark:text-gray-400">今日: ${{ data.usage.today_cost_usd.toFixed(4) }}</p>
            </div>
            <div class="flex h-12 w-12 items-center justify-center rounded-xl bg-gradient-to-br from-orange-500 to-yellow-500 text-white shadow-lg">
              <el-icon :size="22"><Coin /></el-icon>
            </div>
          </div>
        </div>
      </div>

      <!-- Token 统计行 -->
      <div class="mb-6 grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <div class="stat-card">
          <p class="mb-1 text-xs font-semibold text-gray-600 dark:text-gray-400">今日 Token</p>
          <p class="text-2xl font-bold text-blue-600">{{ fmtNum((data.usage.today_input_tokens || 0) + (data.usage.today_output_tokens || 0)) }}</p>
          <div class="mt-2 space-y-1 text-xs text-gray-500 dark:text-gray-400">
            <div class="flex justify-between"><span>输入</span><span class="font-medium">{{ fmtNum(data.usage.today_input_tokens) }}</span></div>
            <div class="flex justify-between"><span>输出</span><span class="font-medium">{{ fmtNum(data.usage.today_output_tokens) }}</span></div>
          </div>
        </div>
        <div class="stat-card">
          <p class="mb-1 text-xs font-semibold text-gray-600 dark:text-gray-400">总 Token 消耗</p>
          <p class="text-2xl font-bold text-emerald-600">{{ fmtNum(data.usage.total_input_tokens + data.usage.total_output_tokens) }}</p>
          <div class="mt-2 space-y-1 text-xs text-gray-500 dark:text-gray-400">
            <div class="flex justify-between"><span>输入</span><span class="font-medium">{{ fmtNum(data.usage.total_input_tokens) }}</span></div>
            <div class="flex justify-between"><span>输出</span><span class="font-medium">{{ fmtNum(data.usage.total_output_tokens) }}</span></div>
          </div>
        </div>
        <div class="stat-card">
          <p class="mb-1 text-xs font-semibold text-gray-600 dark:text-gray-400">今日成本</p>
          <p class="text-2xl font-bold text-indigo-600">${{ data.usage.today_cost_usd.toFixed(4) }}</p>
          <div class="mt-2 text-xs text-gray-500 dark:text-gray-400">
            <div class="flex justify-between">
              <span>均价/请求</span>
              <span class="font-medium">${{ data.usage.today_requests > 0 ? (data.usage.today_cost_usd / data.usage.today_requests).toFixed(6) : '0' }}</span>
            </div>
          </div>
        </div>
        <div class="stat-card">
          <p class="mb-1 text-xs font-semibold text-gray-600 dark:text-gray-400">平台分布</p>
          <div class="mt-2 space-y-1.5">
            <div v-for="p in data.accounts.by_platform" :key="p.platform" class="flex items-center justify-between text-xs">
              <span class="capitalize text-gray-600 dark:text-gray-300">{{ platformLabel(p.platform) }}</span>
              <el-tag size="small" :type="platformTagType(p.platform)">{{ p.count }}</el-tag>
            </div>
            <div v-if="!data.accounts.by_platform?.length" class="text-xs text-gray-400">暂无账户</div>
          </div>
        </div>
      </div>

      <!-- 图表 -->
      <div class="mb-6 grid grid-cols-1 gap-6 lg:grid-cols-2">
        <div class="rounded-xl border border-gray-200 bg-white p-6 shadow dark:border-gray-700 dark:bg-gray-800">
          <div class="mb-4 flex items-center justify-between">
            <h3 class="text-sm font-semibold text-gray-900 dark:text-gray-100">用量趋势</h3>
            <el-radio-group v-model="trendDays" size="small" @change="loadTrends">
              <el-radio-button :value="7">7天</el-radio-button>
              <el-radio-button :value="14">14天</el-radio-button>
              <el-radio-button :value="30">30天</el-radio-button>
            </el-radio-group>
          </div>
          <div class="relative h-72"><canvas ref="trendChartRef" /></div>
        </div>
        <div class="rounded-xl border border-gray-200 bg-white p-6 shadow dark:border-gray-700 dark:bg-gray-800">
          <div class="mb-4 flex items-center justify-between">
            <h3 class="text-sm font-semibold text-gray-900 dark:text-gray-100">模型消耗分布</h3>
            <el-radio-group v-model="modelPeriod" size="small" @change="loadModelStats">
              <el-radio-button value="today">今日</el-radio-button>
              <el-radio-button value="7d">7天</el-radio-button>
              <el-radio-button value="30d">30天</el-radio-button>
              <el-radio-button value="all">全部</el-radio-button>
            </el-radio-group>
          </div>
          <div class="relative h-72"><canvas ref="modelChartRef" /></div>
        </div>
      </div>

      <!-- 模型消耗明细 -->
      <div class="rounded-xl border border-gray-200 bg-white p-6 shadow dark:border-gray-700 dark:bg-gray-800">
        <h3 class="mb-4 text-sm font-semibold text-gray-900 dark:text-gray-100">模型消耗明细</h3>
        <el-table :data="modelStatsData" stripe size="small">
          <el-table-column prop="model" label="模型" min-width="180" />
          <el-table-column label="请求数" width="100" align="right">
            <template #default="{ row }">{{ fmtNum(row.requests) }}</template>
          </el-table-column>
          <el-table-column label="输入 Token" width="120" align="right">
            <template #default="{ row }">{{ fmtNum(row.input_tokens) }}</template>
          </el-table-column>
          <el-table-column label="输出 Token" width="120" align="right">
            <template #default="{ row }">{{ fmtNum(row.output_tokens) }}</template>
          </el-table-column>
          <el-table-column label="成本 (USD)" width="120" align="right">
            <template #default="{ row }">${{ row.cost_usd.toFixed(4) }}</template>
          </el-table-column>
          <el-table-column label="占比" width="80" align="right">
            <template #default="{ row }">{{ totalModelCost > 0 ? ((row.cost_usd / totalModelCost) * 100).toFixed(1) : '0' }}%</template>
          </el-table-column>
        </el-table>
      </div>
    </template>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted, watch, computed, nextTick } from 'vue'
import { storeToRefs } from 'pinia'
import { useDashboardStore } from '@/stores/dashboard'
import { useThemeStore } from '@/stores/theme'
import { getUsageTrends, getUsageByModel } from '@/utils/api'
import { Loading, Refresh, Key, User, DataLine, Coin } from '@element-plus/icons-vue'
import Chart from 'chart.js/auto'

const store = useDashboardStore()
const themeStore = useThemeStore()
const { data, loading } = storeToRefs(store)

const trendChartRef = ref(null)
const modelChartRef = ref(null)
let trendChart = null
let modelChart = null

const autoRefresh = ref(false)
const countdown = ref(30)
let countdownTimer = null
const trendDays = ref(7)
const modelPeriod = ref('7d')
const trendData = ref([])
const modelStatsData = ref([])

const totalModelCost = computed(() => modelStatsData.value.reduce((s, m) => s + (m.cost_usd || 0), 0))

const PLATFORMS = {
  claude: { label: 'Claude', icon: '🧠', color: 'text-indigo-600', tag: '' },
  'claude-console': { label: 'Console', icon: '💻', color: 'text-purple-600', tag: 'warning' },
  bedrock: { label: 'Bedrock', icon: '☁️', color: 'text-orange-600', tag: 'danger' },
  gemini: { label: 'Gemini', icon: '✨', color: 'text-yellow-600', tag: 'warning' },
  'gemini-api': { label: 'Gemini API', icon: '🔑', color: 'text-amber-600', tag: 'warning' },
  openai: { label: 'OpenAI', icon: '🤖', color: 'text-green-600', tag: 'success' },
  'openai-responses': { label: 'OpenAI Resp', icon: '📡', color: 'text-cyan-600', tag: 'info' },
  'azure-openai': { label: 'Azure', icon: '☁️', color: 'text-blue-600', tag: '' },
  droid: { label: 'Droid', icon: '🤖', color: 'text-pink-600', tag: 'danger' },
  ccr: { label: 'CCR', icon: '🔗', color: 'text-gray-600', tag: 'info' }
}
function platformLabel(p) { return PLATFORMS[p]?.label || p }
function platformIcon(p) { return PLATFORMS[p]?.icon || '📦' }
function platformColor(p) { return PLATFORMS[p]?.color || 'text-gray-500' }
function platformTagType(p) { return PLATFORMS[p]?.tag || '' }

function fmtNum(n) {
  if (!n && n !== 0) return '0'
  if (n >= 1e9) return (n / 1e9).toFixed(2) + 'B'
  if (n >= 1e6) return (n / 1e6).toFixed(2) + 'M'
  if (n >= 1e3) return (n / 1e3).toFixed(1) + 'K'
  return String(n)
}

const COLORS = ['#3B82F6','#10B981','#F59E0B','#EF4444','#8B5CF6','#EC4899','#06B6D4','#F97316','#6366F1','#14B8A6','#84CC16','#D946EF','#0EA5E9','#FB923C','#A3E635']
function isDark() { return document.documentElement.classList.contains('dark') }
function txColor() { return isDark() ? '#9CA3AF' : '#6B7280' }
function gridColor() { return isDark() ? 'rgba(75,85,99,0.3)' : 'rgba(229,231,235,0.8)' }

async function loadTrends() {
  try {
    const res = await getUsageTrends({ days: trendDays.value, granularity: 'day' })
    trendData.value = Array.isArray(res) ? res : (res?.data || [])
    await nextTick()
    renderTrendChart()
  } catch (_e) { /* ignore */ }
}

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
        { label: '输入 Token', data: trendData.value.map((d) => d.input_tokens || 0), borderColor: '#10B981', yAxisID: 'y1', tension: 0.3, borderDash: [5, 5] },
        { label: '输出 Token', data: trendData.value.map((d) => d.output_tokens || 0), borderColor: '#F59E0B', yAxisID: 'y1', tension: 0.3, borderDash: [5, 5] },
        { label: '成本 ($)', data: trendData.value.map((d) => d.cost_usd || 0), borderColor: '#EF4444', yAxisID: 'y2', tension: 0.3, borderDash: [2, 2] }
      ]
    },
    options: {
      responsive: true, maintainAspectRatio: false,
      interaction: { mode: 'index', intersect: false },
      plugins: {
        legend: { position: 'top', labels: { color: txColor(), usePointStyle: true, padding: 12, font: { size: 11 } } },
        tooltip: { callbacks: { label(c) { const l = c.dataset.label, v = c.parsed.y; if (l.includes('成本')) return `${l}: $${v.toFixed(4)}`; if (l.includes('Token')) return `${l}: ${fmtNum(v)}`; return `${l}: ${v}` } } }
      },
      scales: {
        x: { ticks: { color: txColor(), font: { size: 10 } }, grid: { color: gridColor() } },
        y: { type: 'linear', position: 'left', title: { display: true, text: '请求数', color: txColor() }, ticks: { color: txColor() }, grid: { color: gridColor() } },
        y1: { type: 'linear', position: 'right', title: { display: true, text: 'Tokens', color: txColor() }, ticks: { color: txColor(), callback: (v) => fmtNum(v) }, grid: { display: false } },
        y2: { type: 'linear', position: 'right', display: false }
      }
    }
  })
}

async function loadModelStats() {
  try {
    const days = { today: 1, '7d': 7, '30d': 30, all: 365 }[modelPeriod.value] || 7
    const res = await getUsageByModel({ days })
    modelStatsData.value = Array.isArray(res) ? res : (res?.data || [])
    await nextTick()
    renderModelChart()
  } catch (_e) { /* ignore */ }
}

function renderModelChart() {
  if (!modelChartRef.value) return
  if (modelChart) modelChart.destroy()
  const sorted = [...modelStatsData.value].sort((a, b) => b.cost_usd - a.cost_usd)
  const top = sorted.slice(0, 10)
  const otherCost = sorted.slice(10).reduce((s, m) => s + m.cost_usd, 0)
  const labels = top.map((m) => m.model)
  const costs = top.map((m) => m.cost_usd)
  if (otherCost > 0) { labels.push('其他'); costs.push(otherCost) }
  modelChart = new Chart(modelChartRef.value, {
    type: 'doughnut',
    data: { labels, datasets: [{ data: costs, backgroundColor: COLORS.slice(0, labels.length), borderColor: isDark() ? '#1F2937' : '#FFF', borderWidth: 2 }] },
    options: {
      responsive: true, maintainAspectRatio: false,
      plugins: {
        legend: { position: 'right', labels: { color: txColor(), padding: 10, font: { size: 11 }, usePointStyle: true } },
        tooltip: { callbacks: { label(c) { const v = c.parsed, t = c.dataset.data.reduce((s, x) => s + x, 0); return `${c.label}: $${v.toFixed(4)} (${t > 0 ? ((v / t) * 100).toFixed(1) : '0'}%)` } } }
      }
    }
  })
}

function toggleAutoRefresh(v) { v ? startAuto() : stopAuto() }
function startAuto() { countdown.value = 30; countdownTimer = setInterval(() => { countdown.value--; if (countdown.value <= 0) { countdown.value = 30; refreshAll() } }, 1000) }
function stopAuto() { clearInterval(countdownTimer); countdownTimer = null }
async function refreshAll() { await store.fetch(); await Promise.all([loadTrends(), loadModelStats()]) }

watch(() => themeStore.isDark, () => { renderTrendChart(); renderModelChart() })
onMounted(async () => { await store.fetch(); await Promise.all([loadTrends(), loadModelStats()]) })
onUnmounted(() => { stopAuto(); trendChart?.destroy(); modelChart?.destroy() })
</script>

<style scoped>
.stat-card { @apply rounded-xl border border-gray-200 bg-white p-5 shadow-sm transition-all duration-200 hover:shadow-md dark:border-gray-700 dark:bg-gray-800; }
</style>

<template>
  <div>
    <h2 class="text-2xl font-bold text-gray-800 dark:text-gray-100 mb-6">用量统计</h2>

    <!-- Filters -->
    <div class="flex gap-4 mb-6 flex-wrap">
      <el-select v-model="days" @change="fetchAll" class="w-32">
        <el-option :value="1" label="今天" />
        <el-option :value="7" label="近 7 天" />
        <el-option :value="30" label="近 30 天" />
        <el-option :value="90" label="近 90 天" />
      </el-select>
    </div>

    <!-- Model Usage -->
    <div class="bg-white dark:bg-gray-800 rounded-xl shadow border border-gray-200 dark:border-gray-700 p-6 mb-6">
      <h3 class="text-lg font-semibold text-gray-700 dark:text-gray-200 mb-4">模型用量分布</h3>
      <el-table :data="modelData" v-loading="modelLoading" stripe size="small">
        <el-table-column prop="model" label="模型" min-width="200" />
        <el-table-column prop="requests" label="请求数" width="100" sortable />
        <el-table-column label="输入 Tokens" width="120" sortable>
          <template #default="{ row }">{{ formatNum(row.input_tokens) }}</template>
        </el-table-column>
        <el-table-column label="输出 Tokens" width="120" sortable>
          <template #default="{ row }">{{ formatNum(row.output_tokens) }}</template>
        </el-table-column>
        <el-table-column label="费用 (USD)" width="120" sortable>
          <template #default="{ row }">${{ row.cost_usd.toFixed(4) }}</template>
        </el-table-column>
      </el-table>
    </div>

    <!-- Trend -->
    <div class="bg-white dark:bg-gray-800 rounded-xl shadow border border-gray-200 dark:border-gray-700 p-6 mb-6">
      <h3 class="text-lg font-semibold text-gray-700 dark:text-gray-200 mb-4">每日趋势</h3>
      <el-table :data="trendData" v-loading="trendLoading" stripe size="small">
        <el-table-column prop="date" label="日期" width="120" />
        <el-table-column prop="requests" label="请求数" width="100" />
        <el-table-column label="输入 Tokens" width="120">
          <template #default="{ row }">{{ formatNum(row.input_tokens) }}</template>
        </el-table-column>
        <el-table-column label="输出 Tokens" width="120">
          <template #default="{ row }">{{ formatNum(row.output_tokens) }}</template>
        </el-table-column>
        <el-table-column label="费用 (USD)" width="120">
          <template #default="{ row }">${{ row.cost_usd.toFixed(4) }}</template>
        </el-table-column>
      </el-table>
    </div>

    <!-- Records -->
    <div class="bg-white dark:bg-gray-800 rounded-xl shadow border border-gray-200 dark:border-gray-700 p-6">
      <h3 class="text-lg font-semibold text-gray-700 dark:text-gray-200 mb-4">请求记录</h3>
      <el-table :data="records" v-loading="recordsLoading" stripe size="small">
        <el-table-column label="时间" width="160">
          <template #default="{ row }">{{ dayjs(row.created_at).format('MM-DD HH:mm:ss') }}</template>
        </el-table-column>
        <el-table-column prop="model" label="模型" min-width="180" show-overflow-tooltip />
        <el-table-column label="输入" width="90">
          <template #default="{ row }">{{ formatNum(row.input_tokens) }}</template>
        </el-table-column>
        <el-table-column label="输出" width="90">
          <template #default="{ row }">{{ formatNum(row.output_tokens) }}</template>
        </el-table-column>
        <el-table-column label="费用" width="80">
          <template #default="{ row }">${{ row.cost_usd.toFixed(4) }}</template>
        </el-table-column>
      </el-table>
      <div class="mt-4 flex justify-end">
        <el-pagination
          v-model:current-page="page"
          :page-size="pageSize"
          :total="total"
          layout="total, prev, pager, next"
          @current-change="fetchRecords"
        />
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { getUsageRecords, getUsageTrends, getUsageByModel } from '@/utils/api'
import dayjs from 'dayjs'

const days = ref(30)
const page = ref(1)
const pageSize = 50
const total = ref(0)

const modelData = ref([])
const modelLoading = ref(false)
const trendData = ref([])
const trendLoading = ref(false)
const records = ref([])
const recordsLoading = ref(false)

function formatNum(n) {
  if (n >= 1e6) return (n / 1e6).toFixed(1) + 'M'
  if (n >= 1e3) return (n / 1e3).toFixed(1) + 'K'
  return String(n)
}

async function fetchModels() {
  modelLoading.value = true
  try { modelData.value = await getUsageByModel({ days: days.value }) } finally { modelLoading.value = false }
}

async function fetchTrends() {
  trendLoading.value = true
  try { trendData.value = await getUsageTrends({ days: days.value }) } finally { trendLoading.value = false }
}

async function fetchRecords() {
  recordsLoading.value = true
  try {
    const res = await getUsageRecords({ days: days.value, page: page.value, page_size: pageSize })
    records.value = res.records
    total.value = res.total
  } finally { recordsLoading.value = false }
}

function fetchAll() {
  page.value = 1
  fetchModels()
  fetchTrends()
  fetchRecords()
}

onMounted(fetchAll)
</script>

<template>
  <div>
    <div class="flex justify-between items-center mb-6">
      <h2 class="text-2xl font-bold text-gray-800 dark:text-gray-100">请求详情</h2>
      <el-button size="small" @click="fetchData">
        <el-icon class="mr-1"><Refresh /></el-icon>刷新
      </el-button>
    </div>

    <!-- Filters -->
    <div class="mb-4 flex gap-3 flex-wrap">
      <el-input v-model="filters.search" placeholder="搜索模型 / Key" style="width: 200px" clearable @change="fetchData" />
      <el-select v-model="filters.platform" placeholder="平台" clearable style="width: 140px" @change="fetchData">
        <el-option v-for="p in platforms" :key="p" :value="p" :label="p" />
      </el-select>
      <el-date-picker v-model="filters.dateRange" type="daterange" range-separator="-" start-placeholder="开始" end-placeholder="结束" @change="fetchData" />
    </div>

    <div class="bg-white dark:bg-gray-800 rounded-xl shadow border border-gray-200 dark:border-gray-700">
      <el-table :data="records" v-loading="loading" stripe>
        <el-table-column prop="id" label="ID" width="100" show-overflow-tooltip />
        <el-table-column prop="model" label="模型" min-width="160" show-overflow-tooltip />
        <el-table-column prop="platform" label="平台" width="120">
          <template #default="{ row }">
            <el-tag size="small">{{ row.platform }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="status_code" label="状态码" width="80" />
        <el-table-column prop="input_tokens" label="输入" width="80" />
        <el-table-column prop="output_tokens" label="输出" width="80" />
        <el-table-column prop="duration_ms" label="耗时(ms)" width="90" />
        <el-table-column prop="created_at" label="时间" width="180" />
      </el-table>

      <div class="p-4 flex justify-center">
        <el-pagination
          v-model:current-page="page"
          :page-size="pageSize"
          :total="total"
          layout="prev, pager, next, total"
          @current-change="fetchData"
        />
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { Refresh } from '@element-plus/icons-vue'
import { getRequestDetails } from '@/utils/api'

const platforms = [
  'claude', 'claude-console', 'bedrock', 'ccr',
  'gemini', 'gemini-api', 'openai', 'openai-responses',
  'azure-openai', 'droid'
]

const records = ref([])
const loading = ref(false)
const page = ref(1)
const pageSize = ref(50)
const total = ref(0)
const filters = ref({ search: '', platform: '', dateRange: null })

async function fetchData() {
  loading.value = true
  try {
    const params = {
      page: page.value,
      page_size: pageSize.value,
      search: filters.value.search || undefined,
      platform: filters.value.platform || undefined
    }
    if (filters.value.dateRange) {
      params.start_date = filters.value.dateRange[0]?.toISOString()
      params.end_date = filters.value.dateRange[1]?.toISOString()
    }
    const res = await getRequestDetails(params)
    if (Array.isArray(res)) {
      records.value = res
      total.value = res.length
    } else {
      records.value = res.data || res.records || []
      total.value = res.total || records.value.length
    }
  } finally {
    loading.value = false
  }
}

onMounted(fetchData)
</script>

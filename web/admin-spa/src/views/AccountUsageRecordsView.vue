<template>
  <div>
    <div class="flex items-center gap-3 mb-6">
      <el-button text @click="$router.back()">
        <el-icon><ArrowLeft /></el-icon>
      </el-button>
      <h2 class="text-2xl font-bold text-gray-800 dark:text-gray-100">
        账户用量记录
      </h2>
    </div>

    <!-- Filters -->
    <div class="mb-4 flex gap-3 flex-wrap">
      <el-date-picker v-model="dateRange" type="daterange" range-separator="-" start-placeholder="开始" end-placeholder="结束" @change="fetchRecords" />
      <el-select v-model="modelFilter" placeholder="模型" clearable style="width: 200px" @change="fetchRecords">
        <el-option v-for="m in models" :key="m" :value="m" :label="m" />
      </el-select>
    </div>

    <div class="bg-white dark:bg-gray-800 rounded-xl shadow border border-gray-200 dark:border-gray-700">
      <el-table :data="records" v-loading="loading" stripe>
        <el-table-column prop="model" label="模型" min-width="160" />
        <el-table-column prop="input_tokens" label="输入 Tokens" width="120" />
        <el-table-column prop="output_tokens" label="输出 Tokens" width="120" />
        <el-table-column prop="cost_usd" label="成本 (USD)" width="120">
          <template #default="{ row }">${{ (row.cost_usd || 0).toFixed(4) }}</template>
        </el-table-column>
        <el-table-column prop="created_at" label="时间" width="180" />
      </el-table>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { ArrowLeft } from '@element-plus/icons-vue'
import { getAccountUsageRecords } from '@/utils/api'

const route = useRoute()
const accountId = route.params.accountId
const records = ref([])
const loading = ref(false)
const dateRange = ref(null)
const modelFilter = ref('')
const models = ref([])

async function fetchRecords() {
  loading.value = true
  try {
    const params = { model: modelFilter.value || undefined }
    if (dateRange.value) {
      params.start_date = dateRange.value[0]?.toISOString()
      params.end_date = dateRange.value[1]?.toISOString()
    }
    const res = await getAccountUsageRecords(accountId, params)
    records.value = Array.isArray(res) ? res : (res.records || [])
  } finally {
    loading.value = false
  }
}

onMounted(fetchRecords)
</script>

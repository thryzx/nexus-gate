<template>
  <div>
    <div class="flex justify-between items-center mb-6">
      <h2 class="text-2xl font-bold text-gray-800 dark:text-gray-100">额度卡管理</h2>
      <el-button type="primary" @click="showCreate = true">
        <el-icon class="mr-1"><Plus /></el-icon>创建额度卡
      </el-button>
    </div>

    <!-- Stats -->
    <div class="grid grid-cols-1 md:grid-cols-3 gap-6 mb-6" v-if="stats">
      <StatCard title="总数" :value="stats.total || 0" color="blue" />
      <StatCard title="已使用" :value="stats.used || 0" color="green" />
      <StatCard title="未使用" :value="stats.unused || 0" color="purple" />
    </div>

    <!-- Table -->
    <div class="bg-white dark:bg-gray-800 rounded-xl shadow border border-gray-200 dark:border-gray-700">
      <el-table :data="cards" v-loading="loading" stripe>
        <el-table-column prop="code" label="卡号" min-width="200" show-overflow-tooltip />
        <el-table-column prop="value" label="额度" width="100" />
        <el-table-column prop="status" label="状态" width="100">
          <template #default="{ row }">
            <el-tag :type="row.used_at ? 'success' : 'info'" size="small">
              {{ row.used_at ? '已使用' : '未使用' }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="used_by" label="使用者" min-width="120" show-overflow-tooltip />
        <el-table-column prop="created_at" label="创建时间" width="180" />
        <el-table-column label="操作" width="100">
          <template #default="{ row }">
            <el-popconfirm title="确认删除？" @confirm="handleDelete(row.id)">
              <template #reference>
                <el-button size="small" text type="danger">删除</el-button>
              </template>
            </el-popconfirm>
          </template>
        </el-table-column>
      </el-table>
    </div>

    <!-- Create Dialog -->
    <el-dialog v-model="showCreate" title="创建额度卡" width="500px" destroy-on-close>
      <el-form :model="form" label-width="100px">
        <el-form-item label="单卡额度" required>
          <el-input-number v-model="form.value" :min="1" :max="100000" />
        </el-form-item>
        <el-form-item label="数量" required>
          <el-input-number v-model="form.count" :min="1" :max="100" />
        </el-form-item>
        <el-form-item label="有效期 (天)">
          <el-input-number v-model="form.validity_days" :min="1" :max="365" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showCreate = false">取消</el-button>
        <el-button type="primary" :loading="saving" @click="handleCreate">创建</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { Plus } from '@element-plus/icons-vue'
import { getQuotaCards, createQuotaCard, deleteQuotaCard, getQuotaCardStats } from '@/utils/api'
import { ElMessage } from 'element-plus'
import StatCard from '@/components/common/StatCard.vue'

const cards = ref([])
const stats = ref(null)
const loading = ref(false)
const saving = ref(false)
const showCreate = ref(false)
const form = ref({ value: 100, count: 1, validity_days: 30 })

async function fetchCards() {
  loading.value = true
  try {
    cards.value = await getQuotaCards()
  } finally {
    loading.value = false
  }
}

async function fetchStats() {
  try {
    stats.value = await getQuotaCardStats()
  } catch { /* ignore */ }
}

async function handleCreate() {
  saving.value = true
  try {
    await createQuotaCard(form.value)
    ElMessage.success('创建成功')
    showCreate.value = false
    await fetchCards()
    await fetchStats()
  } catch (e) {
    ElMessage.error(e?.response?.data?.error || '创建失败')
  } finally {
    saving.value = false
  }
}

async function handleDelete(id) {
  await deleteQuotaCard(id)
  ElMessage.success('已删除')
  await fetchCards()
  await fetchStats()
}

onMounted(() => {
  fetchCards()
  fetchStats()
})
</script>

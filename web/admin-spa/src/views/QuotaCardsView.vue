<template>
  <div>
    <div class="flex justify-between items-center mb-6">
      <h2 class="text-2xl font-bold text-gray-800 dark:text-gray-100">额度卡管理</h2>
      <div class="flex gap-2">
        <el-button @click="exportCards" :disabled="cards.length === 0">
          <el-icon class="mr-1"><Download /></el-icon>导出
        </el-button>
        <el-button type="primary" @click="showCreate = true">
          <el-icon class="mr-1"><Plus /></el-icon>创建额度卡
        </el-button>
      </div>
    </div>

    <!-- Stats -->
    <div class="grid grid-cols-1 md:grid-cols-3 gap-4 mb-6" v-if="stats">
      <StatCard title="总数" :value="stats.total || 0" color="blue" />
      <StatCard title="激活" :value="stats.active || 0" color="green" />
      <StatCard title="已兑换" :value="stats.total_redemptions || 0" color="purple" />
    </div>

    <!-- Tabs -->
    <el-tabs v-model="activeTab" type="border-card">
      <el-tab-pane name="cards">
        <template #label>
          <span>额度卡 <el-badge :value="cards.length" class="ml-1" /></span>
        </template>

        <!-- Filters -->
        <div class="flex gap-3 mb-4 flex-wrap">
          <el-input
            v-model="searchCode"
            placeholder="搜索卡号"
            clearable
            style="width: 200px"
            :prefix-icon="Search"
          />
          <el-select v-model="statusFilter" placeholder="状态" clearable style="width: 120px">
            <el-option label="未使用" value="unused" />
            <el-option label="已使用" value="used" />
            <el-option label="已撤销" value="revoked" />
          </el-select>
        </div>

        <div class="bg-white dark:bg-gray-800 rounded-xl shadow border border-gray-200 dark:border-gray-700">
          <el-table :data="filteredCards" v-loading="loading" stripe>
            <el-table-column prop="code" label="卡号" min-width="200" show-overflow-tooltip>
              <template #default="{ row }">
                <span class="font-mono text-xs">{{ row.code }}</span>
              </template>
            </el-table-column>
            <el-table-column prop="value" label="额度" width="100">
              <template #default="{ row }">
                <span class="font-semibold text-green-600">${{ row.value }}</span>
              </template>
            </el-table-column>
            <el-table-column prop="status" label="状态" width="100">
              <template #default="{ row }">
                <el-tag :type="cardStatusType(row)" size="small">
                  {{ cardStatusLabel(row) }}
                </el-tag>
              </template>
            </el-table-column>
            <el-table-column prop="used_by" label="使用者" min-width="120" show-overflow-tooltip />
            <el-table-column label="有效期" width="160">
              <template #default="{ row }">
                <span :class="{ 'text-red-400': isCardExpired(row) }">
                  {{ row.expires_at ? dayjs(row.expires_at).format('YYYY-MM-DD') : '永不' }}
                </span>
              </template>
            </el-table-column>
            <el-table-column prop="created_at" label="创建时间" width="160">
              <template #default="{ row }">
                {{ dayjs(row.created_at).format('YYYY-MM-DD HH:mm') }}
              </template>
            </el-table-column>
            <el-table-column label="操作" width="150">
              <template #default="{ row }">
                <el-button
                  v-if="row.status === 'active'"
                  size="small"
                  text
                  type="warning"
                  @click="handleRevoke(row)"
                >
                  撤销
                </el-button>
                <el-popconfirm title="确认删除？" @confirm="handleDelete(row.id)">
                  <template #reference>
                    <el-button size="small" text type="danger">删除</el-button>
                  </template>
                </el-popconfirm>
              </template>
            </el-table-column>
          </el-table>
        </div>
      </el-tab-pane>

      <el-tab-pane name="redemptions">
        <template #label>
          <span>兑换记录 <el-badge :value="redemptions.length" type="info" class="ml-1" /></span>
        </template>

        <div class="bg-white dark:bg-gray-800 rounded-xl shadow border border-gray-200 dark:border-gray-700">
          <el-table :data="redemptions" v-loading="redemptionLoading" stripe>
            <el-table-column prop="card_code" label="卡号" min-width="200" show-overflow-tooltip>
              <template #default="{ row }">
                <span class="font-mono text-xs">{{ row.card_code }}</span>
              </template>
            </el-table-column>
            <el-table-column prop="value" label="额度" width="100" />
            <el-table-column prop="used_by" label="使用者" min-width="120" />
            <el-table-column prop="used_at" label="兑换时间" width="180">
              <template #default="{ row }">
                {{ row.used_at ? dayjs(row.used_at).format('YYYY-MM-DD HH:mm:ss') : '-' }}
              </template>
            </el-table-column>
          </el-table>
          <div v-if="!redemptions.length && !redemptionLoading" class="text-center py-8 text-gray-400">
            暂无兑换记录
          </div>
        </div>
      </el-tab-pane>
    </el-tabs>

    <!-- Create Dialog -->
    <el-dialog v-model="showCreate" title="创建额度卡" width="520px" destroy-on-close>
      <el-form :model="form" label-width="100px">
        <el-form-item label="单卡额度" required>
          <el-input-number v-model="form.value" :min="1" :max="100000" :step="10" />
          <span class="ml-2 text-xs text-gray-400">USD</span>
        </el-form-item>
        <el-form-item label="数量" required>
          <el-input-number v-model="form.count" :min="1" :max="500" />
          <span class="ml-2 text-xs text-gray-400">最多 500 张</span>
        </el-form-item>
        <el-form-item label="有效期 (天)">
          <el-input-number v-model="form.validity_days" :min="1" :max="365" />
          <span class="ml-2 text-xs text-gray-400">从创建日算起</span>
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
import { ref, computed, onMounted, watch } from 'vue'
import { Plus, Search, Download } from '@element-plus/icons-vue'
import {
  getQuotaCards,
  createQuotaCard,
  deleteQuotaCard,
  getQuotaCardStats,
  revokeQuotaCard,
  getRedemptionHistory,
  batchCreateQuotaCards
} from '@/utils/api'
import { ElMessage, ElMessageBox } from 'element-plus'
import StatCard from '@/components/common/StatCard.vue'
import dayjs from 'dayjs'

const cards = ref([])
const stats = ref(null)
const loading = ref(false)
const saving = ref(false)
const showCreate = ref(false)
const form = ref({ value: 100, count: 1, validity_days: 30 })

const activeTab = ref('cards')
const searchCode = ref('')
const statusFilter = ref('')

// Redemptions
const redemptions = ref([])
const redemptionLoading = ref(false)

const filteredCards = computed(() => {
  let list = cards.value
  if (searchCode.value) {
    const q = searchCode.value.toLowerCase()
    list = list.filter((c) => c.code?.toLowerCase().includes(q))
  }
  if (statusFilter.value) {
    if (statusFilter.value === 'unused') list = list.filter((c) => c.status === 'active' && c.current_redemptions === 0)
    else if (statusFilter.value === 'used') list = list.filter((c) => c.current_redemptions > 0)
    else if (statusFilter.value === 'revoked') list = list.filter((c) => c.status === 'disabled')
  }
  return list
})

function cardStatusType(row) {
  if (row.status === 'disabled') return 'danger'
  if (row.current_redemptions > 0) return 'success'
  if (isCardExpired(row)) return 'warning'
  return 'info'
}

function cardStatusLabel(row) {
  if (row.status === 'disabled') return '已撤销'
  if (row.current_redemptions > 0) return '已使用'
  if (isCardExpired(row)) return '已过期'
  return '未使用'
}

function isCardExpired(row) {
  return row.expires_at && dayjs(row.expires_at).isBefore(dayjs())
}

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
  } catch {
    /* ignore */
  }
}

async function fetchRedemptions() {
  redemptionLoading.value = true
  try {
    redemptions.value = await getRedemptionHistory()
  } catch {
    redemptions.value = []
  } finally {
    redemptionLoading.value = false
  }
}

async function handleCreate() {
  saving.value = true
  try {
    const count = form.value.count || 1
    let created = []
    if (count > 1) {
      const res = await batchCreateQuotaCards({ count, value: form.value.value })
      created = Array.isArray(res) ? res : res?.data || []
    } else {
      const res = await createQuotaCard({ value: form.value.value })
      created = [res]
    }
    ElMessage.success(`成功创建 ${count} 张额度卡`)
    showCreate.value = false

    // Offer CSV download
    if (created.length) {
      const csv = 'code,value,expires_at\n' + created.map((c) => `${c.code},${c.value},${c.expires_at || ''}`).join('\n')
      const blob = new Blob([csv], { type: 'text/csv' })
      const url = URL.createObjectURL(blob)
      const a = document.createElement('a')
      a.href = url
      a.download = `quota-cards-${dayjs().format('YYYYMMDD-HHmm')}.csv`
      a.click()
      URL.revokeObjectURL(url)
    }

    await fetchCards()
    await fetchStats()
  } catch (e) {
    ElMessage.error(e?.response?.data?.error || '创建失败')
  } finally {
    saving.value = false
  }
}

async function handleRevoke(row) {
  try {
    await ElMessageBox.confirm(`确认撤销卡号 ${row.code.substring(0, 16)}...？`, '撤销额度卡')
    await revokeQuotaCard(row.id)
    ElMessage.success('已撤销')
    await fetchCards()
    await fetchStats()
  } catch {
    /* cancelled */
  }
}

async function handleDelete(id) {
  await deleteQuotaCard(id)
  ElMessage.success('已删除')
  await fetchCards()
  await fetchStats()
}

function exportCards() {
  const rows = filteredCards.value
  const csv =
    'code,value,status,used_by,expires_at,created_at\n' +
    rows
      .map(
        (c) =>
          `${c.code},${c.value},${cardStatusLabel(c)},${c.used_by || ''},${c.expires_at || ''},${c.created_at || ''}`
      )
      .join('\n')
  const blob = new Blob([csv], { type: 'text/csv' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `quota-cards-${dayjs().format('YYYYMMDD')}.csv`
  a.click()
  URL.revokeObjectURL(url)
}

watch(activeTab, (tab) => {
  if (tab === 'redemptions') fetchRedemptions()
})

onMounted(() => {
  fetchCards()
  fetchStats()
})
</script>

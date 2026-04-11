<template>
  <div>
    <div class="flex justify-between items-center mb-6">
      <h2 class="text-2xl font-bold text-gray-800 dark:text-gray-100">API Keys</h2>
      <div class="flex gap-2">
        <el-button @click="exportCsv" :disabled="filteredList.length === 0">
          <el-icon class="mr-1"><Download /></el-icon>导出 CSV
        </el-button>
        <el-button type="success" @click="showBatchCreate = true">批量创建</el-button>
        <el-button type="primary" @click="showCreate = true">创建 Key</el-button>
      </div>
    </div>

    <!-- New Key Display -->
    <el-alert v-if="newKey" type="success" :closable="true" @close="newKey = ''" class="mb-4">
      <template #title>
        <div>
          <p>新 Key 已创建，请立即复制保存（不会再次显示）：</p>
          <code class="text-lg font-mono select-all bg-green-50 dark:bg-green-900/30 px-2 py-1 rounded">{{ newKey }}</code>
          <el-button size="small" class="ml-2" @click="copyKey(newKey)">复制</el-button>
        </div>
      </template>
    </el-alert>

    <!-- Batch Keys Display -->
    <el-alert v-if="batchKeys.length > 0" type="success" :closable="true" @close="batchKeys = []" class="mb-4">
      <template #title>
        <div>
          <p class="mb-2">批量创建了 {{ batchKeys.length }} 个 Key：</p>
          <div class="max-h-40 overflow-y-auto space-y-1">
            <div v-for="(k, i) in batchKeys" :key="i" class="flex items-center gap-2">
              <code class="text-xs font-mono select-all bg-green-50 dark:bg-green-900/30 px-2 py-1 rounded">{{ k }}</code>
            </div>
          </div>
          <el-button size="small" class="mt-2" @click="copyAllBatchKeys">复制全部</el-button>
          <el-button size="small" class="mt-2" @click="downloadBatchKeys">下载 CSV</el-button>
        </div>
      </template>
    </el-alert>

    <!-- Tabs: Active / Deleted -->
    <el-tabs v-model="activeTab" type="border-card" class="mb-4">
      <el-tab-pane name="active">
        <template #label>
          <span>活跃 Keys <el-badge :value="list.length" class="ml-1" /></span>
        </template>

        <!-- Filters -->
        <div class="flex gap-3 mb-4 flex-wrap">
          <el-input
            v-model="searchText"
            placeholder="搜索名称"
            clearable
            style="width: 200px"
            :prefix-icon="Search"
          />
          <el-select v-model="statusFilterKey" placeholder="状态" clearable style="width: 120px">
            <el-option label="Active" value="active" />
            <el-option label="Disabled" value="disabled" />
          </el-select>
          <el-select v-model="sortByKey" placeholder="排序" style="width: 140px">
            <el-option label="名称" value="name" />
            <el-option label="创建时间" value="created_at" />
            <el-option label="日限额" value="daily_cost_limit" />
            <el-option label="RPM" value="rate_limit_rpm" />
          </el-select>
        </div>

        <div class="bg-white dark:bg-gray-800 rounded-xl shadow border border-gray-200 dark:border-gray-700">
          <el-table :data="filteredList" v-loading="loading" stripe>
            <el-table-column prop="name" label="名称" min-width="120" sortable />
            <el-table-column label="Key Prefix" width="140" show-overflow-tooltip>
              <template #default="{ row }">
                <span class="font-mono text-xs">{{ (row.key_hash || '').substring(0, 16) }}...</span>
              </template>
            </el-table-column>
            <el-table-column prop="status" label="状态" width="80">
              <template #default="{ row }">
                <el-tag size="small" :type="row.status === 'active' ? 'success' : 'danger'">{{ row.status }}</el-tag>
              </template>
            </el-table-column>
            <el-table-column label="权限" width="150" show-overflow-tooltip>
              <template #default="{ row }">
                {{ parsePerms(row.permissions) }}
              </template>
            </el-table-column>
            <el-table-column label="日限额" width="100">
              <template #default="{ row }">
                <span :class="{ 'text-orange-500': row.daily_cost_limit > 0 }">
                  {{ row.daily_cost_limit > 0 ? '$' + row.daily_cost_limit : '无限' }}
                </span>
              </template>
            </el-table-column>
            <el-table-column label="并发" width="60">
              <template #default="{ row }">
                {{ row.max_concurrency || '∞' }}
              </template>
            </el-table-column>
            <el-table-column label="RPM" width="60">
              <template #default="{ row }">
                {{ row.rate_limit_rpm || '默认' }}
              </template>
            </el-table-column>
            <el-table-column label="过期时间" width="160">
              <template #default="{ row }">
                <span :class="{ 'text-red-400': isExpired(row.expires_at) }">
                  {{ row.expires_at ? dayjs(row.expires_at).format('YYYY-MM-DD HH:mm') : '永不' }}
                </span>
              </template>
            </el-table-column>
            <el-table-column label="操作" width="180" fixed="right">
              <template #default="{ row }">
                <el-button size="small" text type="primary" @click="editRow(row)">编辑</el-button>
                <el-popconfirm title="确认删除此 Key？" @confirm="store.remove(row.id)">
                  <template #reference>
                    <el-button size="small" text type="danger">删除</el-button>
                  </template>
                </el-popconfirm>
              </template>
            </el-table-column>
          </el-table>
        </div>
      </el-tab-pane>

      <el-tab-pane name="deleted">
        <template #label>
          <span>已删除 <el-badge :value="deletedList.length" type="info" class="ml-1" /></span>
        </template>

        <div class="bg-white dark:bg-gray-800 rounded-xl shadow border border-gray-200 dark:border-gray-700">
          <el-table :data="deletedList" v-loading="deletedLoading" stripe>
            <el-table-column prop="name" label="名称" min-width="120" />
            <el-table-column label="Key Hash" width="200" show-overflow-tooltip>
              <template #default="{ row }">
                <span class="font-mono text-xs">{{ (row.key_hash || '').substring(0, 16) }}...</span>
              </template>
            </el-table-column>
            <el-table-column prop="deleted_at" label="删除时间" width="180" />
            <el-table-column label="操作" width="100">
              <template #default="{ row }">
                <el-button size="small" text type="primary" @click="handleRestore(row)">恢复</el-button>
              </template>
            </el-table-column>
          </el-table>
          <div v-if="!deletedList.length && !deletedLoading" class="text-center py-8 text-gray-400">
            暂无已删除的 Key
          </div>
        </div>
      </el-tab-pane>
    </el-tabs>

    <!-- Create/Edit Dialog -->
    <el-dialog v-model="showEditDialog" :title="editing ? '编辑 API Key' : '创建 API Key'" width="600px" destroy-on-close>
      <el-form :model="form" label-width="100px">
        <el-form-item label="名称" required>
          <el-input v-model="form.name" />
        </el-form-item>
        <el-form-item label="权限">
          <el-select v-model="form.permissions" multiple class="w-full" placeholder="空 = 全部权限">
            <el-option value="claude" label="Claude" />
            <el-option value="openai" label="OpenAI" />
            <el-option value="gemini" label="Gemini" />
            <el-option value="bedrock" label="Bedrock" />
            <el-option value="azure" label="Azure" />
          </el-select>
        </el-form-item>
        <el-form-item label="状态" v-if="editing">
          <el-select v-model="form.status" class="w-full">
            <el-option value="active" label="Active" />
            <el-option value="disabled" label="Disabled" />
          </el-select>
        </el-form-item>
        <el-form-item label="日费用限额">
          <el-input-number v-model="form.daily_cost_limit" :min="0" :precision="2" :step="1" />
          <span class="ml-2 text-xs text-gray-400">0 = 无限</span>
        </el-form-item>
        <el-form-item label="总费用限额">
          <el-input-number v-model="form.total_cost_limit" :min="0" :precision="2" :step="10" />
        </el-form-item>
        <el-form-item label="并发限制">
          <el-input-number v-model="form.max_concurrency" :min="0" :max="1000" />
          <span class="ml-2 text-xs text-gray-400">0 = 无限</span>
        </el-form-item>
        <el-form-item label="RPM 限制">
          <el-input-number v-model="form.rate_limit_rpm" :min="0" :max="10000" />
          <span class="ml-2 text-xs text-gray-400">0 = 全局默认</span>
        </el-form-item>
        <el-form-item label="过期时间">
          <el-date-picker v-model="form.expires_at" type="datetime" class="w-full" placeholder="不过期" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showEditDialog = false">取消</el-button>
        <el-button type="primary" :loading="saving" @click="handleSave">保存</el-button>
      </template>
    </el-dialog>

    <!-- Batch Create Dialog -->
    <el-dialog v-model="showBatchCreate" title="批量创建 API Key" width="520px" destroy-on-close>
      <el-form :model="batchForm" label-width="100px">
        <el-form-item label="名称前缀" required>
          <el-input v-model="batchForm.name_prefix" placeholder="batch-key" />
        </el-form-item>
        <el-form-item label="数量" required>
          <el-input-number v-model="batchForm.count" :min="1" :max="500" />
          <span class="ml-2 text-xs text-gray-400">最多 500 个</span>
        </el-form-item>
        <el-form-item label="权限">
          <el-select v-model="batchForm.permissions" multiple class="w-full" placeholder="空 = 全部权限">
            <el-option value="claude" label="Claude" />
            <el-option value="openai" label="OpenAI" />
            <el-option value="gemini" label="Gemini" />
            <el-option value="bedrock" label="Bedrock" />
            <el-option value="azure" label="Azure" />
          </el-select>
        </el-form-item>
        <el-form-item label="日费用限额">
          <el-input-number v-model="batchForm.daily_cost_limit" :min="0" :precision="2" :step="1" />
        </el-form-item>
        <el-form-item label="过期时间">
          <el-date-picker v-model="batchForm.expires_at" type="datetime" class="w-full" placeholder="不过期" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showBatchCreate = false">取消</el-button>
        <el-button type="primary" :loading="batchSaving" @click="handleBatchCreate">创建</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, watch } from 'vue'
import { storeToRefs } from 'pinia'
import { useApiKeysStore } from '@/stores/apiKeys'
import { batchCreateApiKeys, getDeletedApiKeys, restoreApiKey } from '@/utils/api'
import { ElMessage } from 'element-plus'
import { Search, Download } from '@element-plus/icons-vue'
import dayjs from 'dayjs'

const store = useApiKeysStore()
const { list, loading } = storeToRefs(store)

const activeTab = ref('active')
const showCreate = ref(false)
const editing = ref(null)
const saving = ref(false)
const newKey = ref('')

// Filters
const searchText = ref('')
const statusFilterKey = ref('')
const sortByKey = ref('name')

// Batch
const showBatchCreate = ref(false)
const batchSaving = ref(false)
const batchKeys = ref([])
const batchForm = ref({
  name_prefix: '',
  count: 10,
  permissions: [],
  daily_cost_limit: 0,
  expires_at: null
})

// Deleted
const deletedList = ref([])
const deletedLoading = ref(false)

const defaultForm = () => ({
  name: '',
  permissions: [],
  status: 'active',
  daily_cost_limit: 0,
  total_cost_limit: 0,
  max_concurrency: 0,
  rate_limit_rpm: 0,
  expires_at: null
})

const form = ref(defaultForm())

const showEditDialog = computed({
  get: () => showCreate.value || !!editing.value,
  set: (v) => {
    if (!v) {
      showCreate.value = false
      editing.value = null
    }
  }
})

const filteredList = computed(() => {
  let arr = list.value || []
  if (searchText.value) {
    const q = searchText.value.toLowerCase()
    arr = arr.filter((k) => k.name?.toLowerCase().includes(q))
  }
  if (statusFilterKey.value) {
    arr = arr.filter((k) => k.status === statusFilterKey.value)
  }
  const key = sortByKey.value
  arr = [...arr].sort((a, b) => {
    if (key === 'created_at') return new Date(b.created_at) - new Date(a.created_at)
    if (key === 'daily_cost_limit') return (b.daily_cost_limit || 0) - (a.daily_cost_limit || 0)
    if (key === 'rate_limit_rpm') return (b.rate_limit_rpm || 0) - (a.rate_limit_rpm || 0)
    return (a.name || '').localeCompare(b.name || '')
  })
  return arr
})

function isExpired(d) {
  return d && dayjs(d).isBefore(dayjs())
}

function parsePerms(p) {
  try {
    const arr = typeof p === 'string' ? JSON.parse(p) : p
    return arr.length ? arr.join(', ') : '全部'
  } catch {
    return '全部'
  }
}

function editRow(row) {
  editing.value = row.id
  const perms =
    typeof row.permissions === 'string'
      ? JSON.parse(row.permissions || '[]')
      : row.permissions || []
  form.value = { ...row, permissions: perms }
}

async function handleSave() {
  saving.value = true
  try {
    if (editing.value) {
      await store.update(editing.value, form.value)
      ElMessage.success('更新成功')
    } else {
      const res = await store.create(form.value)
      newKey.value = res.key
      ElMessage.success('创建成功')
    }
    showEditDialog.value = false
    form.value = defaultForm()
  } catch {
    /* interceptor */
  } finally {
    saving.value = false
  }
}

async function handleBatchCreate() {
  if (!batchForm.value.name_prefix) {
    ElMessage.warning('请输入名称前缀')
    return
  }
  batchSaving.value = true
  try {
    const res = await batchCreateApiKeys(batchForm.value)
    const items = Array.isArray(res) ? res : res?.keys || res?.data?.keys || []
    batchKeys.value = items.map((k) => k.key || k)
    showBatchCreate.value = false
    ElMessage.success(`成功创建 ${batchKeys.value.length} 个 Key`)
    await store.fetch()
  } catch (e) {
    ElMessage.error(e?.response?.data?.error || '批量创建失败')
  } finally {
    batchSaving.value = false
  }
}

function copyKey(key) {
  navigator.clipboard.writeText(key)
  ElMessage.success('已复制')
}

function copyAllBatchKeys() {
  navigator.clipboard.writeText(batchKeys.value.join('\n'))
  ElMessage.success('已复制全部')
}

function downloadBatchKeys() {
  const csv = 'key\n' + batchKeys.value.join('\n')
  const blob = new Blob([csv], { type: 'text/csv' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `api-keys-${dayjs().format('YYYYMMDD-HHmm')}.csv`
  a.click()
  URL.revokeObjectURL(url)
}

function exportCsv() {
  const rows = filteredList.value.map((k) => ({
    name: k.name,
    status: k.status,
    permissions: parsePerms(k.permissions),
    daily_cost_limit: k.daily_cost_limit || 0,
    rate_limit_rpm: k.rate_limit_rpm || 0,
    max_concurrency: k.max_concurrency || 0,
    expires_at: k.expires_at || '',
    created_at: k.created_at || ''
  }))
  const header = Object.keys(rows[0] || {}).join(',')
  const body = rows.map((r) => Object.values(r).join(',')).join('\n')
  const csv = header + '\n' + body
  const blob = new Blob([csv], { type: 'text/csv' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `api-keys-export-${dayjs().format('YYYYMMDD')}.csv`
  a.click()
  URL.revokeObjectURL(url)
}

async function fetchDeleted() {
  deletedLoading.value = true
  try {
    deletedList.value = await getDeletedApiKeys()
  } catch {
    deletedList.value = []
  } finally {
    deletedLoading.value = false
  }
}

async function handleRestore(row) {
  try {
    await restoreApiKey(row.id)
    ElMessage.success('已恢复')
    await Promise.all([store.fetch(), fetchDeleted()])
  } catch (e) {
    ElMessage.error(e?.response?.data?.error || '恢复失败')
  }
}

watch(activeTab, (tab) => {
  if (tab === 'deleted') fetchDeleted()
})

onMounted(() => store.fetch())
</script>

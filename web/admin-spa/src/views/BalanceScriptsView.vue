<template>
  <div>
    <div class="mb-6 flex items-center justify-between">
      <h2 class="text-2xl font-bold text-gray-800 dark:text-gray-100">余额脚本管理</h2>
      <div class="flex gap-2">
        <el-button size="small" @click="fetchScripts">
          <el-icon class="mr-1"><Refresh /></el-icon>刷新
        </el-button>
        <el-button type="primary" @click="openCreate">
          <el-icon class="mr-1"><Plus /></el-icon>创建脚本
        </el-button>
      </div>
    </div>

    <!-- Stats -->
    <div class="mb-6 grid grid-cols-2 gap-4 lg:grid-cols-4">
      <div class="stat-card">
        <p class="text-xs font-semibold text-gray-500 dark:text-gray-400">总脚本</p>
        <p class="text-2xl font-bold text-gray-900 dark:text-gray-100">{{ scripts.length }}</p>
      </div>
      <div class="stat-card">
        <p class="text-xs font-semibold text-gray-500 dark:text-gray-400">已启用</p>
        <p class="text-2xl font-bold text-green-600">{{ scripts.filter((s) => s.enabled).length }}</p>
      </div>
      <div class="stat-card">
        <p class="text-xs font-semibold text-gray-500 dark:text-gray-400">平台覆盖</p>
        <p class="text-2xl font-bold text-blue-600">{{ new Set(scripts.map((s) => s.platform)).size }}</p>
      </div>
      <div class="stat-card">
        <p class="text-xs font-semibold text-gray-500 dark:text-gray-400">上次运行</p>
        <p class="text-lg font-bold text-purple-600">{{ lastRunAt || '从未' }}</p>
      </div>
    </div>

    <!-- Table -->
    <div class="card">
      <el-table :data="filteredScripts" v-loading="loading" stripe size="small">
        <el-table-column prop="name" label="脚本名称" min-width="160" show-overflow-tooltip />
        <el-table-column prop="platform" label="平台" width="120">
          <template #default="{ row }">
            <el-tag size="small">{{ row.platform }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column label="状态" width="80">
          <template #default="{ row }">
            <el-switch :model-value="row.enabled" size="small" @change="toggleEnabled(row)" />
          </template>
        </el-table-column>
        <el-table-column prop="description" label="描述" min-width="200" show-overflow-tooltip />
        <el-table-column label="上次运行" width="140">
          <template #default="{ row }">
            <span class="text-xs text-gray-400">{{ row.last_run_at ? dayjs(row.last_run_at).format('MM-DD HH:mm') : '从未' }}</span>
          </template>
        </el-table-column>
        <el-table-column label="结果" width="80">
          <template #default="{ row }">
            <el-tag v-if="row.last_result" :type="row.last_result === 'success' ? 'success' : 'danger'" size="small">
              {{ row.last_result === 'success' ? '成功' : '失败' }}
            </el-tag>
            <span v-else class="text-xs text-gray-400">-</span>
          </template>
        </el-table-column>
        <el-table-column label="操作" width="200" fixed="right">
          <template #default="{ row }">
            <el-button size="small" text type="primary" @click="editScript(row)">编辑</el-button>
            <el-button size="small" text type="success" @click="testRun(row)" :loading="row._testing">测试</el-button>
            <el-popconfirm title="确认删除？" @confirm="handleDelete(row)">
              <template #reference>
                <el-button size="small" text type="danger">删除</el-button>
              </template>
            </el-popconfirm>
          </template>
        </el-table-column>
      </el-table>
    </div>

    <!-- Test Result -->
    <el-dialog v-model="showTestResult" title="测试结果" width="600px" destroy-on-close>
      <div v-if="testResult">
        <div class="mb-3">
          <el-tag :type="testResult.success ? 'success' : 'danger'" size="small">
            {{ testResult.success ? '成功' : '失败' }}
          </el-tag>
          <span class="ml-2 text-sm text-gray-500">{{ testResult.duration }}ms</span>
        </div>
        <pre class="max-h-64 overflow-auto rounded bg-gray-100 p-3 text-xs dark:bg-gray-900">{{ testResult.output || testResult.error || '无输出' }}</pre>
      </div>
    </el-dialog>

    <!-- Edit / Create Dialog -->
    <el-dialog v-model="showEdit" :title="isCreating ? '创建脚本' : '编辑脚本'" width="750px" destroy-on-close>
      <el-form :model="editForm" label-width="100px">
        <el-form-item label="名称" required>
          <el-input v-model="editForm.name" :disabled="!isCreating" placeholder="balance_check_claude" />
        </el-form-item>
        <el-form-item label="平台" required>
          <el-select v-model="editForm.platform" class="w-full" placeholder="选择平台">
            <el-option v-for="p in allPlatforms" :key="p" :value="p" :label="p" />
          </el-select>
        </el-form-item>
        <el-form-item label="描述">
          <el-input v-model="editForm.description" placeholder="脚本用途描述" />
        </el-form-item>
        <el-form-item label="启用">
          <el-switch v-model="editForm.enabled" />
        </el-form-item>
        <el-form-item label="脚本内容" required>
          <el-input v-model="editForm.script" type="textarea" :rows="16" class="font-mono" placeholder="// 余额检查脚本..." />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showEdit = false">取消</el-button>
        <el-button type="primary" :loading="saving" @click="handleSave">保存</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import { Plus, Refresh } from '@element-plus/icons-vue'
import { getBalanceScripts, getBalanceScript, updateBalanceScript, createBalanceScript, deleteBalanceScript, testBalanceScript } from '@/utils/api'
import { ElMessage } from 'element-plus'
import dayjs from 'dayjs'

const allPlatforms = ['claude','claude-console','bedrock','ccr','gemini','gemini-api','openai','openai-responses','azure-openai','droid']

const scripts = ref([])
const loading = ref(false)
const saving = ref(false)
const showEdit = ref(false)
const isCreating = ref(false)
const editForm = ref({ name: '', platform: '', description: '', enabled: false, script: '' })

const showTestResult = ref(false)
const testResult = ref(null)

const filteredScripts = computed(() => scripts.value)
const lastRunAt = computed(() => {
  const dates = scripts.value.map((s) => s.last_run_at).filter(Boolean).sort().reverse()
  return dates.length ? dayjs(dates[0]).format('MM-DD HH:mm') : null
})

async function fetchScripts() {
  loading.value = true
  try {
    const res = await getBalanceScripts()
    scripts.value = (Array.isArray(res) ? res : res?.data || []).map((s) => ({ ...s, _testing: false }))
  } finally { loading.value = false }
}

function openCreate() {
  isCreating.value = true
  editForm.value = { name: '', platform: '', description: '', enabled: true, script: '' }
  showEdit.value = true
}

async function editScript(row) {
  isCreating.value = false
  try {
    const detail = await getBalanceScript(row.name)
    editForm.value = { ...detail }
  } catch {
    editForm.value = { ...row }
  }
  showEdit.value = true
}

async function handleSave() {
  if (!editForm.value.name) { ElMessage.warning('请输入脚本名称'); return }
  saving.value = true
  try {
    if (isCreating.value) {
      await createBalanceScript(editForm.value)
      ElMessage.success('创建成功')
    } else {
      await updateBalanceScript(editForm.value.name, editForm.value)
      ElMessage.success('保存成功')
    }
    showEdit.value = false
    await fetchScripts()
  } catch (e) { ElMessage.error(e?.response?.data?.error || '操作失败') }
  finally { saving.value = false }
}

async function handleDelete(row) {
  try {
    await deleteBalanceScript(row.name)
    ElMessage.success('已删除')
    await fetchScripts()
  } catch (e) { ElMessage.error(e?.response?.data?.error || '删除失败') }
}

async function toggleEnabled(row) {
  try {
    await updateBalanceScript(row.name, { ...row, enabled: !row.enabled })
    row.enabled = !row.enabled
    ElMessage.success(row.enabled ? '已启用' : '已禁用')
  } catch (e) { ElMessage.error(e?.response?.data?.error || '操作失败') }
}

async function testRun(row) {
  row._testing = true
  try {
    const res = await testBalanceScript(row.name)
    testResult.value = res
    showTestResult.value = true
  } catch (e) {
    testResult.value = { success: false, error: e?.response?.data?.error || e.message || '测试失败' }
    showTestResult.value = true
  } finally { row._testing = false }
}

onMounted(fetchScripts)
</script>

<style scoped>
.stat-card { @apply rounded-xl border border-gray-200 bg-white p-4 shadow-sm dark:border-gray-700 dark:bg-gray-800; }
.card { @apply rounded-xl border border-gray-200 bg-white shadow dark:border-gray-700 dark:bg-gray-800; }
</style>

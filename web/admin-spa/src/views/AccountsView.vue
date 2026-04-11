<template>
  <div>
    <div class="flex justify-between items-center mb-6">
      <h2 class="text-2xl font-bold text-gray-800 dark:text-gray-100">账户管理</h2>
      <div class="flex gap-2">
        <el-button type="success" @click="showOAuth = true">
          <el-icon class="mr-1"><Link /></el-icon>OAuth 认证
        </el-button>
        <el-button type="primary" @click="openCreate">
          <el-icon class="mr-1"><Plus /></el-icon>添加账户
        </el-button>
      </div>
    </div>

    <!-- Platform Tabs -->
    <el-tabs v-model="activePlatform" type="border-card" class="mb-4 account-tabs">
      <el-tab-pane
        v-for="p in platformList"
        :key="p.key"
        :label="p.label"
        :name="p.key"
      >
        <template #label>
          <span class="flex items-center gap-1">
            {{ p.label }}
            <el-badge
              :value="getPlatformCount(p.key)"
              :type="getPlatformCount(p.key) > 0 ? '' : 'info'"
              class="ml-1"
            />
          </span>
        </template>

        <div class="bg-white dark:bg-gray-800 rounded-lg overflow-hidden">
          <el-table
            :data="getCurrentPlatformAccounts()"
            v-loading="store.loading"
            stripe
            class="w-full"
            row-key="id"
          >
            <el-table-column prop="name" label="名称" min-width="140" show-overflow-tooltip />
            <el-table-column prop="status" label="状态" width="100">
              <template #default="{ row }">
                <el-tag
                  size="small"
                  :type="statusType(row.status)"
                  effect="light"
                  @click="handleToggleStatus(row)"
                  class="cursor-pointer"
                >
                  {{ statusLabel(row.status) }}
                </el-tag>
              </template>
            </el-table-column>
            <el-table-column label="可调度" width="80" align="center">
              <template #default="{ row }">
                <el-switch
                  :model-value="row.schedulable !== false"
                  size="small"
                  @change="handleToggleSchedulable(row)"
                />
              </template>
            </el-table-column>
            <el-table-column prop="priority" label="优先级" width="80" />
            <el-table-column prop="max_concurrency" label="并发" width="60" />
            <el-table-column prop="description" label="描述" min-width="120" show-overflow-tooltip />
            <el-table-column prop="proxy_url" label="代理" min-width="120" show-overflow-tooltip />
            <el-table-column label="操作" width="220" fixed="right">
              <template #default="{ row }">
                <el-button size="small" text type="primary" @click="editRow(row)">编辑</el-button>
                <el-button
                  v-if="canTest(activePlatform)"
                  size="small"
                  text
                  type="success"
                  @click="handleTest(row)"
                >
                  测试
                </el-button>
                <el-button
                  v-if="canReset(activePlatform)"
                  size="small"
                  text
                  type="warning"
                  @click="handleReset(row)"
                >
                  重置
                </el-button>
                <el-popconfirm title="确认删除此账户？" @confirm="handleDelete(row)">
                  <template #reference>
                    <el-button size="small" text type="danger">删除</el-button>
                  </template>
                </el-popconfirm>
              </template>
            </el-table-column>
          </el-table>
        </div>
      </el-tab-pane>
    </el-tabs>

    <!-- Account Groups -->
    <div class="mt-6 bg-white dark:bg-gray-800 rounded-xl shadow border border-gray-200 dark:border-gray-700 p-6">
      <div class="flex justify-between items-center mb-4">
        <h3 class="text-lg font-semibold text-gray-700 dark:text-gray-200">账户分组</h3>
        <el-button size="small" type="primary" @click="showGroupCreate = true">新建分组</el-button>
      </div>
      <el-table :data="store.groups" stripe size="small">
        <el-table-column prop="name" label="名称" />
        <el-table-column prop="description" label="描述" />
        <el-table-column label="操作" width="150">
          <template #default="{ row }">
            <el-button size="small" text type="primary" @click="editGroup(row)">编辑</el-button>
            <el-popconfirm title="确认删除此分组？" @confirm="store.deleteGroup(row.id)">
              <template #reference>
                <el-button size="small" text type="danger">删除</el-button>
              </template>
            </el-popconfirm>
          </template>
        </el-table-column>
      </el-table>
    </div>

    <!-- Create/Edit Dialog -->
    <el-dialog
      v-model="showDialog"
      :title="editing ? '编辑账户' : '添加账户'"
      width="640px"
      destroy-on-close
    >
      <AccountForm
        :platform="editing ? editingPlatform : activePlatform"
        :initial-data="editing ? editFormData : null"
        :fingerprints="fpStore.list"
        :groups="store.groups"
        :is-edit="!!editing"
        @save="handleSave"
        @cancel="showDialog = false"
      />
    </el-dialog>

    <!-- OAuth Dialog -->
    <el-dialog v-model="showOAuth" title="OAuth 认证" width="720px" destroy-on-close>
      <OAuthFlow
        :platform="activePlatform"
        @success="handleOAuthSuccess"
        @close="showOAuth = false"
      />
    </el-dialog>

    <!-- Group Create/Edit Dialog -->
    <el-dialog v-model="showGroupDialog" :title="editingGroup ? '编辑分组' : '新建分组'" width="500px" destroy-on-close>
      <el-form :model="groupForm" label-width="80px">
        <el-form-item label="名称" required>
          <el-input v-model="groupForm.name" />
        </el-form-item>
        <el-form-item label="描述">
          <el-input v-model="groupForm.description" type="textarea" :rows="2" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showGroupDialog = false">取消</el-button>
        <el-button type="primary" @click="handleGroupSave">保存</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import { useAccountsStore } from '@/stores/accounts'
import { useFingerprintsStore } from '@/stores/fingerprints'
import { ElMessage } from 'element-plus'
import { Plus, Link } from '@element-plus/icons-vue'
import AccountForm from '@/components/accounts/AccountForm.vue'
import OAuthFlow from '@/components/accounts/OAuthFlow.vue'

const store = useAccountsStore()
const fpStore = useFingerprintsStore()

const platformList = [
  { key: 'claude', label: 'Claude' },
  { key: 'claude-console', label: 'Claude Console' },
  { key: 'bedrock', label: 'Bedrock' },
  { key: 'ccr', label: 'CCR' },
  { key: 'gemini', label: 'Gemini' },
  { key: 'gemini-api', label: 'Gemini API' },
  { key: 'openai', label: 'OpenAI' },
  { key: 'openai-responses', label: 'OpenAI Responses' },
  { key: 'azure-openai', label: 'Azure OpenAI' },
  { key: 'droid', label: 'Droid' }
]

const activePlatform = ref('claude')
const showOAuth = ref(false)
const editing = ref(null)
const editingPlatform = ref('claude')
const editFormData = ref(null)
const saving = ref(false)

const showCreate = ref(false)
const showDialog = computed({
  get: () => showCreate.value || !!editing.value,
  set: (v) => {
    if (!v) {
      showCreate.value = false
      editing.value = null
      editFormData.value = null
    }
  }
})

// Groups
const showGroupCreate = ref(false)
const editingGroup = ref(null)
const groupForm = ref({ name: '', description: '' })
const showGroupDialog = computed({
  get: () => showGroupCreate.value || !!editingGroup.value,
  set: (v) => {
    if (!v) {
      showGroupCreate.value = false
      editingGroup.value = null
    }
  }
})

const TESTABLE = ['claude', 'bedrock', 'gemini', 'openai-responses', 'azure-openai', 'droid']
const RESETTABLE = ['claude', 'bedrock', 'gemini', 'openai', 'openai-responses', 'droid']

function canTest(platform) {
  return TESTABLE.includes(platform)
}

function canReset(platform) {
  return RESETTABLE.includes(platform)
}

function getPlatformCount(platform) {
  const cfg = store.PLATFORM_CONFIG[platform]
  return cfg ? (store.stateMap[cfg.stateKey]?.value?.length || 0) : 0
}

function getCurrentPlatformAccounts() {
  const cfg = store.PLATFORM_CONFIG[activePlatform.value]
  return cfg ? (store.stateMap[cfg.stateKey]?.value || []) : []
}

function statusType(s) {
  const map = {
    active: 'success',
    disabled: 'danger',
    unavailable: 'warning',
    error: 'danger',
    blocked: 'danger'
  }
  return map[s] || 'info'
}

function statusLabel(s) {
  const map = {
    active: '在线',
    disabled: '禁用',
    unavailable: '不可用',
    error: '错误',
    blocked: '封禁'
  }
  return map[s] || s
}

function openCreate() {
  editing.value = null
  editFormData.value = null
  showCreate.value = true
}

function editRow(row) {
  editing.value = row.id
  editingPlatform.value = activePlatform.value
  editFormData.value = { ...row }
}

async function handleSave(formData) {
  saving.value = true
  try {
    const platform = editing.value ? editingPlatform.value : activePlatform.value
    const cfg = store.PLATFORM_CONFIG[platform]
    if (editing.value) {
      const updateFn = `update${capitalize(cfg.stateKey.replace('Accounts', ''))}`
      await store[updateFn]?.(editing.value, formData) ||
        (await import('@/utils/api'))[cfg.updateFn]?.(editing.value, formData)
      await store.fetchByPlatform(platform)
    } else {
      const createFn = `create${capitalize(cfg.stateKey.replace('Accounts', ''))}`
      await store[createFn]?.(formData) ||
        (await import('@/utils/api'))[cfg.createFn]?.(formData)
      await store.fetchByPlatform(platform)
    }
    ElMessage.success('保存成功')
    showDialog.value = false
  } catch (e) {
    ElMessage.error(e?.response?.data?.error || '保存失败')
  } finally {
    saving.value = false
  }
}

function capitalize(s) {
  return s.charAt(0).toUpperCase() + s.slice(1)
}

async function handleDelete(row) {
  try {
    const platform = activePlatform.value
    const cfg = store.PLATFORM_CONFIG[platform]
    const deleteFnName = `delete${capitalize(cfg.stateKey.replace('Accounts', ''))}`
    await store[deleteFnName]?.(row.id)
    ElMessage.success('已删除')
  } catch (e) {
    ElMessage.error('删除失败')
  }
}

async function handleToggleStatus(row) {
  try {
    await store.toggleStatus(activePlatform.value, row.id)
    ElMessage.success('状态已切换')
  } catch {
    ElMessage.warning('该平台不支持切换状态')
  }
}

async function handleToggleSchedulable(row) {
  try {
    await store.toggleSchedulable(activePlatform.value, row.id)
  } catch {
    ElMessage.warning('操作失败')
  }
}

async function handleTest(row) {
  try {
    const res = await store.testAccount(activePlatform.value, row.id)
    ElMessage.success(res?.message || '测试成功')
  } catch (e) {
    ElMessage.error(e?.response?.data?.error || '测试失败')
  }
}

async function handleReset(row) {
  try {
    await store.resetStatus(activePlatform.value, row.id)
    ElMessage.success('状态已重置')
  } catch {
    ElMessage.warning('重置失败')
  }
}

function handleOAuthSuccess() {
  showOAuth.value = false
  store.fetchByPlatform(activePlatform.value)
  ElMessage.success('OAuth 认证成功')
}

function editGroup(row) {
  editingGroup.value = row.id
  groupForm.value = { name: row.name, description: row.description || '' }
}

async function handleGroupSave() {
  try {
    if (editingGroup.value) {
      await store.updateGroup(editingGroup.value, groupForm.value)
    } else {
      await store.createGroup(groupForm.value)
    }
    ElMessage.success('保存成功')
    showGroupDialog.value = false
    groupForm.value = { name: '', description: '' }
  } catch {
    ElMessage.error('保存失败')
  }
}

onMounted(async () => {
  await Promise.all([store.fetchAll(), fpStore.fetch(), store.fetchGroups()])
})
</script>

<style scoped>
.account-tabs :deep(.el-tabs__content) {
  padding: 0;
}
</style>

<template>
  <div>
    <div class="flex justify-between items-center mb-6">
      <h2 class="text-2xl font-bold text-gray-800 dark:text-gray-100">账户管理</h2>
      <el-button type="primary" @click="showCreate = true">添加账户</el-button>
    </div>

    <div class="bg-white dark:bg-gray-800 rounded-xl shadow border border-gray-200 dark:border-gray-700">
      <el-table :data="list" v-loading="loading" stripe class="w-full">
        <el-table-column prop="name" label="名称" min-width="120" />
        <el-table-column prop="platform" label="平台" width="100">
          <template #default="{ row }">
            <el-tag size="small" :type="platformType(row.platform)">{{ row.platform }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="account_type" label="类型" width="80" />
        <el-table-column prop="status" label="状态" width="100">
          <template #default="{ row }">
            <el-tag size="small" :type="statusType(row.status)">{{ row.status }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="priority" label="优先级" width="80" />
        <el-table-column prop="max_concurrency" label="并发" width="60" />
        <el-table-column prop="proxy_url" label="代理" min-width="120" show-overflow-tooltip />
        <el-table-column label="操作" width="180" fixed="right">
          <template #default="{ row }">
            <el-button size="small" text type="primary" @click="editRow(row)">编辑</el-button>
            <el-popconfirm title="确认删除此账户？" @confirm="store.remove(row.id)">
              <template #reference>
                <el-button size="small" text type="danger">删除</el-button>
              </template>
            </el-popconfirm>
          </template>
        </el-table-column>
      </el-table>
    </div>

    <!-- Create/Edit Dialog -->
    <el-dialog v-model="showDialog" :title="editing ? '编辑账户' : '添加账户'" width="600px" destroy-on-close>
      <el-form :model="form" label-width="100px">
        <el-form-item label="名称" required>
          <el-input v-model="form.name" />
        </el-form-item>
        <el-form-item label="平台" required>
          <el-select v-model="form.platform" class="w-full">
            <el-option v-for="p in platforms" :key="p" :value="p" :label="p" />
          </el-select>
        </el-form-item>
        <el-form-item label="账户类型" required>
          <el-select v-model="form.account_type" class="w-full">
            <el-option value="oauth" label="OAuth" />
            <el-option value="apikey" label="API Key" />
            <el-option value="bedrock" label="Bedrock" />
          </el-select>
        </el-form-item>
        <el-form-item v-if="!editing" label="凭据 JSON" required>
          <el-input v-model="form.credentials" type="textarea" :rows="4" placeholder='{"access_token":"..."}' />
        </el-form-item>
        <el-form-item label="状态">
          <el-select v-model="form.status" class="w-full">
            <el-option value="active" label="Active" />
            <el-option value="disabled" label="Disabled" />
            <el-option value="unavailable" label="Unavailable" />
          </el-select>
        </el-form-item>
        <el-form-item label="优先级">
          <el-input-number v-model="form.priority" :min="0" :max="100" />
        </el-form-item>
        <el-form-item label="最大并发">
          <el-input-number v-model="form.max_concurrency" :min="1" :max="100" />
        </el-form-item>
        <el-form-item label="代理 URL">
          <el-input v-model="form.proxy_url" placeholder="socks5://127.0.0.1:1080" />
        </el-form-item>
        <el-form-item label="指纹配置">
          <el-select v-model="form.fingerprint_profile_id" class="w-full" clearable placeholder="无">
            <el-option v-for="fp in fingerprints" :key="fp.id" :value="fp.id" :label="fp.name" />
          </el-select>
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showDialog = false">取消</el-button>
        <el-button type="primary" :loading="saving" @click="handleSave">保存</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import { storeToRefs } from 'pinia'
import { useAccountsStore } from '@/stores/accounts'
import { useFingerprintsStore } from '@/stores/fingerprints'
import { ElMessage } from 'element-plus'

const store = useAccountsStore()
const fpStore = useFingerprintsStore()
const { list, loading } = storeToRefs(store)
const fingerprints = computed(() => fpStore.list)

const platforms = ['claude', 'openai', 'gemini', 'bedrock', 'azure']
const showCreate = ref(false)
const editing = ref(null)
const saving = ref(false)

const defaultForm = () => ({
  name: '', platform: 'claude', account_type: 'oauth',
  credentials: '', status: 'active',
  priority: 50, max_concurrency: 1,
  proxy_url: '', fingerprint_profile_id: null
})

const form = ref(defaultForm())

const showDialog = computed({
  get: () => showCreate.value || !!editing.value,
  set: (v) => { if (!v) { showCreate.value = false; editing.value = null } }
})

function editRow(row) {
  editing.value = row.id
  form.value = { ...row, credentials: '' }
}

function platformType(p) {
  const map = { claude: '', openai: 'success', gemini: 'warning', bedrock: 'info', azure: 'danger' }
  return map[p] || ''
}

function statusType(s) {
  const map = { active: 'success', disabled: 'danger', unavailable: 'warning', error: 'danger', blocked: 'danger' }
  return map[s] || 'info'
}

async function handleSave() {
  saving.value = true
  try {
    if (editing.value) {
      const { credentials, ...data } = form.value
      await store.update(editing.value, data)
    } else {
      await store.create(form.value)
    }
    ElMessage.success('保存成功')
    showDialog.value = false
    form.value = defaultForm()
  } catch { /* handled by interceptor */ } finally {
    saving.value = false
  }
}

onMounted(() => {
  store.fetch()
  fpStore.fetch()
})
</script>

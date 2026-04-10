<template>
  <div>
    <div class="flex justify-between items-center mb-6">
      <h2 class="text-2xl font-bold text-gray-800 dark:text-gray-100">API Keys</h2>
      <el-button type="primary" @click="showCreate = true">创建 Key</el-button>
    </div>

    <!-- New Key Display -->
    <el-alert v-if="newKey" type="success" :closable="true" @close="newKey = ''" class="mb-4">
      <template #title>
        <div>
          <p>新 Key 已创建，请立即复制保存（不会再次显示）：</p>
          <code class="text-lg font-mono select-all bg-green-50 dark:bg-green-900/30 px-2 py-1 rounded">{{ newKey }}</code>
          <el-button size="small" class="ml-2" @click="copyKey">复制</el-button>
        </div>
      </template>
    </el-alert>

    <div class="bg-white dark:bg-gray-800 rounded-xl shadow border border-gray-200 dark:border-gray-700">
      <el-table :data="list" v-loading="loading" stripe>
        <el-table-column prop="name" label="名称" min-width="120" />
        <el-table-column label="Key Hash" width="200" show-overflow-tooltip>
          <template #default="{ row }">
            <span class="font-mono text-xs">{{ row.key_hash.substring(0, 16) }}...</span>
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
        <el-table-column label="日限额" width="90">
          <template #default="{ row }">
            {{ row.daily_cost_limit > 0 ? '$' + row.daily_cost_limit : '无限' }}
          </template>
        </el-table-column>
        <el-table-column label="RPM" width="60" prop="rate_limit_rpm">
          <template #default="{ row }">
            {{ row.rate_limit_rpm || '默认' }}
          </template>
        </el-table-column>
        <el-table-column label="过期时间" width="160">
          <template #default="{ row }">
            {{ row.expires_at ? dayjs(row.expires_at).format('YYYY-MM-DD HH:mm') : '永不' }}
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

    <!-- Create/Edit Dialog -->
    <el-dialog v-model="showDialog" :title="editing ? '编辑 API Key' : '创建 API Key'" width="600px" destroy-on-close>
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
        <el-button @click="showDialog = false">取消</el-button>
        <el-button type="primary" :loading="saving" @click="handleSave">保存</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import { storeToRefs } from 'pinia'
import { useApiKeysStore } from '@/stores/apiKeys'
import { ElMessage } from 'element-plus'
import dayjs from 'dayjs'

const store = useApiKeysStore()
const { list, loading } = storeToRefs(store)

const showCreate = ref(false)
const editing = ref(null)
const saving = ref(false)
const newKey = ref('')

const defaultForm = () => ({
  name: '', permissions: [], status: 'active',
  daily_cost_limit: 0, total_cost_limit: 0,
  max_concurrency: 0, rate_limit_rpm: 0,
  expires_at: null
})

const form = ref(defaultForm())

const showDialog = computed({
  get: () => showCreate.value || !!editing.value,
  set: (v) => { if (!v) { showCreate.value = false; editing.value = null } }
})

function parsePerms(p) {
  try {
    const arr = typeof p === 'string' ? JSON.parse(p) : p
    return arr.length ? arr.join(', ') : '全部'
  } catch { return '全部' }
}

function editRow(row) {
  editing.value = row.id
  const perms = typeof row.permissions === 'string' ? JSON.parse(row.permissions || '[]') : (row.permissions || [])
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
    showDialog.value = false
    form.value = defaultForm()
  } catch { /* interceptor */ } finally {
    saving.value = false
  }
}

function copyKey() {
  navigator.clipboard.writeText(newKey.value)
  ElMessage.success('已复制')
}

onMounted(() => store.fetch())
</script>

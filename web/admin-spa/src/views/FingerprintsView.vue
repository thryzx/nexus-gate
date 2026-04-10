<template>
  <div>
    <div class="flex justify-between items-center mb-6">
      <h2 class="text-2xl font-bold text-gray-800 dark:text-gray-100">指纹配置</h2>
      <el-button type="primary" @click="showCreate = true">添加配置</el-button>
    </div>

    <div class="bg-white dark:bg-gray-800 rounded-xl shadow border border-gray-200 dark:border-gray-700">
      <el-table :data="list" v-loading="loading" stripe>
        <el-table-column prop="name" label="名称" min-width="150" />
        <el-table-column label="TLS Profile" min-width="150" show-overflow-tooltip>
          <template #default="{ row }">
            <span class="font-mono text-xs">{{ JSON.stringify(row.tls_profile || {}).substring(0, 50) }}</span>
          </template>
        </el-table-column>
        <el-table-column label="User-Agent" min-width="200" show-overflow-tooltip>
          <template #default="{ row }">{{ row.user_agent_template || '-' }}</template>
        </el-table-column>
        <el-table-column label="创建时间" width="160">
          <template #default="{ row }">{{ dayjs(row.created_at).format('YYYY-MM-DD HH:mm') }}</template>
        </el-table-column>
        <el-table-column label="操作" width="180" fixed="right">
          <template #default="{ row }">
            <el-button size="small" text type="primary" @click="editRow(row)">编辑</el-button>
            <el-popconfirm title="确认删除？" @confirm="store.remove(row.id)">
              <template #reference>
                <el-button size="small" text type="danger">删除</el-button>
              </template>
            </el-popconfirm>
          </template>
        </el-table-column>
      </el-table>
    </div>

    <el-dialog v-model="showDialog" :title="editing ? '编辑指纹配置' : '添加指纹配置'" width="700px" destroy-on-close>
      <el-form :model="form" label-width="140px">
        <el-form-item label="名称" required>
          <el-input v-model="form.name" />
        </el-form-item>
        <el-form-item label="TLS Profile">
          <el-input v-model="form.tls_profile_str" type="textarea" :rows="4" placeholder='{"min_version":"1.2"}' />
        </el-form-item>
        <el-form-item label="HTTP/2 Settings">
          <el-input v-model="form.http2_settings_str" type="textarea" :rows="3" placeholder='{"header_table_size":65536}' />
        </el-form-item>
        <el-form-item label="Header Order">
          <el-input v-model="form.header_order_str" type="textarea" :rows="2" placeholder='["host","user-agent","accept"]' />
        </el-form-item>
        <el-form-item label="User-Agent 模板">
          <el-input v-model="form.user_agent_template" placeholder="Mozilla/5.0..." />
        </el-form-item>
        <el-form-item label="Extra Headers">
          <el-input v-model="form.extra_headers_str" type="textarea" :rows="3" placeholder='{"sec-ch-ua":"\"Chromium\""}' />
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
import { useFingerprintsStore } from '@/stores/fingerprints'
import { ElMessage } from 'element-plus'
import dayjs from 'dayjs'

const store = useFingerprintsStore()
const { list, loading } = storeToRefs(store)

const showCreate = ref(false)
const editing = ref(null)
const saving = ref(false)

const defaultForm = () => ({
  name: '', tls_profile_str: '{}', http2_settings_str: '{}',
  header_order_str: '[]', user_agent_template: '', extra_headers_str: '{}'
})

const form = ref(defaultForm())

const showDialog = computed({
  get: () => showCreate.value || !!editing.value,
  set: (v) => { if (!v) { showCreate.value = false; editing.value = null } }
})

function editRow(row) {
  editing.value = row.id
  form.value = {
    name: row.name,
    tls_profile_str: JSON.stringify(row.tls_profile || {}, null, 2),
    http2_settings_str: JSON.stringify(row.http2_settings || {}, null, 2),
    header_order_str: JSON.stringify(row.header_order || [], null, 2),
    user_agent_template: row.user_agent_template || '',
    extra_headers_str: JSON.stringify(row.extra_headers || {}, null, 2)
  }
}

function parseJSON(str, fallback) {
  try { return JSON.parse(str) } catch { return fallback }
}

async function handleSave() {
  saving.value = true
  try {
    const data = {
      name: form.value.name,
      tls_profile: parseJSON(form.value.tls_profile_str, {}),
      http2_settings: parseJSON(form.value.http2_settings_str, {}),
      header_order: parseJSON(form.value.header_order_str, []),
      user_agent_template: form.value.user_agent_template,
      extra_headers: parseJSON(form.value.extra_headers_str, {})
    }
    if (editing.value) {
      await store.update(editing.value, data)
    } else {
      await store.create(data)
    }
    ElMessage.success('保存成功')
    showDialog.value = false
    form.value = defaultForm()
  } catch { /* interceptor */ } finally {
    saving.value = false
  }
}

onMounted(() => store.fetch())
</script>

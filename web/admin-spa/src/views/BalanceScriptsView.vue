<template>
  <div>
    <h2 class="text-2xl font-bold text-gray-800 dark:text-gray-100 mb-6">余额脚本管理</h2>

    <div class="bg-white dark:bg-gray-800 rounded-xl shadow border border-gray-200 dark:border-gray-700">
      <el-table :data="scripts" v-loading="loading" stripe>
        <el-table-column prop="name" label="脚本名称" min-width="160" />
        <el-table-column prop="platform" label="平台" width="120">
          <template #default="{ row }">
            <el-tag size="small">{{ row.platform }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="enabled" label="状态" width="80">
          <template #default="{ row }">
            <el-tag :type="row.enabled ? 'success' : 'info'" size="small">
              {{ row.enabled ? '启用' : '禁用' }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="description" label="描述" min-width="200" show-overflow-tooltip />
        <el-table-column label="操作" width="120">
          <template #default="{ row }">
            <el-button size="small" text type="primary" @click="editScript(row)">编辑</el-button>
          </template>
        </el-table-column>
      </el-table>
    </div>

    <el-dialog v-model="showEdit" title="编辑脚本" width="700px" destroy-on-close>
      <el-form :model="editForm" label-width="100px">
        <el-form-item label="名称">
          <el-input :model-value="editForm.name" disabled />
        </el-form-item>
        <el-form-item label="启用">
          <el-switch v-model="editForm.enabled" />
        </el-form-item>
        <el-form-item label="脚本内容">
          <el-input v-model="editForm.script" type="textarea" :rows="12" class="font-mono" />
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
import { ref, onMounted } from 'vue'
import { getBalanceScripts, getBalanceScript, updateBalanceScript } from '@/utils/api'
import { ElMessage } from 'element-plus'

const scripts = ref([])
const loading = ref(false)
const saving = ref(false)
const showEdit = ref(false)
const editForm = ref({ name: '', enabled: false, script: '' })

async function fetchScripts() {
  loading.value = true
  try {
    scripts.value = await getBalanceScripts()
  } finally {
    loading.value = false
  }
}

async function editScript(row) {
  try {
    const detail = await getBalanceScript(row.name)
    editForm.value = { ...detail }
    showEdit.value = true
  } catch {
    editForm.value = { ...row }
    showEdit.value = true
  }
}

async function handleSave() {
  saving.value = true
  try {
    await updateBalanceScript(editForm.value.name, editForm.value)
    ElMessage.success('保存成功')
    showEdit.value = false
    await fetchScripts()
  } catch (e) {
    ElMessage.error(e?.response?.data?.error || '保存失败')
  } finally {
    saving.value = false
  }
}

onMounted(fetchScripts)
</script>

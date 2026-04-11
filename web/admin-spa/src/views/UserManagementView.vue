<template>
  <div>
    <div class="flex justify-between items-center mb-6">
      <h2 class="text-2xl font-bold text-gray-800 dark:text-gray-100">用户管理</h2>
      <el-button type="primary" @click="openCreate">
        <el-icon class="mr-1"><Plus /></el-icon>创建用户
      </el-button>
    </div>

    <!-- Stats -->
    <div class="grid grid-cols-1 md:grid-cols-4 gap-4 mb-6">
      <StatCard title="总用户数" :value="users.length" color="blue" />
      <StatCard title="管理员" :value="users.filter((u) => u.role === 'admin').length" color="red" />
      <StatCard title="普通用户" :value="users.filter((u) => u.role !== 'admin').length" color="green" />
      <StatCard
        title="活跃用户"
        :value="users.filter((u) => u.status === 'active').length"
        color="purple"
      />
    </div>

    <!-- Filters -->
    <div class="flex gap-3 mb-4 flex-wrap">
      <el-input
        v-model="search"
        placeholder="搜索用户名 / 显示名"
        clearable
        style="width: 240px"
        :prefix-icon="Search"
      />
      <el-select v-model="roleFilter" placeholder="角色" clearable style="width: 120px">
        <el-option label="管理员" value="admin" />
        <el-option label="用户" value="user" />
      </el-select>
      <el-select v-model="sourceFilter" placeholder="来源" clearable style="width: 120px">
        <el-option label="本地" value="local" />
        <el-option label="LDAP" value="ldap" />
      </el-select>
    </div>

    <div class="bg-white dark:bg-gray-800 rounded-xl shadow border border-gray-200 dark:border-gray-700">
      <el-table :data="filteredUsers" v-loading="loading" stripe>
        <el-table-column prop="username" label="用户名" min-width="120" />
        <el-table-column prop="display_name" label="显示名" min-width="120" show-overflow-tooltip />
        <el-table-column prop="role" label="角色" width="100">
          <template #default="{ row }">
            <el-tag
              :type="row.role === 'admin' ? 'danger' : 'info'"
              size="small"
              class="cursor-pointer"
              @click="handleChangeRole(row)"
            >
              {{ row.role || 'user' }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column label="状态" width="80">
          <template #default="{ row }">
            <el-switch
              :model-value="row.status === 'active'"
              size="small"
              @change="handleToggleStatus(row)"
            />
          </template>
        </el-table-column>
        <el-table-column prop="source" label="来源" width="80">
          <template #default="{ row }">
            <el-tag size="small" :type="row.source === 'ldap' ? 'warning' : ''">
              {{ row.source || 'local' }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="max_keys" label="最大Key数" width="100" />
        <el-table-column label="创建时间" width="180">
          <template #default="{ row }">
            {{ dayjs(row.created_at).format('YYYY-MM-DD HH:mm') }}
          </template>
        </el-table-column>
        <el-table-column label="操作" width="180" fixed="right">
          <template #default="{ row }">
            <el-button size="small" text type="primary" @click="editRow(row)">编辑</el-button>
            <el-popconfirm
              title="确认删除此用户？"
              @confirm="handleDelete(row)"
            >
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
      :title="editing ? '编辑用户' : '创建用户'"
      width="520px"
      destroy-on-close
    >
      <el-form :model="form" label-width="100px">
        <el-form-item label="用户名" required>
          <el-input v-model="form.username" :disabled="!!editing" placeholder="用户名" />
        </el-form-item>
        <el-form-item label="显示名">
          <el-input v-model="form.display_name" placeholder="显示名称" />
        </el-form-item>
        <el-form-item label="角色">
          <el-select v-model="form.role" class="w-full">
            <el-option label="管理员" value="admin" />
            <el-option label="普通用户" value="user" />
          </el-select>
        </el-form-item>
        <el-form-item label="来源">
          <el-select v-model="form.source" class="w-full">
            <el-option label="本地" value="local" />
            <el-option label="LDAP" value="ldap" />
          </el-select>
        </el-form-item>
        <el-form-item label="最大Key数">
          <el-input-number v-model="form.max_keys" :min="1" :max="100" />
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
import { Plus, Search } from '@element-plus/icons-vue'
import { getUsers, createUser, updateUser, deleteUser, toggleUserStatus } from '@/utils/api'
import { ElMessage, ElMessageBox } from 'element-plus'
import StatCard from '@/components/common/StatCard.vue'
import dayjs from 'dayjs'

const users = ref([])
const loading = ref(false)
const saving = ref(false)
const search = ref('')
const roleFilter = ref('')
const sourceFilter = ref('')

const showCreate = ref(false)
const editing = ref(null)

const defaultForm = () => ({
  username: '',
  display_name: '',
  role: 'user',
  source: 'local',
  max_keys: 5
})
const form = ref(defaultForm())

const showDialog = computed({
  get: () => showCreate.value || !!editing.value,
  set: (v) => {
    if (!v) {
      showCreate.value = false
      editing.value = null
    }
  }
})

const filteredUsers = computed(() => {
  let list = users.value
  if (search.value) {
    const q = search.value.toLowerCase()
    list = list.filter(
      (u) =>
        u.username?.toLowerCase().includes(q) || u.display_name?.toLowerCase().includes(q)
    )
  }
  if (roleFilter.value) {
    list = list.filter((u) => (u.role || 'user') === roleFilter.value)
  }
  if (sourceFilter.value) {
    list = list.filter((u) => (u.source || 'local') === sourceFilter.value)
  }
  return list
})

async function fetchUsers() {
  loading.value = true
  try {
    users.value = await getUsers()
  } finally {
    loading.value = false
  }
}

function openCreate() {
  editing.value = null
  form.value = defaultForm()
  showCreate.value = true
}

function editRow(row) {
  editing.value = row.id
  form.value = {
    username: row.username,
    display_name: row.display_name || '',
    role: row.role || 'user',
    source: row.source || 'local',
    max_keys: row.max_keys ?? 5
  }
}

async function handleSave() {
  saving.value = true
  try {
    if (editing.value) {
      await updateUser(editing.value, {
        display_name: form.value.display_name,
        role: form.value.role,
        source: form.value.source,
        max_keys: form.value.max_keys
      })
      ElMessage.success('更新成功')
    } else {
      if (!form.value.username) {
        ElMessage.warning('用户名为必填项')
        return
      }
      await createUser(form.value)
      ElMessage.success('创建成功')
    }
    showDialog.value = false
    await fetchUsers()
  } catch (e) {
    ElMessage.error(e?.response?.data?.error?.message || '操作失败')
  } finally {
    saving.value = false
  }
}

async function handleDelete(row) {
  try {
    await deleteUser(row.id)
    ElMessage.success('已删除')
    await fetchUsers()
  } catch (e) {
    ElMessage.error(e?.response?.data?.error?.message || '删除失败')
  }
}

async function handleToggleStatus(row) {
  try {
    await toggleUserStatus(row.id)
    ElMessage.success('状态已切换')
    await fetchUsers()
  } catch (e) {
    ElMessage.error(e?.response?.data?.error?.message || '操作失败')
  }
}

async function handleChangeRole(row) {
  const newRole = row.role === 'admin' ? 'user' : 'admin'
  try {
    await ElMessageBox.confirm(
      `确认将 ${row.username} 的角色从 ${row.role || 'user'} 改为 ${newRole}？`,
      '角色变更'
    )
    await updateUser(row.id, { role: newRole })
    ElMessage.success('角色已更新')
    await fetchUsers()
  } catch {
    /* cancelled */
  }
}

onMounted(fetchUsers)
</script>

<template>
  <div>
    <h2 class="text-2xl font-bold text-gray-800 dark:text-gray-100 mb-6">用户管理</h2>

    <div class="bg-white dark:bg-gray-800 rounded-xl shadow border border-gray-200 dark:border-gray-700">
      <el-table :data="users" v-loading="loading" stripe>
        <el-table-column prop="id" label="ID" width="220" show-overflow-tooltip />
        <el-table-column prop="username" label="用户名" min-width="120" />
        <el-table-column prop="role" label="角色" width="100">
          <template #default="{ row }">
            <el-tag
              :type="row.role === 'admin' ? 'danger' : 'info'"
              size="small"
            >
              {{ row.role || 'user' }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="email" label="邮箱" min-width="160" show-overflow-tooltip />
        <el-table-column prop="last_login_at" label="最后登录" width="180" />
        <el-table-column prop="created_at" label="创建时间" width="180" />
      </el-table>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { getUsers } from '@/utils/api'

const users = ref([])
const loading = ref(false)

async function fetchUsers() {
  loading.value = true
  try {
    users.value = await getUsers()
  } finally {
    loading.value = false
  }
}

onMounted(fetchUsers)
</script>

<template>
  <div class="min-h-screen flex items-center justify-center bg-gradient-to-br from-blue-600 to-purple-700">
    <div class="w-full max-w-md p-8 bg-white dark:bg-gray-800 rounded-2xl shadow-2xl">
      <div class="text-center mb-8">
        <h1 class="text-3xl font-bold bg-gradient-to-r from-blue-600 to-purple-600 bg-clip-text text-transparent">Nexus Gate</h1>
        <p class="mt-2 text-gray-500 dark:text-gray-400">管理后台登录</p>
      </div>
      <el-form @submit.prevent="handleLogin" :model="form" :rules="rules" ref="formRef">
        <el-form-item prop="username">
          <el-input v-model="form.username" placeholder="用户名" prefix-icon="User" size="large" />
        </el-form-item>
        <el-form-item prop="password">
          <el-input v-model="form.password" type="password" placeholder="密码" prefix-icon="Lock" size="large" show-password @keyup.enter="handleLogin" />
        </el-form-item>
        <el-button type="primary" class="w-full" size="large" :loading="loading" @click="handleLogin">
          登录
        </el-button>
      </el-form>
    </div>
  </div>
</template>

<script setup>
import { ref, reactive } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { ElMessage } from 'element-plus'

const router = useRouter()
const authStore = useAuthStore()
const formRef = ref()
const loading = ref(false)

const form = reactive({
  username: '',
  password: ''
})

const rules = {
  username: [{ required: true, message: '请输入用户名', trigger: 'blur' }],
  password: [{ required: true, message: '请输入密码', trigger: 'blur' }]
}

async function handleLogin() {
  const valid = await formRef.value?.validate().catch(() => false)
  if (!valid) return
  loading.value = true
  try {
    await authStore.login(form)
    ElMessage.success('登录成功')
    router.push('/dashboard')
  } catch {
    // error handled by interceptor
  } finally {
    loading.value = false
  }
}
</script>

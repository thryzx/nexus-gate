<template>
  <div>
    <h2 class="text-2xl font-bold text-gray-800 dark:text-gray-100 mb-6">系统设置</h2>

    <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
      <!-- 基本设置 -->
      <div class="bg-white dark:bg-gray-800 rounded-xl shadow border border-gray-200 dark:border-gray-700 p-6">
        <h3 class="text-lg font-semibold text-gray-700 dark:text-gray-200 mb-4">基本设置</h3>
        <el-form label-width="160px" label-position="left">
          <el-form-item label="默认调度策略">
            <el-select v-model="settings.scheduler_strategy" placeholder="选择策略">
              <el-option label="Round Robin" value="round_robin" />
              <el-option label="最少并发" value="least_connections" />
              <el-option label="随机" value="random" />
            </el-select>
          </el-form-item>
          <el-form-item label="并发排队超时 (秒)">
            <el-input-number v-model="settings.queue_timeout" :min="1" :max="120" />
          </el-form-item>
          <el-form-item label="529 冷却时间 (秒)">
            <el-input-number v-model="settings.overload_cooldown" :min="10" :max="3600" />
          </el-form-item>
          <el-form-item label="粘性会话 TTL (秒)">
            <el-input-number v-model="settings.sticky_session_ttl" :min="60" :max="86400" />
          </el-form-item>
        </el-form>
      </div>

      <!-- 安全设置 -->
      <div class="bg-white dark:bg-gray-800 rounded-xl shadow border border-gray-200 dark:border-gray-700 p-6">
        <h3 class="text-lg font-semibold text-gray-700 dark:text-gray-200 mb-4">安全设置</h3>
        <el-form label-width="160px" label-position="left">
          <el-form-item label="API Key 前缀">
            <el-input v-model="settings.api_key_prefix" placeholder="ng_" />
          </el-form-item>
          <el-form-item label="全局速率限制 (RPM)">
            <el-input-number v-model="settings.global_rate_limit" :min="0" :max="100000" />
          </el-form-item>
          <el-form-item label="Token 自动刷新">
            <el-switch v-model="settings.auto_token_refresh" />
          </el-form-item>
        </el-form>
      </div>

      <!-- 日志设置 -->
      <div class="bg-white dark:bg-gray-800 rounded-xl shadow border border-gray-200 dark:border-gray-700 p-6">
        <h3 class="text-lg font-semibold text-gray-700 dark:text-gray-200 mb-4">日志与监控</h3>
        <el-form label-width="160px" label-position="left">
          <el-form-item label="日志级别">
            <el-select v-model="settings.log_level" placeholder="选择级别">
              <el-option label="Debug" value="debug" />
              <el-option label="Info" value="info" />
              <el-option label="Warn" value="warn" />
              <el-option label="Error" value="error" />
            </el-select>
          </el-form-item>
          <el-form-item label="Usage 记录保留 (天)">
            <el-input-number v-model="settings.usage_retention_days" :min="1" :max="365" />
          </el-form-item>
        </el-form>
      </div>

      <!-- 代理设置 -->
      <div class="bg-white dark:bg-gray-800 rounded-xl shadow border border-gray-200 dark:border-gray-700 p-6">
        <h3 class="text-lg font-semibold text-gray-700 dark:text-gray-200 mb-4">代理设置</h3>
        <el-form label-width="160px" label-position="left">
          <el-form-item label="HTTP 代理">
            <el-input v-model="settings.http_proxy" placeholder="http://proxy:port" />
          </el-form-item>
          <el-form-item label="HTTPS 代理">
            <el-input v-model="settings.https_proxy" placeholder="http://proxy:port" />
          </el-form-item>
          <el-form-item label="No Proxy">
            <el-input v-model="settings.no_proxy" placeholder="localhost,127.0.0.1" />
          </el-form-item>
        </el-form>
      </div>
    </div>

    <div class="mt-6 text-right">
      <el-button type="primary" :loading="saving" @click="handleSave">保存设置</el-button>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { getSettings, updateSetting } from '@/utils/api'
import { ElMessage } from 'element-plus'

const saving = ref(false)

const settings = ref({
  scheduler_strategy: 'round_robin',
  queue_timeout: 30,
  overload_cooldown: 300,
  sticky_session_ttl: 3600,
  api_key_prefix: 'ng_',
  global_rate_limit: 0,
  auto_token_refresh: true,
  log_level: 'info',
  usage_retention_days: 90,
  http_proxy: '',
  https_proxy: '',
  no_proxy: ''
})

async function loadSettings() {
  try {
    const res = await getSettings()
    const data = res.data?.data || res.data || {}
    Object.keys(settings.value).forEach((key) => {
      if (data[key] !== undefined) settings.value[key] = data[key]
    })
  } catch {
    /* interceptor handles */
  }
}

async function handleSave() {
  saving.value = true
  try {
    const tasks = Object.entries(settings.value).map(([key, value]) =>
      updateSetting(key, value)
    )
    await Promise.all(tasks)
    ElMessage.success('设置已保存')
  } catch {
    /* interceptor handles */
  } finally {
    saving.value = false
  }
}

onMounted(loadSettings)
</script>

<template>
  <div>
    <h2 class="text-2xl font-bold text-gray-800 dark:text-gray-100 mb-6">系统设置</h2>

    <el-tabs v-model="activeTab" type="border-card">
      <!-- OEM Branding -->
      <el-tab-pane label="品牌设置" name="oem">
        <div class="p-4">
          <el-form :model="oemForm" label-width="160px" label-position="left">
            <el-form-item label="系统名称">
              <el-input v-model="oemForm.system_name" placeholder="Nexus Gate" />
            </el-form-item>
            <el-form-item label="Logo URL">
              <el-input v-model="oemForm.logo_url" placeholder="https://..." />
            </el-form-item>
            <el-form-item label="主题色">
              <el-color-picker v-model="oemForm.primary_color" />
            </el-form-item>
            <el-form-item label="版权信息">
              <el-input v-model="oemForm.copyright" />
            </el-form-item>
            <el-form-item label="公告内容">
              <el-input v-model="oemForm.announcement" type="textarea" :rows="3" />
            </el-form-item>
          </el-form>
          <div class="text-right mt-4">
            <el-button type="primary" :loading="saving" @click="saveOem">保存品牌设置</el-button>
          </div>
        </div>
      </el-tab-pane>

      <!-- General Config -->
      <el-tab-pane label="基本设置" name="general">
        <div class="p-4">
          <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
            <div>
              <h3 class="text-base font-semibold text-gray-700 dark:text-gray-200 mb-3">调度设置</h3>
              <el-form :model="settings" label-width="160px" label-position="left">
                <el-form-item label="默认调度策略">
                  <el-select v-model="settings.scheduler_strategy" class="w-full">
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
            <div>
              <h3 class="text-base font-semibold text-gray-700 dark:text-gray-200 mb-3">安全设置</h3>
              <el-form :model="settings" label-width="160px" label-position="left">
                <el-form-item label="API Key 前缀">
                  <el-input v-model="settings.api_key_prefix" placeholder="ng_" />
                </el-form-item>
                <el-form-item label="全局速率限制 (RPM)">
                  <el-input-number v-model="settings.global_rate_limit" :min="0" :max="100000" />
                </el-form-item>
                <el-form-item label="Token 自动刷新">
                  <el-switch v-model="settings.auto_token_refresh" />
                </el-form-item>
                <el-form-item label="日志级别">
                  <el-select v-model="settings.log_level" class="w-full">
                    <el-option label="Debug" value="debug" />
                    <el-option label="Info" value="info" />
                    <el-option label="Warn" value="warn" />
                    <el-option label="Error" value="error" />
                  </el-select>
                </el-form-item>
              </el-form>
            </div>
          </div>
          <div class="text-right mt-4">
            <el-button type="primary" :loading="saving" @click="saveGeneral">保存设置</el-button>
          </div>
        </div>
      </el-tab-pane>

      <!-- Webhook -->
      <el-tab-pane label="Webhook 通知" name="webhook">
        <div class="p-4">
          <div v-if="webhookConfig">
            <el-form :model="webhookConfig" label-width="120px" label-position="left">
              <el-form-item label="启用 Webhook">
                <el-switch v-model="webhookConfig.enabled" />
              </el-form-item>
              <el-form-item label="通知事件">
                <el-checkbox-group v-model="webhookConfig.events" v-if="webhookConfig.events">
                  <el-checkbox v-for="e in availableEvents" :key="e" :value="e">{{ e }}</el-checkbox>
                </el-checkbox-group>
              </el-form-item>
            </el-form>

            <h4 class="text-sm font-semibold mt-4 mb-2 text-gray-700 dark:text-gray-300">平台配置</h4>
            <el-table :data="webhookConfig.platforms || []" stripe size="small">
              <el-table-column prop="platform" label="平台" />
              <el-table-column prop="webhook_url" label="URL" show-overflow-tooltip />
              <el-table-column label="操作" width="150">
                <template #default="{ row }">
                  <el-button size="small" text type="danger" @click="removeWebhookPlatform(row.id)">删除</el-button>
                </template>
              </el-table-column>
            </el-table>

            <div class="mt-3 flex gap-2">
              <el-input v-model="newPlatform.platform" placeholder="平台名称" style="width: 150px" />
              <el-input v-model="newPlatform.webhook_url" placeholder="Webhook URL" />
              <el-button type="primary" size="small" @click="addPlatform">添加</el-button>
            </div>
          </div>
          <div class="text-right mt-4">
            <el-button @click="testWebhookNow" :loading="testing">测试通知</el-button>
            <el-button type="primary" :loading="saving" @click="saveWebhook">保存 Webhook</el-button>
          </div>
        </div>
      </el-tab-pane>

      <!-- Service Rates -->
      <el-tab-pane label="服务费率" name="rates">
        <div class="p-4">
          <el-table :data="ratesList" stripe>
            <el-table-column prop="service" label="服务" />
            <el-table-column label="费率" width="150">
              <template #default="{ row }">
                <el-input-number v-model="row.rate" :min="0" :max="100" :step="0.1" size="small" />
              </template>
            </el-table-column>
          </el-table>
          <div class="text-right mt-4">
            <el-button type="primary" :loading="saving" @click="saveRates">保存费率</el-button>
          </div>
        </div>
      </el-tab-pane>

      <!-- Model Pricing -->
      <el-tab-pane label="模型定价" name="pricing">
        <div class="p-4">
          <div class="flex justify-between items-center mb-4">
            <span class="text-gray-500 dark:text-gray-400 text-sm">模型定价用于成本计算</span>
            <el-button size="small" :loading="refreshing" @click="handleRefreshPricing">刷新定价</el-button>
          </div>
          <el-table :data="modelPricing" stripe size="small">
            <el-table-column prop="model" label="模型" min-width="200" />
            <el-table-column prop="input_price" label="输入价格 ($/1M)" width="160" />
            <el-table-column prop="output_price" label="输出价格 ($/1M)" width="160" />
            <el-table-column prop="source" label="来源" width="100" />
          </el-table>
        </div>
      </el-tab-pane>

      <!-- Proxy -->
      <el-tab-pane label="代理设置" name="proxy">
        <div class="p-4">
          <el-form :model="settings" label-width="160px" label-position="left">
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
          <div class="text-right mt-4">
            <el-button type="primary" :loading="saving" @click="saveGeneral">保存代理设置</el-button>
          </div>
        </div>
      </el-tab-pane>
    </el-tabs>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import { useSettingsStore } from '@/stores/settings'
import { getSettings, updateSetting } from '@/utils/api'
import { ElMessage } from 'element-plus'

const settingsStore = useSettingsStore()
const activeTab = ref('oem')
const saving = ref(false)
const testing = ref(false)
const refreshing = ref(false)

// OEM
const oemForm = ref({
  system_name: '',
  logo_url: '',
  primary_color: '#409EFF',
  copyright: '',
  announcement: ''
})

// General settings
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

// Webhook
const webhookConfig = computed(() => settingsStore.webhookConfig)
const availableEvents = [
  'account_error', 'account_blocked', 'token_refresh_failed',
  'high_usage', 'key_expired', 'system_alert'
]
const newPlatform = ref({ platform: '', webhook_url: '' })

// Rates
const ratesList = ref([])

// Pricing
const modelPricing = computed(() => settingsStore.modelPricing || [])

async function loadAll() {
  await settingsStore.fetchAll()

  // Load OEM
  const oem = settingsStore.oemSettings
  if (oem) {
    Object.keys(oemForm.value).forEach((k) => {
      if (oem[k] !== undefined) oemForm.value[k] = oem[k]
    })
  }

  // Load general
  try {
    const res = await getSettings()
    const data = res.data?.data || res.data || res || {}
    Object.keys(settings.value).forEach((k) => {
      if (data[k] !== undefined) settings.value[k] = data[k]
    })
  } catch { /* ignore */ }

  // Load rates
  const rates = settingsStore.serviceRates
  if (rates && typeof rates === 'object') {
    ratesList.value = Object.entries(rates).map(([service, rate]) => ({ service, rate }))
  }
}

async function saveOem() {
  saving.value = true
  try {
    await settingsStore.updateOem(oemForm.value)
    ElMessage.success('品牌设置已保存')
  } catch { ElMessage.error('保存失败') }
  finally { saving.value = false }
}

async function saveGeneral() {
  saving.value = true
  try {
    await Promise.all(
      Object.entries(settings.value).map(([key, value]) =>
        updateSetting({ key, value })
      )
    )
    ElMessage.success('设置已保存')
  } catch { ElMessage.error('保存失败') }
  finally { saving.value = false }
}

async function saveWebhook() {
  saving.value = true
  try {
    await settingsStore.updateWebhook(webhookConfig.value)
    ElMessage.success('Webhook 设置已保存')
  } catch { ElMessage.error('保存失败') }
  finally { saving.value = false }
}

async function addPlatform() {
  if (!newPlatform.value.platform || !newPlatform.value.webhook_url) return
  try {
    await settingsStore.addWebhookPlatform(newPlatform.value)
    newPlatform.value = { platform: '', webhook_url: '' }
    ElMessage.success('平台已添加')
  } catch { ElMessage.error('添加失败') }
}

async function removeWebhookPlatform(id) {
  await settingsStore.removeWebhookPlatform(id)
  ElMessage.success('已删除')
}

async function testWebhookNow() {
  testing.value = true
  try {
    await settingsStore.testWebhookNotification({ type: 'test' })
    ElMessage.success('测试通知已发送')
  } catch { ElMessage.error('发送失败') }
  finally { testing.value = false }
}

async function saveRates() {
  saving.value = true
  try {
    const data = {}
    ratesList.value.forEach((r) => { data[r.service] = r.rate })
    await settingsStore.updateRates(data)
    ElMessage.success('费率已保存')
  } catch { ElMessage.error('保存失败') }
  finally { saving.value = false }
}

async function handleRefreshPricing() {
  refreshing.value = true
  try {
    await settingsStore.refreshPricing()
    ElMessage.success('定价已刷新')
  } catch { ElMessage.error('刷新失败') }
  finally { refreshing.value = false }
}

onMounted(loadAll)
</script>

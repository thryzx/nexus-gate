<template>
  <div>
    <h2 class="mb-6 text-2xl font-bold text-gray-800 dark:text-gray-100">系统设置</h2>

    <el-tabs v-model="activeTab" type="border-card">
      <!-- OEM Branding -->
      <el-tab-pane label="品牌设置" name="oem">
        <div class="p-4">
          <el-form :model="oemForm" label-width="160px" label-position="left">
            <el-form-item label="系统名称">
              <el-input v-model="oemForm.system_name" placeholder="Nexus Gate" />
            </el-form-item>
            <el-form-item label="Logo URL">
              <el-input v-model="oemForm.logo_url" placeholder="https://...">
                <template #append v-if="oemForm.logo_url">
                  <img :src="oemForm.logo_url" class="h-6 w-6 object-contain" alt="logo" />
                </template>
              </el-input>
            </el-form-item>
            <el-form-item label="Favicon URL">
              <el-input v-model="oemForm.favicon_url" placeholder="https://..." />
            </el-form-item>
            <el-form-item label="主题色">
              <el-color-picker v-model="oemForm.primary_color" show-alpha />
              <span class="ml-3 text-sm text-gray-400">{{ oemForm.primary_color }}</span>
            </el-form-item>
            <el-form-item label="版权信息">
              <el-input v-model="oemForm.copyright" />
            </el-form-item>
            <el-form-item label="公告内容">
              <el-input v-model="oemForm.announcement" type="textarea" :rows="3" />
            </el-form-item>
            <el-form-item label="自定义 Footer HTML">
              <el-input v-model="oemForm.footer_html" type="textarea" :rows="2" placeholder="<a href='...'>链接</a>" />
            </el-form-item>
          </el-form>
          <div class="mt-4 text-right">
            <el-button type="primary" :loading="saving" @click="saveOem">保存品牌设置</el-button>
          </div>
        </div>
      </el-tab-pane>

      <!-- General Config -->
      <el-tab-pane label="基本设置" name="general">
        <div class="p-4">
          <div class="grid grid-cols-1 gap-6 lg:grid-cols-2">
            <div>
              <h3 class="mb-3 text-base font-semibold text-gray-700 dark:text-gray-200">调度设置</h3>
              <el-form :model="settings" label-width="160px" label-position="left">
                <el-form-item label="默认调度策略">
                  <el-select v-model="settings.scheduler_strategy" class="w-full">
                    <el-option label="Round Robin" value="round_robin" />
                    <el-option label="最少并发" value="least_connections" />
                    <el-option label="随机" value="random" />
                    <el-option label="加权随机" value="weighted_random" />
                    <el-option label="优先级" value="priority" />
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
                <el-form-item label="最大并发/账户">
                  <el-input-number v-model="settings.max_concurrent_per_account" :min="1" :max="100" />
                </el-form-item>
              </el-form>
            </div>
            <div>
              <h3 class="mb-3 text-base font-semibold text-gray-700 dark:text-gray-200">安全设置</h3>
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
                    <el-option label="Trace" value="trace" />
                    <el-option label="Debug" value="debug" />
                    <el-option label="Info" value="info" />
                    <el-option label="Warn" value="warn" />
                    <el-option label="Error" value="error" />
                  </el-select>
                </el-form-item>
                <el-form-item label="Usage 保留天数">
                  <el-input-number v-model="settings.usage_retention_days" :min="7" :max="365" />
                </el-form-item>
              </el-form>
            </div>
          </div>

          <el-divider />

          <h3 class="mb-3 text-base font-semibold text-gray-700 dark:text-gray-200">转发设置</h3>
          <div class="grid grid-cols-1 gap-6 lg:grid-cols-2">
            <el-form :model="settings" label-width="160px" label-position="left">
              <el-form-item label="请求超时 (秒)">
                <el-input-number v-model="settings.request_timeout" :min="5" :max="600" />
              </el-form-item>
              <el-form-item label="自动重试次数">
                <el-input-number v-model="settings.retry_count" :min="0" :max="5" />
              </el-form-item>
            </el-form>
            <el-form :model="settings" label-width="160px" label-position="left">
              <el-form-item label="流式响应">
                <el-switch v-model="settings.enable_streaming" />
              </el-form-item>
              <el-form-item label="启用缓存">
                <el-switch v-model="settings.enable_cache" />
              </el-form-item>
            </el-form>
          </div>

          <div class="mt-4 text-right">
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
                  <el-checkbox v-for="e in availableEvents" :key="e" :value="e">{{ eventLabels[e] || e }}</el-checkbox>
                </el-checkbox-group>
              </el-form-item>
            </el-form>

            <el-divider />

            <h4 class="mb-3 text-sm font-semibold text-gray-700 dark:text-gray-300">通知平台</h4>
            <el-table :data="webhookConfig.platforms || []" stripe size="small">
              <el-table-column prop="platform" label="平台" width="120">
                <template #default="{ row }">
                  <el-tag size="small" :type="platformTypes[row.platform] || ''">{{ row.platform }}</el-tag>
                </template>
              </el-table-column>
              <el-table-column prop="webhook_url" label="URL" show-overflow-tooltip />
              <el-table-column prop="secret" label="Secret" width="120" show-overflow-tooltip>
                <template #default="{ row }">{{ row.secret ? '••••••' : '-' }}</template>
              </el-table-column>
              <el-table-column label="操作" width="100">
                <template #default="{ row }">
                  <el-button size="small" text type="danger" @click="removeWebhookPlatform(row.id)">删除</el-button>
                </template>
              </el-table-column>
            </el-table>

            <div class="mt-3 rounded-lg border border-gray-200 p-3 dark:border-gray-700">
              <p class="mb-2 text-xs font-semibold text-gray-500 dark:text-gray-400">添加通知平台</p>
              <div class="flex flex-wrap gap-2">
                <el-select v-model="newPlatform.platform" placeholder="选择平台" style="width: 160px" size="small">
                  <el-option v-for="p in webhookPlatformOptions" :key="p.value" :value="p.value" :label="p.label" />
                </el-select>
                <el-input v-model="newPlatform.webhook_url" placeholder="Webhook URL" size="small" class="flex-1" style="min-width: 300px" />
                <el-input v-model="newPlatform.secret" placeholder="Secret (可选)" size="small" style="width: 160px" show-password />
                <el-button type="primary" size="small" @click="addPlatform" :disabled="!newPlatform.platform || !newPlatform.webhook_url">添加</el-button>
              </div>
            </div>
          </div>
          <div class="mt-4 flex justify-end gap-2">
            <el-button @click="testWebhookNow" :loading="testing">
              <el-icon class="mr-1"><Bell /></el-icon>测试通知
            </el-button>
            <el-button type="primary" :loading="saving" @click="saveWebhook">保存 Webhook</el-button>
          </div>
        </div>
      </el-tab-pane>

      <!-- Service Rates -->
      <el-tab-pane label="服务费率" name="rates">
        <div class="p-4">
          <div class="mb-4 flex items-center justify-between">
            <p class="text-sm text-gray-500 dark:text-gray-400">配置各平台服务的价格倍率</p>
            <el-button size="small" @click="addRate">添加费率</el-button>
          </div>
          <el-table :data="ratesList" stripe size="small">
            <el-table-column prop="service" label="服务" min-width="200">
              <template #default="{ row, $index }">
                <el-input v-if="row._editing" v-model="ratesList[$index].service" size="small" />
                <span v-else>{{ row.service }}</span>
              </template>
            </el-table-column>
            <el-table-column label="费率" width="150">
              <template #default="{ row }">
                <el-input-number v-model="row.rate" :min="0" :max="100" :step="0.1" size="small" />
              </template>
            </el-table-column>
            <el-table-column label="操作" width="80">
              <template #default="{ row, $index }">
                <el-button text size="small" type="danger" @click="ratesList.splice($index, 1)">删除</el-button>
              </template>
            </el-table-column>
          </el-table>
          <div class="mt-4 text-right">
            <el-button type="primary" :loading="saving" @click="saveRates">保存费率</el-button>
          </div>
        </div>
      </el-tab-pane>

      <!-- Model Pricing -->
      <el-tab-pane label="模型定价" name="pricing">
        <div class="p-4">
          <div class="mb-4 flex items-center justify-between">
            <div>
              <span class="text-sm text-gray-500 dark:text-gray-400">模型定价用于成本计算</span>
              <el-tag size="small" type="info" class="ml-2">{{ modelPricing.length }} 个模型</el-tag>
            </div>
            <el-button size="small" :loading="refreshing" @click="handleRefreshPricing">
              <el-icon class="mr-1"><Refresh /></el-icon>刷新定价
            </el-button>
          </div>
          <el-input v-model="pricingSearch" placeholder="搜索模型名..." clearable class="mb-3" style="max-width: 300px" />
          <el-table :data="filteredPricing" stripe size="small" max-height="400">
            <el-table-column prop="model" label="模型" min-width="200" show-overflow-tooltip sortable />
            <el-table-column prop="input_price" label="输入价格 ($/1M tokens)" width="180" sortable />
            <el-table-column prop="output_price" label="输出价格 ($/1M tokens)" width="180" sortable />
            <el-table-column prop="source" label="来源" width="100">
              <template #default="{ row }">
                <el-tag size="small" :type="row.source === 'manual' ? 'warning' : 'info'">{{ row.source || 'auto' }}</el-tag>
              </template>
            </el-table-column>
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
            <el-form-item label="SOCKS5 代理">
              <el-input v-model="settings.socks5_proxy" placeholder="socks5://proxy:port" />
            </el-form-item>
            <el-form-item label="No Proxy">
              <el-input v-model="settings.no_proxy" placeholder="localhost,127.0.0.1" />
            </el-form-item>
            <el-form-item label="代理认证用户名">
              <el-input v-model="settings.proxy_username" placeholder="可选" />
            </el-form-item>
            <el-form-item label="代理认证密码">
              <el-input v-model="settings.proxy_password" placeholder="可选" show-password />
            </el-form-item>
          </el-form>
          <div class="mt-4 text-right">
            <el-button @click="testProxy" :loading="proxyTesting">测试连接</el-button>
            <el-button type="primary" :loading="saving" @click="saveGeneral">保存代理设置</el-button>
          </div>
        </div>
      </el-tab-pane>

      <!-- System Info -->
      <el-tab-pane label="系统信息" name="sysinfo">
        <div class="p-4">
          <div class="grid grid-cols-1 gap-4 lg:grid-cols-2">
            <div class="rounded-lg border border-gray-200 p-4 dark:border-gray-700">
              <h4 class="mb-2 text-sm font-semibold text-gray-700 dark:text-gray-300">运行状态</h4>
              <el-descriptions :column="1" border size="small">
                <el-descriptions-item label="版本">{{ sysInfo.version || '-' }}</el-descriptions-item>
                <el-descriptions-item label="运行时间">{{ sysInfo.uptime || '-' }}</el-descriptions-item>
                <el-descriptions-item label="数据库">
                  <el-tag :type="sysInfo.db_ok ? 'success' : 'danger'" size="small">{{ sysInfo.db_ok ? '正常' : '异常' }}</el-tag>
                </el-descriptions-item>
                <el-descriptions-item label="Redis">
                  <el-tag :type="sysInfo.redis_ok ? 'success' : 'danger'" size="small">{{ sysInfo.redis_ok ? '正常' : '异常' }}</el-tag>
                </el-descriptions-item>
              </el-descriptions>
            </div>
            <div class="rounded-lg border border-gray-200 p-4 dark:border-gray-700">
              <h4 class="mb-2 text-sm font-semibold text-gray-700 dark:text-gray-300">环境变量</h4>
              <el-descriptions :column="1" border size="small">
                <el-descriptions-item label="LOG_LEVEL">{{ settings.log_level }}</el-descriptions-item>
                <el-descriptions-item label="API_KEY_PREFIX">{{ settings.api_key_prefix }}</el-descriptions-item>
                <el-descriptions-item label="调度策略">{{ settings.scheduler_strategy }}</el-descriptions-item>
                <el-descriptions-item label="Redis">已连接</el-descriptions-item>
              </el-descriptions>
            </div>
          </div>
        </div>
      </el-tab-pane>
    </el-tabs>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import { Refresh, Bell } from '@element-plus/icons-vue'
import { useSettingsStore } from '@/stores/settings'
import { getSettings, updateSetting, getHealth } from '@/utils/api'
import { ElMessage } from 'element-plus'

const settingsStore = useSettingsStore()
const activeTab = ref('oem')
const saving = ref(false)
const testing = ref(false)
const refreshing = ref(false)
const proxyTesting = ref(false)
const pricingSearch = ref('')

// OEM
const oemForm = ref({
  system_name: '', logo_url: '', favicon_url: '', primary_color: '#409EFF',
  copyright: '', announcement: '', footer_html: ''
})

// General settings
const settings = ref({
  scheduler_strategy: 'round_robin', queue_timeout: 30, overload_cooldown: 300,
  sticky_session_ttl: 3600, max_concurrent_per_account: 5, api_key_prefix: 'ng_',
  global_rate_limit: 0, auto_token_refresh: true, log_level: 'info',
  usage_retention_days: 90, request_timeout: 120, retry_count: 1,
  enable_streaming: true, enable_cache: false,
  http_proxy: '', https_proxy: '', socks5_proxy: '', no_proxy: '',
  proxy_username: '', proxy_password: ''
})

// Webhook
const webhookConfig = computed(() => settingsStore.webhookConfig)
const availableEvents = ['account_error','account_blocked','token_refresh_failed','high_usage','key_expired','system_alert','overload_detected','balance_low']
const eventLabels = {
  account_error: '账户错误', account_blocked: '账户封禁', token_refresh_failed: 'Token刷新失败',
  high_usage: '高用量告警', key_expired: 'Key过期', system_alert: '系统告警',
  overload_detected: '过载检测', balance_low: '余额不足'
}
const platformTypes = { wechat: 'success', dingtalk: 'warning', telegram: 'info', slack: '', feishu: 'success', discord: '' }
const webhookPlatformOptions = [
  { value: 'generic', label: '通用 Webhook' },
  { value: 'wechat', label: '企业微信' },
  { value: 'dingtalk', label: '钉钉' },
  { value: 'feishu', label: '飞书' },
  { value: 'telegram', label: 'Telegram' },
  { value: 'slack', label: 'Slack' },
  { value: 'discord', label: 'Discord' }
]
const newPlatform = ref({ platform: '', webhook_url: '', secret: '' })

// Rates
const ratesList = ref([])

// Pricing
const modelPricing = computed(() => settingsStore.modelPricing || [])
const filteredPricing = computed(() => {
  if (!pricingSearch.value) return modelPricing.value
  const q = pricingSearch.value.toLowerCase()
  return modelPricing.value.filter((m) => m.model?.toLowerCase().includes(q))
})

// System Info
const sysInfo = ref({ version: '', uptime: '', db_ok: false, redis_ok: false })

async function loadAll() {
  await settingsStore.fetchAll()
  const oem = settingsStore.oemSettings
  if (oem) Object.keys(oemForm.value).forEach((k) => { if (oem[k] !== undefined) oemForm.value[k] = oem[k] })
  try {
    const res = await getSettings()
    const data = res.data?.data || res.data || res || {}
    Object.keys(settings.value).forEach((k) => { if (data[k] !== undefined) settings.value[k] = data[k] })
  } catch { /* ignore */ }
  const rates = settingsStore.serviceRates
  if (rates && typeof rates === 'object') ratesList.value = Object.entries(rates).map(([service, rate]) => ({ service, rate }))
  try {
    const h = await getHealth()
    sysInfo.value = { version: h.version || h.data?.version || '-', uptime: h.uptime || '-', db_ok: h.database === 'ok' || h.db_ok || false, redis_ok: h.redis === 'ok' || h.redis_ok || false }
  } catch { /* ignore */ }
}

async function saveOem() {
  saving.value = true
  try { await settingsStore.updateOem(oemForm.value); ElMessage.success('品牌设置已保存') }
  catch { ElMessage.error('保存失败') } finally { saving.value = false }
}

async function saveGeneral() {
  saving.value = true
  try {
    await Promise.all(Object.entries(settings.value).map(([key, value]) => updateSetting({ key, value })))
    ElMessage.success('设置已保存')
  } catch { ElMessage.error('保存失败') } finally { saving.value = false }
}

async function saveWebhook() {
  saving.value = true
  try { await settingsStore.updateWebhook(webhookConfig.value); ElMessage.success('Webhook 设置已保存') }
  catch { ElMessage.error('保存失败') } finally { saving.value = false }
}

async function addPlatform() {
  if (!newPlatform.value.platform || !newPlatform.value.webhook_url) return
  try {
    await settingsStore.addWebhookPlatform(newPlatform.value)
    newPlatform.value = { platform: '', webhook_url: '', secret: '' }
    ElMessage.success('平台已添加')
  } catch { ElMessage.error('添加失败') }
}

async function removeWebhookPlatform(id) {
  await settingsStore.removeWebhookPlatform(id); ElMessage.success('已删除')
}

async function testWebhookNow() {
  testing.value = true
  try { await settingsStore.testWebhookNotification({ type: 'test' }); ElMessage.success('测试通知已发送') }
  catch { ElMessage.error('发送失败') } finally { testing.value = false }
}

function addRate() { ratesList.value.push({ service: '', rate: 1, _editing: true }) }

async function saveRates() {
  saving.value = true
  try {
    const data = {}
    ratesList.value.forEach((r) => { if (r.service) data[r.service] = r.rate })
    await settingsStore.updateRates(data); ElMessage.success('费率已保存')
  } catch { ElMessage.error('保存失败') } finally { saving.value = false }
}

async function handleRefreshPricing() {
  refreshing.value = true
  try { await settingsStore.refreshPricing(); ElMessage.success('定价已刷新') }
  catch { ElMessage.error('刷新失败') } finally { refreshing.value = false }
}

async function testProxy() {
  proxyTesting.value = true
  try { ElMessage.info('代理测试功能将在后续版本中支持') }
  finally { proxyTesting.value = false }
}

onMounted(loadAll)
</script>

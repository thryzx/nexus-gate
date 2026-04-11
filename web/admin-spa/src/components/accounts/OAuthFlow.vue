<template>
  <div class="space-y-6">
    <!-- Platform Selection -->
    <div class="flex gap-2 mb-4">
      <el-radio-group v-model="selectedPlatform" size="default">
        <el-radio-button
          v-for="p in oauthPlatforms"
          :key="p.key"
          :value="p.key"
        >
          {{ p.label }}
        </el-radio-button>
      </el-radio-group>
    </div>

    <!-- Claude OAuth -->
    <template v-if="selectedPlatform === 'claude'">
      <el-tabs v-model="claudeMethod" type="card">
        <!-- Manual OAuth -->
        <el-tab-pane label="手动 OAuth" name="manual">
          <div class="space-y-4 p-4">
            <el-steps :active="claudeStep" align-center class="mb-6">
              <el-step title="获取链接" />
              <el-step title="浏览器认证" />
              <el-step title="输入代码" />
            </el-steps>

            <template v-if="claudeStep === 0">
              <el-form label-width="100px">
                <el-form-item label="名称">
                  <el-input v-model="claudeForm.name" placeholder="账户名称（可选）" />
                </el-form-item>
                <el-form-item label="代理 URL">
                  <el-input v-model="claudeForm.proxy_url" placeholder="socks5://127.0.0.1:1080" />
                </el-form-item>
              </el-form>
              <el-button type="primary" :loading="loading" @click="claudeGenerateUrl">
                生成认证链接
              </el-button>
            </template>

            <template v-if="claudeStep === 1">
              <el-alert type="warning" :closable="false" class="mb-4">
                请在浏览器中打开以下链接完成认证，然后将获取的 code 填入下方。
              </el-alert>
              <div class="bg-gray-50 dark:bg-gray-700 p-3 rounded break-all text-sm font-mono">
                {{ authUrl }}
              </div>
              <div class="flex gap-2 mt-3">
                <el-button size="small" @click="copyUrl">复制链接</el-button>
                <el-button size="small" type="primary" @click="claudeStep = 2">下一步</el-button>
              </div>
            </template>

            <template v-if="claudeStep === 2">
              <el-form label-width="100px">
                <el-form-item label="Auth Code" required>
                  <el-input v-model="claudeForm.code" placeholder="粘贴认证代码" type="textarea" :rows="3" />
                </el-form-item>
              </el-form>
              <el-button type="primary" :loading="loading" @click="claudeExchange">
                完成认证
              </el-button>
            </template>
          </div>
        </el-tab-pane>

        <!-- Cookie Auth -->
        <el-tab-pane label="Cookie 认证" name="cookie">
          <div class="space-y-4 p-4">
            <el-alert type="info" :closable="false" class="mb-4">
              输入 Claude.ai 的 sessionKey (cookie) 进行批量自动认证。每行一个 sessionKey。
            </el-alert>
            <el-form label-width="120px">
              <el-form-item label="Session Keys" required>
                <el-input
                  v-model="cookieForm.sessionKeys"
                  type="textarea"
                  :rows="6"
                  placeholder="sk-ant-sid01-...&#10;sk-ant-sid01-..."
                />
              </el-form-item>
              <el-form-item label="代理 URL">
                <el-input v-model="cookieForm.proxy_url" placeholder="socks5://127.0.0.1:1080" />
              </el-form-item>
            </el-form>
            <el-button type="primary" :loading="loading" @click="claudeCookieAuth">
              批量认证
            </el-button>

            <!-- Batch results -->
            <div v-if="batchResults.length > 0" class="mt-4">
              <h4 class="text-sm font-semibold mb-2 text-gray-700 dark:text-gray-300">认证结果：</h4>
              <div v-for="(r, i) in batchResults" :key="i" class="flex items-center gap-2 py-1">
                <el-tag :type="r.success ? 'success' : 'danger'" size="small">
                  {{ r.success ? '成功' : '失败' }}
                </el-tag>
                <span class="text-sm text-gray-600 dark:text-gray-400 truncate">
                  {{ r.name || r.sessionKey?.slice(0, 20) + '...' }}
                </span>
                <span v-if="r.error" class="text-xs text-red-400">{{ r.error }}</span>
              </div>
            </div>
          </div>
        </el-tab-pane>

        <!-- Setup Token -->
        <el-tab-pane label="Setup Token" name="setup-token">
          <div class="space-y-4 p-4">
            <el-steps :active="setupTokenStep" align-center class="mb-6">
              <el-step title="获取链接" />
              <el-step title="获取 Token" />
              <el-step title="完成认证" />
            </el-steps>

            <template v-if="setupTokenStep === 0">
              <el-form label-width="100px">
                <el-form-item label="名称">
                  <el-input v-model="setupTokenForm.name" placeholder="账户名称（可选）" />
                </el-form-item>
                <el-form-item label="代理 URL">
                  <el-input v-model="setupTokenForm.proxy_url" placeholder="socks5://127.0.0.1:1080" />
                </el-form-item>
              </el-form>
              <el-button type="primary" :loading="loading" @click="claudeGenerateSetupTokenUrl">
                生成 Setup Token 链接
              </el-button>
            </template>

            <template v-if="setupTokenStep === 1">
              <el-alert type="warning" :closable="false" class="mb-4">
                请在浏览器中打开以下链接获取 Setup Token。
              </el-alert>
              <div class="bg-gray-50 dark:bg-gray-700 p-3 rounded break-all text-sm font-mono">
                {{ authUrl }}
              </div>
              <div class="flex gap-2 mt-3">
                <el-button size="small" @click="copyUrl">复制链接</el-button>
                <el-button size="small" type="primary" @click="setupTokenStep = 2">下一步</el-button>
              </div>
            </template>

            <template v-if="setupTokenStep === 2">
              <el-form label-width="120px">
                <el-form-item label="Setup Token" required>
                  <el-input v-model="setupTokenForm.code" placeholder="粘贴 setup token" type="textarea" :rows="3" />
                </el-form-item>
              </el-form>
              <el-button type="primary" :loading="loading" @click="claudeExchangeSetupToken">
                完成认证
              </el-button>
            </template>
          </div>
        </el-tab-pane>
      </el-tabs>
    </template>

    <!-- Gemini OAuth -->
    <template v-if="selectedPlatform === 'gemini'">
      <div class="space-y-4 p-4">
        <el-steps :active="geminiStep" align-center class="mb-6">
          <el-step title="获取链接" />
          <el-step title="浏览器认证" />
          <el-step title="输入代码" />
        </el-steps>

        <template v-if="geminiStep === 0">
          <el-form label-width="100px">
            <el-form-item label="名称">
              <el-input v-model="geminiForm.name" placeholder="账户名称（可选）" />
            </el-form-item>
            <el-form-item label="代理 URL">
              <el-input v-model="geminiForm.proxy_url" placeholder="socks5://127.0.0.1:1080" />
            </el-form-item>
          </el-form>
          <el-button type="success" :loading="loading" @click="geminiGenerateUrl">
            生成 Gemini 认证链接
          </el-button>
        </template>

        <template v-if="geminiStep === 1">
          <el-alert type="success" :closable="false" class="mb-4">
            请在浏览器中打开以下链接完成 Google 认证。
          </el-alert>
          <div class="bg-green-50 dark:bg-green-900/30 p-3 rounded break-all text-sm font-mono">
            {{ authUrl }}
          </div>
          <div class="flex gap-2 mt-3">
            <el-button size="small" @click="copyUrl">复制链接</el-button>
            <el-button size="small" type="success" @click="geminiStep = 2">下一步</el-button>
          </div>
        </template>

        <template v-if="geminiStep === 2">
          <el-form label-width="100px">
            <el-form-item label="Auth Code" required>
              <el-input v-model="geminiForm.code" placeholder="粘贴认证代码" type="textarea" :rows="3" />
            </el-form-item>
          </el-form>
          <el-button type="success" :loading="loading" @click="geminiExchange">
            完成认证
          </el-button>
        </template>
      </div>
    </template>

    <!-- OpenAI OAuth -->
    <template v-if="selectedPlatform === 'openai'">
      <div class="space-y-4 p-4">
        <el-steps :active="openaiStep" align-center class="mb-6">
          <el-step title="获取链接" />
          <el-step title="浏览器认证" />
          <el-step title="输入代码" />
        </el-steps>

        <template v-if="openaiStep === 0">
          <el-form label-width="100px">
            <el-form-item label="名称">
              <el-input v-model="openaiForm.name" placeholder="账户名称（可选）" />
            </el-form-item>
            <el-form-item label="代理 URL">
              <el-input v-model="openaiForm.proxy_url" placeholder="socks5://127.0.0.1:1080" />
            </el-form-item>
          </el-form>
          <el-button type="primary" :loading="loading" @click="openaiGenerateUrl">
            生成 OpenAI 认证链接
          </el-button>
        </template>

        <template v-if="openaiStep === 1">
          <el-alert type="info" :closable="false" class="mb-4">
            请在浏览器中打开以下链接完成 OpenAI 认证。
          </el-alert>
          <div class="bg-blue-50 dark:bg-blue-900/30 p-3 rounded break-all text-sm font-mono">
            {{ authUrl }}
          </div>
          <div class="flex gap-2 mt-3">
            <el-button size="small" @click="copyUrl">复制链接</el-button>
            <el-button size="small" type="primary" @click="openaiStep = 2">下一步</el-button>
          </div>
        </template>

        <template v-if="openaiStep === 2">
          <el-form label-width="100px">
            <el-form-item label="Auth Code" required>
              <el-input v-model="openaiForm.code" placeholder="粘贴认证代码" type="textarea" :rows="3" />
            </el-form-item>
          </el-form>
          <el-button type="primary" :loading="loading" @click="openaiExchange">
            完成认证
          </el-button>
        </template>
      </div>
    </template>

    <!-- Droid OAuth -->
    <template v-if="selectedPlatform === 'droid'">
      <div class="space-y-4 p-4">
        <el-steps :active="droidStep" align-center class="mb-6">
          <el-step title="获取链接" />
          <el-step title="浏览器认证" />
          <el-step title="输入代码" />
        </el-steps>

        <template v-if="droidStep === 0">
          <el-form label-width="100px">
            <el-form-item label="名称">
              <el-input v-model="droidForm.name" placeholder="账户名称（可选）" />
            </el-form-item>
            <el-form-item label="代理 URL">
              <el-input v-model="droidForm.proxy_url" placeholder="socks5://127.0.0.1:1080" />
            </el-form-item>
          </el-form>
          <el-button type="warning" :loading="loading" @click="droidGenerateUrl">
            生成 Droid 认证链接
          </el-button>
        </template>

        <template v-if="droidStep === 1">
          <el-alert type="warning" :closable="false" class="mb-4">
            请在浏览器中打开以下链接完成 Droid 认证。
          </el-alert>
          <div class="bg-yellow-50 dark:bg-yellow-900/30 p-3 rounded break-all text-sm font-mono">
            {{ authUrl }}
          </div>
          <div class="flex gap-2 mt-3">
            <el-button size="small" @click="copyUrl">复制链接</el-button>
            <el-button size="small" type="warning" @click="droidStep = 2">下一步</el-button>
          </div>
        </template>

        <template v-if="droidStep === 2">
          <el-form label-width="100px">
            <el-form-item label="Auth Code" required>
              <el-input v-model="droidForm.code" placeholder="粘贴认证代码" type="textarea" :rows="3" />
            </el-form-item>
          </el-form>
          <el-button type="warning" :loading="loading" @click="droidExchange">
            完成认证
          </el-button>
        </template>
      </div>
    </template>
  </div>
</template>

<script setup>
import { ref, watch } from 'vue'
import { useAccountsStore } from '@/stores/accounts'
import { ElMessage } from 'element-plus'

const props = defineProps({
  platform: { type: String, default: 'claude' }
})

const emit = defineEmits(['success', 'close'])

const store = useAccountsStore()
const loading = ref(false)
const authUrl = ref('')
const batchResults = ref([])

const oauthPlatforms = [
  { key: 'claude', label: 'Claude' },
  { key: 'gemini', label: 'Gemini' },
  { key: 'openai', label: 'OpenAI' },
  { key: 'droid', label: 'Droid' }
]

const selectedPlatform = ref(
  oauthPlatforms.find((p) => p.key === props.platform) ? props.platform : 'claude'
)

watch(
  () => props.platform,
  (val) => {
    if (oauthPlatforms.find((p) => p.key === val)) {
      selectedPlatform.value = val
    }
  }
)

// Claude
const claudeMethod = ref('manual')
const claudeStep = ref(0)
const claudeForm = ref({ name: '', proxy_url: '', code: '' })
const cookieForm = ref({ sessionKeys: '', proxy_url: '' })
const setupTokenStep = ref(0)
const setupTokenForm = ref({ name: '', proxy_url: '', code: '' })

async function claudeGenerateUrl() {
  loading.value = true
  try {
    const res = await store.claudeGenerateAuthUrl({
      name: claudeForm.value.name,
      proxy_url: claudeForm.value.proxy_url
    })
    authUrl.value = res.url || res.auth_url
    claudeStep.value = 1
  } catch (e) {
    ElMessage.error(e?.response?.data?.error || '生成链接失败')
  } finally {
    loading.value = false
  }
}

async function claudeExchange() {
  loading.value = true
  try {
    await store.claudeExchangeCode({
      code: claudeForm.value.code,
      name: claudeForm.value.name,
      proxy_url: claudeForm.value.proxy_url
    })
    ElMessage.success('Claude OAuth 认证成功')
    emit('success')
  } catch (e) {
    ElMessage.error(e?.response?.data?.error || '认证失败')
  } finally {
    loading.value = false
  }
}

async function claudeCookieAuth() {
  loading.value = true
  batchResults.value = []
  try {
    const keys = cookieForm.value.sessionKeys
      .split('\n')
      .map((s) => s.trim())
      .filter(Boolean)
    for (const sessionKey of keys) {
      try {
        await store.claudeOAuthWithCookie({
          sessionKey,
          proxy_url: cookieForm.value.proxy_url
        })
        batchResults.value.push({ sessionKey, success: true })
      } catch (e) {
        batchResults.value.push({
          sessionKey,
          success: false,
          error: e?.response?.data?.error || '失败'
        })
      }
    }
    const successCount = batchResults.value.filter((r) => r.success).length
    if (successCount > 0) {
      ElMessage.success(`${successCount}/${keys.length} 个账户认证成功`)
      emit('success')
    } else {
      ElMessage.error('所有账户认证失败')
    }
  } finally {
    loading.value = false
  }
}

async function claudeGenerateSetupTokenUrl() {
  loading.value = true
  try {
    const res = await store.claudeGenerateSetupTokenUrl({
      name: setupTokenForm.value.name,
      proxy_url: setupTokenForm.value.proxy_url
    })
    authUrl.value = res.url || res.auth_url
    setupTokenStep.value = 1
  } catch (e) {
    ElMessage.error(e?.response?.data?.error || '生成链接失败')
  } finally {
    loading.value = false
  }
}

async function claudeExchangeSetupToken() {
  loading.value = true
  try {
    await store.claudeExchangeSetupToken({
      code: setupTokenForm.value.code,
      name: setupTokenForm.value.name,
      proxy_url: setupTokenForm.value.proxy_url
    })
    ElMessage.success('Setup Token 认证成功')
    emit('success')
  } catch (e) {
    ElMessage.error(e?.response?.data?.error || '认证失败')
  } finally {
    loading.value = false
  }
}

// Gemini
const geminiStep = ref(0)
const geminiForm = ref({ name: '', proxy_url: '', code: '' })

async function geminiGenerateUrl() {
  loading.value = true
  try {
    const res = await store.geminiGenerateAuthUrl({
      name: geminiForm.value.name,
      proxy_url: geminiForm.value.proxy_url
    })
    authUrl.value = res.url || res.auth_url
    geminiStep.value = 1
  } catch (e) {
    ElMessage.error(e?.response?.data?.error || '生成链接失败')
  } finally {
    loading.value = false
  }
}

async function geminiExchange() {
  loading.value = true
  try {
    await store.geminiExchangeCode({
      code: geminiForm.value.code,
      name: geminiForm.value.name,
      proxy_url: geminiForm.value.proxy_url
    })
    ElMessage.success('Gemini OAuth 认证成功')
    emit('success')
  } catch (e) {
    ElMessage.error(e?.response?.data?.error || '认证失败')
  } finally {
    loading.value = false
  }
}

// OpenAI
const openaiStep = ref(0)
const openaiForm = ref({ name: '', proxy_url: '', code: '' })

async function openaiGenerateUrl() {
  loading.value = true
  try {
    const res = await store.openaiGenerateAuthUrl({
      name: openaiForm.value.name,
      proxy_url: openaiForm.value.proxy_url
    })
    authUrl.value = res.url || res.auth_url
    openaiStep.value = 1
  } catch (e) {
    ElMessage.error(e?.response?.data?.error || '生成链接失败')
  } finally {
    loading.value = false
  }
}

async function openaiExchange() {
  loading.value = true
  try {
    await store.openaiExchangeCode({
      code: openaiForm.value.code,
      name: openaiForm.value.name,
      proxy_url: openaiForm.value.proxy_url
    })
    ElMessage.success('OpenAI OAuth 认证成功')
    emit('success')
  } catch (e) {
    ElMessage.error(e?.response?.data?.error || '认证失败')
  } finally {
    loading.value = false
  }
}

// Droid
const droidStep = ref(0)
const droidForm = ref({ name: '', proxy_url: '', code: '' })

async function droidGenerateUrl() {
  loading.value = true
  try {
    const res = await store.droidGenerateAuthUrl({
      name: droidForm.value.name,
      proxy_url: droidForm.value.proxy_url
    })
    authUrl.value = res.url || res.auth_url
    droidStep.value = 1
  } catch (e) {
    ElMessage.error(e?.response?.data?.error || '生成链接失败')
  } finally {
    loading.value = false
  }
}

async function droidExchange() {
  loading.value = true
  try {
    await store.droidExchangeCode({
      code: droidForm.value.code,
      name: droidForm.value.name,
      proxy_url: droidForm.value.proxy_url
    })
    ElMessage.success('Droid OAuth 认证成功')
    emit('success')
  } catch (e) {
    ElMessage.error(e?.response?.data?.error || '认证失败')
  } finally {
    loading.value = false
  }
}

function copyUrl() {
  navigator.clipboard.writeText(authUrl.value)
  ElMessage.success('已复制到剪贴板')
}
</script>

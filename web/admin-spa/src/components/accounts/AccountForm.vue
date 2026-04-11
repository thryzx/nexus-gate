<template>
  <el-form :model="form" label-width="120px" @submit.prevent>
    <el-form-item label="名称" required>
      <el-input v-model="form.name" placeholder="账户名称" />
    </el-form-item>

    <el-form-item label="描述">
      <el-input v-model="form.description" type="textarea" :rows="2" />
    </el-form-item>

    <!-- Platform-Specific Credential Fields -->
    <template v-if="!isEdit">
      <!-- Claude (OAuth) - credentials via OAuthFlow -->
      <template v-if="platform === 'claude'">
        <el-alert type="info" :closable="false" class="mb-4">
          Claude OAuth 账户建议通过「OAuth 认证」按钮添加。手动添加需填入 access_token。
        </el-alert>
        <el-form-item label="Access Token">
          <el-input v-model="form.credentials" type="textarea" :rows="3" placeholder="OAuth access_token" />
        </el-form-item>
      </template>

      <!-- Claude Console (API Key) -->
      <template v-if="platform === 'claude-console'">
        <el-form-item label="API Key" required>
          <el-input v-model="credFields.apiKey" placeholder="sk-ant-..." show-password />
        </el-form-item>
        <el-form-item label="组织 ID">
          <el-input v-model="credFields.organizationId" placeholder="可选" />
        </el-form-item>
      </template>

      <!-- Bedrock (AWS) -->
      <template v-if="platform === 'bedrock'">
        <el-form-item label="Access Key ID" required>
          <el-input v-model="credFields.accessKeyId" placeholder="AKIA..." />
        </el-form-item>
        <el-form-item label="Secret Key" required>
          <el-input v-model="credFields.secretAccessKey" show-password placeholder="Secret Access Key" />
        </el-form-item>
        <el-form-item label="Region" required>
          <el-select v-model="credFields.region" class="w-full" filterable>
            <el-option v-for="r in awsRegions" :key="r" :value="r" :label="r" />
          </el-select>
        </el-form-item>
        <el-form-item label="Session Token">
          <el-input v-model="credFields.sessionToken" placeholder="可选 (临时凭据)" />
        </el-form-item>
      </template>

      <!-- CCR -->
      <template v-if="platform === 'ccr'">
        <el-form-item label="API Key" required>
          <el-input v-model="credFields.apiKey" placeholder="CCR API Key" show-password />
        </el-form-item>
        <el-form-item label="Endpoint">
          <el-input v-model="credFields.endpoint" placeholder="https://..." />
        </el-form-item>
      </template>

      <!-- Gemini (OAuth) -->
      <template v-if="platform === 'gemini'">
        <el-alert type="info" :closable="false" class="mb-4">
          Gemini OAuth 账户建议通过「OAuth 认证」按钮添加。
        </el-alert>
        <el-form-item label="Access Token">
          <el-input v-model="form.credentials" type="textarea" :rows="3" placeholder="OAuth access_token (可选)" />
        </el-form-item>
      </template>

      <!-- Gemini API -->
      <template v-if="platform === 'gemini-api'">
        <el-form-item label="API Key" required>
          <el-input v-model="credFields.apiKey" placeholder="AIza..." show-password />
        </el-form-item>
      </template>

      <!-- OpenAI (OAuth / Codex) -->
      <template v-if="platform === 'openai'">
        <el-alert type="info" :closable="false" class="mb-4">
          OpenAI Codex OAuth 账户建议通过「OAuth 认证」按钮添加。
        </el-alert>
        <el-form-item label="Access Token">
          <el-input v-model="form.credentials" type="textarea" :rows="3" placeholder="OAuth access_token (可选)" />
        </el-form-item>
      </template>

      <!-- OpenAI Responses (API Key) -->
      <template v-if="platform === 'openai-responses'">
        <el-form-item label="API Key" required>
          <el-input v-model="credFields.apiKey" placeholder="sk-..." show-password />
        </el-form-item>
        <el-form-item label="Organization ID">
          <el-input v-model="credFields.orgId" placeholder="org-... (可选)" />
        </el-form-item>
      </template>

      <!-- Azure OpenAI -->
      <template v-if="platform === 'azure-openai'">
        <el-form-item label="API Key" required>
          <el-input v-model="credFields.apiKey" show-password placeholder="Azure API Key" />
        </el-form-item>
        <el-form-item label="Endpoint" required>
          <el-input v-model="credFields.endpoint" placeholder="https://xxx.openai.azure.com/" />
        </el-form-item>
        <el-form-item label="Deployment">
          <el-input v-model="credFields.deployment" placeholder="模型部署名称" />
        </el-form-item>
        <el-form-item label="API Version">
          <el-input v-model="credFields.apiVersion" placeholder="2024-02-15-preview" />
        </el-form-item>
      </template>

      <!-- Droid (OAuth) -->
      <template v-if="platform === 'droid'">
        <el-alert type="info" :closable="false" class="mb-4">
          Droid 账户建议通过「OAuth 认证」按钮添加。
        </el-alert>
        <el-form-item label="Access Token">
          <el-input v-model="form.credentials" type="textarea" :rows="3" placeholder="OAuth access_token (可选)" />
        </el-form-item>
      </template>
    </template>

    <!-- Common Fields -->
    <el-form-item label="状态">
      <el-select v-model="form.status" class="w-full">
        <el-option value="active" label="在线" />
        <el-option value="disabled" label="禁用" />
      </el-select>
    </el-form-item>

    <el-form-item label="优先级">
      <el-input-number v-model="form.priority" :min="0" :max="100" />
    </el-form-item>

    <el-form-item label="最大并发">
      <el-input-number v-model="form.max_concurrency" :min="1" :max="100" />
    </el-form-item>

    <el-form-item label="速率限制 (RPM)">
      <el-input-number v-model="form.rate_limit" :min="0" :max="100000" />
    </el-form-item>

    <el-form-item label="代理 URL">
      <el-input v-model="form.proxy_url" placeholder="socks5://127.0.0.1:1080" />
    </el-form-item>

    <el-form-item label="指纹配置">
      <el-select v-model="form.fingerprint_profile_id" class="w-full" clearable placeholder="无指纹">
        <el-option v-for="fp in fingerprints" :key="fp.id" :value="fp.id" :label="fp.name" />
      </el-select>
    </el-form-item>

    <el-form-item label="分组">
      <el-select v-model="form.group_id" class="w-full" clearable placeholder="无分组">
        <el-option v-for="g in groups" :key="g.id" :value="g.id" :label="g.name" />
      </el-select>
    </el-form-item>

    <el-form-item label="可调度">
      <el-switch v-model="form.schedulable" />
    </el-form-item>

    <div class="flex justify-end gap-2 mt-4">
      <el-button @click="$emit('cancel')">取消</el-button>
      <el-button type="primary" @click="handleSubmit">保存</el-button>
    </div>
  </el-form>
</template>

<script setup>
import { ref, watch, onMounted } from 'vue'

const props = defineProps({
  platform: { type: String, required: true },
  initialData: { type: Object, default: null },
  fingerprints: { type: Array, default: () => [] },
  groups: { type: Array, default: () => [] },
  isEdit: { type: Boolean, default: false }
})

const emit = defineEmits(['save', 'cancel'])

const awsRegions = [
  'us-east-1', 'us-east-2', 'us-west-2', 'eu-west-1', 'eu-west-2',
  'eu-central-1', 'ap-southeast-1', 'ap-southeast-2', 'ap-northeast-1'
]

const defaultForm = () => ({
  name: '',
  description: '',
  platform: props.platform,
  account_type: getDefaultAccountType(props.platform),
  credentials: '',
  status: 'active',
  priority: 50,
  max_concurrency: 1,
  rate_limit: 0,
  proxy_url: '',
  fingerprint_profile_id: null,
  group_id: null,
  schedulable: true
})

const credFields = ref({
  apiKey: '',
  accessKeyId: '',
  secretAccessKey: '',
  region: 'us-east-1',
  sessionToken: '',
  organizationId: '',
  orgId: '',
  endpoint: '',
  deployment: '',
  apiVersion: '2024-02-15-preview'
})

const form = ref(defaultForm())

function getDefaultAccountType(platform) {
  const typeMap = {
    claude: 'oauth',
    'claude-console': 'apikey',
    bedrock: 'bedrock',
    ccr: 'apikey',
    gemini: 'oauth',
    'gemini-api': 'apikey',
    openai: 'oauth',
    'openai-responses': 'apikey',
    'azure-openai': 'apikey',
    droid: 'oauth'
  }
  return typeMap[platform] || 'apikey'
}

function buildCredentials() {
  const p = props.platform
  if (['claude', 'gemini', 'openai', 'droid'].includes(p)) {
    return form.value.credentials || ''
  }
  if (p === 'claude-console') {
    return JSON.stringify({ apiKey: credFields.value.apiKey, organizationId: credFields.value.organizationId })
  }
  if (p === 'bedrock') {
    return JSON.stringify({
      accessKeyId: credFields.value.accessKeyId,
      secretAccessKey: credFields.value.secretAccessKey,
      region: credFields.value.region,
      sessionToken: credFields.value.sessionToken || undefined
    })
  }
  if (p === 'ccr') {
    return JSON.stringify({ apiKey: credFields.value.apiKey, endpoint: credFields.value.endpoint })
  }
  if (p === 'gemini-api') {
    return JSON.stringify({ apiKey: credFields.value.apiKey })
  }
  if (p === 'openai-responses') {
    return JSON.stringify({ apiKey: credFields.value.apiKey, orgId: credFields.value.orgId })
  }
  if (p === 'azure-openai') {
    return JSON.stringify({
      apiKey: credFields.value.apiKey,
      endpoint: credFields.value.endpoint,
      deployment: credFields.value.deployment,
      apiVersion: credFields.value.apiVersion
    })
  }
  return form.value.credentials
}

function handleSubmit() {
  const data = { ...form.value }
  data.platform = props.platform
  data.account_type = getDefaultAccountType(props.platform)
  if (!props.isEdit) {
    data.credentials = buildCredentials()
  } else {
    delete data.credentials
  }
  emit('save', data)
}

watch(
  () => props.initialData,
  (val) => {
    if (val) {
      form.value = { ...defaultForm(), ...val }
    }
  },
  { immediate: true }
)

onMounted(() => {
  if (!props.initialData) {
    form.value = defaultForm()
  }
})
</script>

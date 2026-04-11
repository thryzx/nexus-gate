import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import * as api from '@/utils/api'

const PLATFORM_CONFIG = {
  claude: {
    endpoint: 'claude-accounts',
    stateKey: 'claudeAccounts',
    fetchFn: 'getClaudeAccounts',
    createFn: 'createClaudeAccount',
    updateFn: 'updateClaudeAccount',
    deleteFn: 'deleteClaudeAccount'
  },
  'claude-console': {
    endpoint: 'claude-console-accounts',
    stateKey: 'claudeConsoleAccounts',
    fetchFn: 'getClaudeConsoleAccounts',
    createFn: 'createClaudeConsoleAccount',
    updateFn: 'updateClaudeConsoleAccount',
    deleteFn: 'deleteClaudeConsoleAccount'
  },
  bedrock: {
    endpoint: 'bedrock-accounts',
    stateKey: 'bedrockAccounts',
    fetchFn: 'getBedrockAccounts',
    createFn: 'createBedrockAccount',
    updateFn: 'updateBedrockAccount',
    deleteFn: 'deleteBedrockAccount'
  },
  ccr: {
    endpoint: 'ccr-accounts',
    stateKey: 'ccrAccounts',
    fetchFn: 'getCcrAccounts',
    createFn: 'createCcrAccount',
    updateFn: 'updateCcrAccount',
    deleteFn: 'deleteCcrAccount'
  },
  gemini: {
    endpoint: 'gemini-accounts',
    stateKey: 'geminiAccounts',
    fetchFn: 'getGeminiAccounts',
    createFn: 'createGeminiAccount',
    updateFn: 'updateGeminiAccount',
    deleteFn: 'deleteGeminiAccount'
  },
  'gemini-api': {
    endpoint: 'gemini-api-accounts',
    stateKey: 'geminiApiAccounts',
    fetchFn: 'getGeminiApiAccounts',
    createFn: 'createGeminiApiAccount',
    updateFn: 'updateGeminiApiAccount',
    deleteFn: 'deleteGeminiApiAccount'
  },
  openai: {
    endpoint: 'openai-accounts',
    stateKey: 'openaiAccounts',
    fetchFn: 'getOpenAIAccounts',
    createFn: 'createOpenAIAccount',
    updateFn: 'updateOpenAIAccount',
    deleteFn: 'deleteOpenAIAccount'
  },
  'openai-responses': {
    endpoint: 'openai-responses-accounts',
    stateKey: 'openaiResponsesAccounts',
    fetchFn: 'getOpenAIResponsesAccounts',
    createFn: 'createOpenAIResponsesAccount',
    updateFn: 'updateOpenAIResponsesAccount',
    deleteFn: 'deleteOpenAIResponsesAccount'
  },
  'azure-openai': {
    endpoint: 'azure-openai-accounts',
    stateKey: 'azureOpenaiAccounts',
    fetchFn: 'getAzureOpenAIAccounts',
    createFn: 'createAzureOpenAIAccount',
    updateFn: 'updateAzureOpenAIAccount',
    deleteFn: 'deleteAzureOpenAIAccount'
  },
  droid: {
    endpoint: 'droid-accounts',
    stateKey: 'droidAccounts',
    fetchFn: 'getDroidAccounts',
    createFn: 'createDroidAccount',
    updateFn: 'updateDroidAccount',
    deleteFn: 'deleteDroidAccount'
  }
}

export const useAccountsStore = defineStore('accounts', () => {
  // Per-platform state
  const claudeAccounts = ref([])
  const claudeConsoleAccounts = ref([])
  const bedrockAccounts = ref([])
  const ccrAccounts = ref([])
  const geminiAccounts = ref([])
  const geminiApiAccounts = ref([])
  const openaiAccounts = ref([])
  const openaiResponsesAccounts = ref([])
  const azureOpenaiAccounts = ref([])
  const droidAccounts = ref([])
  const loading = ref(false)

  // Groups
  const groups = ref([])

  // State map for dynamic access
  const stateMap = {
    claudeAccounts,
    claudeConsoleAccounts,
    bedrockAccounts,
    ccrAccounts,
    geminiAccounts,
    geminiApiAccounts,
    openaiAccounts,
    openaiResponsesAccounts,
    azureOpenaiAccounts,
    droidAccounts
  }

  const allAccounts = computed(() => [
    ...claudeAccounts.value.map((a) => ({ ...a, _platform: 'claude' })),
    ...claudeConsoleAccounts.value.map((a) => ({
      ...a,
      _platform: 'claude-console'
    })),
    ...bedrockAccounts.value.map((a) => ({ ...a, _platform: 'bedrock' })),
    ...ccrAccounts.value.map((a) => ({ ...a, _platform: 'ccr' })),
    ...geminiAccounts.value.map((a) => ({ ...a, _platform: 'gemini' })),
    ...geminiApiAccounts.value.map((a) => ({ ...a, _platform: 'gemini-api' })),
    ...openaiAccounts.value.map((a) => ({ ...a, _platform: 'openai' })),
    ...openaiResponsesAccounts.value.map((a) => ({
      ...a,
      _platform: 'openai-responses'
    })),
    ...azureOpenaiAccounts.value.map((a) => ({
      ...a,
      _platform: 'azure-openai'
    })),
    ...droidAccounts.value.map((a) => ({ ...a, _platform: 'droid' }))
  ])

  // Generic helpers
  async function fetchAccounts(apiFn, stateRef) {
    loading.value = true
    try {
      stateRef.value = await apiFn()
    } catch (e) {
      stateRef.value = []
      throw e
    } finally {
      loading.value = false
    }
  }

  async function mutateAccount(apiFn, fetchFn, ...args) {
    const res = await apiFn(...args)
    await fetchFn()
    return res
  }

  // Fetch all platforms
  async function fetchAll() {
    loading.value = true
    try {
      const results = await Promise.allSettled(
        Object.entries(PLATFORM_CONFIG).map(([, cfg]) =>
          api[cfg.fetchFn]().then((data) => {
            stateMap[cfg.stateKey].value = data
          })
        )
      )
      results.forEach((r, i) => {
        if (r.status === 'rejected') {
          const key = Object.values(PLATFORM_CONFIG)[i].stateKey
          stateMap[key].value = []
        }
      })
    } finally {
      loading.value = false
    }
  }

  // Per-platform fetch
  const fetchClaude = () => fetchAccounts(api.getClaudeAccounts, claudeAccounts)
  const fetchClaudeConsole = () =>
    fetchAccounts(api.getClaudeConsoleAccounts, claudeConsoleAccounts)
  const fetchBedrock = () => fetchAccounts(api.getBedrockAccounts, bedrockAccounts)
  const fetchCcr = () => fetchAccounts(api.getCcrAccounts, ccrAccounts)
  const fetchGemini = () => fetchAccounts(api.getGeminiAccounts, geminiAccounts)
  const fetchGeminiApi = () =>
    fetchAccounts(api.getGeminiApiAccounts, geminiApiAccounts)
  const fetchOpenai = () => fetchAccounts(api.getOpenAIAccounts, openaiAccounts)
  const fetchOpenaiResponses = () =>
    fetchAccounts(api.getOpenAIResponsesAccounts, openaiResponsesAccounts)
  const fetchAzureOpenai = () =>
    fetchAccounts(api.getAzureOpenAIAccounts, azureOpenaiAccounts)
  const fetchDroid = () => fetchAccounts(api.getDroidAccounts, droidAccounts)

  // Per-platform CRUD
  const createClaude = (data) =>
    mutateAccount(api.createClaudeAccount, fetchClaude, data)
  const updateClaude = (id, data) =>
    mutateAccount(api.updateClaudeAccount, fetchClaude, id, data)
  const deleteClaude = (id) =>
    mutateAccount(api.deleteClaudeAccount, fetchClaude, id)

  const createClaudeConsole = (data) =>
    mutateAccount(api.createClaudeConsoleAccount, fetchClaudeConsole, data)
  const updateClaudeConsole = (id, data) =>
    mutateAccount(api.updateClaudeConsoleAccount, fetchClaudeConsole, id, data)
  const deleteClaudeConsole = (id) =>
    mutateAccount(api.deleteClaudeConsoleAccount, fetchClaudeConsole, id)

  const createBedrock = (data) =>
    mutateAccount(api.createBedrockAccount, fetchBedrock, data)
  const updateBedrock = (id, data) =>
    mutateAccount(api.updateBedrockAccount, fetchBedrock, id, data)
  const deleteBedrock = (id) =>
    mutateAccount(api.deleteBedrockAccount, fetchBedrock, id)

  const createCcr = (data) => mutateAccount(api.createCcrAccount, fetchCcr, data)
  const updateCcr = (id, data) =>
    mutateAccount(api.updateCcrAccount, fetchCcr, id, data)
  const deleteCcr = (id) => mutateAccount(api.deleteCcrAccount, fetchCcr, id)

  const createGemini = (data) =>
    mutateAccount(api.createGeminiAccount, fetchGemini, data)
  const updateGemini = (id, data) =>
    mutateAccount(api.updateGeminiAccount, fetchGemini, id, data)
  const deleteGemini = (id) =>
    mutateAccount(api.deleteGeminiAccount, fetchGemini, id)

  const createGeminiApi = (data) =>
    mutateAccount(api.createGeminiApiAccount, fetchGeminiApi, data)
  const updateGeminiApi = (id, data) =>
    mutateAccount(api.updateGeminiApiAccount, fetchGeminiApi, id, data)
  const deleteGeminiApi = (id) =>
    mutateAccount(api.deleteGeminiApiAccount, fetchGeminiApi, id)

  const createOpenai = (data) =>
    mutateAccount(api.createOpenAIAccount, fetchOpenai, data)
  const updateOpenai = (id, data) =>
    mutateAccount(api.updateOpenAIAccount, fetchOpenai, id, data)
  const deleteOpenai = (id) =>
    mutateAccount(api.deleteOpenAIAccount, fetchOpenai, id)

  const createOpenaiResponses = (data) =>
    mutateAccount(api.createOpenAIResponsesAccount, fetchOpenaiResponses, data)
  const updateOpenaiResponses = (id, data) =>
    mutateAccount(api.updateOpenAIResponsesAccount, fetchOpenaiResponses, id, data)
  const deleteOpenaiResponses = (id) =>
    mutateAccount(api.deleteOpenAIResponsesAccount, fetchOpenaiResponses, id)

  const createAzureOpenai = (data) =>
    mutateAccount(api.createAzureOpenAIAccount, fetchAzureOpenai, data)
  const updateAzureOpenai = (id, data) =>
    mutateAccount(api.updateAzureOpenAIAccount, fetchAzureOpenai, id, data)
  const deleteAzureOpenai = (id) =>
    mutateAccount(api.deleteAzureOpenAIAccount, fetchAzureOpenai, id)

  const createDroid = (data) =>
    mutateAccount(api.createDroidAccount, fetchDroid, data)
  const updateDroid = (id, data) =>
    mutateAccount(api.updateDroidAccount, fetchDroid, id, data)
  const deleteDroid = (id) =>
    mutateAccount(api.deleteDroidAccount, fetchDroid, id)

  // OAuth flows
  const claudeGenerateAuthUrl = (data) => api.generateClaudeAuthUrl(data)
  const claudeExchangeCode = (data) => api.exchangeClaudeCode(data)
  const claudeOAuthWithCookie = (data) => api.claudeOAuthWithCookie(data)
  const claudeGenerateSetupTokenUrl = (data) =>
    api.generateClaudeSetupTokenUrl(data)
  const claudeExchangeSetupToken = (data) => api.exchangeClaudeSetupToken(data)

  const geminiGenerateAuthUrl = (data) => api.generateGeminiAuthUrl(data)
  const geminiExchangeCode = (data) => api.exchangeGeminiCode(data)

  const openaiGenerateAuthUrl = (data) => api.generateOpenAIAuthUrl(data)
  const openaiExchangeCode = (data) => api.exchangeOpenAICode(data)

  const droidGenerateAuthUrl = (data) => api.generateDroidAuthUrl(data)
  const droidExchangeCode = (data) => api.exchangeDroidCode(data)

  // Toggle & reset
  const toggleStatus = (platform, id) => {
    const toggleMap = {
      claude: api.toggleClaudeAccount,
      'claude-console': api.toggleClaudeConsole,
      bedrock: api.toggleBedrock,
      ccr: api.toggleCcr,
      openai: api.toggleOpenAI,
      'openai-responses': api.toggleOpenAIResponses,
      'azure-openai': api.toggleAzureOpenAI,
      'gemini-api': api.toggleGeminiApi
    }
    const fn = toggleMap[platform]
    if (!fn) return Promise.reject(new Error(`No toggle for ${platform}`))
    const cfg = PLATFORM_CONFIG[platform]
    return mutateAccount(fn, () => fetchAccounts(api[cfg.fetchFn], stateMap[cfg.stateKey]), id)
  }

  const toggleSchedulable = (platform, id) => {
    const toggleMap = {
      claude: api.toggleClaudeSchedulable,
      'claude-console': api.toggleClaudeConsoleSchedulable,
      bedrock: api.toggleBedrockSchedulable,
      ccr: api.toggleCcrSchedulable,
      gemini: api.toggleGeminiSchedulable,
      'gemini-api': api.toggleGeminiApiSchedulable,
      openai: api.toggleOpenAISchedulable,
      'openai-responses': api.toggleOpenAIResponsesSchedulable,
      'azure-openai': api.toggleAzureOpenAISchedulable,
      droid: api.toggleDroidSchedulable
    }
    const fn = toggleMap[platform]
    if (!fn) return Promise.reject(new Error(`No toggleSchedulable for ${platform}`))
    const cfg = PLATFORM_CONFIG[platform]
    return mutateAccount(fn, () => fetchAccounts(api[cfg.fetchFn], stateMap[cfg.stateKey]), id)
  }

  const resetStatus = (platform, id) => {
    const resetMap = {
      claude: api.resetClaudeStatus,
      bedrock: api.resetBedrockStatus,
      gemini: api.resetGeminiStatus,
      openai: api.resetOpenAIStatus,
      'openai-responses': api.resetOpenAIResponsesStatus,
      droid: api.resetDroidStatus
    }
    const fn = resetMap[platform]
    if (!fn) return Promise.resolve()
    const cfg = PLATFORM_CONFIG[platform]
    return mutateAccount(fn, () => fetchAccounts(api[cfg.fetchFn], stateMap[cfg.stateKey]), id)
  }

  const testAccount = (platform, id) => {
    const testMap = {
      claude: api.testClaudeAccount,
      bedrock: api.testBedrockAccount,
      gemini: api.testGeminiAccount,
      'openai-responses': api.testOpenAIResponsesAccount,
      'azure-openai': api.testAzureOpenAIAccount,
      droid: api.testDroidAccount
    }
    const fn = testMap[platform]
    if (!fn) return Promise.reject(new Error(`No test for ${platform}`))
    return fn(id)
  }

  // Groups
  const fetchGroups = async () => {
    groups.value = await api.getAccountGroups()
  }
  const createGroup = (data) =>
    api.createAccountGroup(data).then(() => fetchGroups())
  const updateGroup = (id, data) =>
    api.updateAccountGroup(id, data).then(() => fetchGroups())
  const deleteGroup = (id) =>
    api.deleteAccountGroup(id).then(() => fetchGroups())
  const getGroupMembers = (id) => api.getAccountGroupMembers(id)

  // Fetch by platform key
  function fetchByPlatform(platform) {
    const fetchMap = {
      claude: fetchClaude,
      'claude-console': fetchClaudeConsole,
      bedrock: fetchBedrock,
      ccr: fetchCcr,
      gemini: fetchGemini,
      'gemini-api': fetchGeminiApi,
      openai: fetchOpenai,
      'openai-responses': fetchOpenaiResponses,
      'azure-openai': fetchAzureOpenai,
      droid: fetchDroid
    }
    return fetchMap[platform]?.() || Promise.resolve()
  }

  return {
    // State
    claudeAccounts,
    claudeConsoleAccounts,
    bedrockAccounts,
    ccrAccounts,
    geminiAccounts,
    geminiApiAccounts,
    openaiAccounts,
    openaiResponsesAccounts,
    azureOpenaiAccounts,
    droidAccounts,
    loading,
    groups,
    allAccounts,
    stateMap,
    PLATFORM_CONFIG,

    // Fetch
    fetchAll,
    fetchByPlatform,
    fetchClaude,
    fetchClaudeConsole,
    fetchBedrock,
    fetchCcr,
    fetchGemini,
    fetchGeminiApi,
    fetchOpenai,
    fetchOpenaiResponses,
    fetchAzureOpenai,
    fetchDroid,

    // CRUD
    createClaude,
    updateClaude,
    deleteClaude,
    createClaudeConsole,
    updateClaudeConsole,
    deleteClaudeConsole,
    createBedrock,
    updateBedrock,
    deleteBedrock,
    createCcr,
    updateCcr,
    deleteCcr,
    createGemini,
    updateGemini,
    deleteGemini,
    createGeminiApi,
    updateGeminiApi,
    deleteGeminiApi,
    createOpenai,
    updateOpenai,
    deleteOpenai,
    createOpenaiResponses,
    updateOpenaiResponses,
    deleteOpenaiResponses,
    createAzureOpenai,
    updateAzureOpenai,
    deleteAzureOpenai,
    createDroid,
    updateDroid,
    deleteDroid,

    // OAuth
    claudeGenerateAuthUrl,
    claudeExchangeCode,
    claudeOAuthWithCookie,
    claudeGenerateSetupTokenUrl,
    claudeExchangeSetupToken,
    geminiGenerateAuthUrl,
    geminiExchangeCode,
    openaiGenerateAuthUrl,
    openaiExchangeCode,
    droidGenerateAuthUrl,
    droidExchangeCode,

    // Toggle/Reset/Test
    toggleStatus,
    toggleSchedulable,
    resetStatus,
    testAccount,

    // Groups
    fetchGroups,
    createGroup,
    updateGroup,
    deleteGroup,
    getGroupMembers
  }
})

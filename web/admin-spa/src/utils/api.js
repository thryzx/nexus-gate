import request from './request'

// ── Auth ──
export const login = (data) => request.post('/admin/login', data)
export const changePassword = (data) => request.post('/admin/change-password', data)

// ── Dashboard ──
export const getDashboard = () => request.get('/admin/dashboard')

// ── Accounts (generic) ──
export const getAccounts = () => request.get('/admin/accounts')
export const getAccount = (id) => request.get(`/admin/accounts/${id}`)
export const createAccount = (data) => request.post('/admin/accounts', data)
export const updateAccount = (id, data) => request.put(`/admin/accounts/${id}`, data)
export const deleteAccount = (id) => request.delete(`/admin/accounts/${id}`)
export const getAccountUsageRecords = (id, params) =>
  request.get(`/admin/accounts/${id}/usage-records`, { params })
export const getAccountsUsageStats = () => request.get('/admin/accounts/usage-stats')

// ── Claude Accounts ──
export const getClaudeAccounts = () => request.get('/admin/claude-accounts')
export const createClaudeAccount = (data) => request.post('/admin/claude-accounts', data)
export const updateClaudeAccount = (id, data) =>
  request.put(`/admin/claude-accounts/${id}`, data)
export const deleteClaudeAccount = (id) => request.delete(`/admin/claude-accounts/${id}`)
export const toggleClaudeAccount = (id) =>
  request.put(`/admin/claude-accounts/${id}/toggle`)
export const toggleClaudeSchedulable = (id) =>
  request.put(`/admin/claude-accounts/${id}/toggle-schedulable`)
export const resetClaudeStatus = (id) =>
  request.post(`/admin/claude-accounts/${id}/reset-status`)
export const testClaudeAccount = (id) =>
  request.post(`/admin/claude-accounts/${id}/test`)
export const refreshClaudeToken = (id) =>
  request.post(`/admin/claude-accounts/${id}/refresh`)
export const generateClaudeAuthUrl = (data) =>
  request.post('/admin/claude-accounts/generate-auth-url', data)
export const exchangeClaudeCode = (data) =>
  request.post('/admin/claude-accounts/exchange-code', data)
export const claudeOAuthWithCookie = (data) =>
  request.post('/admin/claude-accounts/oauth-with-cookie', data)
export const generateClaudeSetupTokenUrl = (data) =>
  request.post('/admin/claude-accounts/generate-setup-token-url', data)
export const exchangeClaudeSetupToken = (data) =>
  request.post('/admin/claude-accounts/exchange-setup-token-code', data)

// ── Claude Console Accounts ──
export const getClaudeConsoleAccounts = () =>
  request.get('/admin/claude-console-accounts')
export const createClaudeConsoleAccount = (data) =>
  request.post('/admin/claude-console-accounts', data)
export const updateClaudeConsoleAccount = (id, data) =>
  request.put(`/admin/claude-console-accounts/${id}`, data)
export const deleteClaudeConsoleAccount = (id) =>
  request.delete(`/admin/claude-console-accounts/${id}`)
export const toggleClaudeConsole = (id) =>
  request.put(`/admin/claude-console-accounts/${id}/toggle`)
export const toggleClaudeConsoleSchedulable = (id) =>
  request.put(`/admin/claude-console-accounts/${id}/toggle-schedulable`)

// ── Bedrock Accounts ──
export const getBedrockAccounts = () => request.get('/admin/bedrock-accounts')
export const createBedrockAccount = (data) =>
  request.post('/admin/bedrock-accounts', data)
export const updateBedrockAccount = (id, data) =>
  request.put(`/admin/bedrock-accounts/${id}`, data)
export const deleteBedrockAccount = (id) =>
  request.delete(`/admin/bedrock-accounts/${id}`)
export const toggleBedrock = (id) =>
  request.put(`/admin/bedrock-accounts/${id}/toggle`)
export const toggleBedrockSchedulable = (id) =>
  request.put(`/admin/bedrock-accounts/${id}/toggle-schedulable`)
export const resetBedrockStatus = (id) =>
  request.post(`/admin/bedrock-accounts/${id}/reset-status`)
export const testBedrockAccount = (id) =>
  request.post(`/admin/bedrock-accounts/${id}/test`)

// ── Gemini Accounts (OAuth) ──
export const getGeminiAccounts = () => request.get('/admin/gemini-accounts')
export const createGeminiAccount = (data) =>
  request.post('/admin/gemini-accounts', data)
export const updateGeminiAccount = (id, data) =>
  request.put(`/admin/gemini-accounts/${id}`, data)
export const deleteGeminiAccount = (id) =>
  request.delete(`/admin/gemini-accounts/${id}`)
export const toggleGeminiSchedulable = (id) =>
  request.put(`/admin/gemini-accounts/${id}/toggle-schedulable`)
export const resetGeminiStatus = (id) =>
  request.post(`/admin/gemini-accounts/${id}/reset-status`)
export const testGeminiAccount = (id) =>
  request.post(`/admin/gemini-accounts/${id}/test`)
export const generateGeminiAuthUrl = (data) =>
  request.post('/admin/gemini-accounts/generate-auth-url', data)
export const exchangeGeminiCode = (data) =>
  request.post('/admin/gemini-accounts/exchange-code', data)

// ── Gemini API Accounts ──
export const getGeminiApiAccounts = () => request.get('/admin/gemini-api-accounts')
export const createGeminiApiAccount = (data) =>
  request.post('/admin/gemini-api-accounts', data)
export const updateGeminiApiAccount = (id, data) =>
  request.put(`/admin/gemini-api-accounts/${id}`, data)
export const deleteGeminiApiAccount = (id) =>
  request.delete(`/admin/gemini-api-accounts/${id}`)
export const toggleGeminiApi = (id) =>
  request.put(`/admin/gemini-api-accounts/${id}/toggle`)
export const toggleGeminiApiSchedulable = (id) =>
  request.put(`/admin/gemini-api-accounts/${id}/toggle-schedulable`)

// ── OpenAI Accounts (OAuth) ──
export const getOpenAIAccounts = () => request.get('/admin/openai-accounts')
export const createOpenAIAccount = (data) =>
  request.post('/admin/openai-accounts', data)
export const updateOpenAIAccount = (id, data) =>
  request.put(`/admin/openai-accounts/${id}`, data)
export const deleteOpenAIAccount = (id) =>
  request.delete(`/admin/openai-accounts/${id}`)
export const toggleOpenAI = (id) =>
  request.put(`/admin/openai-accounts/${id}/toggle`)
export const toggleOpenAISchedulable = (id) =>
  request.put(`/admin/openai-accounts/${id}/toggle-schedulable`)
export const resetOpenAIStatus = (id) =>
  request.post(`/admin/openai-accounts/${id}/reset-status`)
export const generateOpenAIAuthUrl = (data) =>
  request.post('/admin/openai-accounts/generate-auth-url', data)
export const exchangeOpenAICode = (data) =>
  request.post('/admin/openai-accounts/exchange-code', data)

// ── OpenAI Responses Accounts ──
export const getOpenAIResponsesAccounts = () =>
  request.get('/admin/openai-responses-accounts')
export const createOpenAIResponsesAccount = (data) =>
  request.post('/admin/openai-responses-accounts', data)
export const updateOpenAIResponsesAccount = (id, data) =>
  request.put(`/admin/openai-responses-accounts/${id}`, data)
export const deleteOpenAIResponsesAccount = (id) =>
  request.delete(`/admin/openai-responses-accounts/${id}`)
export const toggleOpenAIResponses = (id) =>
  request.put(`/admin/openai-responses-accounts/${id}/toggle`)
export const toggleOpenAIResponsesSchedulable = (id) =>
  request.put(`/admin/openai-responses-accounts/${id}/toggle-schedulable`)
export const resetOpenAIResponsesStatus = (id) =>
  request.post(`/admin/openai-responses-accounts/${id}/reset-status`)
export const testOpenAIResponsesAccount = (id) =>
  request.post(`/admin/openai-responses-accounts/${id}/test`)

// ── Azure OpenAI Accounts ──
export const getAzureOpenAIAccounts = () =>
  request.get('/admin/azure-openai-accounts')
export const createAzureOpenAIAccount = (data) =>
  request.post('/admin/azure-openai-accounts', data)
export const updateAzureOpenAIAccount = (id, data) =>
  request.put(`/admin/azure-openai-accounts/${id}`, data)
export const deleteAzureOpenAIAccount = (id) =>
  request.delete(`/admin/azure-openai-accounts/${id}`)
export const toggleAzureOpenAI = (id) =>
  request.put(`/admin/azure-openai-accounts/${id}/toggle`)
export const toggleAzureOpenAISchedulable = (id) =>
  request.put(`/admin/azure-openai-accounts/${id}/toggle-schedulable`)
export const testAzureOpenAIAccount = (id) =>
  request.post(`/admin/azure-openai-accounts/${id}/test`)

// ── Droid Accounts ──
export const getDroidAccounts = () => request.get('/admin/droid-accounts')
export const createDroidAccount = (data) =>
  request.post('/admin/droid-accounts', data)
export const updateDroidAccount = (id, data) =>
  request.put(`/admin/droid-accounts/${id}`, data)
export const deleteDroidAccount = (id) =>
  request.delete(`/admin/droid-accounts/${id}`)
export const toggleDroidSchedulable = (id) =>
  request.put(`/admin/droid-accounts/${id}/toggle-schedulable`)
export const resetDroidStatus = (id) =>
  request.post(`/admin/droid-accounts/${id}/reset-status`)
export const testDroidAccount = (id) =>
  request.post(`/admin/droid-accounts/${id}/test`)
export const generateDroidAuthUrl = (data) =>
  request.post('/admin/droid-accounts/generate-auth-url', data)
export const exchangeDroidCode = (data) =>
  request.post('/admin/droid-accounts/exchange-code', data)

// ── CCR Accounts ──
export const getCcrAccounts = () => request.get('/admin/ccr-accounts')
export const createCcrAccount = (data) => request.post('/admin/ccr-accounts', data)
export const updateCcrAccount = (id, data) =>
  request.put(`/admin/ccr-accounts/${id}`, data)
export const deleteCcrAccount = (id) =>
  request.delete(`/admin/ccr-accounts/${id}`)
export const toggleCcr = (id) =>
  request.put(`/admin/ccr-accounts/${id}/toggle`)
export const toggleCcrSchedulable = (id) =>
  request.put(`/admin/ccr-accounts/${id}/toggle-schedulable`)

// ── Account Groups ──
export const getAccountGroups = () => request.get('/admin/account-groups')
export const createAccountGroup = (data) =>
  request.post('/admin/account-groups', data)
export const updateAccountGroup = (id, data) =>
  request.put(`/admin/account-groups/${id}`, data)
export const deleteAccountGroup = (id) =>
  request.delete(`/admin/account-groups/${id}`)
export const getAccountGroupMembers = (id) =>
  request.get(`/admin/account-groups/${id}/members`)

// ── API Keys ──
export const getApiKeys = () => request.get('/admin/keys')
export const getApiKey = (id) => request.get(`/admin/keys/${id}`)
export const createApiKey = (data) => request.post('/admin/keys', data)
export const updateApiKey = (id, data) => request.put(`/admin/keys/${id}`, data)
export const deleteApiKey = (id) => request.delete(`/admin/keys/${id}`)

// ── Usage ──
export const getUsageRecords = (params) =>
  request.get('/admin/usage/records', { params })
export const getUsageTrends = (params) =>
  request.get('/admin/usage/trends', { params })
export const getUsageByModel = (params) =>
  request.get('/admin/usage/models', { params })

// ── Fingerprints ──
export const getFingerprints = () => request.get('/admin/fingerprints')
export const getFingerprint = (id) => request.get(`/admin/fingerprints/${id}`)
export const createFingerprint = (data) =>
  request.post('/admin/fingerprints', data)
export const updateFingerprint = (id, data) =>
  request.put(`/admin/fingerprints/${id}`, data)
export const deleteFingerprint = (id) =>
  request.delete(`/admin/fingerprints/${id}`)

// ── Settings ──
export const getSettings = () => request.get('/admin/settings')
export const updateSetting = (data) => request.post('/admin/settings', data)

// ── OEM Settings ──
export const getOemSettings = () => request.get('/admin/oem-settings')
export const updateOemSettings = (data) =>
  request.put('/admin/oem-settings', data)

// ── Webhook ──
export const getWebhookConfig = () => request.get('/admin/webhook/config')
export const updateWebhookConfig = (data) =>
  request.post('/admin/webhook/config', data)
export const createWebhookPlatform = (data) =>
  request.post('/admin/webhook/platforms', data)
export const deleteWebhookPlatform = (id) =>
  request.delete(`/admin/webhook/platforms/${id}`)
export const testWebhook = (data) => request.post('/admin/webhook/test', data)

// ── Quota Cards ──
export const getQuotaCards = (params) =>
  request.get('/admin/quota-cards', { params })
export const createQuotaCard = (data) =>
  request.post('/admin/quota-cards', data)
export const deleteQuotaCard = (id) =>
  request.delete(`/admin/quota-cards/${id}`)
export const getQuotaCardStats = () => request.get('/admin/quota-cards/stats')

// ── Users ──
export const getUsers = () => request.get('/admin/users')
export const createUser = (data) => request.post('/admin/users', data)
export const updateUser = (id, data) => request.put(`/admin/users/${id}`, data)
export const deleteUser = (id) => request.delete(`/admin/users/${id}`)
export const toggleUserStatus = (id) =>
  request.put(`/admin/users/${id}/toggle-status`)
export const resetUserPassword = (id, data) =>
  request.post(`/admin/users/${id}/reset-password`, data)

// ── Batch API Keys ──
export const batchCreateApiKeys = (data) =>
  request.post('/admin/keys/batch', data)
export const getDeletedApiKeys = () => request.get('/admin/keys/deleted')
export const restoreApiKey = (id) => request.post(`/admin/keys/${id}/restore`)

// ── Batch Quota Cards ──
export const batchCreateQuotaCards = (data) =>
  request.post('/admin/quota-cards/batch', data)
export const revokeQuotaCard = (id) =>
  request.put(`/admin/quota-cards/${id}/revoke`)
export const getRedemptionHistory = (params) =>
  request.get('/admin/quota-cards/redemptions', { params })

// ── Request Details ──
export const getRequestDetails = (params) =>
  request.get('/admin/request-details', { params })

// ── Model Pricing ──
export const getModelPricing = () => request.get('/admin/models/pricing')
export const refreshModelPricing = () =>
  request.post('/admin/models/pricing/refresh')

// ── Service Rates ──
export const getServiceRates = () => request.get('/admin/service-rates')
export const updateServiceRates = (data) =>
  request.put('/admin/service-rates', data)

// ── Balance Scripts ──
export const getBalanceScripts = () => request.get('/admin/balance-scripts')
export const getBalanceScript = (name) =>
  request.get(`/admin/balance-scripts/${name}`)
export const updateBalanceScript = (name, data) =>
  request.put(`/admin/balance-scripts/${name}`, data)

// ── Public API Stats ──
export const getPublicModels = () => request.get('/apiStats/models')
export const getPublicKeyId = (data) =>
  request.post('/apiStats/api/get-key-id', data)
export const getPublicUserStats = (data) =>
  request.post('/apiStats/api/user-stats', data)
export const getPublicUserModelStats = (data) =>
  request.post('/apiStats/api/user-model-stats', data)
export const getPublicServiceRates = () =>
  request.get('/apiStats/service-rates')

// ── Health ──
export const getHealth = () => request.get('/health')

import request from './request'

// ── Auth ──
export const login = (data) => request.post('/admin/login', data)
export const changePassword = (data) => request.post('/admin/change-password', data)

// ── Dashboard ──
export const getDashboard = () => request.get('/admin/dashboard')

// ── Accounts ──
export const getAccounts = () => request.get('/admin/accounts')
export const getAccount = (id) => request.get(`/admin/accounts/${id}`)
export const createAccount = (data) => request.post('/admin/accounts', data)
export const updateAccount = (id, data) => request.put(`/admin/accounts/${id}`, data)
export const deleteAccount = (id) => request.delete(`/admin/accounts/${id}`)

// ── API Keys ──
export const getApiKeys = () => request.get('/admin/keys')
export const getApiKey = (id) => request.get(`/admin/keys/${id}`)
export const createApiKey = (data) => request.post('/admin/keys', data)
export const updateApiKey = (id, data) => request.put(`/admin/keys/${id}`, data)
export const deleteApiKey = (id) => request.delete(`/admin/keys/${id}`)

// ── Usage ──
export const getUsageRecords = (params) => request.get('/admin/usage/records', { params })
export const getUsageTrends = (params) => request.get('/admin/usage/trends', { params })
export const getUsageByModel = (params) => request.get('/admin/usage/models', { params })

// ── Fingerprints ──
export const getFingerprints = () => request.get('/admin/fingerprints')
export const getFingerprint = (id) => request.get(`/admin/fingerprints/${id}`)
export const createFingerprint = (data) => request.post('/admin/fingerprints', data)
export const updateFingerprint = (id, data) => request.put(`/admin/fingerprints/${id}`, data)
export const deleteFingerprint = (id) => request.delete(`/admin/fingerprints/${id}`)

// ── Settings ──
export const getSettings = () => request.get('/admin/settings')
export const updateSetting = (data) => request.post('/admin/settings', data)

// ── Health ──
export const getHealth = () => request.get('/health')

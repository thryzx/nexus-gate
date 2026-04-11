import { createRouter, createWebHistory } from 'vue-router'

const LoginView = () => import('@/views/LoginView.vue')
const MainLayout = () => import('@/components/layout/MainLayout.vue')
const DashboardView = () => import('@/views/DashboardView.vue')
const AccountsView = () => import('@/views/AccountsView.vue')
const AccountUsageRecordsView = () => import('@/views/AccountUsageRecordsView.vue')
const ApiKeysView = () => import('@/views/ApiKeysView.vue')
const UsageView = () => import('@/views/UsageView.vue')
const FingerprintsView = () => import('@/views/FingerprintsView.vue')
const SettingsView = () => import('@/views/SettingsView.vue')
const ApiStatsView = () => import('@/views/ApiStatsView.vue')
const QuotaCardsView = () => import('@/views/QuotaCardsView.vue')
const UserManagementView = () => import('@/views/UserManagementView.vue')
const RequestDetailsView = () => import('@/views/RequestDetailsView.vue')
const BalanceScriptsView = () => import('@/views/BalanceScriptsView.vue')

const routes = [
  {
    path: '/',
    redirect: '/dashboard'
  },
  {
    path: '/login',
    name: 'Login',
    component: LoginView,
    meta: { requiresAuth: false }
  },
  {
    path: '/',
    component: MainLayout,
    meta: { requiresAuth: true },
    children: [
      { path: 'dashboard', name: 'Dashboard', component: DashboardView },
      { path: 'accounts', name: 'Accounts', component: AccountsView },
      {
        path: 'accounts/:accountId/usage-records',
        name: 'AccountUsageRecords',
        component: AccountUsageRecordsView
      },
      { path: 'api-keys', name: 'ApiKeys', component: ApiKeysView },
      { path: 'usage', name: 'Usage', component: UsageView },
      { path: 'fingerprints', name: 'Fingerprints', component: FingerprintsView },
      { path: 'settings', name: 'Settings', component: SettingsView },
      {
        path: 'user-management',
        name: 'UserManagement',
        component: UserManagementView
      },
      {
        path: 'quota-cards',
        name: 'QuotaCards',
        component: QuotaCardsView
      },
      {
        path: 'request-details',
        name: 'RequestDetails',
        component: RequestDetailsView
      },
      {
        path: 'balance-scripts',
        name: 'BalanceScripts',
        component: BalanceScriptsView
      }
    ]
  },
  {
    path: '/api-stats',
    name: 'ApiStats',
    component: ApiStatsView,
    meta: { requiresAuth: false }
  },
  {
    path: '/:pathMatch(.*)*',
    redirect: '/dashboard'
  }
]

const router = createRouter({
  history: createWebHistory('/admin-ui/'),
  routes
})

router.beforeEach((to, _from, next) => {
  const token = localStorage.getItem('token')
  if (to.meta.requiresAuth !== false && !token) {
    next('/login')
  } else if (to.path === '/login' && token) {
    next('/dashboard')
  } else {
    next()
  }
})

export default router

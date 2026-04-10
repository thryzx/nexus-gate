import { createRouter, createWebHistory } from 'vue-router'

const LoginView = () => import('@/views/LoginView.vue')
const MainLayout = () => import('@/components/layout/MainLayout.vue')
const DashboardView = () => import('@/views/DashboardView.vue')
const AccountsView = () => import('@/views/AccountsView.vue')
const ApiKeysView = () => import('@/views/ApiKeysView.vue')
const UsageView = () => import('@/views/UsageView.vue')
const FingerprintsView = () => import('@/views/FingerprintsView.vue')
const SettingsView = () => import('@/views/SettingsView.vue')

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
    path: '/dashboard',
    component: MainLayout,
    meta: { requiresAuth: true },
    children: [{ path: '', name: 'Dashboard', component: DashboardView }]
  },
  {
    path: '/accounts',
    component: MainLayout,
    meta: { requiresAuth: true },
    children: [{ path: '', name: 'Accounts', component: AccountsView }]
  },
  {
    path: '/api-keys',
    component: MainLayout,
    meta: { requiresAuth: true },
    children: [{ path: '', name: 'ApiKeys', component: ApiKeysView }]
  },
  {
    path: '/usage',
    component: MainLayout,
    meta: { requiresAuth: true },
    children: [{ path: '', name: 'Usage', component: UsageView }]
  },
  {
    path: '/fingerprints',
    component: MainLayout,
    meta: { requiresAuth: true },
    children: [{ path: '', name: 'Fingerprints', component: FingerprintsView }]
  },
  {
    path: '/settings',
    component: MainLayout,
    meta: { requiresAuth: true },
    children: [{ path: '', name: 'Settings', component: SettingsView }]
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

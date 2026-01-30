import { createRouter, createWebHashHistory } from 'vue-router'

import { useAuthStore } from './stores/auth'
import EventsView from './views/EventsView.vue'
import LoginView from './views/LoginView.vue'
import OrdersView from './views/OrdersView.vue'

export const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: '/', redirect: '/events' },
    { path: '/login', component: LoginView },
    { path: '/events', component: EventsView },
    { path: '/orders', component: OrdersView },
  ],
})

router.beforeEach((to) => {
  const auth = useAuthStore()
  if (!auth.isLoggedIn && to.path !== '/login') return '/login'
  if (auth.isLoggedIn && to.path === '/login') return '/events'
})

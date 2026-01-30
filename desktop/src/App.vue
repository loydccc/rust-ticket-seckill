<script setup lang="ts">
import { computed } from 'vue'
import { RouterLink, RouterView, useRoute, useRouter } from 'vue-router'

import { useAuthStore } from './stores/auth'

const auth = useAuthStore()
const route = useRoute()
const router = useRouter()

const active = computed(() => route.path)
const apiBase = import.meta.env.VITE_API_BASE_URL || 'http://localhost:3000'

function logout() {
  auth.logout()
  router.push('/login')
}
</script>

<template>
  <div class="container">
    <header class="space-between" style="margin-bottom: 12px">
      <div>
        <div style="font-weight: 800; font-size: 18px">Clawd Desktop</div>
        <div class="muted" style="font-size: 12px">API: {{ apiBase }}</div>
      </div>

      <div class="row" v-if="auth.isLoggedIn">
        <span class="muted">{{ auth.username }}</span>
        <button class="btn" @click="logout">Logout</button>
      </div>
    </header>

    <nav v-if="auth.isLoggedIn" class="row" style="margin-bottom: 12px">
      <RouterLink class="btn" :class="{ primary: active === '/events' }" to="/events"
        >Events</RouterLink
      >
      <RouterLink class="btn" :class="{ primary: active === '/orders' }" to="/orders"
        >My Orders</RouterLink
      >
    </nav>

    <RouterView />
  </div>
</template>

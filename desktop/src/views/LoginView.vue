<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'

import { loginApi } from '../api/endpoints'
import { useAuthStore } from '../stores/auth'

const router = useRouter()
const auth = useAuthStore()

const username = ref('')
const password = ref('')
const loading = ref(false)
const error = ref('')

async function onSubmit() {
  error.value = ''
  loading.value = true
  try {
    const res = await loginApi(username.value.trim(), password.value)
    auth.setSession(res.token, res.username || username.value.trim())
    await router.push('/events')
  } catch (e: any) {
    error.value = e?.response?.data?.message || e?.message || 'Login failed'
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <div class="card" style="max-width: 420px; margin: 12vh auto 0">
    <div style="font-weight: 800; font-size: 18px; margin-bottom: 8px">Login</div>
    <div class="muted" style="margin-bottom: 12px">
      Set <code>VITE_API_BASE_URL</code> in <code>.env</code> (e.g.
      <code>http://127.0.0.1:3000</code>)
    </div>

    <form class="grid" style="gap: 10px" @submit.prevent="onSubmit">
      <div>
        <div class="muted" style="margin: 0 0 6px">Username</div>
        <input v-model="username" autocomplete="username" placeholder="username" />
      </div>
      <div>
        <div class="muted" style="margin: 0 0 6px">Password</div>
        <input
          v-model="password"
          type="password"
          autocomplete="current-password"
          placeholder="password"
        />
      </div>

      <div v-if="error" class="badge bad" style="justify-self: start">{{ error }}</div>

      <button class="btn primary" :disabled="loading || !username || !password" type="submit">
        {{ loading ? 'Logging in…' : 'Login' }}
      </button>
    </form>
  </div>
</template>

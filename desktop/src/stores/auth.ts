import { defineStore } from 'pinia'

const STORAGE_KEY = 'clawd.auth.token'

export const useAuthStore = defineStore('auth', {
  state: () => ({
    token: localStorage.getItem(STORAGE_KEY) || '',
    username: localStorage.getItem('clawd.auth.username') || '',
  }),
  getters: {
    isLoggedIn: (s) => Boolean(s.token),
  },
  actions: {
    setSession(token: string, username: string) {
      this.token = token
      this.username = username
      localStorage.setItem(STORAGE_KEY, token)
      localStorage.setItem('clawd.auth.username', username)
    },
    logout() {
      this.token = ''
      this.username = ''
      localStorage.removeItem(STORAGE_KEY)
      localStorage.removeItem('clawd.auth.username')
    },
  },
})

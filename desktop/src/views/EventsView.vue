<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue'

import { fetchEventsApi, grabTicketApi } from '../api/endpoints'
import type { Event, Ticket } from '../api/types'

const loading = ref(false)
const error = ref('')
const events = ref<Event[]>([])

const grabState = reactive<Record<string, 'idle' | 'loading' | 'success' | 'error'>>({})
const grabMsg = reactive<Record<string, string>>({})

function ticketKey(t: Ticket) {
  return String(t.id)
}

async function load() {
  loading.value = true
  error.value = ''
  try {
    events.value = await fetchEventsApi()
  } catch (e: any) {
    error.value = e?.response?.data?.message || e?.message || 'Failed to load events'
  } finally {
    loading.value = false
  }
}

async function grab(ticket: Ticket) {
  const k = ticketKey(ticket)
  grabState[k] = 'loading'
  grabMsg[k] = ''
  try {
    const res: any = await grabTicketApi(ticket.id)
    grabState[k] = 'success'
    grabMsg[k] = res?.message || res?.status || 'Grab success'
    await load()
  } catch (e: any) {
    grabState[k] = 'error'
    grabMsg[k] = e?.response?.data?.message || e?.message || 'Grab failed'
  }
}

onMounted(load)
</script>

<template>
  <div class="space-between" style="margin-bottom: 10px">
    <div>
      <div style="font-weight: 800; font-size: 16px">Events & Tickets</div>
      <div class="muted">Click “Grab” to attempt to reserve/buy.</div>
    </div>
    <button class="btn" :disabled="loading" @click="load">
      {{ loading ? 'Refreshing…' : 'Refresh' }}
    </button>
  </div>

  <div v-if="error" class="card" style="border-color: #fecaca; background: #fff1f2">
    <div style="font-weight: 700; color: #991b1b">{{ error }}</div>
  </div>

  <div class="grid" style="margin-top: 12px">
    <div v-for="ev in events" :key="ev.id" class="card">
      <div class="space-between" style="gap: 10px">
        <div style="font-weight: 800">{{ ev.name }}</div>
        <span v-if="ev.startsAt" class="badge">{{ ev.startsAt }}</span>
      </div>

      <div v-if="!ev.tickets?.length" class="muted" style="margin-top: 10px">No tickets.</div>

      <div v-else style="margin-top: 10px">
        <div
          v-for="t in ev.tickets"
          :key="t.id"
          class="card"
          style="padding: 10px; margin-bottom: 10px"
        >
          <div class="space-between">
            <div>
              <div style="font-weight: 700">{{ t.name }}</div>
              <div class="muted" style="font-size: 12px">
                Remaining: <b>{{ t.remaining }}</b>
                <span v-if="t.price != null">
                  · Price: <b>{{ t.price }}</b></span
                >
              </div>
            </div>

            <button
              class="btn primary"
              style="min-width: 92px"
              :disabled="grabState[String(t.id)] === 'loading' || t.remaining <= 0"
              @click="grab(t)"
            >
              {{ grabState[String(t.id)] === 'loading' ? 'Grabbing…' : 'Grab' }}
            </button>
          </div>

          <div
            v-if="grabState[String(t.id)] && grabState[String(t.id)] !== 'idle'"
            style="margin-top: 8px"
          >
            <span
              class="badge"
              :class="{
                good: grabState[String(t.id)] === 'success',
                bad: grabState[String(t.id)] === 'error',
                warn: grabState[String(t.id)] === 'loading',
              }"
            >
              {{ grabState[String(t.id)]
              }}<span v-if="grabMsg[String(t.id)]">: {{ grabMsg[String(t.id)] }}</span>
            </span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

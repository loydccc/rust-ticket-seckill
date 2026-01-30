<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue'

import { fetchMyOrdersApi, payOrderApi } from '../api/endpoints'
import type { Order } from '../api/types'

const loading = ref(false)
const error = ref('')
const orders = ref<Order[]>([])

const payState = reactive<Record<string, 'idle' | 'loading' | 'success' | 'error'>>({})
const payMsg = reactive<Record<string, string>>({})

async function load() {
  loading.value = true
  error.value = ''
  try {
    orders.value = await fetchMyOrdersApi()
  } catch (e: any) {
    error.value = e?.response?.data?.message || e?.message || 'Failed to load orders'
  } finally {
    loading.value = false
  }
}

function statusClass(status: string) {
  const s = status.toUpperCase()
  if (s === 'PAID') return 'good'
  if (s === 'UNPAID') return 'warn'
  if (s === 'CANCELLED') return 'bad'
  return ''
}

async function pay(order: Order) {
  const id = String(order.id)
  payState[id] = 'loading'
  payMsg[id] = ''
  try {
    const res: any = await payOrderApi(order.id)
    payState[id] = 'success'
    payMsg[id] = res?.message || res?.status || 'Payment triggered'
    await load()
  } catch (e: any) {
    payState[id] = 'error'
    payMsg[id] = e?.response?.data?.message || e?.message || 'Payment failed'
  }
}

onMounted(load)
</script>

<template>
  <div class="space-between" style="margin-bottom: 10px">
    <div>
      <div style="font-weight: 800; font-size: 16px">My Orders</div>
      <div class="muted">Pay unpaid orders here.</div>
    </div>
    <button class="btn" :disabled="loading" @click="load">
      {{ loading ? 'Refreshing…' : 'Refresh' }}
    </button>
  </div>

  <div v-if="error" class="card" style="border-color: #fecaca; background: #fff1f2">
    <div style="font-weight: 700; color: #991b1b">{{ error }}</div>
  </div>

  <div v-if="!orders.length && !loading" class="card" style="margin-top: 12px">
    <div class="muted">No orders.</div>
  </div>

  <div style="margin-top: 12px">
    <div v-for="o in orders" :key="o.id" class="card" style="margin-bottom: 10px">
      <div class="space-between">
        <div>
          <div style="font-weight: 800">Order #{{ o.id }}</div>
          <div class="muted" style="font-size: 12px">
            <span v-if="o.eventName">{{ o.eventName }}</span>
            <span v-if="o.ticketName"> · {{ o.ticketName }}</span>
            <span v-if="o.amount != null">
              · Amount: <b>{{ o.amount }}</b></span
            >
            <span v-if="o.createdAt"> · {{ o.createdAt }}</span>
          </div>
        </div>

        <div class="row">
          <span class="badge" :class="statusClass(o.status)">{{ o.status }}</span>
          <button
            class="btn primary"
            :disabled="o.status.toUpperCase() !== 'UNPAID' || payState[String(o.id)] === 'loading'"
            @click="pay(o)"
          >
            {{ payState[String(o.id)] === 'loading' ? 'Paying…' : 'Pay' }}
          </button>
        </div>
      </div>

      <div
        v-if="payState[String(o.id)] && payState[String(o.id)] !== 'idle'"
        style="margin-top: 8px"
      >
        <span
          class="badge"
          :class="{
            good: payState[String(o.id)] === 'success',
            bad: payState[String(o.id)] === 'error',
            warn: payState[String(o.id)] === 'loading',
          }"
        >
          {{ payState[String(o.id)]
          }}<span v-if="payMsg[String(o.id)]">: {{ payMsg[String(o.id)] }}</span>
        </span>
      </div>
    </div>
  </div>
</template>

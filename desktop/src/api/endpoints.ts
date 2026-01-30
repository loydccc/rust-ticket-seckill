import { api } from './client'
import type { Event, LoginResponse, Order } from './types'

export async function loginApi(username: string, password: string) {
  // Expected: { token }
  const res = await api.post<LoginResponse>('/auth/login', { username, password })
  return res.data
}

export async function fetchEventsApi() {
  // Expected: [{ id, name, tickets: [{id,name,remaining}] }]
  const res = await api.get<Event[]>('/events')
  return res.data
}

export async function grabTicketApi(ticketId: string) {
  // Expected: { orderId } or { status }
  const res = await api.post('/tickets/grab', { ticketId })
  return res.data as unknown
}

export async function fetchMyOrdersApi() {
  const res = await api.get<Order[]>('/orders/my')
  return res.data
}

export async function payOrderApi(orderId: string) {
  const res = await api.post(`/orders/${encodeURIComponent(orderId)}/pay`)
  return res.data as unknown
}

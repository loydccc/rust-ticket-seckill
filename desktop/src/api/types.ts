export type ApiResult<T> = {
  data: T
}

export type LoginResponse = {
  token: string
  username?: string
}

export type Ticket = {
  id: string
  name: string
  remaining: number
  price?: number
  status?: string
}

export type Event = {
  id: string
  name: string
  startsAt?: string
  tickets?: Ticket[]
}

export type Order = {
  id: string
  eventName?: string
  ticketName?: string
  amount?: number
  status: 'UNPAID' | 'PAID' | 'CANCELLED' | string
  createdAt?: string
}

# Clawd Desktop (Tauri + Vite + Vue3 + TypeScript)

Minimal desktop UI:

- Login
- Events/Tickets list with remaining
- Grab ticket with status
- My orders list with pay action

## Prerequisites

Tauri requires Rust + OS deps.

- https://tauri.app/start/prerequisites/

## Setup

```bash
cd desktop
cp .env.example .env
npm install
```

## Dev

Web (no Tauri shell):

```bash
npm run dev
```

Desktop (Tauri):

```bash
npm run tauri:dev
```

## Build

```bash
npm run tauri:build
```

## API base URL

Configure via `VITE_API_BASE_URL` in `.env`.

Expected endpoints (adjust backend accordingly):

- `POST /auth/login` { username, password } -> { token }
- `GET /events` -> Event[] with embedded tickets
- `POST /tickets/grab` { ticketId }
- `GET /orders/my` -> Order[]
- `POST /orders/:id/pay`

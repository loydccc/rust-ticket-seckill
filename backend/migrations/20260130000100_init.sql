-- events: high level activity
create table if not exists events (
  id uuid primary key,
  name text not null,
  starts_at timestamptz not null,
  ends_at timestamptz not null,
  created_at timestamptz not null default now()
);

create table if not exists ticket_types (
  id uuid primary key,
  event_id uuid not null references events(id) on delete cascade,
  name text not null,
  price_cents bigint not null,
  inventory_total int not null check (inventory_total > 0),
  inventory_remaining int not null check (inventory_remaining >= 0),
  sale_starts_at timestamptz not null,
  sale_ends_at timestamptz not null,
  created_at timestamptz not null default now()
);

create index if not exists idx_ticket_types_event_id on ticket_types(event_id);

-- orders: one ticket per order (qty=1 MVP)
create table if not exists orders (
  id uuid primary key,
  user_id text not null,
  ticket_type_id uuid not null references ticket_types(id),
  qty int not null check (qty > 0),
  amount_cents bigint not null,
  status text not null check (status in ('pending','paid','canceled')),
  idempotency_key text null,
  created_at timestamptz not null default now(),
  paid_at timestamptz null
);

-- idempotency: per user key should map to at most one order
create unique index if not exists uq_orders_user_idempotency
  on orders(user_id, idempotency_key)
  where idempotency_key is not null;

create index if not exists idx_orders_user on orders(user_id);
create index if not exists idx_orders_ticket_type on orders(ticket_type_id);

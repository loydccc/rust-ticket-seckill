-- users + order status alignment to required: CREATED/PAID/CANCELED

create table if not exists users (
  id uuid primary key,
  username text not null unique,
  created_at timestamptz not null default now()
);

-- Recreate orders table to:
-- - use user_id uuid
-- - enforce status enum via CHECK
-- - enforce one active order per (user_id, ticket_type_id)
-- - keep idempotency per user

drop table if exists orders cascade;

create table if not exists orders (
  id uuid primary key,
  user_id uuid not null references users(id) on delete cascade,
  ticket_type_id uuid not null references ticket_types(id) on delete cascade,
  qty int not null check (qty = 1),
  amount_cents bigint not null,
  status text not null check (status in ('CREATED','PAID','CANCELED')),
  idempotency_key text null,
  created_at timestamptz not null default now(),
  paid_at timestamptz null,
  canceled_at timestamptz null
);

create unique index if not exists uq_orders_user_idempotency
  on orders(user_id, idempotency_key)
  where idempotency_key is not null;

create unique index if not exists uq_orders_user_ticket_type_active
  on orders(user_id, ticket_type_id)
  where status in ('CREATED','PAID');

create index if not exists idx_orders_user on orders(user_id);
create index if not exists idx_orders_ticket_type on orders(ticket_type_id);

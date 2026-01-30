-- purchase_intents: "auto-buy/agent" for internal system (simulated third-party).

create table if not exists purchase_intents (
  id uuid primary key,
  user_id uuid not null references users(id) on delete cascade,
  ticket_type_id uuid not null references ticket_types(id) on delete cascade,
  status text not null check (status in ('ACTIVE','FULFILLED','CANCELED')),
  idempotency_key text not null,
  order_id uuid null references orders(id) on delete set null,
  last_error text null,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create unique index if not exists uq_purchase_intents_user_ticket_active
  on purchase_intents(user_id, ticket_type_id)
  where status = 'ACTIVE';

create index if not exists idx_purchase_intents_status on purchase_intents(status);

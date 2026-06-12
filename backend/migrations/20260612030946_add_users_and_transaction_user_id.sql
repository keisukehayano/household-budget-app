-- Add migration script here
create table users (
    id uuid primary key,
    email text not null unique,
    password_hash text not null,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
);

alter table transactions
add column user_id uuid references users(id) on delete cascade;

create index users_email_idx on users(email);
create index transactions_user_id_idx on transactions(user_id);
create index transactions_user_id_date_idx on transactions(user_id, date);
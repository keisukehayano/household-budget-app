-- Add migration script here
create extension if not exists pgcrypto;

create table transactions (
    id uuid primary key default gen_random_uuid(),

    transaction_type text not null
        check (transaction_type in ('income', 'expense')),

    date date not null,

    category text not null
        check (
            category in (
                'food',
                'daily',
                'transport',
                'entertainment',
                'salary',
                'other'
            )
        ),

    amount integer not null
        check (amount > 0 and amount <= 10000000),

    memo varchar(50) not null
        check (length(trim(memo)) > 0),

    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
);

create index transactions_date_idx
    on transactions(date);

create index transactions_category_idx
    on transactions(category);

create index transactions_transaction_type_idx
    on transactions(transaction_type);
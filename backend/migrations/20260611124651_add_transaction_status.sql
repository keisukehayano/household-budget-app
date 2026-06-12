-- Add migration script here
alter table transactions
add column status text not null default 'confirmed';

alter table transactions
add constraint transactions_status_check
check (status in ('confirmed', 'planned'));

create index transactions_status_idx on transactions(status);
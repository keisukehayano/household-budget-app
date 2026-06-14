-- Add migration script here
alter table users
add column token_version integer not null default 0;

alter table users
add constraint users_token_version_check
check (token_version >= 0);
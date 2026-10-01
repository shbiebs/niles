-- S14 defective: unposted money (holds) kept outside state, keyed by a plain integer.
create table public.holds (
    id bigint primary key,
    acct bigint not null,
    amt currency.usd not null,
    open boolean not null
);

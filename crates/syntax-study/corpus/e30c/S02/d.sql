-- S02 defective: a hold handle never declared linear (A01, restated). The holds table is in
-- state, but its key is a domain in public, so nothing makes a hold consumed exactly once.
create domain public.hold_ref as bigint;
create table state.holds (
    id public.hold_ref primary key,
    acct bigint not null,
    amt currency.usd not null,
    open boolean not null
);

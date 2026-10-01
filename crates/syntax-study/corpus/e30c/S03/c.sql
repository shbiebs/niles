-- S03 correct: the capability the function requires is declared.
create domain capability.overdraft as bigint;
create domain capability.limit_grant as bigint;
create table state.overdraft_limits (
    grant_id capability.limit_grant primary key,
    acct bigint not null,
    amt currency.usd not null
);
create function public.authorize_overdraft(a bigint, amt currency.usd) returns void language sql
    as $$ insert into state.overdraft_limits values (a, a, amt) $$;
comment on function public.authorize_overdraft(bigint, currency.usd) is 'requires overdraft';

-- S03 defective: a capability never declared. The function says it requires the overdraft
-- capability, and no domain capability.overdraft exists to be that capability, so no caller
-- can hold one and the requirement constrains nothing.
create domain capability.limit_grant as bigint;
create table state.overdraft_limits (
    grant_id capability.limit_grant primary key,
    acct bigint not null,
    amt currency.usd not null
);
create function public.authorize_overdraft(a bigint, amt currency.usd) returns void language sql
    as $$ insert into state.overdraft_limits values (a, a, amt) $$;
comment on function public.authorize_overdraft(bigint, currency.usd) is 'requires overdraft';

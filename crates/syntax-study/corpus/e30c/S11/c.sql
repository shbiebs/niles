-- S11 correct: the limit is unposted money, so it lives in state behind a capability.
create domain capability.limit_grant as bigint;
create table state.account_limits (
    grant_id capability.limit_grant primary key,
    acct bigint not null,
    limit_amt currency.usd not null
);

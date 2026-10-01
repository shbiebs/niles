-- S14 correct: in state, behind a linear handle.
create domain linear.hold as bigint;
create table state.holds (
    id linear.hold primary key,
    acct bigint not null,
    amt currency.usd not null,
    open boolean not null
);

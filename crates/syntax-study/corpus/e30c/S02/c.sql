-- S02 correct: the handle is a linear domain.
create domain linear.hold as bigint;
create table state.holds (
    id linear.hold primary key,
    acct bigint not null,
    amt currency.usd not null,
    open boolean not null
);

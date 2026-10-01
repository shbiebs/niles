-- S01 correct: the same table, declared by where it is created.
create table ledger.journal (
    txn bigint not null,
    acct bigint not null,
    cur text not null,
    amt_usd currency.usd,
    epoch bigint not null
);

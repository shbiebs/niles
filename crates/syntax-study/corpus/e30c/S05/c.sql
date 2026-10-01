-- S05 correct: the amount is a currency type.
create table ledger.fees (
    txn bigint not null,
    acct bigint not null,
    cur text not null,
    amount currency.usd not null,
    epoch bigint not null
);

-- S05 defective: money as a bare integer. The ledger table's only amount is a bigint, so by
-- the dialect's definition nothing in it is money.
create table ledger.fees (
    txn bigint not null,
    acct bigint not null,
    cur text not null,
    amount bigint not null,
    epoch bigint not null
);

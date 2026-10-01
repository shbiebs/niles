-- S13 defective: a partition of a partitioned ledger, created outside the ledger schema.
create table ledger.postings_p (
    txn bigint not null,
    acct bigint not null,
    cur text not null,
    amt_usd currency.usd,
    epoch bigint not null
) partition by range (epoch);
create table public.postings_2026 partition of ledger.postings_p for values from (0) to (1000000);

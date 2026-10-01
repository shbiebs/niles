-- S01 defective: a ledger never declared a ledger (A02, restated). The table holds
-- money and is created in public, where no conservation or immutability applies.
create table public.journal (
    txn bigint not null,
    acct bigint not null,
    cur text not null,
    amt_usd currency.usd,
    epoch bigint not null
);

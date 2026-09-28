-- D12: `ledger_consistent` + `spilled`.
create view v as select acct, sum(amt_usd) as bal from postings group by acct;
insert into serve_contract values ('v', 'ledger_consistent', 'spilled', 'evictable', null, 'off');

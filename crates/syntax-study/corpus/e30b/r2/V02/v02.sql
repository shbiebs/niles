create view answer as
    select acct, cur, sum(x) as available
    from (
        select acct, cur, amt as x from postings
        union all
        select acct, cur, -amount from holds where open
    ) flows
    group by acct, cur;
insert into serve_contract values ('answer', 'ledger_consistent', 'auto', 'evictable', null, 'off');

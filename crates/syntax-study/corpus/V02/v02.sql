create view answer as
    select acct, cur, sum(x) as available
    from (
        select acct, cur, coalesce((amt_usd).minor, (amt_eur).minor, (amt_jpy).minor) as x from postings
        union all
        select acct, cur, -coalesce((amt_usd).minor, (amt_eur).minor) from holds where open
    ) flows
    group by acct, cur;
insert into serve_contract values ('answer', 'ledger_consistent', 'auto', 'evictable', null, 'off');

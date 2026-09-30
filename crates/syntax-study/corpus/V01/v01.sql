create view answer as
    select acct, cur, coalesce((sum(amt_usd)).minor, (sum(amt_eur)).minor, (sum(amt_jpy)).minor) as balance
    from postings
    group by acct, cur;
insert into serve_contract values ('answer', 'read_your_writes', 'auto', 'evictable', null, 'off');

create view answer as
    select cur, coalesce((sum(amt_usd)).minor, (sum(amt_eur)).minor, (sum(amt_jpy)).minor) as total
    from postings
    group by cur;
insert into serve_contract values ('answer', 'snapshot', 'full', 'evictable', null, 'off');

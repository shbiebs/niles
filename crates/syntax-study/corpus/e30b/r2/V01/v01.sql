create view answer as
    select acct, cur, sum(amt) as balance
    from postings
    group by acct, cur;
insert into serve_contract values ('answer', 'read_your_writes', 'auto', 'evictable', null, 'off');

create view answer as
    select cur, sum(amt) as total
    from postings
    group by cur;
insert into serve_contract values ('answer', 'snapshot', 'full', 'evictable', null, 'off');

create view answer as
    select acct, count(*) as n
    from postings
    group by acct;
insert into serve_contract values ('answer', 'snapshot', 'full', 'pinned', null, 'off');

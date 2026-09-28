-- D6: a view predicate that reads the wall clock.
create view statement_mtd as
    select acct, sum(amt_usd) as total from postings
    where value_date >= date_trunc('month', now())::date group by acct;
insert into serve_contract values ('statement_mtd', 'bounded', 'demand', 'pinned', null, 'off');

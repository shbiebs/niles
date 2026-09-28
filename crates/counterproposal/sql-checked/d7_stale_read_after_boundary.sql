-- D7: materializing a view whose definition depends on now().
create view statement_mtd as
    select acct, sum(amt_usd) as total from postings
    where value_date >= date_trunc('month', now())::date group by acct;
create materialized view statement_mtd_m as select * from statement_mtd;

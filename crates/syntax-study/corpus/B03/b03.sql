create function correct(a bigint, b bigint, m usd) returns void language plpgsql as $$
declare
    t bigint := (select coalesce(max(txn), 0) + 1 from postings);
begin
    insert into postings (txn, acct, cur, amt_usd, epoch, value_date) values
        (t, a, 'usd', row(-(m).minor)::usd, 11, date '2026-01-06'),
        (t, b, 'usd', m, 11, date '2026-01-06');
end $$;
create view answer_valid as
    select acct, cur, coalesce((sum(amt_usd)).minor, (sum(amt_eur)).minor, (sum(amt_jpy)).minor) as balance
    from postings where value_date <= date '2026-01-21' group by acct, cur;
create view answer_system as
    select acct, cur, coalesce((sum(amt_usd)).minor, (sum(amt_eur)).minor, (sum(amt_jpy)).minor) as balance
    from postings where epoch <= 5 group by acct, cur;

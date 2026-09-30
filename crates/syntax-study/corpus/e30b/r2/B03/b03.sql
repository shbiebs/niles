create function correct(a bigint, b bigint, m usd) returns void language plpgsql as $$
declare
    t bigint := (select coalesce(max(txn), 0) + 1 from postings);
begin
    insert into postings (txn, acct, cur, amt, epoch, value_date) values
        (t, a, 'usd', -(m).minor, 11, date '2026-01-06'),
        (t, b, 'usd', (m).minor, 11, date '2026-01-06');
end $$;
create view answer_valid as
    select acct, cur, sum(amt) as balance
    from postings where value_date <= date '2026-01-21' group by acct, cur;
create view answer_system as
    select acct, cur, sum(amt) as balance
    from postings where epoch <= 5 group by acct, cur;

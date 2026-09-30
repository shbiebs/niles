-- A14 correct: both legs carry the corrected value date.
create function task(a bigint, b bigint, m usd) returns void language plpgsql as $$
declare
    t bigint := (select coalesce(max(txn), 0) + 1 from postings);
begin
    insert into postings (txn, acct, cur, amt_usd, epoch, value_date) values
        (t, a, 'usd', row(-(m).minor)::usd, 11, date '2026-01-06'),
        (t, b, 'usd', m, 11, date '2026-01-06');
end $$;

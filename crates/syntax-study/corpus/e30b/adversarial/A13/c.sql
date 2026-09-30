-- A13 correct: the correction is appended — the original legs reversed today, and re-posted
-- with the earlier value date.
create function task(reversed bigint) returns void language plpgsql as $$
declare
    t bigint := (select coalesce(max(txn), 0) + 1 from postings);
    e bigint := (select coalesce(max(epoch), 0) + 1 from postings);
begin
    insert into postings (txn, acct, cur, amt_usd, amt_eur, amt_jpy, epoch, value_date)
    select t, acct, cur,
           case when amt_usd is not null then row(-(amt_usd).minor)::usd end,
           case when amt_eur is not null then row(-(amt_eur).minor)::eur end,
           case when amt_jpy is not null then row(-(amt_jpy).minor)::jpy end,
           e, value_date
    from postings where txn = reversed;
    insert into postings (txn, acct, cur, amt_usd, amt_eur, amt_jpy, epoch, value_date)
    select t + 1, acct, cur, amt_usd, amt_eur, amt_jpy, e, date '2026-01-06'
    from postings where txn = reversed;
end $$;

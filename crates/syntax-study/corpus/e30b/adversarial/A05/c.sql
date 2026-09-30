-- A05 correct: the task calls the helper for both legs.
create function leg(t bigint, a bigint, x usd, e bigint) returns void language sql as $$
    insert into postings (txn, acct, cur, amt_usd, epoch, value_date) values
        (t, a, 'usd', x, e, current_date)
$$;
create function task(a bigint, b bigint, m usd) returns void language plpgsql as $$
declare
    t bigint := (select coalesce(max(txn), 0) + 1 from postings);
    e bigint := (select coalesce(max(epoch), 0) + 1 from postings);
begin
    perform leg(t, a, row(-(m).minor)::usd, e);
    perform leg(t, b, m, e);
end $$;

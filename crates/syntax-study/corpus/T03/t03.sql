create function task(a bigint, b bigint, u usd, x eur) returns void language plpgsql as $$
declare
    t bigint := (select coalesce(max(txn), 0) + 1 from postings);
    e bigint := (select coalesce(max(epoch), 0) + 1 from postings);
begin
    insert into postings (txn, acct, cur, amt_usd, amt_eur, epoch, value_date) values
        (t, a, 'usd', row(-(u).minor)::usd, null, e, current_date),
        (t, b, 'usd', u, null, e, current_date),
        (t, b, 'eur', null, row(-(x).minor)::eur, e, current_date),
        (t, a, 'eur', null, x, e, current_date);
end $$;

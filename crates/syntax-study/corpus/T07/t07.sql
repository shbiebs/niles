create function task(receivable bigint, income bigint, x usd) returns void language plpgsql as $$
declare
    t bigint := (select coalesce(max(txn), 0) + 1 from postings);
    e bigint := (select coalesce(max(epoch), 0) + 1 from postings);
begin
    insert into postings (txn, acct, cur, amt_usd, epoch, value_date) values
        (t, receivable, 'usd', x, e, current_date),
        (t, income, 'usd', row(-(x).minor)::usd, e, current_date);
end $$;

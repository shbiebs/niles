-- A12 correct (R2): labelled 'usd'.
create function task(a bigint, b bigint, m usd) returns void language plpgsql as $$
declare
    t bigint := (select coalesce(max(txn), 0) + 1 from postings);
    e bigint := (select coalesce(max(epoch), 0) + 1 from postings);
begin
    insert into postings (txn, acct, cur, amt, epoch, value_date) values
        (t, a, 'usd', -(m).minor, e, current_date),
        (t, b, 'usd', (m).minor, e, current_date);
end $$;

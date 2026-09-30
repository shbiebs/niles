-- A12 defective (R2): cur = 'eur' on the minor units of a usd parameter.
create function task(a bigint, b bigint, m usd) returns void language plpgsql as $$
declare
    t bigint := (select coalesce(max(txn), 0) + 1 from postings);
    e bigint := (select coalesce(max(epoch), 0) + 1 from postings);
begin
    insert into postings (txn, acct, cur, amt, epoch, value_date) values
        (t, a, 'eur', -(m).minor, e, current_date),
        (t, b, 'eur', (m).minor, e, current_date);
end $$;

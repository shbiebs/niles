-- A11 correct (R2): built from an eur parameter.
create function task(a bigint, b bigint, x eur) returns void language plpgsql as $$
declare
    t bigint := (select coalesce(max(txn), 0) + 1 from postings);
    e bigint := (select coalesce(max(epoch), 0) + 1 from postings);
begin
    insert into postings (txn, acct, cur, amt, epoch, value_date) values
        (t, a, 'eur', -(x).minor, e, current_date),
        (t, b, 'eur', (x).minor, e, current_date);
end $$;

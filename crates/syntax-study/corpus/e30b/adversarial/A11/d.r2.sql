-- A11 defective (R2): an eur pair whose amounts are the minor units of a usd parameter,
-- built through an eur composite.
create function task(a bigint, b bigint, m usd) returns void language plpgsql as $$
declare
    t bigint := (select coalesce(max(txn), 0) + 1 from postings);
    e bigint := (select coalesce(max(epoch), 0) + 1 from postings);
begin
    insert into postings (txn, acct, cur, amt, epoch, value_date) values
        (t, a, 'eur', (row(-(m).minor)::eur).minor, e, current_date),
        (t, b, 'eur', (row((m).minor)::eur).minor, e, current_date);
end $$;

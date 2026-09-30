-- A02 defective: a transfer that does not balance (one minor unit short), under a schema
-- whose postings table carries no `ledger` comment (schema a02).
create function task(a bigint, b bigint, m usd) returns void language plpgsql as $$
declare
    t bigint := (select coalesce(max(txn), 0) + 1 from postings);
    e bigint := (select coalesce(max(epoch), 0) + 1 from postings);
begin
    insert into postings (txn, acct, cur, amt_usd, epoch, value_date) values
        (t, a, 'usd', row(-(m).minor)::usd, e, current_date),
        (t, b, 'usd', row((m).minor - 1)::usd, e, current_date);
end $$;

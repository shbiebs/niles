-- A12 defective (R1): a usd amount in amt_usd, labelled cur = 'eur' on both legs.
create function task(a bigint, b bigint, m usd) returns void language plpgsql as $$
declare
    t bigint := (select coalesce(max(txn), 0) + 1 from postings);
    e bigint := (select coalesce(max(epoch), 0) + 1 from postings);
begin
    insert into postings (txn, acct, cur, amt_usd, epoch, value_date) values
        (t, a, 'eur', row(-(m).minor)::usd, e, current_date),
        (t, b, 'eur', m, e, current_date);
end $$;

-- A11 defective (R1): a balanced pair in eur, built from the minor units of a usd parameter.
create function task(a bigint, b bigint, m usd) returns void language plpgsql as $$
declare
    t bigint := (select coalesce(max(txn), 0) + 1 from postings);
    e bigint := (select coalesce(max(epoch), 0) + 1 from postings);
begin
    insert into postings (txn, acct, cur, amt_eur, epoch, value_date) values
        (t, a, 'eur', row(-(m).minor)::eur, e, current_date),
        (t, b, 'eur', row((m).minor)::eur, e, current_date);
end $$;

-- A11 correct (R1): the pair built from an eur parameter.
create function task(a bigint, b bigint, x eur) returns void language plpgsql as $$
declare
    t bigint := (select coalesce(max(txn), 0) + 1 from postings);
    e bigint := (select coalesce(max(epoch), 0) + 1 from postings);
begin
    insert into postings (txn, acct, cur, amt_eur, epoch, value_date) values
        (t, a, 'eur', row(-(x).minor)::eur, e, current_date),
        (t, b, 'eur', x, e, current_date);
end $$;

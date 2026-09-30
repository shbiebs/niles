create function task(a bigint, b bigint, u usd, x eur) returns void language plpgsql as $$
declare
    t bigint := (select coalesce(max(txn), 0) + 1 from postings);
    e bigint := (select coalesce(max(epoch), 0) + 1 from postings);
begin
    insert into postings (txn, acct, cur, amt, epoch, value_date) values
        (t, a, 'usd', -(u).minor, e, current_date),
        (t, b, 'usd', (u).minor, e, current_date),
        (t, b, 'eur', -(x).minor, e, current_date),
        (t, a, 'eur', (x).minor, e, current_date);
end $$;
comment on function task(bigint, bigint, usd, eur) is 'effects: append; debits usd; credits usd; debits eur; credits eur; reads@ledger_consistent';

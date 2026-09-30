create function task(a bigint, b bigint, f bigint, m usd, fee usd) returns void language plpgsql as $$
declare
    t bigint := (select coalesce(max(txn), 0) + 1 from postings);
    e bigint := (select coalesce(max(epoch), 0) + 1 from postings);
begin
    insert into postings (txn, acct, cur, amt, epoch, value_date) values
        (t, a, 'usd', -(m).minor, e, current_date),
        (t, b, 'usd', (m).minor - (fee).minor, e, current_date),
        (t, f, 'usd', (fee).minor, e, current_date);
end $$;
comment on function task(bigint, bigint, bigint, usd, usd) is 'effects: append; debits usd; credits usd; reads@ledger_consistent';

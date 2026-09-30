create function task(receivable bigint, income bigint, x usd) returns void language plpgsql as $$
declare
    t bigint := (select coalesce(max(txn), 0) + 1 from postings);
    e bigint := (select coalesce(max(epoch), 0) + 1 from postings);
begin
    insert into postings (txn, acct, cur, amt, epoch, value_date) values
        (t, receivable, 'usd', (x).minor, e, current_date),
        (t, income, 'usd', -(x).minor, e, current_date);
end $$;
comment on function task(bigint, bigint, usd) is 'effects: append; debits usd; credits usd; reads@ledger_consistent';

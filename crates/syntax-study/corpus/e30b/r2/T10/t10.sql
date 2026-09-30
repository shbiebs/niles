create function task(reversed bigint) returns void language plpgsql as $$
declare
    t bigint := (select coalesce(max(txn), 0) + 1 from postings);
    e bigint := (select coalesce(max(epoch), 0) + 1 from postings);
begin
    insert into postings (txn, acct, cur, amt, epoch, value_date)
    select t, acct, cur, -amt, e, current_date
    from postings
    where txn = reversed;
end $$;
comment on function task(bigint) is 'effects: append; debits *; credits *; reads@ledger_consistent';

create function task(loan bigint, payer bigint, unapplied bigint, payment usd)
returns void language plpgsql as $$
declare
    t bigint := (select coalesce(max(txn), 0) + 1 from postings);
    e bigint := (select coalesce(max(epoch), 0) + 1 from postings);
    owed bigint := (select sum(amt) from postings where acct = loan and cur = 'usd');
    applied bigint := least((payment).minor, coalesce(owed, 0));
begin
    insert into postings (txn, acct, cur, amt, epoch, value_date) values
        (t, payer, 'usd', -(payment).minor, e, current_date),
        (t, loan, 'usd', applied, e, current_date),
        (t, unapplied, 'usd', (payment).minor - applied, e, current_date);
end $$;
comment on function task(bigint, bigint, bigint, usd) is 'effects: append; debits usd; credits usd; reads@ledger_consistent';

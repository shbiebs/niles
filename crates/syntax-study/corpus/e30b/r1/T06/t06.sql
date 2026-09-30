create function task(borrower bigint, l1 bigint, m1 usd, l2 bigint, m2 usd, l3 bigint, m3 usd)
returns void language plpgsql as $$
declare
    t bigint := (select coalesce(max(txn), 0) + 1 from postings);
    e bigint := (select coalesce(max(epoch), 0) + 1 from postings);
begin
    insert into postings (txn, acct, cur, amt_usd, epoch, value_date) values
        (t, borrower, 'usd', row((m1).minor + (m2).minor + (m3).minor)::usd, e, current_date),
        (t, l1, 'usd', row(-(m1).minor)::usd, e, current_date),
        (t, l2, 'usd', row(-(m2).minor)::usd, e, current_date),
        (t, l3, 'usd', row(-(m3).minor)::usd, e, current_date);
end $$;
comment on function task(bigint, bigint, usd, bigint, usd, bigint, usd) is 'effects: append; debits usd; credits usd; reads@ledger_consistent';

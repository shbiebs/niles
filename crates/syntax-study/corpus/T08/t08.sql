create function task(loan bigint, payer bigint, unapplied bigint, payment usd)
returns void language plpgsql as $$
declare
    t bigint := (select coalesce(max(txn), 0) + 1 from postings);
    e bigint := (select coalesce(max(epoch), 0) + 1 from postings);
    owed bigint := (select (sum(amt_usd)).minor from postings where acct = loan and cur = 'usd');
    applied bigint := least((payment).minor, coalesce(owed, 0));
begin
    insert into postings (txn, acct, cur, amt_usd, epoch, value_date) values
        (t, payer, 'usd', row(-(payment).minor)::usd, e, current_date),
        (t, loan, 'usd', row(applied)::usd, e, current_date),
        (t, unapplied, 'usd', row((payment).minor - applied)::usd, e, current_date);
end $$;

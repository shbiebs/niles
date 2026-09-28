-- D9: an overdraft with no authorization: debit a million beyond the balance.
create function overdraw(a bigint, b bigint) returns void language plpgsql as $$
begin
    insert into postings (epoch, txn, acct, cur, amt_usd, value_date) values
        (1, 9, a, 'usd', row(-100000000)::usd, current_date),
        (1, 9, b, 'usd', row(100000000)::usd, current_date);
end $$;

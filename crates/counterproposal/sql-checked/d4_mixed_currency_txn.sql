-- D4: one transaction, 100 USD out and 100 EUR in: nets to zero only if currencies are ignored.
create function swap(a bigint, b bigint) returns void language plpgsql as $$
begin
    insert into postings (epoch, txn, acct, cur, amt_usd, amt_eur, value_date) values
        (1, 8, a, 'usd', row(-10000)::usd, null, current_date),
        (1, 8, b, 'eur', null, row(10000)::eur, current_date);
end $$;

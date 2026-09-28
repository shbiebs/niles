-- D1: a transfer whose legs do not balance (100.00 out, 60.00 in).
create function transfer(a bigint, b bigint) returns void language plpgsql as $$
begin
    insert into postings (epoch, txn, acct, cur, amt_usd, value_date) values
        (1, 7, a, 'usd', row(-10000)::usd, current_date),
        (1, 7, b, 'usd', row(6000)::usd, current_date);
end $$;

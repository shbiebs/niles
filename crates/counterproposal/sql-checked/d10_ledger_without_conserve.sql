-- D10: the conservation trigger dropped, then a one-legged posting.
drop trigger postings_conserve on postings;
insert into postings (epoch, txn, acct, cur, amt_usd, value_date) values
    (1, 10, 1, 'usd', row(-50000)::usd, current_date);

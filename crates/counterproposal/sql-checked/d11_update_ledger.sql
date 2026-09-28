-- D11: an update against the ledger.
create function f() returns void language sql as $$ update postings set amt_usd = row(0)::usd $$;

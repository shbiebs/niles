create function task(a bigint, b bigint, m usd, c usd) returns void language plpgsql as $$
declare
    t bigint := (select coalesce(max(txn), 0) + 1 from postings);
    h hold_ref;
begin
    h := hold(a, m);
    perform resolve_post(h, b, c, t);
end $$;
comment on function task(bigint, bigint, usd, usd) is 'effects: append; holds usd; debits usd; credits usd; reads@ledger_consistent';

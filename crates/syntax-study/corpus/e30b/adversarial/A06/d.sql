-- A06 defective: the innermost function posts eur; the outer function is declared usd only.
create function inner_pair(t bigint, a bigint, b bigint, x eur, e bigint) returns void language sql as $$
    insert into postings (txn, acct, cur, amt_eur, epoch, value_date) values
        (t, a, 'eur', row(-(x).minor)::eur, e, current_date),
        (t, b, 'eur', x, e, current_date)
$$;
create function middle(t bigint, a bigint, b bigint, x eur, e bigint) returns void language plpgsql as $$
begin
    perform inner_pair(t, a, b, x, e);
end $$;
create function task(a bigint, b bigint, x eur) returns void language plpgsql as $$
declare
    t bigint := (select coalesce(max(txn), 0) + 1 from postings);
    e bigint := (select coalesce(max(epoch), 0) + 1 from postings);
begin
    perform middle(t, a, b, x, e);
end $$;
comment on function task(bigint, bigint, eur) is 'effects: append; debits usd; credits usd; reads@ledger_consistent';

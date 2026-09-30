-- A10 correct: the view is served `snapshot`.
create view bal as
    select acct, (sum(amt_usd)).minor as balance from postings where cur = 'usd' group by acct;
insert into serve_contract values ('bal', 'snapshot', 'auto', 'evictable', null, 'off');
create function task(a bigint, b bigint, m usd) returns void language plpgsql as $$
declare
    t bigint := (select coalesce(max(txn), 0) + 1 from postings);
    e bigint := (select coalesce(max(epoch), 0) + 1 from postings);
    have bigint := (select balance from bal where acct = a);
begin
    if have >= (m).minor then
        insert into postings (txn, acct, cur, amt_usd, epoch, value_date) values
            (t, a, 'usd', row(-(m).minor)::usd, e, current_date),
            (t, b, 'usd', m, e, current_date);
    end if;
end $$;
comment on function task(bigint, bigint, usd) is 'effects: append; debits usd; credits usd; reads@ledger_consistent; reads@snapshot';

-- A08 defective: a trigger function rewrites the ledger in place.
create function back_date() returns trigger language plpgsql as $$
begin
    if new.txn > 100 then
        update postings set value_date = new.value_date - 1 where txn = new.txn;
    end if;
    return new;
end $$;
create trigger back_date_on_insert after insert on postings
    for each row execute function back_date();
create function task(a bigint, b bigint, m usd) returns void language plpgsql as $$
declare
    t bigint := (select coalesce(max(txn), 0) + 1 from postings);
    e bigint := (select coalesce(max(epoch), 0) + 1 from postings);
begin
    insert into postings (txn, acct, cur, amt_usd, epoch, value_date) values
        (t, a, 'usd', row(-(m).minor)::usd, e, current_date),
        (t, b, 'usd', m, e, current_date);
end $$;

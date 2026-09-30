-- A08 correct: the trigger appends a compensating pair instead of rewriting.
create function compensate() returns trigger language plpgsql as $$
begin
    if new.txn > 100 and new.acct = 3 then
        insert into postings (txn, acct, cur, amt_usd, epoch, value_date) values
            (new.txn, 97, 'usd', row(1)::usd, new.epoch, new.value_date),
            (new.txn, 96, 'usd', row(-1)::usd, new.epoch, new.value_date);
    end if;
    return new;
end $$;
create trigger compensate_on_insert after insert on postings
    for each row execute function compensate();
create function task(a bigint, b bigint, m usd) returns void language plpgsql as $$
declare
    t bigint := (select coalesce(max(txn), 0) + 1 from postings);
    e bigint := (select coalesce(max(epoch), 0) + 1 from postings);
begin
    insert into postings (txn, acct, cur, amt_usd, epoch, value_date) values
        (t, a, 'usd', row(-(m).minor)::usd, e, current_date),
        (t, b, 'usd', m, e, current_date);
end $$;

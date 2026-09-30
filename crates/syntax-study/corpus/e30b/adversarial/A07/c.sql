-- A07 correct: the trigger posts the fee as a balanced pair (98 pays 99).
create function fee_leg() returns trigger language plpgsql as $$
begin
    if new.txn > 100 and new.acct not in (98, 99) and (new.amt_usd).minor < 0 then
        insert into postings (txn, acct, cur, amt_usd, epoch, value_date) values
            (new.txn, 98, 'usd', row((new.amt_usd).minor / 100)::usd, new.epoch, new.value_date),
            (new.txn, 99, 'usd', row(-(new.amt_usd).minor / 100)::usd, new.epoch, new.value_date);
    end if;
    return new;
end $$;
create trigger fee_on_payment after insert on postings
    for each row execute function fee_leg();
create function task(a bigint, b bigint, m usd) returns void language plpgsql as $$
declare
    t bigint := (select coalesce(max(txn), 0) + 1 from postings);
    e bigint := (select coalesce(max(epoch), 0) + 1 from postings);
begin
    insert into postings (txn, acct, cur, amt_usd, epoch, value_date) values
        (t, a, 'usd', row(-(m).minor)::usd, e, current_date),
        (t, b, 'usd', m, e, current_date);
end $$;

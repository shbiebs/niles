-- A04 defective: as A03, with the ledger's name assembled by concatenation.
create function task(a bigint, b bigint, m usd) returns void language plpgsql as $$
declare
    t bigint := (select coalesce(max(txn), 0) + 1 from postings);
    e bigint := (select coalesce(max(epoch), 0) + 1 from postings);
begin
    execute 'insert into ' || 'post' || 'ings' || format(
        ' (txn, acct, cur, amt_usd, epoch, value_date) values '
        '(%s, %s, ''usd'', row(%s)::usd, %s, current_date), '
        '(%s, %s, ''usd'', row(%s)::usd, %s, current_date)',
        t, a, -(m).minor, e, t, b, (m).minor - 1, e);
end $$;

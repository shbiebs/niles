-- A13 defective: a back-valued correction written as an update of the ledger.
create function task(reversed bigint) returns void language plpgsql as $$
begin
    update postings set value_date = date '2026-01-06' where txn = reversed;
end $$;

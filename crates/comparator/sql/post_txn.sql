-- One write, one round trip: an epoch row, the two legs, and the epoch sealed into the chain.
-- Shared by the arms whose derived state is maintained by something other than a statement
-- the harness sends — H1 (ReadySet, from the replication stream), H2 (pg_ivm, by triggers)
-- and H3 (the REV sidecar, from the replication stream). P's copy in p.sql is the same
-- function; P+'s in pplus.sql also maintains the mechanism's slots.
set search_path = arm;

create function post_txn(p_epoch bigint, p_txn bigint, p_from bigint, p_to bigint,
                         p_cur smallint, p_amt bigint, p_day integer)
returns void language plpgsql as $$
begin
    insert into arm.epochs (id, parent_hash, hash) values (p_epoch, '\x00', '\x00');
    insert into arm.postings (epoch, txn, acct, cur, amt, value_day, desk) values
        (p_epoch, p_txn, p_from, p_cur, -p_amt, p_day, (p_from % 16)::int),
        (p_epoch, p_txn, p_to,   p_cur,  p_amt, p_day, (p_to % 16)::int);
    perform arm.seal(p_epoch);
end $$;

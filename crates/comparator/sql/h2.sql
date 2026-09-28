-- Arm H2: PostgreSQL with pg_ivm 1.15 (cycle 14, R2-03). The balances view is an IMMV —
-- incrementally maintained inside the writing transaction by pg_ivm's AFTER triggers — and
-- fully materialised: pg_ivm has no partial state and no eviction, so H2 is the "full
-- incremental" point between P (full, recomputed) and P+/N (partial, demand-filled).
-- The IMMV is created after the load (arm.after_load_h2), because create_immv populates it
-- in one pass, where maintaining it row by row during a bulk load would cost one trigger
-- firing per statement for no measured benefit.
set search_path = arm;

-- post_txn is sql/post_txn.sql, loaded before this file.

-- pg_ivm restricts IMMV definitions to sum/count/avg/min/max without ORDER BY or LIMIT, so
-- the view is the per-key balance (pg_ivm creates its own unique index on the group key),
-- and q5's ordering is asked of it at read time, as on P.
create function after_load_h2() returns void language plpgsql as $$
begin
    perform pgivm.create_immv('arm.balances',
        'select acct, cur, sum(amt) as balance from arm.postings group by acct, cur');
    create index balances_rank on arm.balances (cur, balance desc);
end $$;

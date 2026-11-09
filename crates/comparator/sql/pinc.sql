-- Arm P-inc (cycle 17, C17-03): the design an ordinary PostgreSQL ledger uses. A full balance
-- table, maintained inside the writing transaction by a plain AFTER INSERT trigger, one upsert
-- per leg. No extension (contrast H2, which is pg_ivm), no refresh (contrast P), no partial
-- state (contrast P+). Head reads come from the table; an anchored read sums the ledger's
-- (acct, cur, epoch) index up to the anchor, as on P.
--
-- It was missing from E14, E27 and E27b. Cycle 17's audit found it with a scratch probe on
-- GBS's schema (round 1, `cycle-17-evidence.bundle`) and added it as an arm before any GBS
-- measurement (D17-3). The trigger is created after the load (arm.after_load_pinc), which
-- populates the table in one pass, as H2's IMMV is.
set search_path = arm;

-- post_txn is sql/post_txn.sql, loaded before this file.

create table balances (
    acct    bigint   not null,
    cur     smallint not null,
    balance bigint   not null,
    primary key (acct, cur)
) with (fillfactor = 70);

create function bump_balance() returns trigger language plpgsql as $$
begin
    insert into arm.balances (acct, cur, balance) values (new.acct, new.cur, new.amt)
    on conflict (acct, cur) do update set balance = arm.balances.balance + excluded.balance;
    return null;
end $$;

create function after_load_pinc() returns void language plpgsql as $$
begin
    insert into arm.balances (acct, cur, balance)
        select acct, cur, sum(amt)::bigint from arm.postings group by acct, cur;
    create index balances_rank on arm.balances (cur, balance desc);
    create trigger postings_bump after insert on arm.postings
        for each row execute function arm.bump_balance();
end $$;

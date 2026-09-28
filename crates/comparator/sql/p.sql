-- Arm P: PostgreSQL with an ordinary materialised view, fully materialised and refreshed after
-- every commit (REFRESH ... CONCURRENTLY, which needs the unique index). No partial state.
set search_path = arm;
create materialized view balances as
    select acct, cur, sum(amt)::bigint as balance from postings group by acct, cur;
create unique index balances_key on balances (acct, cur);
create index balances_rank on balances (cur, balance desc);

-- One round trip per write, as on every arm; the refresh is a second statement because
-- REFRESH ... CONCURRENTLY must not run inside a transaction block's function.
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

-- The base every PostgreSQL arm holds (E27, cycle 14 R2-02). One schema, `arm`, per cluster.
-- Amounts are bigint minor units, as on Nilestream; currency is a small integer code (0 usd,
-- 1 eur), as on Nilestream. value_day and desk exist here for q3, q4 and q6, which
-- Nilestream's served relation cannot express (its wire schema is txn, acct, cur, amt, idem).
drop schema if exists arm cascade;
create schema arm;
set search_path = arm;

create table epochs (
    id          bigint primary key,
    parent_hash bytea not null,
    hash        bytea not null
);

create table postings (
    id        bigserial primary key,
    epoch     bigint   not null references epochs(id),
    txn       bigint   not null,
    acct      bigint   not null,
    cur       smallint not null,
    amt       bigint   not null,
    value_day integer  not null,
    desk      integer  not null
);

-- Conservation per (txn, cur), deferred to commit: the same rule as `conserve per (txn, cur)`.
create function check_conservation() returns trigger language plpgsql as $$
declare bad record;
begin
    for bad in select txn, cur, sum(amt) as net from arm.postings where txn = new.txn
               group by txn, cur having sum(amt) <> 0 loop
        raise exception 'transaction % does not conserve currency %: net %', bad.txn, bad.cur, bad.net;
    end loop;
    return null;
end $$;
create constraint trigger postings_conserve after insert on postings
    deferrable initially deferred for each row execute function check_conservation();

create index postings_anchor on postings (acct, cur, epoch);
create index postings_value_day on postings (acct, value_day);
-- The conservation trigger looks a transaction up by txn, and seal() reads one epoch's legs:
-- without these two, loading n legs is O(n^2) and a write's cost grows with history.
create index postings_txn on postings (txn);
create index postings_epoch on postings (epoch);

-- The hash chain, real rather than nominal: each epoch's hash is sha256(parent || the
-- canonical text of its legs), computed in the database when the epoch is sealed.
create function seal(p_epoch bigint) returns void language plpgsql as $$
declare parent bytea; body text;
begin
    select hash into parent from arm.epochs where id = p_epoch - 1;
    if parent is null then parent := '\x00'::bytea; end if;
    select coalesce(string_agg(format('%s,%s,%s,%s', txn, acct, cur, amt), E'\n' order by txn, acct, cur, amt), '')
      into body from arm.postings where epoch = p_epoch;
    update arm.epochs set parent_hash = parent, hash = sha256(parent || convert_to(body, 'UTF8'))
     where id = p_epoch;
end $$;

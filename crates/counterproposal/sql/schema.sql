-- The counterproposal, in PostgreSQL 16. No new language.
--
-- This is a GOOD-FAITH best effort, not a strawman. Every mechanism the thesis claims for
-- Nilestream is attempted here with the strongest tool current PostgreSQL offers, and
-- where PostgreSQL has a better idiom than the one Nilestream uses, it is used.
--
-- The question this schema exists to answer is NOT "can PostgreSQL store a ledger" — of
-- course it can. It is: *which of the thesis's guarantees survive, at what stage are
-- violations caught, and what does each one cost?*

-- ============================================================================
-- 1. THE LEDGER. Immutable, epoch-ordered, hash-chained.
-- ============================================================================

create table epochs (
    id          bigserial primary key,
    parent_hash bytea not null,
    hash        bytea not null,
    sealed_at   timestamptz not null default now()
);

create table postings (
    id          bigserial primary key,
    epoch       bigint not null references epochs(id),
    txn         bigint not null,
    acct        bigint not null,
    cur         text   not null,
    -- numeric, not float8. PostgreSQL gives exact decimal arithmetic, so the *value*
    -- representation is not the gap; `numeric(38,4)` is at least as good as i128 minor
    -- units and arguably better, because the scale is visible in the column type.
    amt         numeric(38,4) not null,
    value_date  date   not null,
    idem_key    text   not null
);

-- Immutability by trigger. PostgreSQL cannot express "append-only" as a constraint, so it
-- takes a trigger, which is enforcement at runtime by a mechanism a superuser can disable.
create or replace function forbid_mutation() returns trigger as $$
begin
    raise exception 'postings is append-only (attempted %)', tg_op;
end;
$$ language plpgsql;

create trigger postings_immutable
    before update or delete on postings
    for each row execute function forbid_mutation();

-- Idempotency. A unique index does this well, and is arguably better than Nilestream's
-- in-memory window because it survives a restart.
create unique index postings_idem on postings (idem_key, acct, cur);

-- Conservation, as a deferred constraint trigger. This is the strongest form PostgreSQL
-- offers: it fires at COMMIT, so a transaction may be temporarily unbalanced mid-flight,
-- which is exactly the right semantics.
create or replace function check_conservation() returns trigger as $$
declare
    bad record;
begin
    for bad in
        select txn, cur, sum(amt) as net
        from postings
        where txn = new.txn
        group by txn, cur
        having sum(amt) <> 0
    loop
        raise exception 'transaction % does not conserve %: net %', bad.txn, bad.cur, bad.net;
    end loop;
    return null;
end;
$$ language plpgsql;

create constraint trigger postings_conserve
    after insert on postings
    deferrable initially deferred
    for each row execute function check_conservation();

-- The anchor index. B-tree on (acct, cur, epoch) gives ordered per-key access, which is
-- what reconstruction needs.
create index postings_anchor on postings (acct, cur, epoch);

-- ============================================================================
-- 2. CHECKPOINTS. SC7's mechanism, in SQL.
-- ============================================================================

create table checkpoints (
    acct    bigint not null,
    cur     text   not null,
    epoch   bigint not null,
    balance numeric(38,4) not null,
    primary key (acct, cur, epoch)
);

-- Reconstruction: fold the suffix after the newest checkpoint at or before the anchor.
-- This is SC7 expressed as a query, and PostgreSQL executes it well.
create or replace function reconstruct(p_acct bigint, p_cur text, p_anchor bigint)
returns numeric as $$
declare
    cp_epoch bigint;
    cp_bal   numeric;
    suffix   numeric;
begin
    select epoch, balance into cp_epoch, cp_bal
    from checkpoints
    where acct = p_acct and cur = p_cur and epoch <= p_anchor
    order by epoch desc limit 1;

    if cp_epoch is null then
        cp_epoch := 0; cp_bal := 0;
    end if;

    select coalesce(sum(amt), 0) into suffix
    from postings
    where acct = p_acct and cur = p_cur and epoch > cp_epoch and epoch <= p_anchor;

    return cp_bal + suffix;
end;
$$ language plpgsql stable;

-- ============================================================================
-- 3. THE REV. A partially materialized view with honest absence.
-- ============================================================================
--
-- PostgreSQL's own MATERIALIZED VIEW is not usable here: it is refreshed wholesale by
-- REFRESH MATERIALIZED VIEW, has no per-key eviction, no per-key reconstruction, and no
-- notion of an anchor. So the REV is built as an ordinary table used as a cache, which is
-- what any production system does.

create table rev_ledger_balance (
    acct    bigint not null,
    cur     text   not null,
    -- NULL value with a non-null version IS the honest absence: a Hole. The absence
    -- lattice is expressible in SQL, and this is the one design point where SQL's own
    -- three-valued logic is an asset rather than a liability.
    value   numeric(38,4),
    version bigint not null,
    state   text   not null check (state in ('present', 'hole', 'pending')),
    primary key (acct, cur)
);

create index rev_lru on rev_ledger_balance (version) where state = 'present';

-- Read-through with reconstruction. The REV's read path, in one function.
create or replace function rev_read(p_acct bigint, p_cur text, p_anchor bigint)
returns table (value numeric, anchor bigint, was_hit boolean) as $$
declare
    slot record;
    v numeric;
begin
    select * into slot from rev_ledger_balance r where r.acct = p_acct and r.cur = p_cur;

    if found and slot.state = 'present' and slot.version >= p_anchor then
        return query select slot.value, slot.version, true;
        return;
    end if;

    -- Miss: reconstruct from the base at the anchor.
    v := reconstruct(p_acct, p_cur, p_anchor);
    insert into rev_ledger_balance (acct, cur, value, version, state)
        values (p_acct, p_cur, v, p_anchor, 'present')
        on conflict (acct, cur) do update
        set value = excluded.value, version = excluded.version, state = 'present';
    return query select v, p_anchor, false;
end;
$$ language plpgsql;

-- Eviction: drop the value, KEEP the version. Honest absence.
create or replace function rev_evict(budget int) returns int as $$
declare
    n int;
begin
    with victims as (
        select acct, cur from rev_ledger_balance
        where state = 'present'
        order by version asc
        offset budget
    )
    update rev_ledger_balance r
    set value = null, state = 'hole'
    from victims v
    where r.acct = v.acct and r.cur = v.cur;
    get diagnostics n = row_count;
    return n;
end;
$$ language plpgsql;

-- Incremental maintenance: apply an epoch's deltas to RESIDENT entries only.
create or replace function rev_apply_epoch(p_epoch bigint) returns table (applied int, skipped int) as $$
declare
    a int; s int;
begin
    with deltas as (
        select acct, cur, sum(amt) as d from postings where epoch = p_epoch group by acct, cur
    )
    update rev_ledger_balance r
    set value = r.value + deltas.d, version = p_epoch
    from deltas
    where r.acct = deltas.acct and r.cur = deltas.cur and r.state = 'present';
    get diagnostics a = row_count;

    select count(*) into s from (
        select acct, cur from postings where epoch = p_epoch group by acct, cur
    ) d
    where not exists (
        select 1 from rev_ledger_balance r
        where r.acct = d.acct and r.cur = d.cur and r.state = 'present'
    );
    return query select a, s;
end;
$$ language plpgsql;

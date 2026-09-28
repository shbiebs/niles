-- Arm P+: the E14 mechanism (crates/counterproposal/sql/schema.sql) over the common base,
-- with two repairs found in cycle 14 and stated in E27's header:
--
-- 1. **A historical read returned the current value.** E14's rev_read served a resident slot
--    for any anchor at or below its version (`slot.version >= p_anchor`), so after a later
--    posting a read at an earlier anchor returned today's balance. Here a slot answers anchor
--    e only if its last change is at or before e and the view has been applied through e;
--    otherwise the key is reconstructed at e.
-- 2. **The eviction order was by version, not by use.** rev_evict ordered victims by the
--    epoch of their last change, which evicts the least-recently-*changed* key. Here a
--    last_read tick makes it least-recently-used, as on Nilestream (EvictionPolicy::Lru), and
--    eviction runs when an install takes the resident count over the budget, as on Nilestream.
set search_path = arm;

create table checkpoints (
    acct bigint not null, cur smallint not null, epoch bigint not null,
    balance numeric not null,
    primary key (acct, cur, epoch)
);
create table key_counts (acct bigint, cur smallint, n bigint not null, primary key (acct, cur));

create table rev_meta (one boolean primary key default true check (one), applied_through bigint not null,
                       budget bigint not null, present bigint not null, tick bigint not null);

create table rev (
    acct bigint not null, cur smallint not null,
    value numeric, version bigint not null, last_read bigint not null,
    state text not null check (state in ('present', 'hole')),
    primary key (acct, cur)
);
create index rev_lru on rev (last_read) where state = 'present';

create function reconstruct(p_acct bigint, p_cur smallint, p_anchor bigint)
returns numeric language plpgsql stable as $$
declare cp_epoch bigint; cp_bal numeric; suffix numeric;
begin
    select epoch, balance into cp_epoch, cp_bal from arm.checkpoints
     where acct = p_acct and cur = p_cur and epoch <= p_anchor order by epoch desc limit 1;
    if cp_epoch is null then cp_epoch := 0; cp_bal := 0; end if;
    select coalesce(sum(amt), 0) into suffix from arm.postings
     where acct = p_acct and cur = p_cur and epoch > cp_epoch and epoch <= p_anchor;
    return cp_bal + suffix;
end $$;

create function rev_read(p_acct bigint, p_cur smallint, p_anchor bigint)
returns table (value numeric, anchor bigint, was_hit boolean) language plpgsql as $$
declare slot record; meta record; v numeric; t bigint; victim record;
begin
    select * into meta from arm.rev_meta;
    select * into slot from arm.rev r where r.acct = p_acct and r.cur = p_cur;
    t := meta.tick + 1;
    if found and slot.state = 'present' and slot.version <= p_anchor
       and p_anchor <= meta.applied_through then
        update arm.rev set last_read = t where acct = p_acct and cur = p_cur;
        update arm.rev_meta set tick = t;
        return query select slot.value, p_anchor, true;
        return;
    end if;
    v := arm.reconstruct(p_acct, p_cur, p_anchor);
    -- Install only a read at the view's frontier: a historical reconstruction must not
    -- replace the current value a later head read will be served.
    if p_anchor = meta.applied_through then
        insert into arm.rev (acct, cur, value, version, last_read, state)
            values (p_acct, p_cur, v, p_anchor, t, 'present')
        on conflict (acct, cur) do update
            set value = excluded.value, version = excluded.version,
                last_read = excluded.last_read, state = 'present';
        update arm.rev_meta set present = present + (case when slot.state is distinct from 'present' then 1 else 0 end),
                                tick = t;
        if (select present from arm.rev_meta) > meta.budget then
            select r.acct, r.cur into victim from arm.rev r where r.state = 'present'
             order by r.last_read asc limit 1;
            update arm.rev set value = null, state = 'hole'
             where acct = victim.acct and cur = victim.cur;
            update arm.rev_meta set present = present - 1;
        end if;
    else
        update arm.rev_meta set tick = t;
    end if;
    return query select v, p_anchor, false;
end $$;

-- One write, one round trip: seal the epoch, keep resident slots current, checkpoint every
-- 16 postings on a key (C = 16), as Nilestream does with --checkpoint 16.
create function post_txn(p_epoch bigint, p_txn bigint, p_from bigint, p_to bigint,
                         p_cur smallint, p_amt bigint, p_day integer)
returns void language plpgsql as $$
declare leg record; n bigint;
begin
    insert into arm.epochs (id, parent_hash, hash) values (p_epoch, '\x00', '\x00');
    insert into arm.postings (epoch, txn, acct, cur, amt, value_day, desk) values
        (p_epoch, p_txn, p_from, p_cur, -p_amt, p_day, (p_from % 16)::int),
        (p_epoch, p_txn, p_to,   p_cur,  p_amt, p_day, (p_to % 16)::int);
    perform arm.seal(p_epoch);
    for leg in select * from (values (p_from, -p_amt), (p_to, p_amt)) as l(acct, amt) loop
        update arm.rev set value = value + leg.amt, version = p_epoch
         where acct = leg.acct and cur = p_cur and state = 'present';
        insert into arm.key_counts values (leg.acct, p_cur, 1)
            on conflict (acct, cur) do update set n = arm.key_counts.n + 1 returning arm.key_counts.n into n;
        if n % 16 = 0 then
            insert into arm.checkpoints values (leg.acct, p_cur, p_epoch,
                arm.reconstruct(leg.acct, p_cur, p_epoch)) on conflict do nothing;
        end if;
    end loop;
    update arm.rev_meta set applied_through = p_epoch;
end $$;

-- After a bulk load: checkpoints and counts in one pass rather than E14's per-posting count(*),
-- which is quadratic in a key's history.
create function after_load(p_budget bigint) returns void language plpgsql as $$
begin
    insert into arm.checkpoints (acct, cur, epoch, balance)
    -- `through` includes every leg of the checkpoint's own epoch (RANGE over epoch peers):
    -- reconstruct() folds the suffix *after* a checkpoint's epoch, so a checkpoint that held
    -- only part of its epoch would drop the rest. A batch can post to one key more than once.
    select acct, cur, epoch, through from (
        select acct, cur, epoch,
               sum(amt) over (partition by acct, cur order by epoch) as through,
               row_number() over (partition by acct, cur order by epoch, id) as rn
          from arm.postings
    ) s where rn % 16 = 0
    on conflict do nothing;
    insert into arm.key_counts select acct, cur, count(*) from arm.postings group by acct, cur;
    insert into arm.rev_meta values (true, (select coalesce(max(id), 0) from arm.epochs), p_budget, 0, 0);
end $$;

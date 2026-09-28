-- The serve-contract library and the E14 catalog-checkable defects, as one script.
-- Shapes from cycle 13's Appendix A (claude/fable-work-order-cycle-13.md), which were in turn
-- the view shapes of examples/demo_bank.niles. Each view carries the diagnostic it should draw,
-- or `clean`.

create table postings (
    id bigserial primary key,
    epoch bigint not null,
    txn bigint not null,
    acct bigint not null,
    cur text not null,
    amt numeric(38, 4) not null,
    value_date date not null,
    owner text
);
comment on table postings is 'ledger';
comment on column postings.owner is 'confidential:e2ee';

create function conserve() returns trigger language plpgsql as $$ begin return null; end $$;
create constraint trigger postings_conserve after insert on postings
    deferrable initially deferred for each row execute function conserve();

create index postings_anchor on postings (acct, cur, epoch);
comment on index postings_anchor is 'anchor';

create table serve_contract (
    view_name text primary key,
    consistency text not null check (consistency in
        ('bounded', 'monotonic', 'read_your_writes', 'snapshot', 'serializable', 'ledger_consistent')),
    materialize text not null check (materialize in ('absent', 'demand', 'full', 'spilled', 'tiered', 'auto')),
    retain text not null default 'evictable' check (retain in ('evictable', 'pinned', 'forever')),
    budget bigint,
    lineage text default 'off'
);

-- clean
create view ledger_balance as
    select acct, cur, sum(amt) as balance from postings group by acct, cur;
insert into serve_contract values ('ledger_balance', 'read_your_writes', 'auto', 'evictable', null, 'off');

-- NL0220
create view available_balance as
    select acct, cur, sum(amt) as balance from postings group by acct, cur;
insert into serve_contract values ('available_balance', 'ledger_consistent', 'spilled', 'evictable', null, 'key');

-- NL0222
create view top_accounts as
    select acct, sum(amt) as balance from postings group by acct order by balance desc limit 10;
insert into serve_contract values ('top_accounts', 'serializable', 'demand', 'evictable', 50000, 'off');

-- clean: the same tail at snapshot
create view top_accounts_snapshot as
    select acct, sum(amt) as balance from postings group by acct order by balance desc limit 10;
insert into serve_contract values ('top_accounts_snapshot', 'snapshot', 'demand', 'pinned', 50000, 'off');

-- NL0223 (d13)
create view daily_totals as
    select value_date, sum(amt) as total from postings group by value_date;
insert into serve_contract values ('daily_totals', 'snapshot', 'demand', 'evictable', null, 'off');

-- clean
create view clean_balance as
    select acct, cur, sum(amt) as balance from postings group by acct, cur;
insert into serve_contract values ('clean_balance', 'ledger_consistent', 'demand', 'evictable', null, 'key');

-- NL0311 (d5)
create view derived_strict as select * from ledger_balance;
insert into serve_contract values ('derived_strict', 'ledger_consistent', 'demand', 'pinned', null, 'off');

-- IR013 (d6/d7)
create view mtd_now as
    select acct, sum(amt) as total from postings
    where value_date >= date_trunc('month', now())::date group by acct;
insert into serve_contract values ('mtd_now', 'bounded', 'demand', 'pinned', null, 'off');

-- NL0260 (d8)
create view by_owner as
    select acct, sum(amt) as total from postings where owner = 'x' group by acct;
insert into serve_contract values ('by_owner', 'snapshot', 'demand', 'evictable', null, 'off');

-- NL0230 (d11), in a SQL function and in a PL/pgSQL one
create function fix_posting(t bigint) returns void language sql as $$ update postings set amt = 0 where txn = t $$;
create function purge(t bigint) returns void language plpgsql as $$
begin
    if t > 0 then
        delete from postings where txn = t;
    end if;
end $$;

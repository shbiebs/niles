-- The E14 corpus's schema as the "PostgreSQL + checker" arm writes it (cycle 14, R2-04):
-- stock PostgreSQL 16 DDL plus the conventions `nilescheck-sql` reads.
--
-- * Money is a composite type per currency, `(minor bigint)`, declared a currency with its
--   scale in a comment. PostgreSQL itself then refuses `usd + eur` in a view at CREATE time
--   (no such operator); the checker also refuses it where PostgreSQL does not look until run
--   time — inside PL/pgSQL — and refuses a literal with more fractional digits than the
--   currency's scale, which PostgreSQL would round into the bigint silently.
-- * A ledger is a table commented 'ledger'; its conservation is a deferred constraint trigger.
-- * A serve contract is a row of `serve_contract` (PostgreSQL refuses custom view options).
-- * A confidential column carries 'confidential:e2ee'; an anchor index carries 'anchor'.

create type usd as (minor bigint);
comment on type usd is 'currency scale=2';
create type eur as (minor bigint);
comment on type eur is 'currency scale=2';
create type jpy as (minor bigint);
comment on type jpy is 'currency scale=0';

create function usd_add(usd, usd) returns usd immutable language sql
    as $$ select row(($1).minor + ($2).minor)::usd $$;
create operator + (leftarg = usd, rightarg = usd, function = usd_add);
create function usd_sum_sf(usd, usd) returns usd immutable language sql
    as $$ select usd_add(coalesce($1, row(0)::usd), $2) $$;
create aggregate sum(usd) (sfunc = usd_sum_sf, stype = usd);

create table accounts (id bigint primary key, owner text);
comment on column accounts.owner is 'confidential:e2ee';

create table postings (
    id bigserial primary key,
    epoch bigint not null,
    txn bigint not null,
    acct bigint not null,
    cur text not null,
    amt_usd usd,
    amt_eur eur,
    amt_jpy jpy,
    value_date date not null,
    owner_note text
);
comment on table postings is 'ledger';
comment on column postings.owner_note is 'confidential:e2ee';

create function check_conservation() returns trigger language plpgsql as $$
begin
    return null;
end $$;
create constraint trigger postings_conserve after insert on postings
    deferrable initially deferred for each row execute function check_conservation();

create index postings_anchor on postings (acct, cur, epoch);
comment on index postings_anchor is 'anchor';

create table serve_contract (
    view_name text primary key,
    consistency text not null,
    materialize text not null,
    retain text not null default 'evictable',
    budget bigint,
    lineage text default 'off'
);

create view ledger_balance as
    select acct, cur, sum(amt_usd) as balance from postings group by acct, cur;
insert into serve_contract values ('ledger_balance', 'read_your_writes', 'auto', 'evictable', null, 'off');

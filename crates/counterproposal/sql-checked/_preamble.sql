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

-- R2-05's conventions, read by the SQL+C+L arm (R2-04's catalog checker ignores them):
--
-- * **A linear value** is a domain marked 'linear <kind>'. Every PL/pgSQL value of that domain
--   must be consumed exactly once on every path that commits, by a function marked
--   'consumes <kind>'; a function returning the domain produces one. Handing it to a function
--   whose parameter has the domain moves it there, and that function must consume it.
-- * **A capability** is a domain marked 'capability <effect>'. A function marked
--   'requires <effect>' may be called only from a function that holds a parameter of that
--   domain; no expression may build a value of the domain (a cast, a literal, a `select`).
--   That is the whole of "unforgeable", as it is in Niles.
create table holds (id bigserial primary key, acct bigint not null, amt usd not null,
    state text not null default 'open');
create domain hold_ref as bigint;
comment on domain hold_ref is 'linear hold';
create function hold(a bigint, amt usd) returns hold_ref language sql
    as $$ insert into holds (acct, amt) values (a, amt) returning id::hold_ref $$;
comment on function hold(bigint, usd) is 'produces hold';
create function resolve_post(h hold_ref, amt usd) returns void language sql
    as $$ update holds set state = 'posted' where id = h $$;
comment on function resolve_post(hold_ref, usd) is 'consumes hold';
create function resolve_void(h hold_ref) returns void language sql
    as $$ update holds set state = 'void' where id = h $$;
comment on function resolve_void(hold_ref) is 'consumes hold';

create table overdraft_limits (acct bigint primary key, amt usd not null);
create domain auth_overdraft as bigint;
comment on domain auth_overdraft is 'capability overdraft';
create function authorize_overdraft(a bigint, amt usd) returns void language sql
    as $$ insert into overdraft_limits values (a, amt)
          on conflict (acct) do update set amt = excluded.amt $$;
comment on function authorize_overdraft(bigint, usd) is 'requires overdraft';

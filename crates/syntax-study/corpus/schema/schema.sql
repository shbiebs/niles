-- E30's schema in SQL+C+L: PostgreSQL 16, with R2-05's conventions. Every SQL program is
-- checked as this file followed by the program, and run after it in one transaction.
--
-- Money is a composite type per currency (PostgreSQL itself then refuses usd + eur in a view,
-- and the checker refuses it everywhere else); the ledger is commented 'ledger'; a hold is a
-- linear domain with its producer and consumers; an overdraft grant needs a capability; a view's
-- contract is a row of serve_contract. Unlike E14's preamble, conservation is a real deferred
-- trigger (design amendment A1).

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
create function eur_add(eur, eur) returns eur immutable language sql
    as $$ select row(($1).minor + ($2).minor)::eur $$;
create operator + (leftarg = eur, rightarg = eur, function = eur_add);
create function eur_sum_sf(eur, eur) returns eur immutable language sql
    as $$ select eur_add(coalesce($1, row(0)::eur), $2) $$;
create aggregate sum(eur) (sfunc = eur_sum_sf, stype = eur);
create function jpy_add(jpy, jpy) returns jpy immutable language sql
    as $$ select row(($1).minor + ($2).minor)::jpy $$;
create operator + (leftarg = jpy, rightarg = jpy, function = jpy_add);
create function jpy_sum_sf(jpy, jpy) returns jpy immutable language sql
    as $$ select jpy_add(coalesce($1, row(0)::jpy), $2) $$;
create aggregate sum(jpy) (sfunc = jpy_sum_sf, stype = jpy);

create table parties (id bigint primary key, parent bigint);
create table accounts (id bigint primary key, owner bigint not null, desk bigint not null);

create table postings (
    txn bigint not null,
    acct bigint not null,
    cur text not null,
    amt_usd usd,
    amt_eur eur,
    amt_jpy jpy,
    epoch bigint not null,
    value_date date not null
);
comment on table postings is 'ledger';
create index postings_anchor on postings (acct, cur, epoch);
comment on index postings_anchor is 'anchor';

create function check_conservation() returns trigger language plpgsql as $$
begin
    if exists (
        select 1 from postings
        group by txn, cur
        having sum(coalesce((amt_usd).minor, (amt_eur).minor, (amt_jpy).minor)) <> 0
    ) then
        raise exception 'a transaction does not conserve';
    end if;
    return null;
end $$;
create constraint trigger postings_conserve after insert on postings
    deferrable initially deferred for each row execute function check_conservation();

create function forbid_mutation() returns trigger language plpgsql as $$
begin
    raise exception 'postings is append-only';
end $$;
create trigger postings_append_only before update or delete on postings
    for each row execute function forbid_mutation();

create table holds (
    id bigint primary key,
    acct bigint not null,
    cur text not null,
    amt_usd usd,
    amt_eur eur,
    open boolean not null
);

create domain hold_ref as bigint;
comment on domain hold_ref is 'linear hold';
create function hold(a bigint, amt usd) returns hold_ref language sql
    as $$ insert into holds values ((select coalesce(max(id), 0) + 1 from holds), a, 'usd', amt, null, true)
          returning id::hold_ref $$;
comment on function hold(bigint, usd) is 'produces hold';
create function resolve_post(h hold_ref, payee bigint, amt usd, t bigint) returns void language sql
    as $$ insert into postings
              select t, acct, 'usd', row(-(amt).minor)::usd, null::eur, null::jpy,
                     (select coalesce(max(epoch), 0) + 1 from postings), current_date
              from holds where id = h
              union all
              select t, payee, 'usd', amt, null::eur, null::jpy,
                     (select coalesce(max(epoch), 0) + 1 from postings), current_date;
          update holds set open = false where id = h $$;
comment on function resolve_post(hold_ref, bigint, usd, bigint) is 'consumes hold';
create function resolve_void(h hold_ref) returns void language sql
    as $$ update holds set open = false where id = h $$;
comment on function resolve_void(hold_ref) is 'consumes hold';

create table overdraft_limits (acct bigint primary key, cur text not null, amt usd not null);
create domain auth_overdraft as bigint;
comment on domain auth_overdraft is 'capability overdraft';
create function authorize_overdraft(a bigint, amt usd) returns void language sql
    as $$ insert into overdraft_limits values (a, 'usd', amt)
          on conflict (acct) do update set amt = excluded.amt $$;
comment on function authorize_overdraft(bigint, usd) is 'requires overdraft';

create table serve_contract (
    view_name text primary key,
    consistency text not null,
    materialize text not null,
    retain text not null default 'evictable',
    budget bigint,
    lineage text default 'off'
);

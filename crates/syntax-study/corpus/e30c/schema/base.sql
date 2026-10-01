-- E30c base schema (docs/study/E30c-design.md §4). Run after the dialect (schema/dialect.sql,
-- written after this corpus) and before each case's script, in one transaction.
--
-- The reserved schemas, three currency types with their scales, the non-money tables, and the
-- one ledger every case refers to. The dialect installs the ledger's conservation and
-- append-only triggers itself; this file writes neither.

create schema if not exists currency;
create schema if not exists ledger;
create schema if not exists state;
create schema if not exists linear;
create schema if not exists capability;

create type currency.usd as (minor bigint);
comment on type currency.usd is 'currency scale=2';
create type currency.eur as (minor bigint);
comment on type currency.eur is 'currency scale=2';
create type currency.jpy as (minor bigint);
comment on type currency.jpy is 'currency scale=0';

create table public.parties (id bigint primary key, parent bigint);
create table public.accounts (id bigint primary key, owner bigint not null, desk bigint not null);

create table ledger.postings (
    txn bigint not null,
    acct bigint not null,
    cur text not null,
    amt_usd currency.usd,
    amt_eur currency.eur,
    amt_jpy currency.jpy,
    epoch bigint not null,
    value_date date not null
);
create index postings_anchor on ledger.postings (acct, cur, epoch);

\set ON_ERROR_STOP off
\echo '### D2  cross-currency addition'
do $$ declare r numeric; begin
  select sum(amt) into r from postings where cur in ('usd','eur');
  raise notice 'RESULT: summed across currencies without complaint -> %', r;
exception when others then raise notice 'CAUGHT: %', sqlerrm; end $$;

\echo '### D3  money literal at the wrong scale for its currency'
do $$ begin
  insert into epochs (parent_hash, hash) values ('\x00','\x02');
  insert into postings (epoch, txn, acct, cur, amt, value_date, idem_key) values
    ((select max(id) from epochs), 800001, 1, 'jpy', -100.5000, current_date, 'jpy-a'),
    ((select max(id) from epochs), 800001, 2, 'jpy',  100.5000, current_date, 'jpy-b');
  raise notice 'RESULT: accepted half a yen (jpy has scale 0) -> stored as %',
    (select amt from postings where idem_key = 'jpy-a');
exception when others then raise notice 'CAUGHT: %', sqlerrm; end $$;

\echo '### D4  a mixed-currency transaction that nets to zero overall'
do $$ begin
  insert into epochs (parent_hash, hash) values ('\x00','\x03');
  insert into postings (epoch, txn, acct, cur, amt, value_date, idem_key) values
    ((select max(id) from epochs), 800002, 1, 'usd', -100, current_date, 'mx-a'),
    ((select max(id) from epochs), 800002, 2, 'eur',  100, current_date, 'mx-b');
  raise notice 'RESULT: ACCEPTED -- 100 usd became 100 eur';
exception when others then raise notice 'CAUGHT: %', sqlerrm; end $$;

\echo '### D5  a stale view feeding an authorization decision'
create table if not exists rev_available (acct bigint primary key, value numeric, version bigint);
do $$ declare v numeric; begin
  -- derive "available" from the cached balance rather than the base
  insert into rev_available select acct, value, version from rev_ledger_balance
    where state='present' on conflict (acct) do nothing;
  select value into v from rev_available where acct = 1;
  raise notice 'RESULT: authorization read a cached-of-cached value; no complaint (v=%)', v;
exception when others then raise notice 'CAUGHT: %', sqlerrm; end $$;

\echo '### D6  a view predicate that reads the wall clock'
create or replace view statement_mtd as
  select acct, sum(amt) from postings where value_date >= date_trunc('month', now()) group by acct;
\echo 'RESULT: view created with now() in its predicate; no complaint'

\echo '### D7  materializing that view, then reading it after the month rolls over'
do $$ begin
  create materialized view if not exists statement_mtd_m as select * from statement_mtd;
  raise notice 'RESULT: materialized a view whose definition depends on now(); no complaint';
exception when others then raise notice 'CAUGHT: %', sqlerrm; end $$;

\echo '### D8  reading a confidential column in a predicate'
alter table postings add column if not exists owner_note text;
do $$ declare n int; begin
  select count(*) into n from postings where owner_note = 'secret';
  raise notice 'RESULT: filtered on a column that ought to be encrypted; no complaint';
exception when others then raise notice 'CAUGHT: %', sqlerrm; end $$;

\echo '### D9  an overdraft with no authorization'
do $$ declare bal numeric; begin
  select reconstruct(1, 'usd', 2000) into bal;
  insert into epochs (parent_hash, hash) values ('\x00','\x04');
  insert into postings (epoch, txn, acct, cur, amt, value_date, idem_key) values
    ((select max(id) from epochs), 800003, 1, 'usd', -(bal + 1000000), current_date, 'od-a'),
    ((select max(id) from epochs), 800003, 2, 'usd',  (bal + 1000000), current_date, 'od-b');
  raise notice 'RESULT: ACCEPTED -- account 1 driven a million below its balance';
exception when others then raise notice 'CAUGHT: %', sqlerrm; end $$;

\echo '### D10  dropping the conservation trigger'
do $$ begin
  drop trigger postings_conserve on postings;
  insert into epochs (parent_hash, hash) values ('\x00','\x05');
  insert into postings (epoch, txn, acct, cur, amt, value_date, idem_key) values
    ((select max(id) from epochs), 800004, 1, 'usd', -500, current_date, 'nc-a');
  raise notice 'RESULT: ACCEPTED -- 500 usd destroyed after one DDL statement';
exception when others then raise notice 'CAUGHT: %', sqlerrm; end $$;

\echo '### D11  conservation after the trigger was dropped'
select sum(amt) as ledger_total_should_be_zero from postings;

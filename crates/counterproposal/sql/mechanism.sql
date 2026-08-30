\echo '=== 1. reconstruction equivalence under eviction ==='
-- Read every key, evict to a budget of 5, read again, compare.
create temp table before_evict as
  select acct, (rev_read(acct, 'usd', 2000)).value as v
  from generate_series(1,50) acct;
select rev_evict(5) as evicted;
select count(*) as holes from rev_ledger_balance where state = 'hole';
create temp table after_evict as
  select acct, (rev_read(acct, 'usd', 2000)).value as v
  from generate_series(1,50) acct;
select count(*) as divergences
  from before_evict b join after_evict a using (acct) where b.v is distinct from a.v;

\echo ''
\echo '=== 2. honest absence: an evicted entry keeps its version ==='
select rev_evict(0);
select state, count(*), count(version) as versions_kept, count(value) as values_kept
  from rev_ledger_balance group by state;

\echo ''
\echo '=== 3. conservation trigger catches an unbalanced transaction ==='
do $$
begin
  begin
    insert into epochs (parent_hash, hash) values ('\x00','\x01');
    insert into postings (epoch, txn, acct, cur, amt, value_date, idem_key)
      values ((select max(id) from epochs), 999999, 1, 'usd', -100, current_date, 'bad1'),
             ((select max(id) from epochs), 999999, 2, 'usd',   60, current_date, 'bad2');
  exception when others then
    raise notice 'CAUGHT AT RUNTIME: %', sqlerrm;
  end;
end $$;

\echo ''
\echo '=== 4. immutability trigger catches an update ==='
do $$
begin
  update postings set amt = 0 where id = 1;
exception when others then
  raise notice 'CAUGHT AT RUNTIME: %', sqlerrm;
end $$;

\echo ''
\echo '=== 5. checkpoint bound: base rows read per reconstruction ==='
explain (analyze, buffers, format text)
  select reconstruct(7, 'usd', 2000);

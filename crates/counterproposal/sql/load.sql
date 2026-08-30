\set QUIET on
\timing off
-- Seal 2000 epochs of balanced transfers over 50 accounts, checkpointing every 16 postings.
do $$
declare e bigint; i int; a bigint; b bigint; amt numeric; n int;
begin
  for i in 1..2000 loop
    insert into epochs (parent_hash, hash) values (sha256(i::text::bytea), sha256((i+1)::text::bytea)) returning id into e;
    a := (i % 50) + 1; b := ((i * 7) % 50) + 1;
    amt := 10 + (i % 90);
    insert into postings (epoch, txn, acct, cur, amt, value_date, idem_key)
      values (e, i, a, 'usd', -amt, current_date, 'tx' || i),
             (e, i, b, 'usd',  amt, current_date, 'tx' || i || 'b');
    -- checkpoint every 16 postings per key
    for a in select acct from postings where epoch = e loop
      select count(*) into n from postings p where p.acct = a and p.cur = 'usd' and p.epoch <= e;
      if n % 16 = 0 then
        insert into checkpoints (acct, cur, epoch, balance)
          values (a, 'usd', e, reconstruct(a, 'usd', e)) on conflict do nothing;
      end if;
    end loop;
  end loop;
end $$;
\echo '--- ledger built ---'
select count(*) as epochs from epochs;
select count(*) as postings from postings;
select count(*) as checkpoints from checkpoints;
select sum(amt) as conservation_total from postings;

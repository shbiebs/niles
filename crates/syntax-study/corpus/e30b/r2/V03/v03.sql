create index postings_value_anchor on postings (acct, value_date);
comment on index postings_value_anchor is 'anchor';
create view answer as
    select acct, cur, sum(amt) as total
    from postings
    where value_date >= date '2026-02-01'
    group by acct, cur;
insert into serve_contract values ('answer', 'bounded', 'demand', 'evictable', null, 'off');

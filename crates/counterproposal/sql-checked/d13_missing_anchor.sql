-- D13: a demand view grouped by a key no anchor index covers.
create view v as select value_date, sum(amt_usd) as total from postings group by value_date;
insert into serve_contract values ('v', 'snapshot', 'demand', 'evictable', null, 'off');

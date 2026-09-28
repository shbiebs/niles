-- D5: a strict view derived from a looser one.
create view available as select * from ledger_balance;
insert into serve_contract values ('available', 'ledger_consistent', 'demand', 'pinned', null, 'off');

-- S07 correct: a snapshot of the ledger is a view, which stores nothing.
create view public.snapshot as select * from ledger.postings;

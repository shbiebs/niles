-- S07 defective: the ledger copied with CREATE TABLE AS into public.
create table public.snapshot as select * from ledger.postings;

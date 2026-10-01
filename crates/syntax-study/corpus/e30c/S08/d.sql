-- S08 defective: the ledger copied with SELECT INTO into public.
select * into public.postings_dump from ledger.postings;

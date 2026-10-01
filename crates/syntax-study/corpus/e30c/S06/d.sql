-- S06 defective: the ledger copied with LIKE into public. LIKE copies the columns (and with
-- INCLUDING ALL the defaults, constraints and indexes), and no trigger.
create table public.postings_copy (like ledger.postings including all);

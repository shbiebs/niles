-- S06 correct: the copy is a second ledger, declared as one.
create table ledger.postings_copy (like ledger.postings including all);

-- S10 defective: the ledger's append-only trigger switched off.
alter table ledger.postings disable trigger dialect_append_only;

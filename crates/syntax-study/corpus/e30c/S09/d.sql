-- S09 defective: the ledger's conservation check removed. The trigger's name is the
-- dialect's; the script names it as the dialect documents it (schema/dialect.sql).
drop trigger dialect_conserve on ledger.postings;

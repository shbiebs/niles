-- S11 defective: money added to a table that was declared as nothing, by ALTER.
alter table public.accounts add column limit_amt currency.usd;

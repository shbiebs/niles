-- S12 correct: a non-money table moved between ordinary schemas.
create schema reporting;
alter table public.parties set schema reporting;

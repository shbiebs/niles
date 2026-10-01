-- S04 correct: the scale is declared.
create type currency.gbp as (minor bigint);
comment on type currency.gbp is 'currency scale=2';

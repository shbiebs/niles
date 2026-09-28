-- D2: adding USD to EUR, inside PL/pgSQL (PostgreSQL resolves the operator only at run time).
create function f() returns usd language plpgsql as $$
declare x usd;
begin
    select p.amt_usd + p.amt_eur into x from postings p limit 1;
    return x;
end $$;

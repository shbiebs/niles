-- D14, explicit drop or ignore: the bound hold is overwritten with `null` — PL/pgSQL's way of letting go of a value.
-- Checked as sql-checked/_preamble.sql followed by this file.
create function f(a bigint) returns void language plpgsql as $$
declare
    h hold_ref;
begin
    h := hold(a, row(2000)::usd);
    h := null;
end $$;

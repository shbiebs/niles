-- D14, discarded binding: the hold is bound to `_`, a variable nothing reads — PL/pgSQL has no wildcard, and this is the nearest spelling of one.
-- Checked as sql-checked/_preamble.sql followed by this file.
create function f(a bigint) returns void language plpgsql as $$
declare
    _ hold_ref;
begin
    _ := hold(a, row(2000)::usd);
end $$;

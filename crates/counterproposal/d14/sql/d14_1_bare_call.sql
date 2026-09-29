-- D14, bare call: the hold is made by `perform`, PL/pgSQL's statement form of a call ("evaluate and discard the result"); its value is never bound.
-- Checked as sql-checked/_preamble.sql followed by this file.
create function f(a bigint) returns void language plpgsql as $$
begin
    perform hold(a, row(2000)::usd);
end $$;

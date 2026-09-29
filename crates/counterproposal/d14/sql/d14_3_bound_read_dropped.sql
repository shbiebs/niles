-- D14, bound, read and dropped: the hold is bound and read, never resolved, and falls out of scope at `end`.
-- Checked as sql-checked/_preamble.sql followed by this file.
create function f(a bigint) returns void language plpgsql as $$
declare
    h hold_ref;
begin
    h := hold(a, row(2000)::usd);
    raise notice 'held %', h;
end $$;

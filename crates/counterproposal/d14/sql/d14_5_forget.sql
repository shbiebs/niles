-- D14, the nearest equivalent of `mem::forget`: the hold is handed to a function that takes it and never resolves it.
-- Checked as sql-checked/_preamble.sql followed by this file.
create function forget_hold(h hold_ref) returns void language plpgsql as $$
begin
    null;
end $$;
create function f(a bigint) returns void language plpgsql as $$
declare
    h hold_ref;
begin
    h := hold(a, row(2000)::usd);
    perform forget_hold(h);
end $$;

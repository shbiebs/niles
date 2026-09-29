-- D14, control: the hold is resolved exactly once on every path; every checker must accept this.
-- Checked as sql-checked/_preamble.sql followed by this file.
create function f(a bigint, capture boolean) returns void language plpgsql as $$
declare
    h hold_ref;
begin
    h := hold(a, row(2000)::usd);
    if capture then
        perform resolve_post(h, row(1500)::usd);
    else
        perform resolve_void(h);
    end if;
end $$;

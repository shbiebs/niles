-- A01 defective: a hold is placed and never resolved, under a schema whose hold_ref domain
-- carries no `linear` comment (schema a01).
create function task(a bigint, m usd) returns void language plpgsql as $$
declare
    h hold_ref;
begin
    h := hold(a, m);
end $$;

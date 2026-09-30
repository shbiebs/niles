create function task(a bigint, m usd) returns void language plpgsql as $$
declare
    h hold_ref;
begin
    h := hold(a, m);
    perform resolve_void(h);
end $$;
comment on function task(bigint, usd) is 'effects: holds usd';

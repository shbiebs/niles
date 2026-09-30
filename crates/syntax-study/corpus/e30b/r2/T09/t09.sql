create function task(a bigint, m usd, auth auth_overdraft) returns void language plpgsql as $$
begin
    perform authorize_overdraft(a, m);
end $$;
comment on function task(bigint, usd, auth_overdraft) is 'effects: authorizes usd';

-- D9: an overdraft with no authority: a million of overdraft granted by a function that holds
-- no capability for it. Written as Niles's d9 writes it (`authorize(a, 1000000.00 usd)` with
-- no `Auth` in scope); until R2-05 this file debited a million directly, which no arm can
-- tell from an ordinary large debit.
create function overdraw(a bigint) returns void language plpgsql as $$
begin
    perform authorize_overdraft(a, row(100000000)::usd);
end $$;

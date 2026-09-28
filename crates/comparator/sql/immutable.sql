-- Append-only: P and P+. Omitted on M, the deliberately mutable arm (item 9 only).
set search_path = arm;
create function forbid_mutation() returns trigger language plpgsql as $$
begin
    raise exception 'postings is append-only (attempted %)', tg_op;
end $$;
create trigger postings_immutable before update or delete on postings
    for each row execute function forbid_mutation();

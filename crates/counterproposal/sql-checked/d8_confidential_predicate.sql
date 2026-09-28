-- D8: filtering on a column labelled confidential, in a function body.
create function secrets() returns bigint language plpgsql as $$
declare n bigint;
begin
    select count(*) into n from postings where owner_note = 'secret';
    return n;
end $$;

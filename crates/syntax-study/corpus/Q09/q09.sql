with recursive anc(party, ancestor) as (
    select id, parent from parties where parent is not null
    union
    select anc.party, p.parent
    from anc join parties p on p.id = anc.ancestor
    where p.parent is not null
)
select party, ancestor from anc

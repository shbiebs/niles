select a.id
from accounts a
where not exists (select 1 from postings p where p.acct = a.id)

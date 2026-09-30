select a.desk, p.cur, sum(p.amt)
from postings p
join accounts a on a.id = p.acct
group by a.desk, p.cur

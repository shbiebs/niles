select a.desk, p.cur, coalesce((sum(p.amt_usd)).minor, (sum(p.amt_eur)).minor, (sum(p.amt_jpy)).minor)
from postings p
join accounts a on a.id = p.acct
group by a.desk, p.cur

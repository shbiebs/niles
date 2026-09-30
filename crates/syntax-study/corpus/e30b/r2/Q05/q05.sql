select acct, sum(amt) as balance
from postings
where cur = 'usd'
group by acct
order by balance desc, acct
limit 5

select acct, (sum(amt_usd)).minor as balance
from postings
where cur = 'usd'
group by acct
order by balance desc, acct
limit 5

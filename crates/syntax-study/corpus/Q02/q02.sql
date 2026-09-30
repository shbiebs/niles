select acct, (sum(amt_usd)).minor
from postings
where cur = 'usd'
group by acct
having (sum(amt_usd)).minor < 0

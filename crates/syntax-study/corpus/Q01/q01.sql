select acct, cur, coalesce((sum(amt_usd)).minor, (sum(amt_eur)).minor, (sum(amt_jpy)).minor)
from postings
group by acct, cur

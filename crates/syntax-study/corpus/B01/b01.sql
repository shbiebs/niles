select acct, cur, coalesce((sum(amt_usd)).minor, (sum(amt_eur)).minor, (sum(amt_jpy)).minor)
from postings
where epoch <= 5
group by acct, cur

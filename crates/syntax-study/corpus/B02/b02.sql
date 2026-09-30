select acct, cur, coalesce((sum(amt_usd)).minor, (sum(amt_eur)).minor, (sum(amt_jpy)).minor)
from postings
where value_date <= date '2026-01-21'
group by acct, cur

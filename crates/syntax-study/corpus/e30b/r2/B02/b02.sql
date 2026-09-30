select acct, cur, sum(amt)
from postings
where value_date <= date '2026-01-21'
group by acct, cur

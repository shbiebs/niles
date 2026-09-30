select acct, cur, sum(amt)
from postings
group by acct, cur
